"""Fresh test-only foreground PG owner; importing starts no process or fixture.

Lineage: copied K306 OwnedLocalCluster/bounded_bytes, with an injected original
lifecycle owner dict and explicit local path/hash guards. No Linux Campaign.
"""
import hashlib
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time


def bounded_bytes(path,limit=262144):
    before=path.lstat()
    if path.resolve()!=path or not stat.S_ISREG(before.st_mode) or before.st_size>limit:raise ValueError('fixed consumed file')
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
    try:
        key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
        opened=os.fstat(fd)
        if key(before)!=key(opened):raise ValueError('fixed consumed identity')
        raw=bytearray()
        while len(raw)<=limit:
            part=os.read(fd,min(65536,limit+1-len(raw)))
            if not part:break
            raw.extend(part)
        if len(raw)>limit or len(raw)!=opened.st_size or key(opened)!=key(os.fstat(fd)) or key(opened)!=key(path.lstat()):raise ValueError('fixed consumed bytes')
        return bytes(raw)
    finally:os.close(fd)


def private_directory(info):
    if not (stat.S_ISDIR(info.st_mode) and info.st_uid == os.geteuid()
            and stat.S_IMODE(info.st_mode) == 0o700):
        raise ValueError('owned private PostgreSQL directory')


def digest_file(path, maximum=134217728):
    path = Path(path)
    if (not path.is_absolute() or path.resolve(strict=True) != path
            or any(c in str(path) for c in '\n\r\t\0')):
        raise ValueError('canonical public PostgreSQL binary')
    before = path.lstat()
    if (not stat.S_ISREG(before.st_mode) or before.st_uid not in (0, os.geteuid())
            or before.st_mode & 0o022 or before.st_size > maximum):
        raise ValueError('bounded public PostgreSQL binary')
    identity = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_uid, s.st_gid,
                          s.st_nlink, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    try:
        opened = os.fstat(fd)
        if identity(before) != identity(opened):
            raise ValueError('public PostgreSQL binary identity')
        digest = hashlib.sha256()
        seen = 0
        while seen <= maximum:
            try:
                block = os.read(fd, min(65536, maximum + 1 - seen))
            except InterruptedError:
                continue
            if not block:
                break
            seen += len(block)
            digest.update(block)
        if (seen != opened.st_size or seen > maximum or identity(opened)
                != identity(os.fstat(fd)) or identity(opened) != identity(path.lstat())):
            raise ValueError('public PostgreSQL binary changed')
        return digest.hexdigest()
    finally:
        os.close(fd)


