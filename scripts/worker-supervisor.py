#!/usr/bin/env python3
"""K-246: bounded POSIX host supervision; no in-process drain guarantee."""
import argparse
import ipaddress
import json
import math
import os
from pathlib import Path
import re
import selectors
import signal
import socket
import stat
import subprocess
import sys
import threading
import time
from urllib.parse import urlsplit

SCHEMA = "zee-worker-supervisor-1"
BUILD_FIELDS = {"schema", "version", "source_sha", "source_date", "channel", "dirty"}
HEX40 = re.compile(r"[0-9a-f]{40}\Z")
LIMIT = 65536


class Reject(Exception):
    def __init__(self, code):
        self.code = code


def require(ok, code):
    if not ok:
        raise Reject(code)


def pairs_unique(pairs):
    out = {}
    for key, value in pairs:
        require(key not in out, "json")
        out[key] = value
    return out


def decode_json(data):
    try:
        return json.loads(data, object_pairs_hook=pairs_unique,
                          parse_constant=lambda _: (_ for _ in ()).throw(Reject("json")))
    except (ValueError, UnicodeError, RecursionError):
        raise Reject("json") from None


def regular_bytes(path):
    before = path.lstat()
    require(stat.S_ISREG(before.st_mode) and before.st_size <= LIMIT and
            before.st_uid in {0, os.geteuid()} and not before.st_mode & 0o022,
            "application_gate")
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(path, flags)
    try:
        opened = os.fstat(fd)
        key = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns)
        require(stat.S_ISREG(opened.st_mode) and key(before) == key(opened), "application_gate")
        with os.fdopen(fd, "rb") as stream:
            fd = None
            data = stream.read(LIMIT + 1)
            after = os.fstat(stream.fileno())
        current = path.lstat()
        require(len(data) == opened.st_size and len(data) <= LIMIT and
                stat.S_ISREG(current.st_mode) and key(opened) == key(after) == key(current),
                "application_gate")
        return data, current.st_mtime_ns
    finally:
        if fd is not None:
            os.close(fd)


