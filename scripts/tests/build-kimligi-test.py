#!/usr/bin/env python3
"""K-226: actual build.rs/surum.rs in isolated, dependency-free Cargo fixtures.

This tests identity logic and fixture reproducibility, not product release
acceptance. The source checkout, its Cargo target and user Cargo config stay
unchanged. Only an already-installed pinned toolchain is used; Cargo is offline.
"""

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
COMMIT_DATE = "2024-02-29T12:00:00+00:00"
COMMIT_EPOCH = 1709208000
BINARY = "zee-build-fixture" + (".exe" if os.name == "nt" else "")


class BuildIdentityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="zee-build-identity-")
        cls.addClassCleanup(cls.temp.cleanup)
        cls.base = Path(cls.temp.name).resolve()
        cls.version = tomllib.loads((ROOT / "compiler/Cargo.toml").read_text())["package"]["version"]
        cls.toolchain = tomllib.loads((ROOT / "compiler/rust-toolchain.toml").read_text())["toolchain"]["channel"]
        cls.env = os.environ.copy()
        for key in list(cls.env):
            if (key.startswith(("ZEE_SOURCE_", "ZEE_RELEASE_", "CARGO_", "GIT_"))
                    or key in ("SOURCE_DATE_EPOCH", "RUSTFLAGS", "RUSTC", "RUSTDOC",
                               "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTUP_TOOLCHAIN")):
                cls.env.pop(key)
        cls.env.update({"CARGO_HOME": str(cls.base / "cargo-home"),
                        "CARGO_NET_OFFLINE": "true", "GIT_CONFIG_NOSYSTEM": "1",
                        "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull})
        # `rustup which` fails for an absent toolchain; it does not install one.
        cls.cargo = cls.command(["rustup", "which", "--toolchain", cls.toolchain, "cargo"]).stdout.strip()
        cls.env["RUSTC"] = cls.command(["rustup", "which", "--toolchain", cls.toolchain, "rustc"]).stdout.strip()
        cls.env["RUSTDOC"] = cls.command(["rustup", "which", "--toolchain", cls.toolchain, "rustdoc"]).stdout.strip()
        cls.seed = cls.base / "seed"
        compiler = cls.seed / "compiler"
        (compiler / "src").mkdir(parents=True)
        shutil.copyfile(ROOT / "compiler/build.rs", compiler / "build.rs")
        shutil.copyfile(ROOT / "compiler/src/surum.rs", compiler / "src/surum.rs")
        (compiler / "Cargo.toml").write_text(
            '[package]\nname = "zee-build-fixture"\nversion = "' + cls.version +
            '"\nedition = "2021"\nbuild = "build.rs"\n[workspace]\n')
        (compiler / "src/main.rs").write_text(
            'mod surum;\nfn main() { println!("{}", surum::json()); '
            'println!("{}", surum::metin()); }\n')
        (cls.seed / ".gitignore").write_text("compiler/target/\n")
        cls.command([cls.cargo, "generate-lockfile", "--offline"], cwd=compiler)
        cls.git(cls.seed, "init", "-q")
        cls.git(cls.seed, "add", ".")
        cls.commit(cls.seed, "actual build source fixture", COMMIT_DATE)
        cls.sha = cls.git(cls.seed, "rev-parse", "HEAD").stdout.strip()
        if cls.git(cls.seed, "status", "--porcelain").stdout:
            raise AssertionError("seed source must be clean")

    @classmethod
    def command(cls, arguments, cwd=None, env=None, check=True):
        result = subprocess.run(arguments, cwd=cwd, env=env or cls.env, capture_output=True,
                                text=True, timeout=60)
        if check and result.returncode:
            raise AssertionError("Fixture command failed:\n" + result.stdout + result.stderr)
        return result

    @classmethod
    def git(cls, root, *args, env=None):
        return cls.command(["git", "-C", str(root), *args], env=env)

    @classmethod
    def commit(cls, root, message, date):
        environment = {**cls.env, "GIT_AUTHOR_DATE": date, "GIT_COMMITTER_DATE": date}
        cls.git(root, "-c", "user.name=Zee test", "-c", "user.email=zee-test@example.invalid",
                "commit", "-qm", message, env=environment)

    def setUp(self):
        self.directory = self.base / self._testMethodName
        self.directory.mkdir()
        self.repo = self.directory / "source"
        self.clone(self.repo)

    def clone(self, destination):
        self.command(["git", "clone", "--quiet", "--no-local", str(self.seed), str(destination)])
        self.assertEqual(self.git(destination, "status", "--porcelain").stdout, "")

    def build(self, root=None, values=None, check=True):
        root = (root or self.repo).resolve()
        environment = {**self.env, "CARGO_TARGET_DIR": str(root / "compiler/target"),
                       "CARGO_ENCODED_RUSTFLAGS": "--remap-path-prefix=" + str(root) + "=/zee-source"}
        environment.update(values or {})
        result = self.command([self.cargo, "build", "--locked", "--offline", "--release"],
                              cwd=root / "compiler", env=environment, check=check)
        binary = root / "compiler/target/release" / BINARY
        if not check:
            return result
        execution = self.command([str(binary)], cwd=root)
        lines = execution.stdout.splitlines()
        self.assertEqual(len(lines), 2)
        return json.loads(lines[0]), lines[1], hashlib.sha256(binary.read_bytes()).hexdigest()

    def archive(self):
        destination = self.directory / "archive"
        shutil.copytree(self.repo, destination, ignore=shutil.ignore_patterns(".git", "target"))
        return destination

    def assert_build_rejected(self, values, expected):
        result = self.build(values=values, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(expected, result.stdout + result.stderr)

    def test_two_clean_clones_same_commit_identity_and_binary_hash(self):
        second = self.directory / "second"
        self.clone(second)
        first_record, first_text, first_hash = self.build()
        second_record, second_text, second_hash = self.build(second)
        self.assertEqual(first_record, second_record)
        self.assertEqual(first_text, second_text)
        self.assertEqual(first_hash, second_hash)
        self.assertEqual(first_record["source_sha"], self.sha)
        self.assertEqual(first_record["source_date"], "2024-02-29")
        self.assertEqual(first_record["schema"], "zee-build-v1")
        self.assertEqual(first_record["version"], self.version)
        self.assertFalse(first_record["dirty"])
        self.assertEqual(self.git(self.repo, "status", "--porcelain").stdout, "")
        self.assertEqual(self.git(second, "status", "--porcelain").stdout, "")

    def test_second_actual_git_commit_changes_identity_without_source_edit(self):
        first, _, _ = self.build()
        (self.repo / "notes.txt").write_text("second exact source commit\n")
        self.git(self.repo, "add", "notes.txt")
        self.commit(self.repo, "second commit", "2024-03-01T12:00:00+00:00")
        second_sha = self.git(self.repo, "rev-parse", "HEAD").stdout.strip()
        second, _, _ = self.build()
        self.assertNotEqual(second_sha, self.sha)
        self.assertEqual(second["source_sha"], second_sha)
        self.assertNotEqual(second["source_sha"], first["source_sha"])
        self.assertEqual(second["source_date"], "2024-03-01")
        self.assertFalse(second["dirty"])

    def test_declared_sha_must_match_actual_checkout(self):
        self.assert_build_rejected({"ZEE_SOURCE_SHA": "0" * 40},
                                   "declared source SHA differs from checkout HEAD")

    def test_matching_declared_sha_and_exact_epoch_are_accepted(self):
        record, _, _ = self.build(values={"ZEE_SOURCE_SHA": self.sha,
                                         "SOURCE_DATE_EPOCH": str(COMMIT_EPOCH)})
        self.assertEqual(record["source_sha"], self.sha)
        self.assertEqual(record["source_date"], "2024-02-29")

    def test_source_date_epoch_mismatch_rejected_even_with_same_calendar_day(self):
        self.assert_build_rejected({"SOURCE_DATE_EPOCH": str(COMMIT_EPOCH + 1)},
                                   "SOURCE_DATE_EPOCH must equal exact source commit time")

    def test_development_version_cannot_claim_stable(self):
        self.assertTrue(self.version.endswith("-dev"))
        self.assert_build_rejected({"ZEE_RELEASE_CHANNEL": "stable"},
                                   "development series cannot claim stable channel")

    def test_unknown_release_channel_rejected(self):
        self.assert_build_rejected({"ZEE_RELEASE_CHANNEL": "unreviewed-channel"},
                                   "unsupported release channel")

    def test_short_or_uppercase_source_sha_rejected(self):
        for value in (self.sha[:7], "A" * 40):
            with self.subTest(value=value):
                self.assert_build_rejected({"ZEE_SOURCE_SHA": value},
                                           "source SHA must be 40 lowercase hex characters")

    def test_archive_without_git_has_explicit_unknown_identity_and_date(self):
        record, text, _ = self.build(self.archive())
        self.assertEqual(record["source_sha"], "unknown")
        self.assertEqual(record["source_date"], "unknown")
        self.assertIn("unknown", text)
        self.assertFalse(record["dirty"])
        self.assertNotEqual(record["source_date"], datetime.now(timezone.utc).date().isoformat())

    def test_archive_explicit_epoch_calendar_boundaries(self):
        archive = self.archive()
        cases = [(0, "1970-01-01"), (951782400, "2000-02-29"),
                 (1709208000, "2024-02-29"), (946684799, "1999-12-31"),
                 (253402300799, "9999-12-31")]
        for epoch, expected in cases:
            with self.subTest(epoch=epoch):
                record, _, _ = self.build(archive, {"SOURCE_DATE_EPOCH": str(epoch)})
                self.assertEqual(record["source_date"], expected)
                self.assertEqual(record["source_sha"], "unknown")

    def test_archive_epoch_above_year9999_or_negative_rejected(self):
        archive = self.archive()
        for epoch, expected in [("253402300800", "source date exceeds year 9999"),
                                ("-1", "source epoch must be unsigned seconds")]:
            with self.subTest(epoch=epoch):
                result = self.build(archive, {"SOURCE_DATE_EPOCH": epoch}, check=False)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(expected, result.stdout + result.stderr)

    def test_uncommitted_compiler_source_is_marked_dirty(self):
        source = self.repo / "compiler/src/main.rs"
        source.write_text(source.read_text() + "// uncommitted fixture change\n")
        record, text, _ = self.build()
        self.assertEqual(record["source_sha"], self.sha)
        self.assertTrue(record["dirty"])
        self.assertIn("kirli kaynak", text)


if __name__ == "__main__":
    unittest.main()
