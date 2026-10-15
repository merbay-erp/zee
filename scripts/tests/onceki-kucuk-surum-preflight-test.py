import json, os, shutil, subprocess, sys, tempfile, unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

class Preflight(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='zee-k279-preflight-')
        self.root = Path(self.temp.name).resolve()
        self.script = self.root / 'scripts/tests/onceki-kucuk-surum-test.py'
        self.script.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / 'scripts/tests/onceki-kucuk-surum-test.py', self.script)
        shutil.copyfile(ROOT / 'scripts/lifecycle_core.py', self.root / 'scripts/lifecycle_core.py')
        self.fixtures = self.root / 'compiler/tests/fixtures/onceki-kucuk-surum-v07'
        shutil.copytree(ROOT / 'compiler/tests/fixtures/onceki-kucuk-surum-v07', self.fixtures)
        self.marker = self.root / 'unexpected-native-execution'
        self.binary = self.root / 'canary'
        self.binary.write_text('#!' + sys.executable + '\nfrom pathlib import Path\nPath(' + repr(str(self.marker)) + ').write_text("unexpected")\n')
        self.binary.chmod(0o700)

    def tearDown(self):
        self.temp.cleanup()

    def refused_before_native(self):
        result = subprocess.run([sys.executable, '-I', '-B', str(self.script), '--binary', str(self.binary),
            '--expected-source', 'a' * 40, '--output', str(self.root / 'output')],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=3)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.marker.exists())
        self.assertFalse((self.root / 'output').exists())

    def test_source_tamper_refused_before_native(self):
        (self.fixtures / '01-merhaba-dunya.dil').write_bytes(b'changed')
        self.refused_before_native()

    def test_fifo_source_refused_without_block_or_native(self):
        p = self.fixtures / '01-merhaba-dunya.dil'
        p.unlink()
        os.mkfifo(p, 0o600)
        self.refused_before_native()

    def test_duplicate_manifest_key_refused_before_native(self):
        p = self.fixtures / 'manifest.json'
        p.write_text('{"schema":"extra",' + p.read_text().lstrip()[1:])
        self.refused_before_native()

    def test_extra_inventory_entry_refused_before_native(self):
        (self.fixtures / 'unrecorded.dil').write_bytes(b'')
        self.refused_before_native()

    def test_changed_owner_helper_refused_before_native(self):
        (self.root / 'scripts/lifecycle_core.py').write_bytes(b'changed')
        self.refused_before_native()

if __name__ == '__main__':
    unittest.main(verbosity=2)
