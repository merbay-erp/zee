"""K271 short preparation: real owned process/TCP checks are not native acceptance."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock

FILES = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('kit', FILES / 'scripts/owned-web-tests.py')
K = importlib.util.module_from_spec(spec)
spec.loader.exec_module(K)
spec = importlib.util.spec_from_file_location('owner', FILES / 'scripts/worker-supervisor.py')
OWNER = importlib.util.module_from_spec(spec)
spec.loader.exec_module(OWNER)


class Contract(unittest.TestCase):
    def setUp(self):
        K.CANCELLED, K.END = False, float('inf')
        self.temp = tempfile.TemporaryDirectory()
        self.path = Path(self.temp.name).resolve()
        self.root = self.path / 'fixtures'
        shutil.copytree(FILES / 'compiler/tests/fixtures/owned-web-kit', self.root)
        self.suite = json.loads((FILES / 'compiler/tests/fixtures/owned-web-kit-suite.json').read_text())

    def tearDown(self):
        K.CANCELLED, K.END = False, float('inf')
        self.temp.cleanup()

    def reject(self, value):
        with self.assertRaises(K.Rejected):
            K.validate(self.root, value)

    def test_valid_checked_in_suite_is_nonempty(self):
        self.assertEqual(len(K.validate(self.root, self.suite)['cases']), 4)

    def test_manifest_cannot_select_process_shell_or_dsn(self):
        for key in ('argv', 'shell', 'executable', 'database_url'):
            with self.subTest(key=key):
                value = copy.deepcopy(self.suite)
                value['cases'][0][key] = 'PRIVATE_CANARY'
                self.reject(value)

    def test_unknown_top_level_key_rejected(self):
        self.suite['executable'] = '/bin/sh'
        self.reject(self.suite)
        actual = subprocess.run([sys.executable, '-I', '-B', str(FILES / 'scripts/owned-web-tests.py'), '--unknown', 'PRIVATE_DSN_CANARY'], capture_output=True, timeout=3)
        self.assertEqual(actual.returncode, 2)
        self.assertEqual(actual.stdout, b'')
        self.assertNotIn(b'CANARY', actual.stderr)
        self.assertEqual(json.loads(actual.stderr)['type'], 'Rejected')

    def test_duplicate_json_key_is_rejected(self):
        p = self.path / 'duplicate.json'
        p.write_text('{"schema":"a", "schema":"b"}')
        with self.assertRaises(K.Rejected):
            K.load(p)

    def test_json_size_bound_before_parse(self):
        p = self.path / 'large.json'
        p.write_bytes(b' ' * 65537)
        with self.assertRaises(K.Rejected):
            K.load(p)
        # Real CLI guard: an audit hook makes ANY open of the hostile suite
        # exit71. The actual role/size rejection must return2 before that open.
        fifo = self.path / 'suite.fifo'
        os.mkfifo(fifo, 0o600)
        for hostile in (p, fifo):
            with self.subTest(role=hostile.name):
                bootstrap = 'import os,runpy,sys; tool=sys.argv[1]; target=sys.argv[2]; sys.argv=[tool]+sys.argv[3:]\n'
                bootstrap += 'def audit(event,args):\n if event=="open" and isinstance(args[0],str) and args[0]==target:os._exit(71)\n'
                bootstrap += 'sys.addaudithook(audit);runpy.run_path(tool,run_name="__main__")'
                output = self.path / ('never-created-' + hostile.name)
                arguments = ['--fixture-root', str(self.root), '--suite', str(hostile), '--suite-sha256', '0'*64,
                             '--dil', sys.executable, '--binary-sha256', '0'*64, '--expected-source', '1'*40,
                             '--postgres-bin-dir', str(self.path), '--output', str(output)]
                actual = subprocess.run([sys.executable, '-I', '-B', '-c', bootstrap, str(FILES/'scripts/owned-web-tests.py'), str(hostile), *arguments], capture_output=True, timeout=3)
                self.assertEqual(actual.returncode, 2)
                self.assertEqual(actual.stdout, b'')
                self.assertEqual(json.loads(actual.stderr)['type'], 'Rejected')
                self.assertFalse(output.exists())

    def test_changed_fixture_hash_rejected(self):
        (self.root / 'site/ana.dil').write_text('changed')
        self.reject(self.suite)

    def test_parent_escape_and_absolute_inventory_rejected(self):
        for name in ('../outside', '/tmp/outside'):
            with self.subTest(name=name):
                value = copy.deepcopy(self.suite)
                value['inventory'][name] = '0' * 64
                self.reject(value)

    def test_inventory_symlink_rejected(self):
        target = self.root / 'site/ana.dil'
        target.unlink()
        target.symlink_to(self.root / 'panel/ana.dil')
        self.reject(self.suite)

    def test_unpinned_project_file_rejected(self):
        (self.root / 'site/unpinned.dil').write_text('unexpected')
        self.reject(self.suite)

    def test_snapshot_requires_exact_inventory_pin(self):
        self.suite['cases'][0]['steps'][0]['expect']['snapshot'] = 'missing.snapshot'
        self.reject(self.suite)

    def test_forward_csrf_reference_rejected(self):
        self.suite['cases'][1]['steps'][0]['form'] = {'_csrf': {'csrf': 'not_captured'}}
        self.reject(self.suite)

    def test_case_and_step_bounds_reject_before_process(self):
        value = copy.deepcopy(self.suite)
        value['cases'] *= 3
        self.reject(value)
        value = copy.deepcopy(self.suite)
        value['cases'][0]['steps'] *= 33
        self.reject(value)

    def test_sql_psql_meta_and_observer_mutation_rejected(self):
        for name, content in [('db-bootstrap.sql', '\\! echo PRIVATE_CANARY'), ('db-observers.json', '{"total":"DELETE FROM owned_notes"}')]:
            with self.subTest(name=name):
                value = copy.deepcopy(self.suite)
                target = self.root / name
                previous = target.read_bytes()
                target.write_text(content)
                value['inventory'][name] = K.sha(target)
                self.reject(value)
                target.write_bytes(previous)

    def test_csrf_html_requires_bounded_hex(self):
        parser = K.HiddenToken()
        with self.assertRaises(K.Rejected):
            parser.feed('<input name=_csrf value="PRIVATE_CANARY">')

    def finalization_with_mocked_native(self, phase=None, action=None):
        # Execute the real main/finally path; no child, PG or socket is allowed.
        label = 'normal' if phase is None else phase + '-' + action
        fixture = self.path / label
        fixture.mkdir()
        binary = fixture / 'dil'
        binary.write_bytes(b'fixture-only binary; never execute\n')
        pg = fixture / 'pg'
        pg.mkdir()
        for name in ('initdb', 'postgres', 'psql'):
            (pg / name).write_bytes(b'fixture-only runtime; never execute\n')
        suite = FILES / 'compiler/tests/fixtures/owned-web-kit-suite.json'
        output = fixture / 'output'
        build = {'schema': 'zee-build-v1', 'source_sha': '1' * 40, 'dirty': False}
        events, temporary_paths = [], []
        triggered = False
        real_temporary_directory = tempfile.TemporaryDirectory
        real_write_text = Path.write_text
        real_need = K.need

        def interrupt(at):
            nonlocal triggered
            events.append(at)
            if at == phase and not triggered:
                triggered = True
                if action == 'cancel':
                    K.CANCELLED = True
                else:
                    self.assertEqual(action, 'deadline')
                    K.END = time.monotonic() - 1

        class Temporary:
            def __init__(self, *args, **kwargs):
                kwargs['dir'] = fixture
                self.actual = real_temporary_directory(*args, **kwargs)
                temporary_paths.append(Path(self.actual.name))

            def __enter__(self):
                return self.actual.__enter__()

            def __exit__(self, *args):
                result = self.actual.__exit__(*args)
                interrupt('temporary_directory_exit')
                return result

        class CommandOwner:
            children = ()

            def cleanup(self):
                interrupt('command_owner_cleanup')

        class MockKit:
            def __init__(self, owner, binary, pg, work, pins):
                self.command_owner = CommandOwner()
                self.proof = None
                self.coverage_output = None

            def command(self, argv, **kwargs):
                events.append('command')
                if argv[0] == binary:
                    self_outer.assertEqual(argv, [binary, 'sürüm', '--json'])
                    return json.dumps(build).encode()
                self_outer.assertEqual(argv, [pg / 'postgres', '--version'])
                return b'postgres (PostgreSQL) 16.15\n'

            def case(self, root, case, index, actual_build, proof):
                self_outer.assertEqual(actual_build, build)
                events.append('case')
                proof['cleanup'].append({'id': case['id'], 'worker_groups_empty': True, 'pg_groups_empty': True})
                return {'id': case['id'], 'passed': True,
                        'cluster_system_identifier': str(index) if 'database' in case else None,
                        'observer_readonly': 'database' in case}

        def write_text(path, text, *args, **kwargs):
            result = real_write_text(path, text, *args, **kwargs)
            if path == output / 'proof.json':
                interrupt('proof_write')
            return result

        def final_need(value, reason='contract'):
            if reason == 'acceptance_failed':
                interrupt('final_need')
            return real_need(value, reason)

        self_outer = self
        argv = ['--fixture-root', str(self.root), '--suite', str(suite),
                '--suite-sha256', K.sha(suite), '--dil', str(binary),
                '--binary-sha256', K.sha(binary), '--expected-source', build['source_sha'],
                '--postgres-bin-dir', str(pg), '--output', str(output)]
        rejected = None
        with mock.patch.object(K, 'Kit', MockKit), \
                mock.patch.object(K.tempfile, 'TemporaryDirectory', Temporary), \
                mock.patch.object(K.signal, 'signal') as handlers, \
                mock.patch.object(K, 'need', final_need), \
                mock.patch.object(K.subprocess, 'Popen', side_effect=AssertionError('no child')) as popen, \
                mock.patch.object(K.socket, 'socket', side_effect=AssertionError('no socket')) as socket_call, \
                mock.patch.object(K.socket, 'create_connection', side_effect=AssertionError('no connection')) as connection, \
                mock.patch.object(Path, 'write_text', write_text):
            try:
                K.main(argv)
            except K.Rejected as error:
                rejected = error
        self.assertEqual(handlers.call_count, 2)
        popen.assert_not_called()
        socket_call.assert_not_called()
        connection.assert_not_called()
        self.assertEqual(events.count('case'), 4)
        self.assertEqual(events.count('command_owner_cleanup'), 1)
        self.assertTrue(temporary_paths)
        self.assertTrue(all(not path.exists() for path in temporary_paths))
        self.assertEqual(triggered, phase is not None)
        proof = json.loads((output / 'proof.json').read_text())
        self.assertTrue(proof['owned_cleanup'])
        self.assertTrue(proof['command_groups_empty'])
        return rejected, proof

    def test_finalization_cancel_or_deadline_never_passes(self):
        for phase in ('temporary_directory_exit', 'command_owner_cleanup', 'proof_write', 'final_need'):
            for action in ('cancel', 'deadline'):
                with self.subTest(phase=phase, action=action):
                    # main resets its global guard; argv preparation must also start clean.
                    K.CANCELLED, K.END = False, float('inf')
                    rejected, proof = self.finalization_with_mocked_native(phase, action)
                    self.assertEqual(proof['status'], 'failed')
                    self.assertIsInstance(rejected, K.Rejected)
                    self.assertEqual(proof['reason'], 'cancelled_or_deadline')

    def test_finalization_without_cancel_keeps_passed(self):
        rejected, proof = self.finalization_with_mocked_native()
        self.assertIsNone(rejected)
        self.assertEqual(proof['status'], 'passed')
        self.assertEqual(len(proof['cases']), 4)

    def kit(self):
        return K.Kit(OWNER, Path(sys.executable), self.path, self.path, {})

    def test_actual_owned_capture_overflow_reaps(self):
        kit = self.kit()
        with self.assertRaises(K.Rejected):
            kit.command([sys.executable, '-c', 'import sys;sys.stdout.write("x"*10000)'], limit=32)
        self.assertFalse(kit.command_owner.children)

    def test_actual_owned_timeout_reaps(self):
        kit = self.kit()
        with self.assertRaises(K.Rejected):
            kit.command([sys.executable, '-c', 'import time;time.sleep(30)'], timeout=.05)
        self.assertFalse(kit.command_owner.children)

    def test_actual_owned_cancel_flag_reaps(self):
        kit = self.kit()
        thread = threading.Thread(target=lambda: (time.sleep(.05), setattr(K, 'CANCELLED', True)))
        thread.start()
        try:
            with self.assertRaises(K.Rejected):
                kit.command([sys.executable, '-c', 'import time;time.sleep(30)'])
        finally:
            thread.join()
        self.assertFalse(kit.command_owner.children)

    def test_actual_tcp_duplicate_or_truncated_frame_rejected(self):
        for response in (b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok',
                         b'HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nok'):
            with self.subTest(response_length=len(response)), socket.socket() as server:
                server.bind(('127.0.0.1', 0))
                server.listen(1)
                server.settimeout(2)
                port = server.getsockname()[1]
                def serve():
                    with server.accept()[0] as connection:
                        connection.settimeout(2)
                        connection.recv(8192)
                        connection.sendall(response)
                thread = threading.Thread(target=serve)
                thread.start()
                try:
                    with self.assertRaises(K.Rejected):
                        K.exchange(port, 'GET', '/', {}, '', 'same')
                finally:
                    thread.join()

    def test_actual_tcp_header_or_body_budget_rejected(self):
        for response in (b'HTTP/1.1 200 OK\r\nX-Fill: ' + b'x' * 17000,
                         b'HTTP/1.1 200 OK\r\nContent-Length: 70000\r\nConnection: close\r\n\r\n' + b'x' * 70000):
            with self.subTest(size=len(response)), socket.socket() as server:
                server.bind(('127.0.0.1', 0))
                server.listen(1)
                server.settimeout(2)
                port = server.getsockname()[1]
                def serve():
                    with server.accept()[0] as connection:
                        connection.settimeout(2)
                        connection.recv(8192)
                        try:
                            connection.sendall(response)
                        except OSError:
                            pass
                thread = threading.Thread(target=serve)
                thread.start()
                try:
                    with self.assertRaises(K.Rejected):
                        K.exchange(port, 'GET', '/', {}, '', 'same')
                finally:
                    thread.join()

    def test_actual_tcp_absolute_deadline_until_eof(self):
        with socket.socket() as server:
            server.bind(('127.0.0.1', 0))
            server.listen(1)
            server.settimeout(2)
            port = server.getsockname()[1]
            def serve():
                with server.accept()[0] as connection:
                    connection.settimeout(2)
                    connection.recv(8192)
                    connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok')
                    time.sleep(.12)
            thread = threading.Thread(target=serve)
            thread.start()
            K.END = time.monotonic() + .05
            try:
                with self.assertRaises((K.Rejected, OSError)):
                    K.exchange(port, 'GET', '/', {}, '', 'same')
            finally:
                thread.join()


if __name__ == '__main__':
    unittest.main()
