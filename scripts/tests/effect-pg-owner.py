"""DRAFT standalone owned plaintext PostgreSQL adapter, never a native entrypoint.

The caller registers this resource-free object BEFORE start, owns the private
root, and supplies absolute work/cleanup deadlines. No helper globals are copied
or patched. Receipts are derived only from retained future original children;
public_state exports no PID, argv, SQL, path, bytes or hashes.
"""
import os
import selectors
import signal
import stat
import subprocess
import time
from pathlib import Path

STATE = 'DRAFT-NOT-EXECUTABLE'
COMPLETE_FOR_INTEGRATION_REVIEW = True


def check(value, reason):
    if not value:
        raise ValueError(reason)


class PlaintextPG:
    PUBLIC_ROLES = ('initdb', 'version', 'readiness', 'private-command',
                    'fixture-sql', 'observer-sql', 'native-operation')

    def __init__(self, h, pg, work, port, work_deadline, cleanup_deadline, cancelled):
        # Resource free: no filesystem, process, socket, FD, signal or deadline arm.
        self.h = h
        self.pg = Path(pg)
        self.work = Path(work)
        self.data = self.work / 'data'
        self.port = port
        self.work_deadline = work_deadline
        self.cleanup_deadline = cleanup_deadline
        self.cancelled = cancelled
        self.child = None
        self.commands = []
        self.readiness = []
        self.work_identity = None
        self.start_attempted = False
        self.pg_spawn_attempted = False
        self.pg_spawn_uncertain = False
        self.ready = False
        self.closed = False
        self.failed = False
        self.uncertain = False
        self.fallback = False
        self.normal_signal = None
        self.observed_exit = None
        self.exit = None
        self.close_errors = []

    def _h(self, name):
        return self.h[name] if isinstance(self.h, dict) else getattr(self.h, name)

    def _cancelled(self):
        direct = self.h.get('CANCELLED', False) if isinstance(self.h, dict) else getattr(self.h, 'CANCELLED', False)
        globals_ = getattr(self._h('held_status'), '__globals__', {})
        return bool(self.cancelled() or direct or globals_.get('CANCELLED', False))

    def remaining_work(self, limit):
        check(type(limit) in (int, float) and 0 < limit <= 180, 'bounded work timeout')
        left = self.work_deadline() - time.monotonic()
        check(not self._cancelled() and left > 0, 'work deadline/cancelled')
        return min(limit, left)

    def remaining_cleanup(self):
        # Caller chooses END for inline cleanup, or its once-armed terminal90 end.
        left = self.cleanup_deadline() - time.monotonic()
        check(left > 0, 'caller absolute cleanup deadline')
        return left

    def _environment(self):
        env = self._h('controlled_env')()
        for key in list(env):
            if key.startswith(('PG', 'ZEE_')) or key in ('SSLKEYLOGFILE', 'OPENSSL_CONF',
                    'OPENSSL_MODULES', 'OPENSSL_ENGINES'):
                env.pop(key)
        return env

    def private_root(self):
        check(self.work.is_absolute() and self.work.resolve(strict=True) == self.work,
              'canonical registered private root')
        check(not any(c in str(self.work) for c in ("'", '\\', '\x00', '\n', '\r', '\t')),
              'private root safe for fixed PostgreSQL config')
        value = self.work.lstat()
        check(stat.S_ISDIR(value.st_mode) and value.st_uid == os.geteuid()
              and stat.S_IMODE(value.st_mode) == 0o700, 'owned private root0700')
        identity = (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid)
        if self.work_identity is None:
            self.work_identity = identity
        else:
            check(identity == self.work_identity, 'original registered private root identity')

    @staticmethod
    def _pipe_key(pipe):
        fd = pipe.fileno()
        value = os.fstat(fd)
        return fd, value.st_dev, value.st_ino, value.st_mode

    def _mark(self, row, error, uncertain=False):
        self.failed = True
        row['errors'].append(type(error).__name__)
        row['uncertain'] = row['uncertain'] or uncertain
        self.uncertain = self.uncertain or uncertain

    def _stop_original(self, child, observed):
        check(child.returncode is None, 'retained original child authority before reap')
        # H.stop_owned has a final wait(timeout=2); reserve it inside caller end.
        left = self.remaining_cleanup()
        check(left > 2.05, 'original cleanup needs wait reserve inside absolute end')
        result = self._h('stop_owned')(child, grace=min(.2 if observed == 0 else 3, left - 2.05))
        check(time.monotonic() < self.cleanup_deadline(), 'original reap exceeded caller end')
        check(type(child.returncode) is int and result == child.returncode,
              'original child terminal reap receipt')
        if observed is not None:
            check(result == observed, 'same original observed/reaped terminal status')
        return result

    def _close_command(self, retained):
        # Every original resource is attempted; no cleanup error replaces the first.
        primary = None
        row = retained['row']
        child = retained['child']
        if child is not None and child.returncode is None:
            try:
                observed = self._h('held_status')(child)
                if row['observed_exit'] is None:
                    row['observed_exit'] = observed
                row['exit'] = self._stop_original(child, observed)
                row['reaped'] = child.returncode is not None
            except BaseException as error:
                primary = error
                self._mark(row, error, uncertain=True)
        elif child is not None:
            # Never adopt external poll/wait results as an original observation.
            if not row['reaped']:
                error = ValueError('original command already reaped without owned receipt')
                primary = error
                self._mark(row, error, uncertain=True)
        elif retained['spawn_attempted']:
            self._mark(row, ValueError('Popen failed without returned original handle'), uncertain=True)
        selector = retained['selector']
        if selector is not None and not retained['selector_closed']:
            try:
                self.remaining_cleanup()
                if retained['selector_has_fd']:
                    check(retained['selector_identity'] is not None
                          and self._pipe_key(selector) == retained['selector_identity'],
                          'same original selector FD before close')
                selector.close()
                retained['selector_closed'] = True
            except BaseException as error:
                primary = primary or error
                self._mark(row, error, uncertain=True)
        pipe = retained['pipe']
        if pipe is not None and not retained['pipe_closed']:
            try:
                self.remaining_cleanup()
                check(not pipe.closed and retained['pipe_identity'] is not None
                      and self._pipe_key(pipe) == retained['pipe_identity'],
                      'same retained command pipe before close')
                pipe.close()
                check(pipe.closed, 'retained command pipe closed')
                retained['pipe_closed'] = True
            except BaseException as error:
                primary = primary or error
                self._mark(row, error, uncertain=True)
        if retained['selector_attempted'] and selector is None:
            self._mark(row, ValueError('selector construction uncertain'), uncertain=True)
        row['fd_closed'] = retained['pipe_closed'] and retained['selector_closed']
        row['resources_closed'] = (not retained['spawn_attempted'] or child is not None and row['reaped']) and row['fd_closed'] and not row['uncertain']
        if primary is not None:
            raise primary

    def _run_owned(self, argv, timeout, limit, env, role, allowed):
        check(role in self.PUBLIC_ROLES and type(limit) is int and 0 < limit <= 131072,
              'fixed command role and bounded private capture')
        duration = self.remaining_work(timeout)
        check(not self.closed, 'PG adapter closed')
        row = {'role': role, 'status': 'running', 'observed_exit': None, 'exit': None,
               'reaped': False, 'fd_closed': False, 'resources_closed': False,
               'uncertain': False, 'errors': []}
        retained = {'row': row, 'child': None, 'pipe': None, 'pipe_identity': None,
                    'selector': None, 'selector_attempted': False, 'selector_closed': True,
                    'selector_has_fd': False, 'selector_identity': None,
                    'pipe_closed': True, 'spawn_attempted': False}
        self.commands.append(retained)  # Before any allocating/partially failing start.
        end = min(self.work_deadline(), time.monotonic() + duration)
        data = bytearray()
        primary = None
        try:
            self.remaining_work(1)
            retained['spawn_attempted'] = True
            child = subprocess.Popen([str(x) for x in argv], stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, close_fds=True,
                start_new_session=True, env=self._environment() if env is None else env)
            retained['child'] = child
            retained['pipe'] = child.stdout
            retained['pipe_closed'] = False
            check(child.stdout is not None, 'original returned command pipe')
            retained['pipe_identity'] = self._pipe_key(child.stdout)
            os.set_blocking(child.stdout.fileno(), False)
            retained['selector_attempted'] = True
            selector = selectors.DefaultSelector()
            retained['selector'] = selector
            retained['selector_closed'] = False
            if callable(getattr(selector, 'fileno', None)):
                retained['selector_has_fd'] = True
                retained['selector_identity'] = self._pipe_key(selector)
            selector.register(child.stdout, selectors.EVENT_READ)
            while selector.get_map():
                self.remaining_work(1)
                check(time.monotonic() < end, 'original command deadline')
                for key, _ in selector.select(min(.05, max(0, end - time.monotonic()))):
                    check(key.fileobj is child.stdout and self._pipe_key(child.stdout)
                          == retained['pipe_identity'], 'original selected command pipe')
                    block = os.read(key.fileobj.fileno(), min(4096, limit + 1 - len(data)))
                    if not block:
                        selector.unregister(key.fileobj)
                    else:
                        check(len(data) + len(block) <= limit, 'private command output bound')
                        data.extend(block)
            terminal = self._h('held_status')(child)
            while terminal is None:
                self.remaining_work(1)
                check(time.monotonic() < end, 'original terminal command deadline')
                time.sleep(min(.01, max(0, end - time.monotonic())))
                terminal = self._h('held_status')(child)
            row['observed_exit'] = terminal
            check(type(terminal) is int and terminal in allowed, 'original command exit rejected')
        except BaseException as error:
            primary = error
            self._mark(row, error)
        finally:
            try:
                self._close_command(retained)
            except BaseException as error:
                primary = primary or error
        if primary is None:
            try:
                self.remaining_work(1)
                check(row['resources_closed'] and row['exit'] == row['observed_exit'],
                      'original command receipt complete')
            except BaseException as error:
                primary = error
                self._mark(row, error)
        if primary is not None:
            row['status'] = 'failed'
            raise primary
        row['status'] = 'readiness-retry' if row['exit'] == 2 else 'passed'
        return bytes(data), retained

    def run_owned(self, argv, timeout, limit, env=None, role='private-command'):
        """Internal test adapter only; output is private, never exported or hashed."""
        check(role != 'readiness', 'readiness retry is reserved to fixed start probe')
        result, _ = self._run_owned(argv, timeout, limit, env, role, (0,))
        return result

    def live_owned(self):
        check(self.child is not None and self.child.returncode is None,
              'retained original unreaped PG handle')
        return self._h('held_status')(self.child)

    def _probe_argv(self):
        return [self.pg / 'psql', '-X', '-q', '-t', '-A', '-v', 'ON_ERROR_STOP=1',
                '-h', '127.0.0.1', '-p', str(self.port), '-U', 'postgres', '-d', 'postgres',
                '-c', "SHOW data_directory; SHOW server_version; SELECT '1';"]

    def start(self):
        check(not self.start_attempted and not self.closed and self.child is None,
              'one fresh PG start only')
        self.start_attempted = True
        self.private_root()
        check(self.pg.is_absolute() and self.pg.resolve(strict=True) == self.pg,
              'explicit canonical supplied PG binary directory')
        check(type(self.port) is int and 1024 <= self.port <= 65535, 'owned loopback port')
        check(not self.data.exists() and not self.data.is_symlink(), 'fresh owned PG data path')
        self.run_owned([self.pg / 'initdb', '--no-locale', '--encoding=UTF8', '--auth=trust',
                       '--username=postgres', '--pgdata', self.data], 20, 131072, role='initdb')
        (self.data / 'postgresql.conf').write_text(
            f"listen_addresses='127.0.0.1'\nport={self.port}\nunix_socket_directories='{self.work}'\n"
            "max_connections=20\nlog_min_messages=fatal\nssl=off\n")
        check(self.run_owned([self.pg / 'postgres', '--version'], 2, 512, role='version').strip()
              == b'postgres (PostgreSQL) 16.15', 'owned PG16.15 tool')
        self.remaining_work(1)
        self.pg_spawn_attempted = True
        try:
            self.child = subprocess.Popen([str(self.pg / 'postgres'), '-D', str(self.data)],
                stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                close_fds=True, start_new_session=True, env=self._environment())
        except BaseException:
            self.pg_spawn_uncertain = self.uncertain = self.failed = True
            raise
        ready_end = min(self.work_deadline(), time.monotonic() + 10)
        while True:
            self.remaining_work(1)
            check(time.monotonic() < ready_end and self.live_owned() is None,
                  'same original PG alive within readiness bound')
            result, retained = self._run_owned(self._probe_argv(),
                min(3, max(.001, ready_end - time.monotonic())), 8192, None, 'readiness', (0, 2))
            self.readiness.append(retained)
            row = retained['row']
            if row['exit'] == 2:
                check(time.monotonic() < ready_end and self.live_owned() is None,
                      'original PG retry within readiness bound')
                time.sleep(min(.05, max(0, ready_end - time.monotonic())))
                continue
            lines = result.decode().strip().splitlines()
            check(lines == [str(self.data), '16.15', '1'],
                  'original data-directory/version/SELECT1 identity')
            check(self.live_owned() is None, 'same original PG alive after identity query')
            self.private_root()
            self.ready = True
            return

    def normal_stop(self):
        left = self.remaining_cleanup()
        end = min(self.cleanup_deadline() - 2.05, time.monotonic() + 10)
        check(left > 2.05 and time.monotonic() < end, 'PG normal close within caller end')
        terminal = self.live_owned()
        check(terminal is None, 'PG must still be live before normal SIGINT')
        while True:
            check(self.live_owned() is None, 'original PG still live immediately before SIGINT')
            try:
                os.kill(self.child.pid, signal.SIGINT)
                self.normal_signal = 'SIGINT'
                break
            except PermissionError:
                check(self.live_owned() is None and time.monotonic() < end,
                      'original SIGINT permission deadline')
                time.sleep(min(.001, max(0, end - time.monotonic())))
        terminal = self.live_owned()
        while terminal is None:
            check(time.monotonic() < end, 'original PG normal SIGINT deadline')
            time.sleep(min(.02, max(0, end - time.monotonic())))
            terminal = self.live_owned()
        self.observed_exit = terminal
        check(type(terminal) is int and terminal == 0 and self.normal_signal == 'SIGINT',
              'original PG normal SIGINT observed0')
        self.exit = self._stop_original(self.child, terminal)
        check(self.exit == 0 and self.child.returncode == 0, 'original PG reaped0')

    def close(self):
        if self.closed:
            check(not self.close_errors and not self.uncertain, 'previous PG close uncertainty')
            return
        primary = None
        for retained in reversed(self.commands):
            try:
                self._close_command(retained)
            except BaseException as error:
                primary = primary or error
        if self.child is not None and self.child.returncode is None:
            try:
                self.normal_stop()
            except BaseException as error:
                primary = primary or error
                self.fallback = self.failed = True
                try:
                    check(self.child.returncode is None, 'same original PG for fallback')
                    observed = self._h('held_status')(self.child)
                    self.exit = self._stop_original(self.child, observed)
                except BaseException as cleanup_error:
                    self.uncertain = True
                    self.close_errors.append(type(cleanup_error).__name__)
        elif self.child is not None and not (self.normal_signal == 'SIGINT'
                and self.observed_exit == self.exit == self.child.returncode == 0):
            primary = primary or ValueError('PG reaped outside original normal close')
            self.uncertain = self.failed = True
        for retained in reversed(self.commands):
            try:
                self._close_command(retained)
            except BaseException as error:
                primary = primary or error
        try:
            if self.work_identity is not None:
                self.private_root()
            if self.start_attempted:
                check(self.child is not None and self.normal_signal == 'SIGINT'
                      and self.observed_exit == self.exit == self.child.returncode == 0,
                      'started PG requires original normal shutdown receipt')
            check(not self.pg_spawn_uncertain and not self.uncertain and not self.fallback
                  and not self.failed and not self._cancelled(), 'failed/uncertain PG cannot close successfully')
            check(all(x['row']['resources_closed'] for x in self.commands),
                  'every original command/FD closed')
        except BaseException as error:
            primary = primary or error
        if primary is not None:
            self.failed = True
            self.close_errors.append(type(primary).__name__)
            raise primary
        self.closed = True

    def assert_root_removable(self):
        check(self.closed and not self.failed and not self.uncertain and not self.fallback
              and not self.pg_spawn_uncertain, 'PG/root uncertainty retained')
        check(all(x['row']['resources_closed'] and not x['row']['uncertain'] for x in self.commands),
              'all original command and pipe authority settled')
        if self.start_attempted:
            check(self.ready and self.normal_signal == 'SIGINT'
                  and self.observed_exit == self.exit == self.child.returncode == 0,
                  'original ready normal PG receipt before root deletion')
        if self.work_identity is not None:
            self.private_root()
        return True

    def public_state(self):
        rows = [dict(x['row'], errors=list(x['row']['errors'])) for x in self.commands]
        return {'schema': 'zee-effect-owned-plaintext-pg-part-2',
                'profile': 'owned-loopback-trust-plaintext-only',
                'status': 'closed' if self.closed else 'failed' if self.failed else 'prepared',
                'started': self.start_attempted, 'ready_identity_verified': self.ready,
                'normal_signal': self.normal_signal, 'observed_exit': self.observed_exit,
                'exit': self.exit, 'reaped': self.child is not None and self.child.returncode is not None,
                'fallback': self.fallback, 'closed': self.closed, 'uncertain': self.uncertain,
                'cancelled': self._cancelled(), 'cleanup_errors': list(self.close_errors),
                'command_receipts': rows,
                'readiness_command_indexes': [self.commands.index(x) for x in self.readiness],
                'probe_original_owner_receipts_complete': self.ready and bool(self.readiness)
                    and all(x['row']['resources_closed'] and x['row']['reaped'] for x in self.readiness)}


if __name__ == '__main__':
    raise ValueError('DRAFT adapter part: no native/runtime entrypoint; integration review required')