class OwnedLocalCluster:
    """Fresh POSIX fixture only; retain the actual foreground PostgreSQL child."""
    LOG_LIMIT = 8 * 1024 * 1024
    READY_LIMIT = 64 * 1024

    def __init__(self, campaign, directory, cert, key, binaries, owner):
        self.c = campaign
        self.owner = owner
        self.require(isinstance(owner, dict) and all(callable(owner.get(name))
                     for name in ('held_status', 'stop_owned', 'controlled_env'))
                     and type(owner.get('CANCELLED')) is bool, 'injected original owner')
        self.directory, self.cert, self.key = Path(directory), Path(cert), Path(key)
        self.binaries = Path(binaries)
        self.data = self.directory / 'data'
        self.protocol = 'TLSv1.2'
        self.child = None
        self.stream = None
        self.closed = False
        self.started = False
        self.failed = False
        self.probes = []
        self.close_errors = []
        self.row = {'schema': 'zee-owned-foreground-postgresql-1', 'status': 'not-started',
                    'readiness': [], 'fallback_used': False, 'normal_shutdown': False}

    @staticmethod
    def require(value, reason):
        if not value:
            raise ValueError(reason)

    def active(self):
        self.c.need(True)
        self.require(not self.c.stopped and not self.owner['CANCELLED'] and not getattr(self.c, 'cleaning', False),
                     'PostgreSQL fixture cancelled')
        if self.child is not None:
            self.require(self.owner['held_status'](self.child) is None, 'PostgreSQL fixture exited early')
            self.log_size()

    def log_size(self):
        opened, named = os.fstat(self.stream.fileno()), self.log.lstat()
        self.require(stat.S_ISREG(opened.st_mode) and stat.S_ISREG(named.st_mode)
                     and (opened.st_dev, opened.st_ino) == (named.st_dev, named.st_ino)
                     and opened.st_size <= self.LOG_LIMIT, 'PostgreSQL server log identity/budget')
        return opened.st_size

    def remaining_cleanup(self):
        deadline = getattr(self.c, 'cleanup_deadline', None)
        if deadline is None:
            deadline = self.c.deadline
        return max(0.0, deadline - time.monotonic())

    def readiness_command(self, attempt, deadline):
        self.active()
        log = self.c.out / f'postgres-ready-{attempt:03d}.log'
        argv = self.psql + ['--no-psqlrc', '--host', str(self.directory), '--port', str(self.port),
                           '--username', 'postgres', '--dbname', 'postgres', '--tuples-only',
                           '--no-align', '--set', 'ON_ERROR_STOP=1', '--command',
                           'SHOW data_directory; SHOW server_version;']
        row = {'argv': argv, 'log': log.name, 'status': 'failed', 'timeout_seconds': 3}
        self.row['readiness'].append(row)
        retained = {'child': None, 'row': row}
        self.probes.append(retained)
        child = None
        terminal = None
        start = time.monotonic()
        end = min(deadline, start + 3, self.c.deadline)
        row['timeout_seconds'] = max(0, end - start)
        try:
            with log.open('xb') as stream:
                log.chmod(0o600)
                self.active()
                self.require(time.monotonic() < end, 'PostgreSQL readiness command deadline')
                retained['child'] = subprocess.Popen(argv, cwd=self.c.out, env=self.env,
                                                    stdin=subprocess.DEVNULL, stdout=stream,
                                                    stderr=subprocess.STDOUT, start_new_session=True, close_fds=True)
                child = retained['child']
                try:
                    while True:
                        self.active()
                        self.require(time.monotonic() < end and os.fstat(stream.fileno()).st_size <= self.READY_LIMIT,
                                     'PostgreSQL readiness command budget')
                        terminal = self.owner['held_status'](child)
                        if terminal is not None:
                            break
                        time.sleep(min(.05, max(0, end - time.monotonic())))
                    row['observed_exit'] = terminal
                    self.require(terminal in (0, 2), 'PostgreSQL readiness command exit')
                finally:
                    # No cancellation/deadline predicate may skip this original handle.
                    if child.returncode is None:
                        grace = min(.2 if terminal in (0, 2) else 3,
                                    max(0, end - time.monotonic()), self.remaining_cleanup())
                        self.owner['stop_owned'](child, grace=grace)
                    row.update(exit=child.returncode, reaped=child.returncode is not None)
            self.active()
            self.require(time.monotonic() < end and child.returncode == terminal,
                         'PostgreSQL readiness cancelled/deadline after reap')
            raw = bounded_bytes(log, self.READY_LIMIT)
            row.update(status='passed' if terminal == 0 else 'readiness-retry',
                       sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))
            return terminal, raw
        finally:
            # A failed first cleanup retains the same child for one bounded retry.
            if child is not None and child.returncode is None:
                self.owner['stop_owned'](child, grace=min(.2, self.remaining_cleanup()))
                row.update(exit=child.returncode, reaped=child.returncode is not None)
            row['seconds'] = time.monotonic() - start
            if log.exists():
                raw = bounded_bytes(log, self.READY_LIMIT)
                row.update(sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))

    def start(self):
        self.active()
        self.require(self in self.c.clusters and self.child is None and not self.closed
                     and not self.started and self.row['status'] == 'not-started', 'PostgreSQL registration')
        self.c.proof.setdefault('postgres_clusters', []).append(self.row)
        self.row['status'] = 'starting'
        try:
            self.require(self.directory.resolve(strict=True) == self.directory
                         and any(path == self.directory and identity == (self.directory.stat().st_dev, self.directory.stat().st_ino)
                                 for path, identity in self.c.tempdirs), 'registered PostgreSQL private directory')
            private_directory(self.directory.stat())
            self.require(self.cert.parent == self.directory and self.key.parent == self.directory
                         and self.binaries.resolve(strict=True) == self.binaries, 'PostgreSQL fixture paths')
            self.require(all(stat.S_ISREG(path.lstat().st_mode) for path in (self.cert, self.key)),
                         'PostgreSQL certificate/key fixture shape')
            for name in ('initdb', 'postgres', 'psql'):
                path = self.binaries / name
                self.require(digest_file(path) == self.c.pg_pins[str(path)], 'PostgreSQL runtime pin')
            self.env = {k: v for k, v in self.owner['controlled_env']().items() if not k.startswith('PG')}
            self.env['LC_ALL'] = 'C'
            self.c.command('owned-PG-initdb', [self.binaries / 'initdb', '--no-locale', '--encoding=UTF8',
                           '--auth=trust', '--username=postgres', '--pgdata', self.data], 60, self.env)
            self.active()
            self.version = self.c.command('owned-PG-version', [self.binaries / 'postgres', '--version'], 15, self.env).decode().strip()
            self.require(self.version == 'postgres (PostgreSQL) 16.15', 'PostgreSQL fixture version')
            self.psql = [str(self.binaries / 'psql')]
            import socket
            with socket.socket() as listener:
                listener.bind(('127.0.0.1', 0))
                self.port = listener.getsockname()[1]
            self.psql_port = self.port
            self.identity = {'binaries': str(self.binaries), 'platform': sys.platform, 'ownership': 'original-foreground-Popen'}
            config = (f"listen_addresses='127.0.0.1'\nport={self.port}\n"
                      f"unix_socket_directories='{self.directory}'\nssl=on\n"
                      f"ssl_min_protocol_version='{self.protocol}'\nssl_max_protocol_version='{self.protocol}'\n"
                      f"ssl_cert_file='{self.cert}'\nssl_key_file='{self.key}'\n"
                      f"hba_file='{self.directory / 'pg_hba.conf'}'\nlogging_collector=off\n")
            (self.data / 'postgresql.conf').write_text(config)
            self.log = self.c.out / 'postgres-server.log'
            argv = [str(self.binaries / 'postgres'), '-D', str(self.data)]
            self.row.update(argv=argv, log=self.log.name, data_directory=str(self.data),
                            socket_directory=str(self.directory), port=self.port, version=self.version,
                            log_limit_bytes=self.LOG_LIMIT, log_limit_semantics='observed-during-start-readiness-and-close',
                            postgres_sha256=self.c.pg_pins[str(self.binaries / 'postgres')])
            self.stream = self.log.open('xb')
            self.log.chmod(0o600)
            self.active()
            self.child = subprocess.Popen(argv, cwd=self.c.out, env=self.env, stdin=subprocess.DEVNULL,
                                          stdout=self.stream, stderr=subprocess.STDOUT,
                                          start_new_session=True, close_fds=True)
            self.row['pid'] = self.child.pid
            end = min(time.monotonic() + 30, self.c.deadline)
            attempt = 0
            while time.monotonic() < end:
                self.active()
                attempt += 1
                code, raw = self.readiness_command(attempt, end)
                if code == 0:
                    self.require(raw.decode().strip().splitlines() == [str(self.data), '16.15'],
                                 'PostgreSQL data-directory/version identity')
                    self.active()
                    self.require(time.monotonic() < end, 'PostgreSQL terminal readiness deadline')
                    self.started = True
                    self.row.update(status='ready', readiness_identity_verified=True)
                    return
                time.sleep(min(.1, max(0, end - time.monotonic())))
            raise ValueError('PostgreSQL readiness deadline')
        except BaseException as error:
            self.failed = True
            self.row.update(status='failed', failure_type=type(error).__name__)
            raise

    def close(self):
        if self.closed:
            self.require(not self.close_errors, 'previous PostgreSQL cleanup failed')
            return
        errors = self.close_errors
        terminal = None
        end = time.monotonic() + min(30, self.remaining_cleanup())
        self.row['status'] = 'closing'
        try:
            for retained in reversed(self.probes):
                child = retained['child']
                if child is not None and child.returncode is None:
                    try:
                        self.owner['stop_owned'](child, grace=min(.2, self.remaining_cleanup()))
                        retained['row'].update(exit=child.returncode, reaped=child.returncode is not None)
                    except BaseException as error:
                        errors.append('owned PostgreSQL readiness cleanup: ' + type(error).__name__)
            if self.child is not None:
                terminal = (self.owner['held_status'](self.child) if self.child.returncode is None
                            else self.row.get('observed_exit'))
                if terminal is None and time.monotonic() < end:
                    os.kill(self.child.pid, signal.SIGINT)
                    self.row['SIGINT_requested'] = True
                    while terminal is None and time.monotonic() < end:
                        self.log_size()
                        time.sleep(min(.05, max(0, end - time.monotonic())))
                        terminal = self.owner['held_status'](self.child)
                self.row['observed_exit'] = terminal
                if terminal != 0 or not self.row.get('SIGINT_requested') or time.monotonic() >= end:
                    self.row['fallback_used'] = True
                    errors.append('non-normal PostgreSQL terminal status')
        except BaseException as error:
            errors.append(type(error).__name__)
        finally:
            try:
                if self.child is not None:
                    if self.child.returncode is None:
                        if terminal != 0:
                            self.row['fallback_used'] = True
                        # No fresh shutdown budget, and no PID-derived adoption.
                        self.owner['stop_owned'](self.child, grace=min(.2, max(0, end - time.monotonic()), self.remaining_cleanup()))
                    self.row.update(exit=self.child.returncode, reaped=self.child.returncode is not None)
            except BaseException as error:
                errors.append('owned PostgreSQL cleanup: ' + type(error).__name__)
            finally:
                for retained in reversed(self.probes):
                    child = retained['child']
                    if child is not None and child.returncode is None:
                        try:
                            self.owner['stop_owned'](child, grace=min(.2, self.remaining_cleanup()))
                            retained['row'].update(exit=child.returncode, reaped=child.returncode is not None)
                        except BaseException as error:
                            errors.append('final owned readiness cleanup: ' + type(error).__name__)
                try:
                    if self.stream is not None:
                        self.stream.close()
                except BaseException as error:
                    errors.append('PostgreSQL log close: ' + type(error).__name__)
            self.closed = ((self.child is None or self.child.returncode is not None)
                           and all(p['child'] is None or p['child'].returncode is not None for p in self.probes)
                           and (self.stream is None or self.stream.closed))
            if hasattr(self, 'log') and self.log.exists() and self.closed:
                try:
                    raw = bounded_bytes(self.log, self.LOG_LIMIT)
                    self.row.update(log_sha256=hashlib.sha256(raw).hexdigest(), log_bytes=len(raw))
                    self.row['normal_shutdown'] = (self.child is not None and terminal == 0
                        and self.child.returncode == 0 and self.row.get('SIGINT_requested') is True
                        and not self.row['fallback_used'] and not errors
                        and b'database system is shut down' in raw)
                except BaseException as error:
                    errors.append('PostgreSQL log verification: ' + type(error).__name__)
            if self.child is not None and not self.row['normal_shutdown'] and not errors:
                errors.append('normal PostgreSQL shutdown not observed')
            self.row.update(cleanup_errors=errors, cancelled=self.c.stopped or self.owner['CANCELLED'],
                            owned_child_cleanup_complete=self.closed)
            self.row['status'] = ('closed' if self.closed and not errors and not self.failed
                                 and not self.row['cancelled'] else 'failed')
        self.require(not errors and self.closed, 'PostgreSQL owned cleanup failed')
