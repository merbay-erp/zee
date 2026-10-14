#!/usr/bin/env python3
"""Owned loopback PostgreSQL COMMIT fault fixture; plaintext protocol only.

before cuts before forwarding COMMIT. after withholds replies and requires
CommandComplete(COMMIT), ReadyForQuery(Idle), and no ErrorResponse before EOF.
observe forwards replies unchanged, including an acknowledged server refusal.
This is not a production proxy, TLS interceptor, or outcome inference API.
"""
from __future__ import annotations
import argparse
import ipaddress
import json
import select
import socket
import struct
import threading
import time

FRAME_LIMIT = 32 * 1024 * 1024
STARTUP_LIMIT = 64 * 1024


def tam_oku(baglanti: socket.socket, uzunluk: int, deadline: float) -> bytes:
    parcalar = []
    while uzunluk:
        kalan = deadline - time.monotonic()
        if kalan <= 0:
            raise TimeoutError("fixture absolute deadline exceeded")
        baglanti.settimeout(kalan)
        parca = baglanti.recv(min(uzunluk, 64 * 1024))
        if not parca:
            raise EOFError("fixture connection closed")
        parcalar.append(parca)
        uzunluk -= len(parca)
    return b"".join(parcalar)


def cerceve_oku(baglanti: socket.socket, deadline: float) -> tuple[bytes, bytes]:
    tur = tam_oku(baglanti, 1, deadline)
    baslik = tam_oku(baglanti, 4, deadline)
    uzunluk = struct.unpack("!I", baslik)[0]
    if not 4 <= uzunluk <= FRAME_LIMIT:
        raise ValueError("invalid PostgreSQL fixture frame length")
    return tur, baslik + tam_oku(baglanti, uzunluk - 4, deadline)


def kapat(*baglantilar: socket.socket) -> None:
    for baglanti in baglantilar:
        try:
            baglanti.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        baglanti.close()


def loopback(host: str) -> None:
    if host == "localhost":
        return
    try:
        local = ipaddress.ip_address(host).is_loopback
    except ValueError:
        local = False
    if not local:
        raise ValueError("COMMIT fixture endpoints must be loopback")


