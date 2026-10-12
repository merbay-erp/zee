#!/usr/bin/env python3
"""K-192: bozuk kaynak/artefakt, ikili çalıştırılmadan reddedilir."""
import hashlib
import json
import os
import runpy
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "calisma-ortami-denetle.py"
SHA = "1" * 40


class RetTestleri(unittest.TestCase):
    def setUp(self):
        self.gecici = tempfile.TemporaryDirectory(prefix="zee-ortam-ret-")
        self.addCleanup(self.gecici.cleanup)
        self.kok = Path(self.gecici.name)
        self.artefakt = self.kok / "artefakt"
        self.artefakt.mkdir()
        self.rapor = self.kok / "rapor.json"
        self.ad = "dil.exe" if sys.platform == "win32" else "dil"
        self.ikili = self.artefakt / self.ad
        self.ikili.write_bytes(b"calistirilamaz test verisi")
        (self.artefakt / "GIT_SHA").write_text(SHA + "\n", encoding="ascii")
        self.kayit = hashlib.sha256(self.ikili.read_bytes()).hexdigest() + "  " + self.ad
        (self.artefakt / "SHA256SUMS").write_text(self.kayit + "\n", encoding="ascii")

    def ret(self, sha, neden):
        sonuc = subprocess.run([sys.executable, str(SCRIPT), "--artefakt", str(self.artefakt),
                                "--kurulu", str(self.artefakt), "--rapor", str(self.rapor),
                                "--beklenen-sha", sha], capture_output=True, timeout=10,
                                env=dict(os.environ, PYTHONIOENCODING="ascii:strict"))
        self.assertEqual(sonuc.returncode, 1, sonuc.stderr.decode("utf-8", errors="replace"))
        rapor = json.loads(self.rapor.read_text(encoding="utf-8"))
        self.assertEqual(rapor["sonuc"], "kaldi")
        self.assertIn(neden, rapor["hata"])
        self.assertNotIn("prova", rapor)

    def test_farkli_kaynak_sha_ikiliden_once_reddedilir(self):
        self.ret("2" * 40, "GIT_SHA beklenen")

    def test_bozulmus_ikili_calistirilmadan_reddedilir(self):
        self.ikili.write_bytes(b"degistirilmis test verisi")
        self.ret(SHA, "özetiyle uyuşmuyor")

    def test_yinelenen_checksum_calistirmadan_reddedilir(self):
        (self.artefakt / "SHA256SUMS").write_text(self.kayit + "\n" + self.kayit + "\n", encoding="ascii")
        self.ret(SHA, "tekilliği geçersiz")


class CrtTestleri(unittest.TestCase):
    def test_statik_crt_dinamik_crt_importunu_reddeder(self):
        denetle = runpy.run_path(str(SCRIPT))["statik_crt_denetle"]
        for dll in ("VCRUNTIME140.dll", "MSVCP140.dll", "ucrtbase.dll", "msvcrt.dll",
                    "api-ms-win-crt-runtime-l1-1-0.dll"):
            with self.subTest(dll=dll), self.assertRaisesRegex(ValueError, "dinamik CRT"):
                denetle({"dil.exe": {"dogrudan": ["KERNEL32.dll", dll]}})

    def test_eski_gercek_windows_raporu_statik_crt_sozunu_karsilamaz(self):
        yol = SCRIPT.parent.parent / "compiler/tests/fixtures/calisma-ortami-v1/30dcfaeec34f2bb992719f7a210bdf3138a226b0/windows.json"
        rapor = json.loads(yol.read_bytes())
        with self.assertRaisesRegex(ValueError, "dinamik CRT"):
            runpy.run_path(str(SCRIPT))["statik_crt_denetle"](rapor["ikililer"])

    def test_statik_crt_sistem_dlllerini_kabul_eder(self):
        runpy.run_path(str(SCRIPT))["statik_crt_denetle"]({
            "dil.exe": {"dogrudan": ["KERNEL32.dll", "WS2_32.dll", "bcrypt.dll"]},
            "dillsp.exe": {"dogrudan": ["ADVAPI32.dll"]},
        })


if __name__ == "__main__":
    unittest.main()
