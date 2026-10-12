#!/usr/bin/env python3
"""K-198 exact kaynak/byte doğrulaması Docker'dan önce olmalıdır."""
import hashlib
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "temiz-windows-kurulum-prova.py"


class RetTestleri(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="zee-clean-install-ret-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.sha = "a" * 40
        self.api = runpy.run_path(str(SCRIPT))
        (self.root / "GIT_SHA").write_text(self.sha + "\n", encoding="ascii")
        for name in ("dil.exe", "dillsp.exe"):
            (self.root / name).write_bytes(b"test artifact bytes")
        self.hash = hashlib.sha256(b"test artifact bytes").hexdigest()
        (self.root / "SHA256SUMS").write_text("".join(self.hash + "  " + n + "\n" for n in ("dil.exe", "dillsp.exe")), encoding="ascii")

    def ret(self, reason, sha=None):
        with patch("subprocess.run") as command:
            with self.assertRaisesRegex(ValueError, reason):
                self.api["prova"](self.root, sha or self.sha)
            command.assert_not_called()

    def test_yanlis_sha_dockerdan_once_reddedilir(self):
        self.ret("source differs", "b" * 40)

    def test_degismis_ikili_dockerdan_once_reddedilir(self):
        (self.root / "dillsp.exe").write_bytes(b"changed")
        self.ret("binary hash differs")

    def test_yinelenen_kayit_dockerdan_once_reddedilir(self):
        p = self.root / "SHA256SUMS"
        p.write_text(p.read_text(encoding="ascii") + self.hash + "  dil.exe\n", encoding="ascii")
        self.ret("name/uniqueness")

    def test_manifest_yabanci_yol_kayip_ikili_ve_yanlis_kaynagi_reddeder(self):
        hashes = {n: self.hash for n in ("dil.exe", "dillsp.exe")}
        manifest = '# zee-kurulum-1\n' + ''.join(self.api['BIN'] + '\\' + n + '\t' + self.hash + '\t' + self.sha + '\n' for n in hashes)
        validate = self.api['manifest_dogrula']
        validate(manifest, self.sha, hashes)
        for bad in (manifest.replace(self.sha, 'b' * 40), manifest.replace('dil.exe', 'foreign.exe'), manifest.rsplit('\n', 2)[0] + '\n'):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                validate(bad, self.sha, hashes)

    def test_temizlik_hatasi_asli_hatayi_ortmez_ve_basarida_reddedilir(self):
        cleanup = self.api['container_temizle']
        with patch('subprocess.run', return_value=subprocess.CompletedProcess([], 1)):
            cleanup('zee-k198-test', ValueError)
            with self.assertRaisesRegex(ValueError, 'cleanup'):
                cleanup('zee-k198-test', None)
        with patch('subprocess.run', side_effect=subprocess.TimeoutExpired('docker', 30)):
            cleanup('zee-k198-test', ValueError)
            with self.assertRaises(subprocess.TimeoutExpired):
                cleanup('zee-k198-test', None)


if __name__ == "__main__":
    unittest.main()
