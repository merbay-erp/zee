#!/usr/bin/env python3
"""K-204: exact geçmiş istisnası yeni anahtar eşleşmesini susturamaz."""
import json
import base64
import hashlib
from pathlib import Path
import runpy
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
CHECK = runpy.run_path(str(ROOT / "scripts/sir-yanlis-pozitif-denetle.py"))["dogrula"]
RECORD = ROOT / "docs/sir-yanlis-pozitif-v1.tsv"


class FalsePositiveTests(unittest.TestCase):
    def test_immutable_kayitlar_dort_exact_fingerprint_tasir(self):
        self.assertEqual(len(CHECK(RECORD)), 4)

    def test_bayat_hash_reddedilir(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "wrong.tsv"
            lines = RECORD.read_text().splitlines()
            fields = lines[2].split("\t"); fields[1] = "0" * 64
            lines[2] = "\t".join(fields)
            path.write_text("\n".join(lines)+"\n")
            with self.assertRaisesRegex(ValueError, "hash"):
                CHECK(path)

    def test_genis_istisna_ve_tekrar_reddedilir(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "wrong.tsv"
            text = RECORD.read_text()
            path.write_text(text + text.splitlines()[2] + "\n")
            with self.assertRaisesRegex(ValueError, "fingerprint"):
                CHECK(path)
            fields = text.splitlines()[2].split("\t"); fields[0] = "docs/.*"
            path.write_text("# zee-sir-yanlis-pozitif-1\n" + "\t".join(fields) + "\n")
            with self.assertRaisesRegex(ValueError, "fingerprint"):
                CHECK(path)

    def test_yeni_anahtar_gitleaks_tarafindan_reddedilir(self):
        if not shutil.which("gitleaks"):
            self.skipTest("Gitleaks CI security işinde kurulur")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = root / "default.toml"; config.write_text("[extend]\nuseDefault = true\n")
            ignore = root / "ignore"; ignore.write_text("\n".join(CHECK(RECORD))+"\n")
            report = root / "redacted.json"
            value = base64.urlsafe_b64encode(hashlib.sha256(b"zee generated detector fixture").digest()).decode().rstrip("=")
            payload = ('api_' + 'key="' + value + '"\n').encode()
            result = subprocess.run(["gitleaks", "stdin", "--redact", "--no-banner",
                "--ignore-gitleaks-allow", "--config", str(config),
                "--gitleaks-ignore-path", str(ignore), "--report-format", "json",
                "--report-path", str(report)], input=payload, capture_output=True)
            self.assertEqual(result.returncode, 1)
            findings = json.loads(report.read_bytes())
            self.assertTrue(any(f["RuleID"] == "generic-api-key" for f in findings))
            self.assertTrue(all(f["Secret"] == "REDACTED" for f in findings))


if __name__ == "__main__":
    unittest.main()
