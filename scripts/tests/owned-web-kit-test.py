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
