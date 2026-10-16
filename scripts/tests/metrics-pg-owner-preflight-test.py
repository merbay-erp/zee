#!/usr/bin/env python3
"""Prepared-only deterministic PG-owner mocks; no native process or socket allowed."""
import ast
import argparse
import builtins
import hashlib
import json
import os
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import types
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
SOURCE = HERE / 'native-pg-owner.py'
LINEAGE_SOURCE = HERE / 'service-native-support.py'
LINEAGE_SHA = '2892a4f172cfbca9f7c31a267617c933ae7b8689d8e8901dd7e2318132dbe7a2'
SOURCE_SHA = None
METRICS_SOURCE_SHA = 'd5d12902358591365ff201c13c44b255d265c49c71befabcdef0575fe7e854c4'
CONSUMER_SOURCE = HERE / 'native-metrics-test.py'
sha = lambda raw: hashlib.sha256(raw).hexdigest()
dump = lambda node: ast.dump(node, include_attributes=False)


def reject_real(*args, **kwargs):
    raise AssertionError('real process/socket/signal boundary is forbidden in preparation mocks')


def public_source(path):
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or before.st_size > 65536:
        raise AssertionError('bounded public source only')
    raw = path.read_bytes()
    after = path.lstat()
    identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
    if len(raw) != before.st_size or identity(before) != identity(after):
        raise AssertionError('public source changed while reading')
    return raw


def selected(path, pin, names):
    if set(names) == {'Campaign'}:
        path, pin = LINEAGE_SOURCE, LINEAGE_SHA
    raw = public_source(path)
    if pin is not None and sha(raw) != pin:
        raise AssertionError('pinned public source changed')
    tree = ast.parse(raw, filename=str(path))
    nodes = [n for n in tree.body if isinstance(n, (ast.ClassDef, ast.FunctionDef)) and n.name in names]
    if {n.name for n in nodes} != set(names) or len(nodes) != len(names):
        raise AssertionError('exact AST owner selection')
    return ast.Module(body=nodes, type_ignores=[])


class Clock:
    def __init__(self):
        self.now = 100.0
    def monotonic(self):
        return self.now
    def sleep(self, seconds):
        if not 0 <= seconds <= 1:
            raise AssertionError('bounded fake sleep')
        self.now += seconds if seconds else .000001


class Child:
    def __init__(self, world, kind, terminal, stream):
        self.world, self.kind, self.terminal, self.stream = world, kind, terminal, stream
        self.pid = 900001 + len(world.children)
        self.returncode = None
        self.stop_failures = 0
        self.observation_advance = 0
    def poll(self):
        raise AssertionError('poll consumes original-child authority')
    def wait(self, *args, **kwargs):
        raise AssertionError('only fake retained owner may reap')
    communicate = wait
    kill = wait
    terminate = wait


class Owner:
    def __init__(self, world):
        self.w = world
        self.mapping = {'CANCELLED': False}
        self.CANCELLED = False
        self.stops = []
        self.observations = []
    @property
    def CANCELLED(self):
        return self.mapping['CANCELLED']
    @CANCELLED.setter
    def CANCELLED(self, value):
        self.mapping['CANCELLED'] = value
    def controlled_env(self):
        return {'PATH': '/mock-only', 'PGHOST': 'forbidden-external-default', 'LC_ALL': 'uncontrolled'}
    def held_status(self, child):
        if child not in self.w.children or child.returncode is not None:
            raise AssertionError('unknown or already reaped original child')
        self.observations.append(child)
        if child.observation_advance:
            self.w.clock.now += child.observation_advance
            child.observation_advance = 0
        return child.terminal
    def stop_owned(self, child, grace=3):
        if child not in self.w.children or child.returncode is not None or not 0 <= grace <= 3:
            raise AssertionError('wrong original object or cleanup grace')
        self.stops.append((child, grace))
        if child.stop_failures:
            child.stop_failures -= 1
            raise RuntimeError('controlled owned cleanup error')
        child.returncode = child.terminal if child.terminal is not None else -9
        return child.returncode


class CampaignFixture:
    def __init__(self, world):
        self.w = world
        self.out = world.out
        self.clusters = []
        self.resources = []
        self.commands = []
        self.units = {}
        self.links = []
        self.previous = {}
        self.proof = {'status': 'running'}
        self.finished = False
        self.stopped = False
        self.failed_body = False
        self.cleaning = False
        self.cleanup_deadline = None
        self.deadline = world.clock.now + 1200
        self.output_created = True
        s = world.directory.stat()
        self.tempdirs = [(world.directory, (s.st_dev, s.st_ino))]
        self.pg_pins = {str(world.binaries / n): sha((world.binaries / n).read_bytes())
                        for n in ('postgres', 'initdb', 'psql', 'pg_ctl')}
        self.cleanup_owner = world.owner
    def need(self, value):
        deadline = self.cleanup_deadline if self.cleaning else self.deadline
        if not value or (not self.cleaning and (self.stopped or self.w.owner.CANCELLED)) or self.w.clock.now >= deadline:
            raise ValueError('fake campaign boundary')
    def cancel(self):
        self.stopped = True
        self.w.owner.CANCELLED = True
    def command(self, label, argv, budget=25, env=None):
        self.need(True)
        args = [str(x) for x in argv]
        self.commands.append((label, args, budget, env))
        if label == 'owned-PG-initdb':
            if self.w.init_failure:
                raise RuntimeError('controlled initdb failure')
            expected = [str(self.w.binaries / 'initdb'), '--no-locale', '--encoding=UTF8',
                        '--auth=trust', '--username=postgres', '--pgdata', str(self.w.directory / 'data')]
            if args != expected or budget != 60:
                raise AssertionError('original initdb invocation changed')
            Path(args[-1]).mkdir(mode=0o700)
            return b'fixture initdb completed\n'
        if label == 'owned-PG-version':
            if args != [str(self.w.binaries / 'postgres'), '--version']:
                raise AssertionError('version argv')
            return b'postgres (PostgreSQL) 16.15\n'
        raise AssertionError('unexpected fake campaign command')
    def unlink(self):
        if self.units or self.links:
            raise AssertionError('these mocks never create actual systemd units')


