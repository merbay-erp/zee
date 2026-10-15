import importlib.util
from pathlib import Path
import tempfile
import os
import subprocess
import sys
import time
import unittest

spec = importlib.util.spec_from_file_location('local_rc', Path(__file__).resolve().parents[1] / 'rc-kampanyasi.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class Stats(unittest.TestCase):
    def good(self, text=None, command=None, crashes=None):
        text = text or 'Done 12345 runs in 1801 second(s)\nstat::number_of_executed_units: 12345\n'
        return module.fuzz_stats(text, command or {'passed': True}, crashes or [])['passed']

    def test_actual_long_success(self):
        self.assertTrue(self.good())

    def test_short_smoke_cannot_pass(self):
        self.assertFalse(self.good('Done 12 runs in 3 second(s)\nstat::number_of_executed_units: 12'))

    def test_missing_final_stats_cannot_pass(self):
        self.assertFalse(self.good('Done 12345 runs in 1801 second(s)'))

    def test_sanitizer_failure_rejects(self):
        self.assertFalse(self.good('ERROR: AddressSanitizer\nDone 12345 runs in 1801 second(s)\nstat::number_of_executed_units: 12345'))

    def test_input_timeout_rejects(self):
        self.assertFalse(self.good('ERROR: libFuzzer: timeout\nDone 12345 runs in 1801 second(s)\nstat::number_of_executed_units: 12345'))

    def test_nonzero_exit_rejects(self):
        self.assertFalse(self.good(command={'passed': False}))

    def test_crash_file_rejects(self):
        self.assertFalse(self.good(crashes=[{'path': 'crash-one'}]))


class Ownership(unittest.TestCase):
    def test_actual_stuck_leader_deadline_never_passes(self):
        with tempfile.TemporaryDirectory(prefix='zee-local-rc-owner-') as root:
            path = Path(root)
            value = module.command('timeout', [sys.executable, '-I', '-B', '-c', 'import time;time.sleep(30)'],
                                   path, dict(os.environ), path / 'run.log', .1)
            self.assertFalse(value['passed'])
            self.assertEqual(value['supervisor_failure'], 'outer-deadline')
            self.assertLess(value['elapsed_seconds'], 5)

    def test_actual_exited_leader_descendant_is_signalled_before_reap(self):
        with tempfile.TemporaryDirectory(prefix='zee-local-rc-group-') as root:
            path = Path(root)
            code = ('import os,time\np=os.fork()\n'
                    'if p==0:\n time.sleep(30)\n os._exit(0)\n'
                    'print(p,flush=True)\nos._exit(0)\n')
            value = module.command('held-group', [sys.executable, '-I', '-B', '-c', code],
                                   path, dict(os.environ), path / 'run.log', 5)
            self.assertTrue(value['passed'])
            child = int((path / 'run.log').read_text().strip())
            limit = time.monotonic() + 3
            while True:
                result = subprocess.run(['ps', '-p', str(child), '-o', 'stat='], capture_output=True, text=True)
                state = result.stdout.strip()
                if not state or state.startswith('Z'):
                    break
                self.assertLess(time.monotonic(), limit, 'owned descendant remains running')
                time.sleep(.05)


if __name__ == '__main__':
    unittest.main()
