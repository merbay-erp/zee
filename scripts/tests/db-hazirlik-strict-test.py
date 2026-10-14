#!/usr/bin/env python3
"""K-251: fail-closed gate ownership and bounded child lifecycle checks."""
import json
import os
from pathlib import Path
import runpy
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
API = runpy.run_path(str(ROOT/'scripts/db-hazirlik.py'))
Gate, Reject, child = API['Gate'], API['Reject'], API['child']
REVISION = '2'*40


class GateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        self.path = self.root/'ready.json'
        self.gate = None

    def tearDown(self):
        if self.gate is not None:
            self.gate.close()
        self.temp.cleanup()

    def open(self):
        self.gate = Gate(self.root, self.path, REVISION)
        return self.gate

    def test_exact_atomic_true_then_false(self):
        gate = self.open()
        for ready in (True, False):
            gate.write(ready)
            self.assertEqual(json.loads(self.path.read_text()), {
                'schema':'zee-worker-readiness-1',
                'application_revision':REVISION, 'ready':ready})
            self.assertEqual(self.path.stat().st_mode & 0o777, 0o600)
        self.assertFalse(list(self.root.glob('*.tmp')))

    def test_symlink_destination_rejected_without_target_change(self):
        target = self.root/'target'
        target.write_text('unchanged')
        self.path.symlink_to(target)
        with self.assertRaises(Reject): self.open().write(False)
        self.assertEqual(target.read_text(), 'unchanged')

    def test_hardlink_destination_rejected(self):
        target = self.root/'target'; target.write_text('unchanged')
        os.link(target, self.path)
        with self.assertRaises(Reject): self.open().write(False)
        self.assertEqual(target.read_text(), 'unchanged')

    def test_public_write_destination_rejected(self):
        self.path.write_text('unchanged'); self.path.chmod(0o666)
        with self.assertRaises(Reject): self.open().write(False)
        self.assertEqual(self.path.read_text(), 'unchanged')

    def test_concurrent_writer_lock_rejected(self):
        self.open()
        with self.assertRaises(OSError): Gate(self.root, self.path, REVISION)

    def test_lock_symlink_rejected(self):
        target = self.root/'target'; target.write_text('unchanged')
        (self.root/'ready.json.lock').symlink_to(target)
        with self.assertRaises(OSError): self.open()
        self.assertEqual(target.read_text(), 'unchanged')

    def test_parent_replacement_rejected(self):
        parent = self.root/'private'; parent.mkdir(mode=0o700)
        self.path = parent/'ready.json'; gate = self.open()
        parent.rename(self.root/'old'); parent.mkdir(mode=0o700)
        with self.assertRaises(Reject): gate.write(False)
        self.assertFalse(self.path.exists())

    def test_public_write_parent_rejected(self):
        self.root.chmod(0o777)
        with self.assertRaises(Reject): self.open()
        self.root.chmod(0o700)

    def test_deadline_kills_and_waits_child(self):
        start = time.monotonic()
        with self.assertRaises(Reject):
            child(Path('/bin/sleep'), ['5'], self.root, start+.1)
        self.assertLess(time.monotonic()-start, 2)

    def test_output_bound_kills_and_waits_child(self):
        start = time.monotonic()
        with self.assertRaises(Reject):
            child(Path('/usr/bin/yes'), [], self.root, start+3)
        self.assertLess(time.monotonic()-start, 2)


if __name__ == '__main__':
    unittest.main()
