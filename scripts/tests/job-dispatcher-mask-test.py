#!/usr/bin/env python3
"""Reject invalid host arguments without echoing their private values."""
import json
from pathlib import Path
import subprocess
import sys
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'job-dispatcher.py'
CANARY = 'PRIVATE_JOB_DISPATCHER_ARGUMENT_CANARY'


class MaskedArguments(unittest.TestCase):
    def check(self, arguments):
        result = subprocess.run([sys.executable, '-I', '-B', str(SCRIPT), *arguments],
                                capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(result.stderr, b'')
        self.assertNotIn(CANARY.encode(), result.stdout)
        self.assertEqual(result.stdout.count(b'\n'), 1)
        record = json.loads(result.stdout)
        self.assertEqual(set(record), {'schema', 'event', 'elapsed_ms'})
        self.assertEqual(record['schema'], 'zee-sql-intent-dispatcher-1')
        self.assertEqual(record['event'], 'rejected')
        self.assertIs(type(record['elapsed_ms']), int)

    def test_invalid_numeric_argument_is_masked(self):
        self.check(['--ticks', CANARY])

    def test_unknown_argument_is_masked(self):
        self.check(['--unknown-' + CANARY])


class JobLifecycleContracts(unittest.TestCase):
    def test_child_free_owner_preparation(self):
        script = Path(__file__).resolve().with_name('job-pg-owner-preflight-test.py')
        result = subprocess.run([sys.executable, '-I', '-B', str(script)],
                                capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        self.assertEqual(result.stdout, b'')
        report = result.stderr.decode()
        self.assertIn('\nRan 44 tests in ', report)
        self.assertTrue(report.rstrip().endswith('\nOK'))


if __name__ == '__main__':
    unittest.main()
