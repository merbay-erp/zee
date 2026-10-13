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
            policy = {"schema": "zee-acik-depo-politikasi-1", "history_base": "0" * 40,
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

    def test_ci_hazirligi_kimlik_bilgisi_olmadan_exact_yerel_kaynagi_kullanir(self):
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory) / "gitconfig"
            envfile = Path(directory) / "env"
            env = dict(os.environ, CI="false", GIT_CONFIG_GLOBAL=str(config),
                       GIT_CONFIG_NOSYSTEM="1", GITHUB_ENV=str(envfile))
            bash = "bash"
            if os.name == "nt":
                # Windows System32 bash may select WSL; this script uses the
                # same Git Bash runtime as the workflow shell.
                git_exe = Path(shutil.which("git"))
                bash = str(git_bash(git_exe))
                env["GIT_CONFIG_GLOBAL"] = config.as_posix()
                env["GITHUB_ENV"] = envfile.as_posix()
            command = [bash, (ROOT / "scripts/ci-oz-kaynak.sh").as_posix()]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True)
            self.assertEqual(result.returncode, 2, result.stderr.decode())
            self.assertFalse(config.exists())
            env["CI"] = "true"
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode())
            self.assertEqual(envfile.read_text(), "CARGO_NET_GIT_FETCH_WITH_CLI=true\n")
            result = subprocess.run(["git", "ls-remote", "https://github.com/merbay-erp/zee", "HEAD"],
                                    cwd=ROOT, env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode())
            head = subprocess.check_output(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
            self.assertIn(head, result.stdout)

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


if __name__ == "__main__":
    unittest.main()
