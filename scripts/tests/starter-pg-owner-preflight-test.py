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
CONSUMER_SOURCE = HERE / 'starter-content-native.py'
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


class StarterFixtureProtocolPreparation(unittest.TestCase):
    def setUp(self):
        ForegroundPostgresPreparation.setUp(self)
    def tearDown(self):
        ForegroundPostgresPreparation.tearDown(self)
    def environment(self):
        source = CONSUMER_SOURCE
        tree = ast.parse(public_source(source))
        nodes = [n for n in ast.walk(tree) if isinstance(n, (ast.ClassDef, ast.FunctionDef))
                 and n.name in ('StarterPgCampaign', 'close_work')]
        self.assertEqual({n.name for n in nodes}, {'StarterPgCampaign', 'close_work'})
        clock = Clock()
        env = {'campaign_cancelled': False, 'cleaning': False, 'campaign_deadline': 600,
               'H': {'CANCELLED': False}, 'time': clock, 'raw': [], 'proof': {}, 'host': None, 'cleanup_end': lambda: clock.now + 90, 'M': types.SimpleNamespace(sha=sha),
               'pgpins': {'postgres': 'p', 'initdb': 'i', 'psql': 's'}, 'Path': Path}
        class Work:
            def __init__(self):
                self.removed = False
                self.changed = False
                self.mode = stat.S_IFDIR | 0o700
            def stat(self):
                return self.lstat()
            def lstat(self):
                return types.SimpleNamespace(st_dev=1, st_ino=3 if self.changed else 2,
                    st_mode=self.mode, st_uid=os.geteuid(), st_gid=os.getegid())
            def resolve(self, strict=False):
                return self
            def exists(self):
                return not self.removed
        work = Work()
        env.update(work=work, work_identity=(1, 2, work.mode, os.geteuid(), os.getegid()),
                   a=types.SimpleNamespace(output=Path('/mock/out'), postgres_bin_dir=Path('/mock/bin')),
                   cluster=types.SimpleNamespace(closed=True, child=types.SimpleNamespace(returncode=0), probes=[]))
        def remove(path):
            self.assertIs(path, work)
            work.removed = True
        def private(info):
            self.assertTrue(stat.S_ISDIR(info.st_mode) and info.st_uid == os.geteuid()
                            and stat.S_IMODE(info.st_mode) == 0o700)
        env.update(shutil=types.SimpleNamespace(rmtree=remove), PG=types.SimpleNamespace(private_directory=private))
        exec(compile(ast.Module(body=nodes, type_ignores=[]), str(source), 'exec'), env)
        env['clock'] = clock
        return env

    def test_starter_work_closed_owners_allow_private_removal(self):
        e = self.environment()
        e['close_work']()
        self.assertFalse(e['work'].exists())

    def test_starter_work_uncertain_original_owner_preserves_private_root(self):
        for kind in ('closed-flag', 'postmaster', 'readiness', 'host'):
            with self.subTest(kind=kind):
                e = self.environment()
                if kind == 'closed-flag':
                    e['cluster'].closed = False
                elif kind == 'postmaster':
                    e['cluster'].child.returncode = None
                elif kind == 'host':
                    e['host'] = types.SimpleNamespace(returncode=None)
                else:
                    e['cluster'].probes = [{'child': types.SimpleNamespace(returncode=None)}]
                with self.assertRaises(ValueError):
                    e['close_work']()
                self.assertTrue(e['work'].exists())
                self.assertTrue(e['proof']['temporary_private_root_preserved_after_uncertain_owner'])

    def test_starter_work_identity_change_preserves_private_root(self):
        for kind in ('inode', 'mode'):
            with self.subTest(kind=kind):
                e = self.environment()
                if kind == 'inode':
                    e['work'].changed = True
                else:
                    e['work'].mode = stat.S_IFDIR | 0o755
                with self.assertRaises(ValueError):
                    e['close_work']()
                self.assertTrue(e['work'].exists())
                self.assertTrue(e['proof']['temporary_private_root_preserved_after_identity_change'])

    def test_starter_cleanup90_is_sticky_and_does_not_extend_acceptance600(self):
        e = self.environment()
        campaign = e['StarterPgCampaign'](e['work'])
        e['cleaning'] = True
        e['clock'].now = 700
        self.assertEqual(campaign.cleanup_deadline, 790)
        e['clock'].now = 750
        self.assertEqual(campaign.cleanup_deadline, 790)
        campaign.need(True)
        e['cleaning'] = False
        with self.assertRaises(ValueError):
            campaign.need(True)

    def test_starter_command_forwards_pinned_owner_and_rechecks_deadline(self):
        for budget, expected, late in ((15, 15, False), (15, 15, True), (60, 30, False), (60, 30, True)):
            with self.subTest(budget=budget, late=late):
                e = self.environment()
                campaign = e['StarterPgCampaign'](e['work'])
                calls = []
                def run(argv, **kwargs):
                    calls.append((argv, kwargs))
                    if late:
                        e['clock'].now = 600
                    return b'postgres (PostgreSQL) 16.15\n'
                e['H']['run'] = run
                args = [Path('/mock/bin/postgres'), '--version']
                if late:
                    with self.assertRaises(ValueError):
                        campaign.command('owned-PG-version', args, budget, {'LC_ALL': 'C'})
                else:
                    self.assertEqual(campaign.command('owned-PG-version', args, budget, {'LC_ALL': 'C'}),
                                     b'postgres (PostgreSQL) 16.15\n')
                self.assertEqual(calls, [(args, {'timeout': expected, 'limit': 65536, 'env': {'LC_ALL': 'C'}})])


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
    if args.target_sha256 is not None and SOURCE_SHA != args.target_sha256:
        parser.error('--target-sha256 does not match the public target')
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)
