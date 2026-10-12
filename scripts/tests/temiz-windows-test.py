#!/usr/bin/env python3
"""K-197: bozuk artefakt Docker/ikili yürütmeden reddedilir; LSP hatası kapanmaz."""
import hashlib
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "temiz-windows-prova.py"


class RetTestleri(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="zee-clean-win-ret-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.sha = "a" * 40
        self.api = runpy.run_path(str(SCRIPT))
        (self.root / "GIT_SHA").write_text(self.sha + "\n", encoding="ascii")
        for name in ("dil.exe", "dillsp.exe"):
            (self.root / name).write_bytes(b"test artifact bytes")
        self.line = hashlib.sha256(b"test artifact bytes").hexdigest()
        (self.root / "SHA256SUMS").write_text("".join(self.line + "  " + n + "\n" for n in ("dil.exe", "dillsp.exe")), encoding="ascii")

    def ret(self, reason, sha=None):
        with patch("subprocess.run") as command:
            with self.assertRaisesRegex(ValueError, reason):
                self.api["prova"](self.root, sha or self.sha)
            command.assert_not_called()

    def test_farkli_sha_dockerdan_once_reddedilir(self):
        self.ret("source differs", "b" * 40)

    def test_degismis_ikili_dockerdan_once_reddedilir(self):
        (self.root / "dil.exe").write_bytes(b"changed")
        self.ret("binary hash differs")

    def test_yol_yonlendirme_dockerdan_once_reddedilir(self):
        (self.root / "SHA256SUMS").write_text(self.line + "  ../foreign.exe\n", encoding="ascii")
        self.ret("name/uniqueness")

    def test_yinelenen_kayit_dockerdan_once_reddedilir(self):
        p = self.root / "SHA256SUMS"
        p.write_text(p.read_text(encoding="ascii") + self.line + "  dil.exe\n", encoding="ascii")
        self.ret("name/uniqueness")

    def test_lsp_hata_eksik_ve_yinelenen_yanit_reddedilir(self):
        validate = self.api["lsp_dogrula"]
        good = [{"id": 1, "result": {"capabilities": {}}}, {"id": 2, "result": None}]
        validate(good)
        for replies in (good[:1], good + [good[1]], [{"id": 1, "error": {}}, good[1]]):
            with self.subTest(replies=replies), self.assertRaisesRegex(ValueError, "LSP"):
                validate(replies)


if __name__ == "__main__":
    unittest.main()
