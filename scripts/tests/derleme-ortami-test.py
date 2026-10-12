#!/usr/bin/env python3
"""K-196 kayıt aracı gerçek komut/araç kimliği olmadan kanıt üretmez."""
import hashlib
import json
from pathlib import Path
import runpy
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "derleme-ortami-kaydet.py"


class KayitTestleri(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="zee-build-env-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.sha = "a" * 40
        self.kaydet = runpy.run_path(str(SCRIPT))["kaydet"]
        self.lines = []
        for binary in ("dil", "dillsp"):
            # Resolver testte Python ikilisine yönlendirilir;
            # --version ve dosya SHA-256 hesabı gerçek süreç/byte kullanır.
            self.lines.append(' INFO rustc_codegen_ssa::back::link "cc" "-o" "' + binary + '-123abc"')
        self.logs = {}
        for clone in ("a", "b"):
            p = self.root / (clone + ".log")
            p.write_text("\n".join(self.lines), encoding="utf-8")
            self.logs[clone] = p

    def test_eksik_ikinci_ikili_reddedilir(self):
        self.logs["b"].write_text(self.lines[0], encoding="utf-8")
        from unittest.mock import patch
        with patch("shutil.which", return_value=sys.executable), self.assertRaisesRegex(ValueError, "İki gerçek"):
            self.kaydet(self.logs, self.sha)

    def test_yanlis_sha_reddedilir(self):
        with self.assertRaisesRegex(ValueError, "SHA"):
            self.kaydet(self.logs, "bad")

    def test_linker_programi_olmayan_log_reddedilir(self):
        self.logs["a"].write_text(' INFO rustc_codegen_ssa::back::link "-o" "dil-123abc"')
        with self.assertRaisesRegex(ValueError, "tekil"):
            self.kaydet(self.logs, self.sha)

    def test_bulunmayan_linker_reddedilir(self):
        from unittest.mock import patch
        with patch("shutil.which", return_value=None), self.assertRaisesRegex(ValueError, "bulunamadı"):
            self.kaydet(self.logs, self.sha)

    def test_gercek_arac_hashi_ve_iki_klon_kaydi(self):
        from unittest.mock import patch
        with patch("shutil.which", return_value=sys.executable):
            data = self.kaydet(self.logs, self.sha)
        self.assertEqual(data["gitCommit"], self.sha)
        self.assertEqual(set(data["clones"]), {"a", "b"})
        expected = hashlib.sha256(Path(sys.executable).resolve().read_bytes()).hexdigest()
        self.assertEqual(data["clones"]["a"]["binaries"]["dil"]["sha256"], expected)
        self.assertTrue(data["clones"]["b"]["binaries"]["dillsp"]["version"])

    def test_windows_debug_quoted_yolu_kayipsiz_cozulur(self):
        parse = runpy.run_path(str(SCRIPT))["quoted"]
        value = r"C:\Program Files\Microsoft Visual Studio\link.exe"
        self.assertEqual(parse("INFO " + json.dumps(value)), [value])
        from unittest.mock import patch
        sdk = r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.lib"
        for clone, path in self.logs.items():
            lines = [' INFO rustc_codegen_ssa::back::link "cc" ' + json.dumps('/OUT:C:\\build\\' + binary + '.exe')
                     for binary in ('dil', 'dillsp')]
            path.write_text('\n'.join(lines) + '\nSearching ' + sdk + '\n', encoding='utf-8')
        with patch('shutil.which', return_value=sys.executable):
            data = self.kaydet(self.logs, self.sha)
        self.assertEqual(set(data['clones']['a']['binaries']), {'dil', 'dillsp'})
        self.assertEqual(data['clones']['b']['sdkVersions'][sdk], '10.0.26100.0')


if __name__ == "__main__":
    unittest.main()