def safe_environment():
    # Credentials are not inherited. K-245 composition is a separate acceptance.
    return {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC"}


class JsonLog:
    def __init__(self):
        self.start = time.monotonic()
        self.dropped = 0
        self.maximum = 4096
        try:
            if stat.S_ISFIFO(os.fstat(1).st_mode):
                self.maximum = min(self.maximum, os.fpathconf(1, "PC_PIPE_BUF"))
            os.set_blocking(1, False)
        except OSError:
            raise Reject("log_descriptor") from None

    def emit(self, event, **fields):
        record = {"schema": SCHEMA, "event": event,
                  "elapsed_ms": min(2147483647, int((time.monotonic() - self.start) * 1000))}
        record.update(fields)
        data = (json.dumps(record, separators=(",", ":"), ensure_ascii=True) + "\n").encode()
        require(len(data) <= self.maximum, "log_record")
        try:
            written = os.write(1, data)
            if written != len(data):
                self.dropped = min(2147483647, self.dropped + 1)
        except OSError:
            self.dropped = min(2147483647, self.dropped + 1)


class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise Reject("arguments")


def arguments(argv):
    p = Parser(description=__doc__, allow_abbrev=False)
    p.add_argument("--dil", required=True)
    p.add_argument("--project", required=True)
    p.add_argument("--origin", required=True)
    p.add_argument("--worker-port", type=int, action="append", required=True)
    p.add_argument("--expected-source", required=True)
    p.add_argument("--application-revision", required=True)
    p.add_argument("--readiness-file", required=True)
    p.add_argument("--health-path", required=True)
    p.add_argument("--health-body", required=True)
    p.add_argument("--pilot", action="store_true")
    p.add_argument("--poll-seconds", type=float, default=2)
    p.add_argument("--probe-timeout", type=float, default=1)
    p.add_argument("--startup-timeout", type=float, default=15)
    p.add_argument("--stop-grace", type=float, default=5)
    p.add_argument("--credential-root")
    p.add_argument("--credential-env", action="append", default=[])
    keys = [value.split("=", 1)[0] for value in argv if value.startswith("--")]
    require(all(keys.count(key) == 1 for key in set(keys) - {"--worker-port", "--credential-env"}), "arguments")
    a = p.parse_args(argv)
    require(os.name == "posix" and all(hasattr(os, name) for name in
            ("O_NOFOLLOW", "O_NONBLOCK", "waitid", "WNOWAIT", "WEXITED", "P_PID")), "platform")
    require(len(a.worker_port) == 2 and len(set(a.worker_port)) == 2 and
            all(1 <= port <= 65535 for port in a.worker_port), "ports")
    require(HEX40.fullmatch(a.expected_source) and HEX40.fullmatch(a.application_revision), "identity")
    for value, low, high in [(a.poll_seconds, 2, 60), (a.probe_timeout, .1, 5),
                             (a.startup_timeout, 1, 120), (a.stop_grace, .1, 30)]:
        require(math.isfinite(value) and low <= value <= high, "budgets")
    require(re.fullmatch(r"/[A-Za-z0-9/_-]{0,127}", a.health_path) and
            not a.health_path.startswith("//"), "health_path")
    require(0 < len(a.health_body.encode("utf-8")) <= 256 and
            not any(ord(c) < 32 or ord(c) == 127 for c in a.health_body), "health_body")
    require(len(a.origin) <= 512 and a.origin.isascii() and not any(c.isspace() for c in a.origin), "origin")
    u = urlsplit(a.origin)
    require(u.scheme == "https" and u.netloc and not u.username and not u.password and
            not u.path and not u.query and not u.fragment, "origin")
    host = u.hostname
    require(host is not None, "origin")
    try:
        address = ipaddress.ip_address(host)
        require(str(address) == host, "origin")
        host = "[" + host + "]" if address.version == 6 else host
    except ValueError:
        require(len(host) <= 253 and all(re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?", x)
                                        for x in host.split(".")), "origin")
    port = u.port
    require(port is None or 1 <= port <= 65535, "origin")
    a.host = host + (":" + str(port) if port is not None and port != 443 else "")
    a.origin = "https://" + a.host
    for name in ["dil", "project", "readiness_file"]:
        path = Path(getattr(a, name))
        require(path.is_absolute() and len(str(path)) <= 4096 and len(path.parts) <= 64, "paths")
        setattr(a, name, path)
    a.dil = a.dil.resolve(strict=True)
    a.project = a.project.resolve(strict=True)
    require(a.project.is_dir() and a.dil.is_file() and os.access(a.dil, os.X_OK) and
            not a.dil.stat().st_mode & 0o6000, "paths")
    require(bool(a.credential_root) == bool(a.credential_env) and len(a.credential_env) <= 16, "credentials")
    names, files = set(), set()
    for binding in a.credential_env:
        require(binding.count("=") == 1, "credentials")
        name, file = binding.split("=")
        require(re.fullmatch(r"[A-Z_][A-Z0-9_]{0,63}", name) and
                re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]{0,63}", file) and
                name not in names and file not in files and name not in os.environ, "credentials")
        names.add(name)
        files.add(file)
    if a.credential_root:
        require(Path(a.credential_root).is_absolute() and len(a.credential_root) <= 4096, "credentials")
    parent = a.readiness_file.parent.resolve(strict=True)
    require(parent == a.readiness_file.parent and parent.is_relative_to(a.project), "application_gate")
    # Gate parent cannot be replaced by another local user.
    cursor = parent
    while True:
        s = cursor.stat()
        require(s.st_uid in {0, os.geteuid()} and not s.st_mode & 0o022, "application_gate")
        if cursor == a.project:
            break
        cursor = cursor.parent
    return a


def free_ports(ports):
    held = []
    try:
        for port in ports:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            held.append(s)
            s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            s.bind(("127.0.0.1", port))
            s.listen(1)
    except OSError:
        raise Reject("port_collision") from None
    finally:
        for s in held:
            s.close()


