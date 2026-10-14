#!/usr/bin/env python3
"""Hermetic wire tests for the owned COMMIT fault fixture, never a DB oracle."""

from pathlib import Path
import json
import queue
import runpy
import socket
import struct
import threading
import time
import unittest

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "postgresql-commit-proxy.py"))
PROXY = MODULE["CommitProxy"]
READ = MODULE["tam_oku"]
FRAME_READ = MODULE["cerceve_oku"]
GUARD_SECONDS = 3
STATE_KEYS = {
    "mode", "commit_seen", "commit_forwarded", "commit_ready", "commit_complete",
    "commit_error", "commits_seen", "completion_delivered", "error_delivered",
}


def frame(tag, body=b""):
    return tag + struct.pack("!I", len(body) + 4) + body


def startup(application="fixture"):
    body = struct.pack("!I", 196608) + (
        b"user\x00fixture\x00database\x00fixture\x00application_name\x00"
        + application.encode("ascii") + b"\x00\x00"
    )
    return struct.pack("!I", len(body) + 4) + body


def receive(connection, count):
    chunks = []
    while count:
        value = connection.recv(count)
        if not value:
            raise EOFError("peer closed")
        chunks.append(value)
        count -= len(value)
    return b"".join(chunks)


def read_frame(connection):
    tag = receive(connection, 1)
    length = struct.unpack("!I", receive(connection, 4))[0]
    if not 4 <= length <= 65536:
        raise AssertionError("test peer received invalid frame")
    return tag, receive(connection, length - 4)


def error_frame():
    # Wire payload is forwarded, but the fixture's JSON must never log M/detail.
    return frame(b"E", b"SERROR\x00VERROR\x00C23514\x00MPRIVATE_SENTINEL\x00\x00")


class Peer:
    def __init__(self, replies):
        self.replies = replies
        self.commits = []
        self.startup = None
        self.errors = []
        self.connection = None
        self.listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.listener.settimeout(GUARD_SECONDS)
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen(1)
        self.port = self.listener.getsockname()[1]
        self.thread = threading.Thread(target=self.run, daemon=True)
        self.thread.start()

    def run(self):
        try:
            self.connection, address = self.listener.accept()
            if address[0] != "127.0.0.1":
                raise AssertionError("test peer must accept loopback only")
            self.connection.settimeout(GUARD_SECONDS)
            header = receive(self.connection, 4)
            length = struct.unpack("!I", header)[0]
            self.startup = header + receive(self.connection, length - 4)
            self.connection.sendall(frame(b"R", struct.pack("!I", 0)) + frame(b"Z", b"I"))
            tag, query = read_frame(self.connection)
            if (tag, query) != (b"Q", b"COMMIT\x00"):
                raise AssertionError("fixture must forward only the expected COMMIT")
            self.commits.append(query)
            self.connection.sendall(b"".join(self.replies))
        except (EOFError, OSError):
            # The before mode and invalid startup deliberately close this peer.
            pass
        except BaseException as error:
            self.errors.append(error)
        finally:
            self.close_sockets()

    def close_sockets(self):
        for connection in (self.connection, self.listener):
            if connection is not None:
                try:
                    connection.shutdown(socket.SHUT_RDWR)
                except OSError:
                    pass
                connection.close()

    def close(self):
        self.close_sockets()
        self.thread.join(GUARD_SECONDS)
        if self.thread.is_alive():
            raise AssertionError("owned backend test thread did not stop")
        if self.errors:
            raise self.errors[0]