class CommitProxy:
    def __init__(self, backend_host, backend_port, mode, listen_host="127.0.0.1",
                 listen_port=0, deadline_seconds=35, application=None,
                 skip_connections=0, skip_commits=0):
        loopback(listen_host)
        loopback(backend_host)
        if mode not in ("before", "after", "observe"):
            raise ValueError("invalid COMMIT fixture mode")
        if not 0 < deadline_seconds <= 120 or skip_connections < 0 or skip_commits < 0:
            raise ValueError("invalid COMMIT fixture limits")
        self.backend_host, self.backend_port, self.mode = backend_host, backend_port, mode
        self.listen_host, self.listen_port = listen_host, listen_port
        self.deadline_seconds, self.application = deadline_seconds, application
        self.skip_connections, self.skip_commits = skip_connections, skip_commits
        self.state = {"mode": mode, "commit_seen": False, "commit_forwarded": False,
                      "commit_ready": False, "commit_complete": False,
                      "commit_error": None, "commits_seen": 0,
                      "completion_delivered": False, "error_delivered": False}
        self.done, self.commit, self.client_closed = threading.Event(), threading.Event(), threading.Event()
        self.reply_ready = threading.Event()
        self.error, self.listener = None, None
        self.sockets, self.threads = [], []

    def start(self):
        self.deadline = time.monotonic() + self.deadline_seconds
        self.listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.listener.bind((self.listen_host, self.listen_port))
        self.listener.listen(4)
        self.port = self.listener.getsockname()[1]
        self.thread = threading.Thread(target=self._run, daemon=True)
        self.thread.start()
        return self

    def _send(self, target, value):
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("fixture absolute deadline exceeded")
        target.settimeout(remaining)
        target.sendall(value)

    def _accept(self):
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("fixture accept deadline exceeded")
        self.listener.settimeout(remaining)
        return self.listener.accept()[0]

    def _backend(self):
        return socket.create_connection((self.backend_host, self.backend_port),
                                        timeout=max(0.001, self.deadline - time.monotonic()))

    def _transparent(self, client):
        backend = self._backend()
        self.sockets.extend((client, backend))
        def copy(source, target):
            try:
                while not self.done.is_set():
                    source.settimeout(max(0.001, self.deadline - time.monotonic()))
                    chunk = source.recv(64 * 1024)
                    if not chunk:
                        break
                    self._send(target, chunk)
            except OSError:
                pass
            finally:
                kapat(client, backend)
        threads = [threading.Thread(target=copy, args=pair, daemon=True)
                   for pair in [(client, backend), (backend, client)]]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(max(0, self.deadline - time.monotonic()))
        if any(thread.is_alive() for thread in threads):
            raise TimeoutError("skipped fixture connection exceeded deadline")

    def _run(self):
        try:
            for _ in range(self.skip_connections):
                self._transparent(self._accept())
            client = self._accept()
            self.sockets.append(client)
            self.listener.close()
            backend = self._backend()
            self.sockets.append(backend)
            header = tam_oku(client, 4, self.deadline)
            length = struct.unpack("!I", header)[0]
            if not 8 <= length <= STARTUP_LIMIT:
                raise ValueError("invalid PostgreSQL fixture startup length")
            startup = tam_oku(client, length - 4, self.deadline)
            if struct.unpack("!I", startup[:4])[0] != 196608:
                raise ValueError("COMMIT fixture requires plaintext protocol3 startup")
            if not startup.endswith(b"\x00\x00"):
                raise ValueError("invalid PostgreSQL fixture startup terminator")
            fields = startup[4:].split(b"\x00")[:-2]
            if len(fields) % 2:
                raise ValueError("invalid PostgreSQL fixture startup fields")
            settings = dict(zip(fields[::2], fields[1::2]))
            if self.application is not None and settings.get(b"application_name") != self.application.encode():
                raise ValueError("COMMIT fixture application identity mismatch")
            self._send(backend, header + startup)
            self._relay(client, backend)
        except (EOFError, OSError, ValueError) as error:
            if not self.done.is_set() and not (isinstance(error, EOFError) and self.state["commit_ready"]):
                self.error = type(error).__name__ + ": " + str(error)
        finally:
            self.done.set()
            kapat(*self.sockets)
            if self.listener is not None:
                self.listener.close()

    def _relay(self, client, backend):
        # One owner reads, forwards, and updates protocol state. Short select
        # polls let explicit cleanup stop the fixture without concurrent socket
        # timeout/shutdown races; the absolute deadline bounds partial frames.
        while not self.done.is_set():
            remaining = self.deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError("fixture absolute deadline exceeded")
            readable, _, _ = select.select([client, backend], [], [], min(0.1, remaining))
            for source in readable:
                if self.done.is_set():
                    return
                try:
                    tag, rest = cerceve_oku(source, self.deadline)
                except EOFError:
                    if source is client and self.state["commit_error"] is not None:
                        self.client_closed.set()
                        if self.state["commit_ready"]:
                            self.done.set()
                            return
                        # ERROR may make the real CLI exit before RFQ. Continue
                        # observing the server through Idle, without replay.
                        while not self.done.is_set():
                            tag, rest = cerceve_oku(backend, self.deadline)
                            if self._reply(client, tag, rest):
                                return
                        return
                    raise
                if source is client:
                    query = rest[4:].rstrip(b"\x00").strip().upper()
                    if tag == b"Q" and query == b"COMMIT":
                        self.state["commits_seen"] += 1
                        if self.state["commits_seen"] > self.skip_commits:
                            self.state["commit_seen"] = True
                            self.commit.set()
                            if self.mode == "before":
                                self.done.set()
                                return
                            self.state["commit_forwarded"] = True
                    self._send(backend, tag + rest)
                elif self._reply(client, tag, rest):
                    return

    def _reply(self, client, tag, rest):
        body = rest[4:]
        if self.commit.is_set():
            if tag == b"E":
                fields = {part[:1]: part[1:] for part in body.split(b"\x00") if part}
                self.state["commit_error"] = {
                    "severity": fields.get(b"V", fields.get(b"S", b"")).decode("ascii", "replace"),
                    "sqlstate": fields.get(b"C", b"").decode("ascii", "replace")}
            elif tag == b"C" and body == b"COMMIT\x00":
                self.state["commit_complete"] = True
            elif tag == b"Z":
                self.state["commit_ready"] = body == b"I"
                if self.mode == "after":
                    self.done.set()
                    return True
            if self.mode == "after":
                return False
        try:
            self._send(client, tag + rest)
            if self.commit.is_set() and tag == b"E":
                self.state["error_delivered"] = True
            if self.commit.is_set() and tag == b"C" and body == b"COMMIT\x00":
                self.state["completion_delivered"] = True
        except OSError:
            if not (self.state["error_delivered"] and self.state["commit_ready"]):
                raise
        if self.commit.is_set() and tag == b"Z" and self.mode == "observe":
            # finish() is called after the owner has its CLI result; preserve
            # the successful response connection until that explicit cleanup.
            self.reply_ready.set()
        if self.client_closed.is_set() and self.state["commit_ready"]:
            self.done.set()
            return True
        return False

    def finish(self):
        completion = self.reply_ready if self.mode == "observe" else self.done
        while not completion.is_set() and not self.done.is_set():
            remaining = self.deadline - time.monotonic()
            if remaining <= 0:
                break
            completion.wait(min(0.05, remaining))
        self.close()
        if self.error:
            raise ValueError(self.error)
        if not self.state["commit_seen"]:
            raise ValueError("target COMMIT was not observed")
        if self.mode != "before" and not self.state["commit_ready"]:
            raise ValueError("COMMIT fixture did not observe ReadyForQuery idle")
        if self.mode == "after" and (not self.state["commit_complete"] or self.state["commit_error"] is not None):
            raise ValueError("after fixture did not prove successful COMMIT completion")
        return dict(self.state)

    def close(self):
        self.done.set()
        if self.listener is not None:
            kapat(self.listener)
        kapat(*self.sockets)
        main = getattr(self, "thread", None)
        if main is not None and main is not threading.current_thread():
            main.join(timeout=2)
            if main.is_alive():
                raise ValueError("COMMIT fixture thread did not stop")
        # Join the owner before inspecting its final acquired socket collection.
        kapat(*self.sockets)
        for thread in self.threads:
            if thread is not None and thread is not threading.current_thread():
                thread.join(timeout=2)
                if thread.is_alive():
                    raise ValueError("COMMIT fixture thread did not stop")


def ana() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("before", "after", "observe"), required=True)
    parser.add_argument("--listen-host", default="127.0.0.1")
    parser.add_argument("--listen-port", type=int, default=55432)
    parser.add_argument("--backend-host", default="127.0.0.1")
    parser.add_argument("--backend-port", type=int, default=5432)
    parser.add_argument("--skip-connections", type=int, default=0)
    parser.add_argument("--skip-commits", type=int, default=0)
    parser.add_argument("--deadline-seconds", type=float, default=35)
    parser.add_argument("--application")
    args = parser.parse_args()
    proxy = CommitProxy(**vars(args)).start()
    print(json.dumps({"ready": True, "host": args.listen_host, "port": proxy.port}, sort_keys=True), flush=True)
    try:
        print(json.dumps(proxy.finish(), sort_keys=True), flush=True)
        return 0
    except ValueError as error:
        print(json.dumps({"status": "failed", "reason": str(error)}, sort_keys=True), flush=True)
        return 2


if __name__ == "__main__":
    raise SystemExit(ana())