def validate_build(record, a):
    require(isinstance(record, dict) and set(record) == BUILD_FIELDS and
            record["schema"] == "zee-build-v1" and record["source_sha"] == a.expected_source and
            isinstance(record["version"], str) and len(record["version"]) <= 96 and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?", record["version"]) and
            isinstance(record["source_date"], str) and re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", record["source_date"]) and
            record["channel"] in {"stable", "beta", "nightly", "development"} and
            type(record["dirty"]) is bool, "build_identity")
    require(a.pilot or (not record["dirty"] and record["channel"] != "development"), "build_policy")
    return record


class Supervisor:
    def __init__(self, a, log):
        self.a, self.log = a, log
        self.children = []
        self.stop = False
        self.reload = False
        self.wakeup = threading.Event()
        self.build = None
        self.gate_epoch_ns = time.time_ns()
        self.metrics = {"probe_total": 0, "probe_failures": 0, "probe_ms_total": 0,
                        "restart_total": 0, "forced_kills": 0}

    def signal(self, number, frame):
        if number == signal.SIGHUP:
            self.reload = True
        else:
            self.stop = True
        self.wakeup.set()

    def increment(self, key, value=1):
        self.metrics[key] = min(2147483647, self.metrics[key] + value)

    def spawn(self, argv, capture=False, environment=None, visible=False):
        # Signal handlers only set flags; even a signal during Popen cannot skip registration.
        child = subprocess.Popen(argv, cwd=self.a.project, env=safe_environment() if environment is None else environment,
                                 stdin=subprocess.DEVNULL, stdout=subprocess.PIPE if capture else (None if visible else subprocess.DEVNULL),
                                 stderr=None if visible else subprocess.DEVNULL, close_fds=True, start_new_session=True)
        self.children.append(child)
        return child

    def stop_child(self, child):
        # Only handles created above may reach killpg. Never discover/adopt a PID from a port/file.
        require(child in self.children, "ownership")
        def group_alive():
            try:
                os.killpg(child.pid, 0)
                return True
            except ProcessLookupError:
                return False
            except PermissionError:
                # Darwin reports EPERM for the vanished group of an unreaped zombie.
                # WNOWAIT still reserves this owned leader PID, so no replacement group is adopted.
                require(self.exit_status(child) is not None, "ownership")
                return False
        if group_alive():
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            except PermissionError:
                require(self.exit_status(child) is not None and not group_alive(), "ownership")
        deadline = time.monotonic() + self.a.stop_grace
        while group_alive() and time.monotonic() < deadline:
            time.sleep(.02)
        if group_alive():
            try:
                os.killpg(child.pid, signal.SIGKILL)
                self.increment("forced_kills")
            except ProcessLookupError:
                pass
            except PermissionError:
                require(self.exit_status(child) is not None and not group_alive(), "ownership")
        child.wait(timeout=2)
        if child.stdout is not None:
            child.stdout.close()
        self.children.remove(child)

    def exit_status(self, child):
        # Never poll/reap leaders until their owned group has been cleaned. This
        # retains PID authority even if a credential launcher dies before dil.
        require(child in self.children and child.returncode is None, "ownership")
        value = os.waitid(os.P_PID, child.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
        if value is None:
            return None
        return value.si_status if value.si_code == os.CLD_EXITED else -value.si_status

    def cleanup(self):
        failed = False
        for child in list(self.children):
            try:
                self.stop_child(child)
            except (Reject, OSError, subprocess.SubprocessError):
                failed = True
        require(not failed, "cleanup")

    def query_json(self, flag):
        child = self.spawn([str(self.a.dil), "sürüm", flag], capture=True)
        data = bytearray()
        deadline = time.monotonic() + 5
        selector = selectors.DefaultSelector()
        selector.register(child.stdout, selectors.EVENT_READ)
        try:
            while selector.get_map():
                require(not self.stop and time.monotonic() < deadline, "build_identity")
                for key, _ in selector.select(min(.1, max(0, deadline - time.monotonic()))):
                    chunk = os.read(key.fileobj.fileno(), 4097)
                    if not chunk:
                        selector.unregister(key.fileobj)
                    else:
                        data.extend(chunk)
                        require(len(data) <= 4096, "build_identity")
            status = self.exit_status(child)
            while status is None and time.monotonic() < deadline and not self.stop:
                time.sleep(.01)
                status = self.exit_status(child)
            require(status == 0 and not self.stop, "build_identity")
            return decode_json(bytes(data))
        finally:
            selector.close()
            self.stop_child(child)

    def application_ready(self):
        try:
            data, written = regular_bytes(self.a.readiness_file)
            value = decode_json(data)
            return (isinstance(value, dict) and set(value) == {"schema", "application_revision", "ready"}
                    and value["schema"] == "zee-worker-readiness-1"
                    and value["application_revision"] == self.a.application_revision
                    and type(value["ready"]) is bool and value["ready"]
                    and self.gate_epoch_ns <= written <= time.time_ns())
        except (Reject, OSError):
            return False

    def probe(self, port, until=None):
        started = time.monotonic()
        self.increment("probe_total")
        try:
            deadline = min(started + self.a.probe_timeout, until) if until is not None else started + self.a.probe_timeout
            require(deadline > started, "probe")
            request = ("GET " + self.a.health_path + " HTTP/1.1\r\nHost: " + self.a.host +
                       "\r\nForwarded: for=127.0.0.1;proto=https;host=" + self.a.host +
                       "\r\nConnection: close\r\n\r\n").encode("ascii")
            with socket.create_connection(("127.0.0.1", port), timeout=deadline - started) as s:
                def read(n):
                    remaining = deadline - time.monotonic()
                    require(remaining > 0 and not self.stop, "probe")
                    s.settimeout(remaining)
                    return s.recv(n)
                s.settimeout(max(.001, deadline - time.monotonic()))
                s.sendall(request)
                data = bytearray()
                while b"\r\n\r\n" not in data:
                    chunk = read(1024)
                    require(chunk and len(data) + len(chunk) <= 16384 + 256, "probe")
                    data.extend(chunk)
                header, body = bytes(data).split(b"\r\n\r\n", 1)
                require(len(header) <= 16384 and b"\0" not in header, "probe")
                lines = header.decode("ascii").split("\r\n")
                require(re.fullmatch(r"HTTP/1\.[01] 200 [ -~]*", lines[0]), "probe")
                fields = {}
                for line in lines[1:]:
                    require(all(32 <= ord(c) <= 126 for c in line), "probe")
                    key, value = line.split(":", 1)
                    require(re.fullmatch(r"[A-Za-z0-9-]+", key) and key.lower() not in fields,
                            "probe")
                    fields[key.lower()] = value.strip()
                expected = self.a.health_body.encode("utf-8")
                require(fields.get("content-length") == str(len(expected)) and
                        fields.get("connection", "").lower() == "close" and
                        "transfer-encoding" not in fields and
                        decode_json(fields.get("x-zee-build", "")) == self.build, "probe")
                require(len(body) <= len(expected), "probe")
                while len(body) < len(expected):
                    chunk = read(len(expected) - len(body))
                    require(chunk, "probe")
                    body += chunk
                require(body == expected and not read(1), "probe")
            require(time.monotonic() < deadline, "probe")
            return True
        except (OSError, Reject, UnicodeError, ValueError):
            self.increment("probe_failures")
            return False
        finally:
            self.increment("probe_ms_total", int((time.monotonic() - started) * 1000))

    def start_worker(self, index):
        require(not self.stop, "stopping")
        command = ["çalıştır", "--web-proxy", self.a.origin,
                   "--web-worker-port", str(self.a.worker_port[index]), str(self.a.project)]
        if self.a.credential_root:
            argv = [sys.executable, "-I", str(Path(__file__).resolve().parent / "credential-baslat.py"),
                    "--credential-root", self.a.credential_root]
            for binding in self.a.credential_env:
                argv.extend(["--env", binding])
            argv.extend(["--dil", str(self.a.dil), "--"] + command)
        else:
            argv = [str(self.a.dil)] + command
        child = self.spawn(argv)
        self.log.emit("worker_started", worker=index, pid=child.pid)
        return child

    def live(self, workers):
        for index, child in enumerate(workers):
            code = self.exit_status(child)
            if code is not None:
                self.log.emit("worker_exit", worker=index, exit_code=code)
                raise Reject("worker_exit")

    def wait_worker(self, workers, index):
        deadline = time.monotonic() + self.a.startup_timeout
        while not self.stop:
            require(time.monotonic() < deadline, "startup")
            self.live(workers)
            if self.probe(self.a.worker_port[index], deadline) and self.application_ready():
                require(time.monotonic() < deadline, "startup")
                return
            require(time.monotonic() < deadline, "startup")
            self.wakeup.wait(min(self.a.poll_seconds, max(0, deadline - time.monotonic())))
            self.wakeup.clear()

    def run(self):
        free_ports(self.a.worker_port)  # Ordinary collisions fail before any child launch.
        if self.stop:
            return
        self.build = validate_build(self.query_json("--json"), self.a)
        configuration = self.a.project / "zee.ayar"
        if configuration.exists() or configuration.is_symlink():
            capability = self.query_json("--ayar-semasi")
            require(isinstance(capability, dict) and
                    capability == {"schema": "zee-config-capability-1", "version": self.build["version"],
                                   "config_schemas": ["zee-ayar-1"], "unknown_security": "reject"}, "config_capability")
        self.log.emit("build", build=self.build)
        workers = []
        try:
            for index in range(2):
                if self.stop:
                    return
                workers.append(self.start_worker(index))
            ready_before, had_ready = None, False
            startup = time.monotonic() + self.a.startup_timeout
            while not self.stop:
                require(had_ready or time.monotonic() < startup, "startup")
                self.live(workers)
                if self.reload:
                    self.reload = False
                    self.gate_epoch_ns = time.time_ns()
                    self.log.emit("state", state="restarting", ready=False, workers=2)
                    for index in range(2):
                        if self.stop:
                            break
                        self.stop_child(workers[index])
                        free_ports([self.a.worker_port[index]])
                        workers[index] = self.start_worker(index)
                        self.wait_worker(workers, index)
                        self.increment("restart_total")
                    ready_before = None
                if self.stop:
                    break
                results = [self.probe(port, None if had_ready else startup) for port in self.a.worker_port]
                self.live(workers)
                application = self.application_ready()
                ready = all(results) and application
                require(had_ready or time.monotonic() < startup, "startup")
                if ready != ready_before:
                    self.log.emit("state", state="ready" if ready else "unready", ready=ready, workers=2,
                                  application_ready=application, healthy_workers=sum(results))
                    ready_before = ready
                had_ready = had_ready or ready
                require(had_ready or time.monotonic() < startup, "startup")
                self.log.emit("metrics", **self.metrics, log_drops=self.log.dropped, managed_workers=2)
                self.wakeup.wait(self.a.poll_seconds if had_ready else min(self.a.poll_seconds, max(0, startup - time.monotonic())))
                self.wakeup.clear()
        finally:
            self.log.emit("state", state="stopping", ready=False, workers=len(self.children))
            self.cleanup()
            self.log.emit("metrics", **self.metrics, log_drops=self.log.dropped, managed_workers=0)
            self.log.emit("state", state="stopped", ready=False, workers=0)


def main(argv=None):
    supervisor = None
    log = None
    previous = {}
    try:
        log = JsonLog()
        a = arguments(sys.argv[1:] if argv is None else argv)
        supervisor = Supervisor(a, log)
        for number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            previous[number] = signal.signal(number, supervisor.signal)
        supervisor.run()
        return 0
    except (Reject, OSError, ValueError, TypeError, RecursionError, subprocess.SubprocessError) as error:
        if log is not None:
            log.emit("rejected", reason=error.code if isinstance(error, Reject) else "host_error")
        return 0 if supervisor is not None and supervisor.stop else 2
    finally:
        failed = False
        if supervisor is not None:
            try:
                supervisor.cleanup()
            except (Reject, OSError, subprocess.SubprocessError):
                failed = True
                if log is not None:
                    log.emit("rejected", reason="cleanup")
        for number, handler in previous.items():
            signal.signal(number, handler)
        if failed:
            return 2


if __name__ == "__main__":
    sys.exit(main())
