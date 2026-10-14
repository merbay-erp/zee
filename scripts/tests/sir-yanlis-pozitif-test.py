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
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
MODULE = runpy.run_path(str(ROOT / "scripts/sir-yanlis-pozitif-denetle.py"))
CHECK = MODULE["dogrula"]
TLS_CHECK = MODULE["tls_fixture_dogrula"]
RECORD = ROOT / "docs/sir-yanlis-pozitif-v1.tsv"
TLS_RECORD = ROOT / "docs/sentetik-tls-fixture-v1.tsv"
TLS_FILE = MODULE["TLS_FIXTURE"]
TLS_FILES = MODULE["TLS_FIXTURES"]
SMTP_FILE = "compiler/tests/fixtures/smtp-mime/server.p12"


def tls_dosyalari_kopyala(tree, proof=False):
    for file, provenance in TLS_FILES.items():
        copied = tree / file
        copied.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / file, copied)
        if proof:
            shutil.copyfile(ROOT / provenance, tree / provenance)


class FalsePositiveTests(unittest.TestCase):
    def test_sentetik_tls_yalniz_exact_immutable_icerik_icin_fingerprint_alir(self):
        self.assertEqual(TLS_CHECK(TLS_RECORD, None), [])
        with tempfile.TemporaryDirectory() as directory:
            tree = Path(directory)
            tls_dosyalari_kopyala(tree)
            expected = sorted(str(tree / file) + ":pkcs12-file:0" for file in TLS_FILES)
            self.assertEqual(TLS_CHECK(TLS_RECORD, tree), expected)
            for file in TLS_FILES:
                copied = tree / file
                original = copied.read_bytes()
                copied.write_bytes(original + b"changed")
                with self.assertRaisesRegex(ValueError, "hash"):
                    TLS_CHECK(TLS_RECORD, tree)
                copied.write_bytes(original)

    def test_sentetik_tls_genis_yol_kural_hash_ve_kaynak_reddedilir(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "wrong.tsv"
            original = TLS_RECORD.read_text().splitlines()
            for row in [2, 3]:
                for index, value in [(0, "compiler/tests/fixtures/.*"), (1, "generic-api-key"),
                                     (2, "0" * 64), (3, "0" * 40), (4, "docs/anything.md")]:
                    lines = original.copy()
                    fields = lines[row].split("\t")
                    fields[index] = value
                    lines[row] = "\t".join(fields)
                    path.write_text("\n".join(lines) + "\n")
                    with self.assertRaises(ValueError):
                        TLS_CHECK(path, None)
            path.write_text(TLS_RECORD.read_text() + original[2] + "\n")
            with self.assertRaisesRegex(ValueError, "sayısı"):
                TLS_CHECK(path, None)

    def test_sentetik_tls_baska_pkcs12_dosyasini_susturmaz(self):
        if not shutil.which("gitleaks"):
            self.skipTest("Gitleaks CI security işinde kurulur")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tree = root / "tree"
            tls_dosyalari_kopyala(tree)
            config = root / "default.toml"
            config.write_text("[extend]\nuseDefault = true\n")
            ignore = root / "ignore"
            ignore.write_text("\n".join(CHECK(RECORD) + TLS_CHECK(TLS_RECORD, tree)) + "\n")
            args = ["gitleaks", "dir", "--redact", "--no-banner", "--ignore-gitleaks-allow",
                    "--config", str(config), "--gitleaks-ignore-path", str(ignore), str(tree)]
            self.assertEqual(subprocess.run(args, capture_output=True).returncode, 0)
            shutil.copyfile(ROOT / TLS_FILE, tree / "unknown.p12")
            self.assertEqual(subprocess.run(args, capture_output=True).returncode, 1)

    def test_tls_kayit_eksik_tekrar_ve_alti_alan_disini_reddeder(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "wrong.tsv"
            original = TLS_RECORD.read_text().splitlines()
            path.write_text("\n".join(original[:-1]) + "\n")
            with self.assertRaisesRegex(ValueError, "sayısı"):
                TLS_CHECK(path, None)
            duplicate = original[:3] + [original[2]]
            path.write_text("\n".join(duplicate) + "\n")
            with self.assertRaisesRegex(ValueError, "exact"):
                TLS_CHECK(path, None)
            for row in [original[3] + "\textra", "\t".join(original[3].split("\t")[:-1])]:
                path.write_text("\n".join(original[:3] + [row]) + "\n")
                with self.assertRaisesRegex(ValueError, "altı"):
                    TLS_CHECK(path, None)

    def test_yeni_tls_ilk_izlenen_blob_ve_bozuk_kaynak_diski_reddedilir(self):
        with tempfile.TemporaryDirectory() as directory:
            tree = Path(directory)
            tls_dosyalari_kopyala(tree, proof=True)
            # Isolate the working disk; immutable reads still come from actual Git.
            real_show = subprocess.check_output

            def source_show(args, **kwargs):
                self.assertEqual(args[:3], ["git", "-C", str(tree)])
                return real_show(["git", "-C", str(ROOT)] + args[3:], **kwargs)

            with mock.patch.object(MODULE["subprocess"], "check_output", side_effect=source_show):
                self.assertEqual(TLS_CHECK(TLS_RECORD, None, root=tree), [])
                copied = tree / SMTP_FILE
                original_bytes = copied.read_bytes()
                copied.write_bytes(original_bytes + b"changed")
                with self.assertRaisesRegex(ValueError, "hash"):
                    TLS_CHECK(TLS_RECORD, None, root=tree)
                copied.write_bytes(original_bytes)
                provenance = tree / TLS_FILES[SMTP_FILE]
                provenance.unlink()
                with self.assertRaisesRegex(ValueError, "kökeni"):
                    TLS_CHECK(TLS_RECORD, None, root=tree)

            path = tree / "wrong.tsv"
            lines = TLS_RECORD.read_text().splitlines()
            fields = lines[3].split("\t")
            # The MIME source checkpoint did not yet track the ignored PKCS12.
            fields[3] = "89c6522c83ad3d383bfe1e819341ab856a5d96d1"
            lines[3] = "\t".join(fields)
            path.write_text("\n".join(lines) + "\n")
            with self.assertRaisesRegex(ValueError, "immutable"):
                TLS_CHECK(path, None)

    def test_yeni_tls_tarama_kopyasi_eksik_ve_symlink_olamaz(self):
        with tempfile.TemporaryDirectory() as directory:
            tree = Path(directory)
            tls_dosyalari_kopyala(tree)
            copied = tree / SMTP_FILE
            copied.unlink()
            with self.assertRaisesRegex(ValueError, "kopyasının"):
                TLS_CHECK(TLS_RECORD, tree)
            copied.symlink_to(ROOT / SMTP_FILE)
            with self.assertRaisesRegex(ValueError, "kopyasının"):
                TLS_CHECK(TLS_RECORD, tree)

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