class World:
    def __init__(self):
        self.temp = tempfile.TemporaryDirectory(prefix='pg-owner-mock-')
        self.root = Path(self.temp.name).resolve()
        self.out, self.directory, self.binaries = (self.root / n for n in ('out', 'fixture', 'bin'))
        for p in (self.out, self.directory, self.binaries):
            p.mkdir(mode=0o700)
        self.cert, self.key = self.directory / 'fake-cert', self.directory / 'fake-key'
        self.cert.write_bytes(b'NOT-A-CERTIFICATE\n')
        self.key.write_bytes(b'NOT-A-PRIVATE-KEY\n')
        for n in ('postgres', 'initdb', 'psql', 'pg_ctl'):
            (self.binaries / n).write_bytes(('non-executable mock ' + n).encode())
        self.clock = Clock()
        self.children = []
        self.signals = []
        self.socket_calls = []
        self.public_reads = []
        self.owner = Owner(self)
        self.init_failure = False
        self.spawn_failure = False
        self.cancel_after_spawn = False
        self.server_initial_exit = None
        self.server_sigint_exit = 0
        self.on_signal = None
        self.on_public_read = None
        self.plans = [{'exit': 0}]
        self.c = CampaignFixture(self)
        self.D = types.SimpleNamespace(snapshot=self.snapshot,
            T=types.SimpleNamespace(D=self.owner, private_directory=self.private_directory))
        namespace = self.namespace()
        source = selected(SOURCE, SOURCE_SHA, ('OwnedLocalCluster', 'bounded_bytes', 'private_directory', 'digest_file'))
        exec(compile(source, str(SOURCE), 'exec'), namespace)
        self.real_bounded = namespace['bounded_bytes']
        namespace['bounded_bytes'] = self.bounded
        self.Cluster = namespace['OwnedLocalCluster']
        self.owner.mapping.update(held_status=self.owner.held_status, stop_owned=self.owner.stop_owned, controlled_env=self.owner.controlled_env)
        self.cluster = self.Cluster(self.c, self.directory, self.cert, self.key, self.binaries, self.owner.mapping)
        self.c.clusters.append(self.cluster)
        self.namespace_dict = namespace
    def namespace(self):
        b = dict(vars(builtins))
        b['__import__'] = self.import_only_socket
        fake_os = types.SimpleNamespace(**{n: getattr(os, n) for n in (
            'fstat', 'open', 'read', 'close', 'geteuid', 'O_RDONLY', 'O_NOFOLLOW', 'O_NONBLOCK', 'O_CLOEXEC')})
        fake_os.kill = self.kill
        return {'__builtins__': b, '__name__': 'reviewed_owned_pg_mock', 'Path': Path,
                'D': self.D, 'time': self.clock, 'os': fake_os, 'stat': stat,
                'hashlib': hashlib, 'sys': types.SimpleNamespace(platform='linux'),
                'signal': types.SimpleNamespace(SIGINT=signal.SIGINT),
                'subprocess': types.SimpleNamespace(Popen=self.popen, DEVNULL=subprocess.DEVNULL, STDOUT=subprocess.STDOUT),
                'json': json, 'shutil': types.SimpleNamespace(rmtree=reject_real)}
    def import_only_socket(self, name, *args, **kwargs):
        if name != 'socket':
            raise AssertionError('candidate attempted an unexpected import')
        world = self
        class Listener:
            def __enter__(self):
                world.socket_calls.append(self)
                return self
            def __exit__(self, *exc):
                return False
            def bind(self, address):
                if address != ('127.0.0.1', 0):
                    raise AssertionError('non-loopback fake listener')
            def getsockname(self):
                return ('127.0.0.1', 54321)
        return types.SimpleNamespace(socket=Listener)
    def private_directory(self, s):
        if not stat.S_ISDIR(s.st_mode) or s.st_mode & 0o077:
            raise AssertionError('private mock directory')
    def snapshot(self, path):
        path = Path(path)
        if path.parent != self.binaries or path.name not in ('postgres', 'initdb', 'psql'):
            raise AssertionError('only public dummy executable hash permitted')
        return sha(path.read_bytes())
    def bounded(self, path, limit=262144):
        if Path(path).parent != self.out or Path(path).name not in (
                'postgres-server.log', *('postgres-ready-%03d.log' % n for n in range(1, 301))):
            raise AssertionError('only own public log may be read')
        self.public_reads.append(Path(path))
        raw = self.real_bounded(path, limit)
        if self.on_public_read:
            callback, self.on_public_read = self.on_public_read, None
            callback(Path(path))
        return raw
    def popen(self, argv, **kwargs):
        if not kwargs.get('start_new_session') or not kwargs.get('close_fds') or kwargs['stdin'] != subprocess.DEVNULL:
            raise AssertionError('owned Popen contract')
        if kwargs['cwd'] != self.out or kwargs['stderr'] != subprocess.STDOUT:
            raise AssertionError('bounded child working directory/output')
        if kwargs['env'].get('LC_ALL') != 'C' or any(k.startswith('PG') for k in kwargs['env']):
            raise AssertionError('isolated PostgreSQL child environment')
        kind = Path(argv[0]).name
        stream = kwargs['stdout']
        if kind == 'postgres':
            if self.spawn_failure:
                raise RuntimeError('controlled postgres Popen failure')
            if argv != [str(self.binaries / 'postgres'), '-D', str(self.directory / 'data')]:
                raise AssertionError('foreground postgres argv')
            child = Child(self, kind, self.server_initial_exit, stream)
            stream.write(b'database system is ready to accept connections\n')
            stream.flush()
        elif kind == 'psql':
            if not self.plans:
                raise AssertionError('unexpected additional readiness command')
            plan = self.plans.pop(0)
            expected = [str(self.binaries / 'psql'), '--no-psqlrc', '--host', str(self.directory),
                        '--port', '54321', '--username', 'postgres', '--dbname', 'postgres',
                        '--tuples-only', '--no-align', '--set', 'ON_ERROR_STOP=1', '--command',
                        'SHOW data_directory; SHOW server_version;']
            if argv != expected:
                raise AssertionError('exact private-socket SQL identity probe')
            child = Child(self, kind, plan.get('exit'), stream)
            child.stop_failures = plan.get('stop_failures', 0)
            child.observation_advance = plan.get('advance', 0)
            raw = plan.get('raw', (str(self.directory / 'data') + '\n16.15\n').encode()
                           if child.terminal == 0 else b'not ready\n')
            stream.write(raw)
            stream.flush()
            if plan.get('cancel'):
                self.c.cancel()
        else:
            raise AssertionError('no other process may be represented')
        self.children.append(child)
        if kind == 'postgres' and self.cancel_after_spawn:
            self.c.cancel()
        return child
    def kill(self, pid, number):
        candidates = [c for c in self.children if c.pid == pid and c.kind == 'postgres' and c.returncode is None]
        if len(candidates) != 1 or number != signal.SIGINT:
            raise AssertionError('signal must use the original live postmaster handle')
        child = candidates[0]
        self.signals.append((child, number))
        child.terminal = self.server_sigint_exit
        if child.terminal == 0:
            child.stream.write(b'database system is shut down\n')
            child.stream.flush()
        if self.on_signal:
            callback, self.on_signal = self.on_signal, None
            callback()
    def begin_cleanup(self, seconds=300):
        self.c.cleaning = True
        self.c.cleanup_deadline = self.clock.now + seconds
    def all_reaped(self):
        return all(c.returncode is not None for c in self.children)
    def dispose(self):
        # No fake child is reaped here: each test must observe its actual cleanup result.
        for child in self.children:
            if not child.stream.closed:
                child.stream.close()
        stream = self.cluster.stream
        if stream is not None and not stream.closed:
            stream.close()
        self.temp.cleanup()


