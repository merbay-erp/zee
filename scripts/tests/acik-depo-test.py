#!/usr/bin/env python3
"""K-214: gerçek müşteri adları yerine hermetik, sentetik politika."""
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
API = runpy.run_path(str(ROOT / "scripts/acik-depo-denetle.py"))
NORMALIZE, DIGEST = API["normalize"], API["digest"]
INSPECT, SCAN, LOAD = API["inspect"], API["scan"], API["load_policy"]


def rules():
    return {"terms": {DIGEST(NORMALIZE("Example Client"))},
            "paths": {DIGEST("/private-host/")}, "lengths": {13}}


def git_bash(git_exe):
    # Git may be found through cmd/, bin/ or mingw64/bin/. The Bash runtime
    # belongs to the same installation, not to Windows System32/WSL.
    for parent in git_exe.parents:
        for relative in ["bin/bash.exe", "usr/bin/bash.exe"]:
            candidate = parent / relative
            if candidate.is_file():
                return candidate
    raise ValueError("Git kurulumunda Git Bash bulunamadı")


class PublicPrivacyTests(unittest.TestCase):
    def test_git_bash_cmd_bin_ve_mingw_duzenlerini_bulur(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "Git"
            bash = root / "bin/bash.exe"
            bash.parent.mkdir(parents=True); bash.touch()
            for relative in ["cmd/git.exe", "bin/git.exe", "mingw64/bin/git.exe"]:
                self.assertEqual(git_bash(root / relative), bash)
            bash.unlink()
            with self.assertRaises(ValueError):
                git_bash(root / "cmd/git.exe")

    def test_buyuk_kucuk_unicode_ve_ayiricilar(self):
        for value in ["Example Client", "EXAMPLE_CLIENT", "ｅｘａｍｐｌｅ client",
                      "example\u200bclient", "example-client", "example.client"]:
            self.assertTrue(INSPECT("source.txt", value.encode(), rules(), {}))
        self.assertFalse(INSPECT("source.txt", b"Site A; 203.0.113.7", rules(), {}))
        self.assertTrue(INSPECT("source.txt", b"prefixExampleClientSettings", rules(), {}))

    def test_dosya_adi_ve_yapilandirilmis_yol(self):
        self.assertTrue(INSPECT("example-client.log", b"safe", rules(), {}))
        self.assertTrue(INSPECT("a", b"/private-host/app", rules(), {}))
        self.assertFalse(INSPECT("a", b"DNS SRV; private-host", rules(), {}))

    def test_utf16_icerigi(self):
        self.assertTrue(INSPECT("a", "Example Client".encode("utf-16"), rules(), {}))

    def test_ikili_icerik_metin_sayilmaz_yol_taranir(self):
        ikili = b"\x00\x01OS/2" + "Example Client".encode() + b"\x00\xff"
        self.assertFalse(INSPECT("assets/font.ttf", ikili, rules(), {}))
        self.assertTrue(INSPECT("assets/example-client.ttf", ikili, rules(), {}))
        self.assertTrue(INSPECT("a", "Example Client".encode("utf-16"), rules(), {}))

    def test_kisa_ad_sozcuk_siniri_ve_ayiricilarla_taranir(self):
        short = dict(rules(), words={DIGEST("xyz")}, word_lengths={3})
        for text in ["XYZ", "X-Y-Z", "xyz.example", "prefix_xyz"]:
            self.assertTrue(INSPECT("a", text.encode(), short, {}))
        self.assertFalse(INSPECT("a", b"prefixXYZsuffix", short, {}))
        self.assertFalse(INSPECT("a", b'graph["x", "y", "z"]', short, {}))

    def test_uzun_kayitta_gomulu_ad_ve_ayri_politika_korunur(self):
        text = "z" * 9000 + "ExampleClient" + "z" * 9000
        self.assertTrue(INSPECT("long.log", text.encode(), rules(), {}))
        other = {"terms": {DIGEST("anotherclient")}, "paths": set(), "lengths": {13}}
        self.assertFalse(INSPECT("long.log", text.encode(), other, {}))

    def test_eski_satir_istisnasi_kopyayi_degistirmeyi_ve_tekrari_kapsamaz(self):
        text = "Example Client"
        fp = API["fingerprint"]("old.md", text, DIGEST(NORMALIZE(text)))
        baseline = {fp: 1}
        self.assertFalse(INSPECT("old.md", text.encode(), rules(), baseline))
        self.assertFalse(INSPECT("old.md", ("safe\n" + text).encode(), rules(), baseline))
        self.assertTrue(INSPECT("new.md", text.encode(), rules(), baseline))
        self.assertTrue(INSPECT("old.md", (text + " changed").encode(), rules(), baseline))
        self.assertTrue(INSPECT("old.md", (text + "\n" + text).encode(), rules(), baseline))

    def test_politika_bos_yinelenen_ve_gecersiz_hash_reddeder(self):
        original = json.loads((ROOT / "docs/acik-depo-politikasi-v1.json").read_text())
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "policy.json"
            for change in [{"rules": []}, {"legacy_fingerprints": {"*": 1}},
                           {"legacy_fingerprints": {"a" * 64: 1}}, {"history_scope": "since-base"},
                           {"history_base": "master"},
                           {"rules": original["rules"] * 2}]:
                path.write_text(json.dumps(dict(original, **change)))
                with self.assertRaises(ValueError):
                    LOAD(path)

    def test_eklenip_silinen_dosya_ve_commit_mesaji_reddedilir(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(["git", "-C", directory, *args], stderr=subprocess.PIPE).decode().strip()
            git("init", "-q")
            git("config", "user.name", "Zee Test")
            git("config", "user.email", "zee@example.invalid")
            git("commit", "--allow-empty", "-qm", "base")
            base = git("rev-parse", "HEAD")
            policy = {"history_base": base, "legacy_fingerprints": {}}
            self.assertFalse(SCAN(root, policy, rules(), worktree=False))
            (root / "a.txt").write_text("Example Client")
            git("add", "."); git("commit", "-qm", "add")
            git("rm", "a.txt"); git("commit", "-qm", "remove")
            self.assertFalse(SCAN(root, policy, rules(), history=False, worktree=False))
            self.assertTrue(SCAN(root, policy, rules(), worktree=False))
            git("reset", "--hard", base)
            git("commit", "--allow-empty", "-qm", "Example Client")
            self.assertTrue(SCAN(root, policy, rules(), worktree=False))

    def test_dosyalar_ve_eslesme_degerleri_loga_yazilmaz(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", directory], check=True)
            subprocess.run(["git", "-C", directory, "-c", "user.name=Zee Test",
                "-c", "user.email=zee@example.invalid", "commit", "--allow-empty", "-qm", "base"], check=True)
            policy = {"schema": "zee-acik-depo-politikasi-1", "history_scope": "all-reachable", "history_base": "0" * 40,
                      "rules": [{"kind": "term", "length": 13,
                                 "sha256": DIGEST("exampleclient")}], "legacy_fingerprints": {}}
            path = root / "policy.json"; path.write_text(json.dumps(policy))
            (root / "example-client.txt").write_text("Example Client")
            result = subprocess.run(["python3", str(ROOT / "scripts/acik-depo-denetle.py"),
                "--depo", directory, "--politika", str(path), "--yalniz-agac"], capture_output=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn(b"MASKEL", result.stdout + result.stderr)
            self.assertNotIn(b"Example Client", result.stdout + result.stderr)
            self.assertNotIn(b"example-client.txt", result.stdout + result.stderr)

    def test_yerel_yama_iki_manifestte_git_erisimini_kaldirir(self):
        import tomllib
        for path, expected in [("compiler/Cargo.toml", "third_party/postgres-protocol"),
                               ("compiler/fuzz/Cargo.toml", "../third_party/postgres-protocol")]:
            manifest = tomllib.loads((ROOT / path).read_text())
            self.assertEqual(manifest["patch"]["crates-io"]["postgres-protocol"], {"path": expected})
        self.assertFalse((ROOT / "scripts/ci-oz-kaynak.sh").exists())

    def test_kok_ve_eski_taban_oncesi_icerik_istisnasiz_taranir(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(["git", "-C", directory, *args], stderr=subprocess.PIPE).decode().strip()
            git("init", "-q"); git("config", "user.name", "Zee Test")
            git("config", "user.email", "zee@example.invalid")
            (root / "old.txt").write_text("Example Client")
            git("add", "."); git("commit", "-qm", "root")
            git("rm", "old.txt"); git("commit", "-qm", "old base")
            base = git("rev-parse", "HEAD")
            self.assertTrue(SCAN(root, {"history_base": base, "legacy_fingerprints": {}}, rules(), worktree=False))

    def test_taban_eslemesi_yalniz_onayli_eski_master_icin_gecerlidir(self):
        api = runpy.run_path(str(ROOT / "scripts/ci-taban-kimligi.py"))
        for before in ["", "0" * 40, "a" * 40]:
            self.assertEqual(api["resolve"](before), before)
        for before in api["OLD_TIPS"]:
            target = api["resolve"](before)
            subprocess.run(["git", "-C", str(ROOT), "merge-base", "--is-ancestor", target, "HEAD"], check=True)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); path = root / "docs/incelemeler/k214/sha-esleme-v1.tsv"
            path.parent.mkdir(parents=True); path.write_text("# empty\n")
            with self.assertRaises(ValueError):
                api["resolve"](api["OLD_MASTER"], root)
            path = root / "docs/incelemeler/k326/sha-esleme-v1.tsv"
            path.parent.mkdir(parents=True); path.write_text("# empty\n")
            for before in api["K326_TIPS"]:
                with self.assertRaises(ValueError):
                    api["resolve"](before, root)


if __name__ == "__main__":
    unittest.main()