class CommitProxyTests(unittest.TestCase):
    def assert_closed(self, proxy):
        self.assertFalse(proxy.thread.is_alive())
        self.assertTrue(all(not thread.is_alive() for thread in proxy.threads))
        self.assertTrue(all(connection.fileno() == -1 for connection in proxy.sockets))
        self.assertEqual(proxy.listener.fileno(), -1)
        proxy.close()  # Idempotent cleanup must remain safe.

    def assert_schema(self, state):
        self.assertEqual(set(state), STATE_KEYS)
        self.assertIn(state["mode"], ("before", "after", "observe"))
        self.assertIs(type(state["commits_seen"]), int)
        self.assertEqual(state["commits_seen"], 1)
        for name in STATE_KEYS - {"mode", "commits_seen", "commit_error"}:
            self.assertIs(type(state[name]), bool, name)
        if state["commit_error"] is not None:
            self.assertEqual(set(state["commit_error"]), {"severity", "sqlstate"})
            self.assertTrue(all(type(value) is str for value in state["commit_error"].values()))
        self.assertNotIn("PRIVATE_SENTINEL", json.dumps(state))

    def start_pair(self, mode, replies, **options):
        peer = Peer(replies)
        self.addCleanup(peer.close)
        proxy = PROXY("127.0.0.1", peer.port, mode=mode, deadline_seconds=2, **options).start()
        self.addCleanup(proxy.close)
        client = socket.create_connection(("127.0.0.1", proxy.port), timeout=GUARD_SECONDS)
        client.settimeout(GUARD_SECONDS)
        self.addCleanup(client.close)
        return peer, proxy, client

    def handshake(self, client, application="fixture"):
        client.sendall(startup(application))
        self.assertEqual(read_frame(client), (b"R", struct.pack("!I", 0)))
        self.assertEqual(read_frame(client), (b"Z", b"I"))

    def test_after_requires_real_commit_ack_and_idle_then_drops_once(self):
        peer, proxy, client = self.start_pair("after", [frame(b"C", b"COMMIT\x00"), frame(b"Z", b"I")])
        self.handshake(client)
        client.sendall(frame(b"Q", b"COMMIT\x00"))
        self.assertEqual(client.recv(1), b"")
        state = proxy.finish()
        self.assert_schema(state)
        self.assertEqual(peer.commits, [b"COMMIT\x00"])
        self.assertTrue(state["commit_seen"] and state["commit_forwarded"])
        self.assertTrue(state["commit_complete"] and state["commit_ready"])
        self.assertFalse(state["completion_delivered"] or state["error_delivered"])
        self.assertIsNone(state["commit_error"])
        self.assert_closed(proxy)

    def test_before_never_forwards_commit(self):
        peer, proxy, client = self.start_pair("before", [])
        self.handshake(client)
        client.sendall(frame(b"Q", b"COMMIT\x00"))
        self.assertEqual(client.recv(1), b"")
        state = proxy.finish()
        peer.thread.join(GUARD_SECONDS)
        self.assert_schema(state)
        self.assertEqual(peer.commits, [])
        self.assertTrue(state["commit_seen"])
        self.assertFalse(state["commit_forwarded"] or state["commit_complete"] or state["commit_ready"])
        self.assertFalse(state["completion_delivered"] or state["error_delivered"])
        self.assert_closed(proxy)

    def test_observe_forwards_error_and_idle_without_claiming_commit_success(self):
        peer, proxy, client = self.start_pair("observe", [error_frame(), frame(b"Z", b"I")])
        self.handshake(client)
        client.sendall(frame(b"Q", b"COMMIT\x00"))
        self.assertEqual(read_frame(client), (b"E", error_frame()[5:]))
        self.assertEqual(read_frame(client), (b"Z", b"I"))
        client.close()
        state = proxy.finish()
        self.assert_schema(state)
        self.assertEqual(peer.commits, [b"COMMIT\x00"])
        self.assertTrue(state["commit_forwarded"] and state["commit_ready"] and state["error_delivered"])
        self.assertFalse(state["commit_complete"] or state["completion_delivered"])
        self.assertEqual(state["commit_error"], {"severity": "ERROR", "sqlstate": "23514"})
        self.assert_closed(proxy)

    def test_after_rejects_missing_or_mismatched_ack_error_and_nonidle(self):
        cases = {
            "missing-command": [frame(b"Z", b"I")],
            "rollback-command": [frame(b"C", b"ROLLBACK\x00"), frame(b"Z", b"I")],
            "other-command": [frame(b"C", b"SELECT 1\x00"), frame(b"Z", b"I")],
            "nonidle": [frame(b"C", b"COMMIT\x00"), frame(b"Z", b"T")],
            "error": [error_frame(), frame(b"Z", b"I")],
            "error-and-commit": [error_frame(), frame(b"C", b"COMMIT\x00"), frame(b"Z", b"I")],
        }
        for name, replies in cases.items():
            with self.subTest(name=name):
                peer, proxy, client = self.start_pair("after", replies)
                self.handshake(client)
                client.sendall(frame(b"Q", b"COMMIT\x00"))
                self.assertEqual(client.recv(1), b"")
                expected = ("ReadyForQuery idle" if name == "nonidle"
                            else "after fixture did not prove successful COMMIT completion")
                with self.assertRaisesRegex(ValueError, expected):
                    proxy.finish()
                self.assertEqual(peer.commits, [b"COMMIT\x00"])
                self.assert_closed(proxy)

    def test_startup_bounds_ssl_and_malformed_protocol_fail_closed(self):
        cases = {
            "below-minimum": struct.pack("!I", 7),
            "above-limit": struct.pack("!I", 65537),
            "ssl-request": struct.pack("!II", 8, 80877103),
            "wrong-protocol": struct.pack("!II", 10, 196609) + b"\x00\x00",
            "missing-terminator": struct.pack("!II", 9, 196608) + b"x",
        }
        for name, message in cases.items():
            with self.subTest(name=name):
                peer, proxy, client = self.start_pair("observe", [])
                client.sendall(message)
                with self.assertRaises(ValueError):
                    proxy.finish()
                self.assertIsNone(peer.startup)
                self.assertEqual(peer.commits, [])
                self.assert_closed(proxy)

    def test_application_identity_mismatch_is_rejected_before_forwarding(self):
        peer, proxy, client = self.start_pair("observe", [], application="expected")
        client.sendall(startup("different"))
        with self.assertRaises(ValueError):
            proxy.finish()
        self.assertIsNone(peer.startup)
        self.assert_closed(proxy)

    def test_frame_limit_is_checked_before_body_read(self):
        for length in (3, 32 * 1024 * 1024 + 1):
            with self.subTest(length=length):
                reader, writer = socket.socketpair()
                try:
                    writer.sendall(b"E" + struct.pack("!I", length))
                    # Writer stays open and supplies no body. Timeout is not a limit rejection.
                    with self.assertRaises(ValueError):
                        FRAME_READ(reader, time.monotonic() + 0.2)
                finally:
                    reader.close()
                    writer.close()
        reader, writer = socket.socketpair()
        try:
            writer.sendall(frame(b"C"))
            self.assertEqual(FRAME_READ(reader, time.monotonic() + 0.2), (b"C", struct.pack("!I", 4)))
        finally:
            reader.close()
            writer.close()

    def test_partial_progress_cannot_reset_absolute_read_deadline(self):
        reader, writer = socket.socketpair()
        results = queue.Queue()
        stop = threading.Event()
        deadline = time.monotonic() + 0.08

        def read():
            try:
                results.put(("ok", READ(reader, 32, deadline)))
            except BaseException as error:
                results.put(("error", error))
            finally:
                stop.set()

        def feed():
            try:
                for _ in range(32):
                    if stop.wait(0.02):
                        return
                    writer.sendall(b"x")
            except OSError:
                pass

        writer.sendall(b"x")
        threads = [threading.Thread(target=read, daemon=True), threading.Thread(target=feed, daemon=True)]
        for thread in threads:
            thread.start()
        try:
            kind, value = results.get(timeout=GUARD_SECONDS)
            self.assertEqual(kind, "error")
            self.assertIsInstance(value, TimeoutError)
        finally:
            stop.set()
            reader.close()
            writer.close()
            for thread in threads:
                thread.join(GUARD_SECONDS)
                self.assertFalse(thread.is_alive())

    def test_accept_deadline_and_cleanup_do_not_require_a_client(self):
        proxy = PROXY("127.0.0.1", 9, mode="before", deadline_seconds=0.08).start()
        self.addCleanup(proxy.close)
        with self.assertRaises(ValueError):
            proxy.finish()
        self.assert_closed(proxy)

    def test_nonloopback_endpoints_and_invalid_limits_are_rejected(self):
        for name in ("backend_host", "listen_host"):
            for host in ("0.0.0.0", "192.0.2.1", "example.invalid"):
                with self.subTest(name=name, host=host):
                    options = {"backend_host": "127.0.0.1", "backend_port": 9, "mode": "observe"}
                    options[name] = host
                    with self.assertRaises(ValueError):
                        PROXY(**options)
        for options in ({"mode": "unknown"}, {"deadline_seconds": 0}, {"deadline_seconds": 121},
                        {"skip_connections": -1}, {"skip_commits": -1}):
            with self.subTest(options=options):
                values = {"backend_host": "127.0.0.1", "backend_port": 9, "mode": "observe"}
                values.update(options)
                with self.assertRaises(ValueError):
                    PROXY(**values)


if __name__ == "__main__":
    unittest.main()