class ForegroundPostgresPreparation(unittest.TestCase):
    def setUp(self):
        self.guard = ExitStack()
        for module, names in ((subprocess, ('Popen', 'run', 'call', 'check_call', 'check_output')),
                              (socket, ('socket', 'create_connection')),
                              (os, ('kill', 'killpg', 'system', 'fork', 'posix_spawn', 'posix_spawnp')),
                              (signal, ('signal', 'raise_signal'))):
            for name in names:
                if hasattr(module, name):
                    self.guard.enter_context(mock.patch.object(module, name, side_effect=reject_real))
        self.worlds = []
    def tearDown(self):
        unreaped = [child.kind for world in self.worlds for child in world.children if child.returncode is None]
        try:
            for world in reversed(self.worlds):
                world.dispose()
        finally:
            self.guard.close()
        self.assertEqual(unreaped, [], 'the tested method must reap every fake original child')
    def world(self):
        world = World()
        self.worlds.append(world)
        return world
    def cleanup(self, world, failed=False, seconds=300):
        world.begin_cleanup(seconds)
        if failed:
            with self.assertRaises(ValueError):
                world.cluster.close()
        else:
            world.cluster.close()
        self.assertTrue(world.all_reaped())

    def test_constructor_child_free(self):
        w = self.world()
        self.assertEqual((w.c.commands, w.children, w.socket_calls), ([], [], []))
        self.assertIsNone(w.cluster.child)
        self.cleanup(w)

    def test_not_registered(self):
        w = self.world()
        w.c.clusters.clear()
        with self.assertRaises(ValueError):
            w.cluster.start()
        self.assertEqual((w.c.commands, w.children), ([], []))
        self.cleanup(w)

    def test_initdb_failure(self):
        w = self.world()
        w.init_failure = True
        with self.assertRaises(RuntimeError):
            w.cluster.start()
        self.assertEqual(w.children, [])
        self.cleanup(w)
        self.assertEqual(w.cluster.row['status'], 'failed')

    def test_postgres_spawn_failure(self):
        w = self.world()
        w.spawn_failure = True
        with self.assertRaises(RuntimeError):
            w.cluster.start()
        self.cleanup(w)
        self.assertTrue(w.cluster.stream.closed)
        self.assertEqual(w.cluster.row['status'], 'failed')

    def test_cancel_at_spawn_return(self):
        w = self.world()
        w.cancel_after_spawn = True
        with self.assertRaises(ValueError):
            w.cluster.start()
        self.assertEqual([c.kind for c in w.children], ['postgres'])
        self.assertIs(w.cluster.child, w.children[0])
        self.cleanup(w)
        self.assertTrue(w.c.stopped and w.owner.CANCELLED)
        self.assertEqual(w.cluster.row['status'], 'failed')

    def test_ready_exit2_then_exact0(self):
        w = self.world()
        w.plans = [{'exit': 2}, {'exit': 0}]
        w.cluster.start()
        self.assertEqual([r['status'] for r in w.cluster.row['readiness']], ['readiness-retry', 'passed'])
        self.assertTrue(w.cluster.row['readiness_identity_verified'])
        self.assertTrue(all(r['reaped'] for r in w.cluster.row['readiness']))
        self.cleanup(w)
        self.assertTrue(w.cluster.row['normal_shutdown'])

    def test_wrong_ready_identity(self):
        for mismatch in ('data-directory', 'version'):
            with self.subTest(mismatch=mismatch):
                w = self.world()
                raw = (b'/wrong/data\n16.15\n' if mismatch == 'data-directory'
                       else (str(w.directory / 'data') + '\n17.0\n').encode())
                w.plans = [{'exit': 0, 'raw': raw}]
                with self.assertRaises(ValueError):
                    w.cluster.start()
                self.assertFalse(w.cluster.started)
                self.assertEqual(len(w.cluster.row['readiness']), 1)
                self.cleanup(w)

    def test_readiness_non2_error(self):
        for plan in ({'exit': 1}, {'exit': -9}, {'exit': None}, {'exit': 0, 'cancel': True},
                     {'exit': 0, 'raw': b'x' * (64 * 1024 + 1)}):
            with self.subTest(plan={k: len(v) if isinstance(v, bytes) else v for k, v in plan.items()}):
                w = self.world()
                w.plans = [plan]
                with self.assertRaises(ValueError):
                    w.cluster.start()
                self.assertFalse(w.cluster.started)
                self.assertNotEqual(w.cluster.row['readiness'][0]['status'], 'readiness-retry')
                self.cleanup(w)

    def test_ready_terminal_deadline(self):
        for boundary in ('held-status', 'public-result-read'):
            with self.subTest(boundary=boundary):
                w = self.world()
                if boundary == 'held-status':
                    w.plans = [{'exit': 0, 'advance': 31}]
                else:
                    w.on_public_read = lambda path: setattr(w.clock, 'now', w.clock.now + 31)
                with self.assertRaises(ValueError):
                    w.cluster.start()
                self.assertFalse(w.cluster.started)
                self.cleanup(w)

    def test_retained_psql_cleanup_error(self):
        w = self.world()
        w.plans = [{'exit': 0, 'stop_failures': 2}]
        with self.assertRaises(RuntimeError):
            w.cluster.start()
        probe = w.children[-1]
        self.assertIs(w.cluster.probes[0]['child'], probe)
        self.assertIsNone(probe.returncode)
        self.assertEqual([c for c, _ in w.owner.stops], [probe, probe])
        self.cleanup(w)
        self.assertIs(w.owner.stops[2][0], probe)
        self.assertEqual(w.cluster.row['status'], 'failed')

    def test_server_early_exit(self):
        for code in (0, 1):
            with self.subTest(code=code):
                w = self.world()
                w.server_initial_exit = code
                with self.assertRaises(ValueError):
                    w.cluster.start()
                self.cleanup(w, failed=True)
                self.assertFalse(w.cluster.row['normal_shutdown'])
                self.assertEqual(w.signals, [])

    def test_normal_sigint_zero_reap(self):
        w = self.world()
        w.cluster.start()
        original = w.cluster.child
        self.cleanup(w)
        self.assertEqual(w.signals, [(original, signal.SIGINT)])
        self.assertEqual((w.cluster.row['observed_exit'], w.cluster.row['exit']), (0, 0))
        self.assertTrue(w.cluster.row['normal_shutdown'] and w.cluster.row['reaped'])
        self.assertFalse(w.cluster.row['fallback_used'])
        before = (len(w.signals), len(w.owner.stops))
        w.cluster.close()
        self.assertEqual(before, (len(w.signals), len(w.owner.stops)))

    def test_shutdown_timeout_or_nonzero(self):
        for exit_code in (None, 1):
            with self.subTest(exit_code=exit_code):
                w = self.world()
                w.cluster.start()
                w.server_sigint_exit = exit_code
                self.cleanup(w, failed=True)
                self.assertTrue(w.cluster.row['fallback_used'])
                self.assertEqual(w.cluster.row['status'], 'failed')
                self.assertLessEqual(w.clock.now, 131)

    def test_exhausted_cleanup_budget(self):
        w = self.world()
        w.cluster.start()
        self.cleanup(w, failed=True, seconds=0)
        self.assertEqual(w.owner.stops[-1], (w.cluster.child, 0))
        self.assertEqual(w.signals, [])
        self.assertFalse(w.cluster.row['normal_shutdown'])

    def test_close_error_sticky(self):
        w = self.world()
        w.cluster.start()
        w.cluster.child.stop_failures = 1
        w.begin_cleanup()
        with self.assertRaises(ValueError):
            w.cluster.close()
        first = list(w.cluster.close_errors)
        with self.assertRaises(ValueError):
            w.cluster.close()
        self.assertTrue(w.all_reaped())
        with self.assertRaises(ValueError):
            w.cluster.close()
        self.assertEqual(w.cluster.close_errors[:len(first)], first)
        self.assertEqual(w.cluster.row['status'], 'failed')

    def test_late_cancel_close_publication(self):
        for boundary in ('SIGINT-callback', 'server-log-read', 'Campaign-proof-write'):
            with self.subTest(boundary=boundary):
                w = self.world()
                w.cluster.start()
                if boundary == 'SIGINT-callback':
                    w.on_signal = w.c.cancel
                    self.cleanup(w)
                elif boundary == 'server-log-read':
                    w.on_public_read = lambda path: w.c.cancel()
                    self.cleanup(w)
                else:
                    # Actual frozen Campaign.close/__exit__/need bodies, fake resource environment.
                    namespace = w.namespace()
                    tree = selected(SOURCE, SOURCE_SHA, ('Campaign',))
                    exec(compile(tree, str(SOURCE), 'exec'), namespace)
                    actual = namespace['Campaign']
                    w.c.need = types.MethodType(actual.need, w.c)
                    w.c.close = types.MethodType(actual.close, w.c)
                    w.c.finished = True
                    w.c.tempdirs = []  # This test covers publication, not real temporary deletion.
                    original_write = Path.write_text
                    fired = []
                    def write(path, data, *args, **kwargs):
                        result = original_write(path, data, *args, **kwargs)
                        if path == w.out / 'proof.json' and not fired:
                            fired.append(True)
                            w.c.cancel()
                        return result
                    with mock.patch.object(Path, 'write_text', write):
                        with self.assertRaises(ValueError):
                            actual.__exit__(w.c, None, None, None)
                    self.assertEqual(json.loads((w.out / 'proof.json').read_text())['status'], 'failed')
                    self.assertTrue(fired and w.all_reaped())
                self.assertTrue(w.c.stopped and w.owner.CANCELLED)
                if boundary != 'Campaign-proof-write':
                    self.assertEqual(w.cluster.row['status'], 'failed')

    def test_bounded_log_and_path(self):
        for variant in ('server-cap', 'log-inode', 'key-symlink'):
            with self.subTest(variant=variant):
                w = self.world()
                if variant == 'key-symlink':
                    w.key.unlink()
                    dummy = w.root / 'outside-dummy'
                    dummy.write_bytes(b'not-a-key')
                    w.key.symlink_to(dummy)
                    with self.assertRaises(ValueError):
                        w.cluster.start()
                    self.assertEqual(w.children, [])
                    self.cleanup(w)
                else:
                    w.cluster.start()
                    if variant == 'server-cap':
                        w.cluster.stream.truncate(w.cluster.LOG_LIMIT + 1)
                        w.cluster.stream.flush()
                    else:
                        w.cluster.log.rename(w.out / 'original-server-log')
                        w.cluster.log.write_bytes(b'replaced\n')
                    with self.assertRaises(ValueError):
                        w.cluster.active()
                    self.cleanup(w, failed=True)
                self.assertNotIn(w.key, w.public_reads)
                self.assertNotIn(w.cert, w.public_reads)

    def test_consumer_oracle_static(self):
        for name, count, pin in (
                ('dispatcher-service-native.py', 4, '00e17464f4d734311eff7a9deaf042f553c5b518c9f0b0e58f70e97e50c44192'),
                ('http-service-native.py', 7, '2f4bd558847c4938e5db819746bc677f193d6aa9f05222de10403ef3f5e6bed9')):
            with self.subTest(consumer=name):
                tree = ast.parse(public_source(LINEAGE_SOURCE.with_name(name)))
                def records(tree):
                    return [n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute)
                            and isinstance(n.func.value, ast.Name) and n.func.value.id == 'c' and n.func.attr == 'record']
                actual = records(tree)
                self.assertEqual(len(actual), count)
                self.assertEqual(sha('\n'.join(dump(n) for n in actual).encode()), pin)



class MetricsLifecyclePreparation(unittest.TestCase):
    """Selected actual definitions, memory owners only; never import main."""
    def setUp(self):
        self.guard=ExitStack()
        for module,names in ((subprocess,('Popen','run','call','check_call','check_output')),
            (socket,('socket','create_connection')),(os,('kill','killpg','fork','system','posix_spawn','posix_spawnp')),
            (signal,('signal','raise_signal'))):
            for name in names:
                if hasattr(module,name):self.guard.enter_context(mock.patch.object(module,name,side_effect=reject_real))
        self.clock=Clock()
        self.ns=dict(time=self.clock,sys=sys,types=types,json=json,hashlib=hashlib,re=__import__('re'),
            Path=Path,signal=signal,os=os,subprocess=subprocess,selectors=types.SimpleNamespace(),
            H={'CANCELLED':False},PG={},root=None,out=None,binary=None,proof={'cases':[]},
            deadline=700.,cleanup_limit=None,cleaning=False,cancelled=False,oldsignals={},
            children=[],child_records={},components=[],fds=[],channels=[],cleanup_errors=[],command_logs=[],
            work=None,work_identity=None,cluster=None,private_uncertain=False,started=100.)
        names=('need','active','cancel','install_handlers','cleanup_end','remaining','cleanup_all',
            'close_child','close_children','remember_fd','own_pipe','fd_row','close_fd','own_channel',
            'close_fds','register_component','close_component','close_components','close_pg','close_work',
            'public_inventory','publish','spawn_worker','command_capture','OwnedChild','MetricsRunning',
            'own_fixture','FixtureContext','PrivateLease','MetricsPgCampaign','pool_result','export_pool')
        exec(compile(selected(CONSUMER_SOURCE,None,names),str(CONSUMER_SOURCE),'exec'),self.ns)
        self.ns['EXPECTED_CASES']=['actual-pg-pool-four-leases-wait-timeout-recovery', 'native-request-status-rate429-db-boundary-no-content', 'actual-native-pipe-backpressure-recovery', 'actual-native-closed-reader-no-sigpipe', 'actual-native-malformed-source-alias-early-rejection', 'actual-native-malformed-source-stderr-alias-early-rejection', 'actual-two-worker-supervisor-collector', 'credential-designated-metadata-composition-two-workers']
        self.ns['POOL_SELECTOR']='postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar'
    def tearDown(self):self.guard.close()
    def good_publication(self):
        records={};work=types.SimpleNamespace(exists=lambda:False)
        class Output:
            def __init__(self,name=''):self.name=name
            def __truediv__(self,name):return Output(name)
            def write_text(self,text):records[self.name]=json.loads(text)
        self.ns.update(out=Output(),work=work,cluster=types.SimpleNamespace(closed=True),
            proof={'cases':[dict(name=name,status='passed') for name in self.ns['EXPECTED_CASES']],'body_completed':True,'postgres_clusters':[{'normal_shutdown':True,'fallback_used':False,'reaped':True}]})
        self.ns['public_inventory']=lambda:[]
        return records
    def test_successful_publication_real_pass_payload(self):
        records=self.good_publication()
        self.ns['publish'](None)
        self.assertEqual(records['proof.json']['status'],'passed')
        self.assertEqual(records['proof.json']['case_count'],8)
        self.assertTrue(records['proof.json']['private_root_removed'])
        self.assertEqual(records['proof.json']['cleanup_errors'],[])
    def test_all_cleanup_callbacks_success_order(self):
        seen=[]
        self.ns['cleanup_all'](*(lambda i=i:seen.append(i) for i in range(5)))
        self.assertEqual(seen,list(range(5)))
    def test_each_close_failure_still_attempts_every_callback(self):
        for bad in range(5):
            with self.subTest(bad=bad):
                seen=[];primary=RuntimeError('inert-first')
                def close(i):
                    seen.append(i)
                    if i==bad:raise primary
                with self.assertRaises(RuntimeError) as got:self.ns['cleanup_all'](*(lambda i=i:close(i) for i in range(5)))
                self.assertIs(got.exception,primary);self.assertEqual(seen,list(range(5)))
    def test_body_primary_identity_survives_cleanup_error(self):
        primary=KeyError('inert-body');seen=[]
        try:raise primary
        except KeyError:
            with self.assertRaises(KeyError) as got:self.ns['cleanup_all'](lambda:(_ for _ in ()).throw(ValueError()),lambda:seen.append('later'))
        self.assertIs(got.exception,primary);self.assertEqual(seen,['later'])
    def test_real_failed_proof_payload_not_pretend_success(self):
        records=self.good_publication();self.ns['cleanup_errors'].append('RuntimeError')
        with self.assertRaises(ValueError):self.ns['publish'](None)
        self.assertEqual(records['proof.json']['status'],'failed')
        self.assertEqual(records['proof.json']['cleanup_errors'],['RuntimeError'])
    def test_cancel_during_cleanup_records_sticky_ledger(self):
        self.ns['cleaning']=True;self.ns['cancel']()
        self.assertTrue(self.ns['cancelled']);self.assertFalse(self.ns['H']['CANCELLED'])
        records=self.good_publication()
        with self.assertRaises(ValueError):self.ns['publish'](None)
        self.assertTrue(records['proof.json']['cancelled'])
    def test_pg_close_masks_only_owner_cancel_and_restores_new_flag(self):
        self.ns['H']['CANCELLED']=True
        def close():self.ns['cancel']()
        self.ns.update(cleaning=True,cluster=types.SimpleNamespace(close=close))
        self.ns['close_pg']();self.assertTrue(self.ns['H']['CANCELLED']);self.assertTrue(self.ns['cancelled'])
    def test_cleanup_deadline_is_one_shot_and_inline_separate(self):
        self.assertEqual(self.ns['remaining'](10),10);self.assertIsNone(self.ns['cleanup_limit'])
        self.assertEqual(self.ns['cleanup_end'](),190.)
        self.clock.now=150.;self.assertEqual(self.ns['cleanup_end'](),190.)
    def test_partial_install_snapshots_before_mutator_all_restored(self):
        for index in range(3):
            with self.subTest(index=index):
                self.ns['oldsignals']={};numbers=[signal.SIGTERM,signal.SIGINT,signal.SIGHUP]
                previous={n:object() for n in numbers};current=dict(previous);events=[]
                def change(n,v):
                    events.append(n);current[n]=v
                    if len(events)==index+1:raise RuntimeError('install')
                self.ns['signal']=types.SimpleNamespace(SIGTERM=signal.SIGTERM,SIGINT=signal.SIGINT,SIGHUP=signal.SIGHUP,getsignal=lambda n:current[n],signal=change)
                with self.assertRaises(RuntimeError):self.ns['install_handlers']()
                self.assertEqual(set(self.ns['oldsignals']),set(numbers[:index+1]))
                self.assertTrue(all(self.ns['oldsignals'][n] is previous[n] for n in numbers[:index+1]))
                primary=RuntimeError('original-partial-install')
                records=self.good_publication();restores=[]
                def restore(n,handler):
                    restores.append(n);current[n]=handler
                self.ns['signal'].signal=restore
                with self.assertRaises(RuntimeError) as got:self.ns['publish'](primary)
                self.assertIs(got.exception,primary)
                self.assertEqual(restores,numbers[:index+1])
                self.assertTrue(all(current[n] is previous[n] for n in numbers[:index+1]))
                self.assertEqual(records['proof.json']['status'],'failed')
    def test_restore_attempts_all_even_first_restore_fails(self):
        records=self.good_publication();seen=[];self.ns['oldsignals']={1:'a',2:'b',3:'c'}
        def restore(n,h):
            seen.append(n)
            if n==1:raise RuntimeError()
        self.ns['signal']=types.SimpleNamespace(signal=restore)
        with self.assertRaises(RuntimeError):self.ns['publish'](None)
        self.assertEqual(seen,[1,2,3]);self.assertEqual(records['proof.json']['status'],'failed')
    def test_publication_cancel_cannot_retain_pass(self):
        records=self.good_publication()
        def inventory():self.ns['cancel']();return []
        self.ns['public_inventory']=inventory
        with self.assertRaises(ValueError):self.ns['publish'](None)
        self.assertEqual(records['proof.json']['status'],'failed')
    def test_restore_cancel_failed_only(self):
        records=self.good_publication();self.ns['oldsignals']={1:'a'}
        self.ns['signal']=types.SimpleNamespace(signal=lambda *_:self.ns['cancel']())
        with self.assertRaises(ValueError):self.ns['publish'](None)
        self.assertTrue(records['proof.json']['cancelled']);self.assertEqual(records['proof.json']['status'],'failed')
    def test_restore_deadline_failed_only(self):
        records=self.good_publication();self.ns['oldsignals']={1:'a'}
        self.ns['signal']=types.SimpleNamespace(signal=lambda *_:setattr(self.clock,'now',701.))
        with self.assertRaises(ValueError):self.ns['publish'](None)
        self.assertEqual(records['proof.json']['status'],'failed')
    def test_first_body_error_preserved_over_failed_write_and_restore(self):
        self.good_publication();primary=RuntimeError('body');seen=[]
        class Output:
            def __truediv__(self,n):return self
            def write_text(self,t):seen.append('write');raise ValueError('write')
        self.ns['out']=Output();self.ns['oldsignals']={1:'a',2:'b'}
        self.ns['signal']=types.SimpleNamespace(signal=lambda n,h:seen.append(n))
        with self.assertRaises(RuntimeError) as got:self.ns['publish'](primary)
        self.assertIs(got.exception,primary);self.assertIn(1,seen);self.assertIn(2,seen)
    def test_foreign_fd_identity_never_closed(self):
        closed=[];self.ns['fds']=[dict(fd=7,identity=(1,2,3),closed=False)]
        self.ns['os']=types.SimpleNamespace(fstat=lambda fd:types.SimpleNamespace(st_dev=9,st_ino=2,st_mode=3),close=closed.append)
        with self.assertRaises(ValueError):self.ns['close_fd'](7)
        self.assertEqual(closed,[]);self.assertFalse(self.ns['fds'][0]['closed'])
    def test_fd_close_error_keeps_registered_authority(self):
        self.ns['fds']=[dict(fd=7,identity=(1,2,3),closed=False)]
        self.ns['os']=types.SimpleNamespace(fstat=lambda fd:types.SimpleNamespace(st_dev=1,st_ino=2,st_mode=3),close=lambda _:(_ for _ in ()).throw(OSError()))
        with self.assertRaises(OSError):self.ns['close_fd'](7)
        self.assertFalse(self.ns['fds'][0]['closed'])
    def test_partial_pipe_setup_registers_both_before_mutation(self):
        self.ns['os']=types.SimpleNamespace(pipe=lambda:(7,8),fstat=lambda fd:types.SimpleNamespace(st_dev=1,st_ino=fd,st_mode=3),set_blocking=lambda *_:(_ for _ in ()).throw(RuntimeError()))
        with self.assertRaises(RuntimeError):self.ns['own_pipe']()
        self.assertEqual([r['fd'] for r in self.ns['fds']],[7,8])
        self.assertFalse(any(r['closed'] for r in self.ns['fds']))
        self.ns['fds']=[]
        def partial_stat(fd):
            if fd==7:raise OSError('first fstat')
            return types.SimpleNamespace(st_dev=1,st_ino=fd,st_mode=3)
        self.ns['os'].fstat=partial_stat
        with self.assertRaises(OSError):self.ns['own_pipe']()
        self.assertEqual([r['fd'] for r in self.ns['fds']],[7,8])
        self.assertIsNone(self.ns['fds'][0]['identity'])
        self.assertEqual(self.ns['fds'][1]['identity'],(1,8,3))
    def test_spawn_failure_does_not_adopt_or_invent_child(self):
        primary=RuntimeError('spawn');self.ns['subprocess']=types.SimpleNamespace(Popen=lambda *a,**k:(_ for _ in ()).throw(primary))
        with self.assertRaises(RuntimeError) as got:self.ns['spawn_worker']([],owner_role='consume')
        self.assertIs(got.exception,primary);self.assertEqual(self.ns['children'],[])
    def test_command_spawn_failure_original_error(self):
        primary=RuntimeError('spawn');self.ns['subprocess']=types.SimpleNamespace(Popen=lambda *a,**k:(_ for _ in ()).throw(primary),DEVNULL=-1,PIPE=-2)
        self.ns['H']['controlled_env']=lambda:{}
        with self.assertRaises(RuntimeError) as got:self.ns['command_capture']([],5)
        self.assertIs(got.exception,primary);self.assertEqual(self.ns['children'],[])
    def test_command_partial_selector_failure_reaps_original(self):
        primary=RuntimeError('selector');events=[];stream=types.SimpleNamespace(close=lambda:events.append('close'))
        child=types.SimpleNamespace(returncode=None,stdin=None,stdout=stream,stderr=stream)
        self.ns['subprocess']=types.SimpleNamespace(Popen=lambda *a,**k:child,DEVNULL=-1,PIPE=-2)
        self.ns['selectors']=types.SimpleNamespace(DefaultSelector=lambda:(_ for _ in ()).throw(primary))
        self.ns['H'].update(controlled_env=lambda:{},stop_owned=lambda c,grace:setattr(c,'returncode',0))
        with self.assertRaises(RuntimeError) as got:self.ns['command_capture']([],5)
        self.assertIs(got.exception,primary);self.assertIs(self.ns['children'][0],child);self.assertEqual(child.returncode,0)
    def test_worker_wait_holds_original_status_then_reaps_once_without_poll(self):
        events=[]
        stream=types.SimpleNamespace(close=lambda:events.append('close'))
        child=types.SimpleNamespace(returncode=None,stdin=None,stdout=stream,stderr=stream,poll=reject_real)
        self.ns['H']['held_status']=lambda c:events.append('held') or 2
        def stop(c,grace):events.append('stop');c.returncode=2
        self.ns['H']['stop_owned']=stop
        self.ns['child_records'][id(child)]={'role':'alias','exit':None,'reaped':False}
        owned=self.ns['OwnedChild'](child)
        self.assertEqual(owned.wait(10),2);self.assertEqual(owned.wait(10),2)
        self.assertEqual(events,['held','stop'])
        self.assertEqual(self.ns['child_records'][id(child)]['observed_exit'],2)
        self.ns['close_child'](child);self.assertEqual(events.count('stop'),1)
        self.assertEqual(events.count('close'),2)
    def test_each_pipe_close_attempted_after_owner_error(self):
        events=[];child=types.SimpleNamespace(returncode=None)
        child.stdin=types.SimpleNamespace(close=lambda:events.append('in'));child.stdout=types.SimpleNamespace(close=lambda:events.append('out'));child.stderr=types.SimpleNamespace(close=lambda:events.append('err'))
        self.ns['H']['stop_owned']=lambda *a,**k:(_ for _ in ()).throw(RuntimeError())
        with self.assertRaises(RuntimeError):self.ns['close_child'](child)
        self.assertEqual(events,['in','out','err']);self.assertIsNone(child.returncode)
    def test_channel_close_failure_retains_fd_row(self):
        self.ns['fds']=[dict(fd=7,identity=(1,2,3),closed=False)]
        self.ns['close_fd']=lambda _:(_ for _ in ()).throw(OSError())
        class Channel:
            def __init__(self,fd):self.fd=fd
        channel=self.ns['own_channel']({'Channel':Channel},7)
        with self.assertRaises(OSError):channel.close()
        self.assertEqual(channel.fd,7);self.assertFalse(self.ns['fds'][0]['closed'])
    def test_actual_Channel_EOF_closes_registry_once(self):
        closed=[];self.ns['fds']=[dict(fd=7,identity=(1,2,3),closed=False)]
        fakeos=types.SimpleNamespace(read=lambda *_:b'',fstat=lambda fd:types.SimpleNamespace(st_dev=1,st_ino=2,st_mode=3),close=closed.append)
        self.ns['os']=fakeos
        api_path=CONSUMER_SOURCE.parents[1]/'native-metrics.py'
        api={'os':fakeos,'time':self.clock}
        exec(compile(selected(api_path,METRICS_SOURCE_SHA,['Channel']),str(api_path),'exec'),api)
        channel=self.ns['own_channel'](api,7)
        channel.receive()
        self.assertIsNone(channel.fd);self.assertTrue(self.ns['fds'][0]['closed'])
        self.ns['close_fds']();channel.close();self.assertEqual(closed,[7])
    def test_partial_Channel_constructor_retains_original_raw_fd(self):
        closed=[];self.ns['fds']=[dict(fd=7,identity=(1,2,3),closed=False)]
        self.ns['os']=types.SimpleNamespace(fstat=lambda fd:types.SimpleNamespace(st_dev=1,st_ino=2,st_mode=3),close=closed.append)
        class Channel:
            def __init__(self,fd):raise RuntimeError('constructor')
        with self.assertRaises(RuntimeError):self.ns['own_channel']({'Channel':Channel},7)
        self.assertFalse(self.ns['fds'][0]['closed']);self.ns['close_fds']()
        self.assertEqual(closed,[7]);self.assertTrue(self.ns['fds'][0]['closed'])
    def test_pool_bad_stderr_rejected_before_any_public_write(self):
        rows=[[4,0,4,1,0,0,0,0],[4,4,0,0,0,1,0,1]]
        raw=('running 1 test\ntest '+self.ns['POOL_SELECTOR']+' ... K257_OWNED_POOL_METRICS '+json.dumps(rows)+'\nok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 2.00s\n').encode()
        self.ns.update(binary=Path('/public/compiler/target/release/dil'),root=Path('/public'),out=None)
        for bad in (b'panic: private',b'postgresql://secret',b'sslmode=require',b'env=private',b'BEGIN PRIVATE KEY'):
            with self.subTest(stderr=bad):
                with self.assertRaises(ValueError):self.ns['export_pool'](types.SimpleNamespace(stdout=raw,stderr=bad))
        self.assertEqual(self.ns['command_logs'],[])
    def test_partial_fixture_constructor_registered_before_error(self):
        primary=RuntimeError('fixture-init')
        class Fixture:
            def __init__(self,**kwargs):raise RuntimeError('fixture-init')
        # Actual function globals are explicitly supplied; no module monkeypatch.
        self.ns['binary']=Path('/public/dummy')
        with self.assertRaises(RuntimeError):self.ns['own_fixture']({'Fixture':Fixture,'listening':lambda _:False},native=True)
        self.assertEqual(len(self.ns['components']),1);self.assertFalse(self.ns['components'][0]['closed'])
    def test_credential_context_body_primary_survives_late_close_failure(self):
        primary=RuntimeError('body');events=[]
        def close():events.append('credential-close');raise ValueError('close')
        context=self.ns['FixtureContext'](types.SimpleNamespace(close=close))
        with self.assertRaises(RuntimeError) as got:
            with context:raise primary
        self.assertIs(got.exception,primary);self.assertEqual(events,['credential-close'])
    def test_private_root_retained_for_each_uncertain_owner(self):
        for key,value in [('private_uncertain',True),('cleanup_errors',['OSError']),('children',[types.SimpleNamespace(returncode=None)]),('components',[dict(closed=False)]),('fds',[dict(closed=False)]),('cluster',types.SimpleNamespace(closed=False,probes=[]))]:
            with self.subTest(owner=key):
                old=self.ns[key];self.ns[key]=value;self.ns['work']=types.SimpleNamespace()
                with self.assertRaises(ValueError):self.ns['close_work']()
                self.assertTrue(self.ns['proof']['private_root_preserved_after_uncertain_owner'])
                self.ns[key]=old
    def test_private_root_identity_deadline_and_happy_remove(self):
        for mode in ('happy','identity','deadline'):
            with self.subTest(mode=mode):
                removed=[];s=types.SimpleNamespace(st_dev=1,st_ino=2,st_mode=3,st_uid=4,st_gid=5)
                class Work:
                    def lstat(self):return s
                    def resolve(self,strict):return self
                    def exists(self):return not removed
                work=Work();self.ns.update(work=work,work_identity=(1,2 if mode!='identity' else 9,3,4,5),cleanup_limit=99 if mode=='deadline' else 190)
                self.ns['PG']={'private_directory':lambda s:None};self.ns['shutil']=types.SimpleNamespace(rmtree=lambda w:removed.append(w))
                if mode=='happy':self.ns['close_work']();self.assertEqual(removed,[work])
                else:
                    with self.assertRaises(ValueError):self.ns['close_work']()
                    self.assertEqual(removed,[])
    def test_pool_single_structured_interleave_and_typed_negatives(self):
        selector=self.ns['POOL_SELECTOR'];rows=[[4,0,4,1,0,0,0,0],[4,4,0,0,0,1,0,1]]
        marker='K257_OWNED_POOL_METRICS '+json.dumps(rows)
        summary='test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 2.00s'
        raw=('running 1 test\ntest '+selector+' ... '+marker+'\nok\n'+summary+'\n').encode()
        self.assertEqual(self.ns['pool_result'](raw),rows)
        rows64=rows*32
        self.assertEqual(self.ns['pool_result'](raw.replace(json.dumps(rows).encode(),json.dumps(rows64).encode())),rows64)
        with self.assertRaises(ValueError):
            self.ns['pool_result'](raw.replace(json.dumps(rows).encode(),json.dumps(rows64+[rows[0]]).encode()))
        for bad in [raw+raw,raw.replace(b'1 passed',b'0 passed'),raw.replace(b'0 failed',b'1 failed'),raw.replace(b'[[4,',b'[[true,'),raw.replace(selector.encode(),b'wrong_selector'),raw.replace(b'\nok\n',b'\nanything ok\n')]:
            with self.subTest(bad=bad[:30]):
                with self.assertRaises(ValueError):self.ns['pool_result'](bad)

    def test_original29_require_and_eight_case_append_AST_preserved(self):
        tree=ast.parse(public_source(CONSUMER_SOURCE))
        requires=sorted(dump(n) for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Name) and n.func.id=='require')
        self.assertEqual(len(requires),29)
        self.assertEqual(sha('\n'.join(requires).encode()),'a94b094baedcdbc3b5f3b4280ad1c30beaf4c3480a0a9fd8f1c0cb0673f4eceb')
        cases=[dump(n) for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr=='append' and isinstance(n.func.value,ast.Subscript) and isinstance(n.func.value.value,ast.Name) and n.func.value.value.id=='proof' and isinstance(n.func.value.slice,ast.Constant) and n.func.value.slice.value=='cases']
        self.assertEqual(len(cases),7) # alias loop expands its site twice ->8 observations
        self.assertEqual(sha('\n'.join(cases).encode()),'d006ff729cf3b6a7f8e3a7e2ee827fedfe8b2a2a224a710fbfb36b572b2ae811')
    def test_metrics_PG_adapter_inline_vs_once_terminal_budget(self):
        directory=types.SimpleNamespace(lstat=lambda:types.SimpleNamespace(st_dev=1,st_ino=2))
        self.ns['PG']={'digest_file':lambda _: 'a'*64}
        campaign=self.ns['MetricsPgCampaign'](directory,Path('/public/bin'))
        self.assertIsNone(self.ns['cleanup_limit']);self.assertEqual(campaign.cleanup_deadline,700.)
        self.ns['H']['CANCELLED']=True;self.assertTrue(campaign.stopped)
        self.ns['cleaning']=True;self.assertEqual(campaign.cleanup_deadline,190.)
        self.clock.now=150.;self.assertEqual(campaign.cleanup_deadline,190.)
    def test_explicit_public_inventory_never_scans_private_material(self):
        reads=[]
        class Named:
            def __init__(self,name):self.name=name
            def exists(self):return True
        class Output:
            def __truediv__(self,name):return Named(name)
        def bounded(path,limit):
            reads.append(path.name)
            if path.name in ('server.key','ca.crt','password','request'):raise AssertionError('private')
            return b'public'
        self.ns.update(out=Output(),PG={'bounded_bytes':bounded},command_logs=[dict(name='command-001-stdout.log')],cluster=types.SimpleNamespace(closed=True,log='postgres-server.log',row={'readiness':[dict(log='postgres-ready-001.log',sha256='a'*64)]}))
        rows=self.ns['public_inventory']()
        self.assertEqual(reads,['command-001-stdout.log','native-metadata.jsonl','postgres-server.log','postgres-ready-001.log'])
        self.assertEqual([r['path'] for r in rows],reads)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', type=Path, default=SOURCE)
    parser.add_argument('--target-sha256')
    parser.add_argument('--consumer-source', type=Path, default=CONSUMER_SOURCE)
    parser.add_argument('--lineage-source', type=Path, default=LINEAGE_SOURCE)
    args, remaining = parser.parse_known_args()
    SOURCE = args.target.resolve(strict=True)
    CONSUMER_SOURCE = args.consumer_source.resolve(strict=True)
    LINEAGE_SOURCE = args.lineage_source.resolve(strict=True)
    if sha(public_source(LINEAGE_SOURCE)) != LINEAGE_SHA:
        parser.error('--lineage-source does not match frozen K306 service owner')
    if SOURCE.name != 'native-pg-owner.py':
        parser.error('--target must name the reviewed public native-pg-owner.py')
    SOURCE_SHA = sha(public_source(SOURCE))
    if SOURCE_SHA != '6e5f004a99af5685bcd9625b73df2ae574f21854f0b0bff06dc8013c18b4d9c2':
        parser.error('unchanged shared PG owner6e5 required')
    if args.target_sha256 is not None and SOURCE_SHA != args.target_sha256:
        parser.error('--target-sha256 does not match the public target')
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)
