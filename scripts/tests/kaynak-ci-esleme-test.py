#!/usr/bin/env python3
"""K-224 exact-source verifier acceptance and tampering tests; stdlib only."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import warnings
import zipfile

TOOL = Path(__file__).resolve().parent.parent / "kaynak-ci-esleme.py"


def digest(value):
    return hashlib.sha256(value).hexdigest()


class SourceCiTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.repo = self.base / "repo"
        self.repo.mkdir()
        self.sources = {"README.md": b"source v1\n", "compiler/src/main.rs": b"fn main() {}\n"}
        self.git("init", "-q")
        for name, value in self.sources.items():
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(value)
        self.git("add", ".")
        self.git("-c", "user.name=Zee test", "-c", "user.email=zee-test@example.invalid",
                 "commit", "-qm", "fixture")
        self.commit = self.git("rev-parse", "HEAD").strip()
        self.zip = self.base / "snapshot.zip"
        self.ci = self.base / "ci.json"
        self.report = self.base / "report.json"
        self.make_zip()
        self.expected_zip_sha = digest(self.zip.read_bytes())
        self.make_ci()

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.repo), *args], check=True,
                              capture_output=True, text=True).stdout

    def make_zip(self, sources=None, manifest=None, comment=None, extra=None):
        sources = self.sources if sources is None else sources
        if manifest is None:
            manifest = "".join(digest(value) + "  " + name + "\n"
                               for name, value in sorted(sources.items())).encode()
        with zipfile.ZipFile(self.zip, "w") as archive:
            archive.comment = (self.commit if comment is None else comment).encode()
            for name, value in sources.items():
                archive.writestr("zee/" + name, value)
            archive.writestr("zee/KAYNAK-SHA256.txt", manifest)
            for name, value in (extra or {}).items():
                archive.writestr(name, value)
        self.expected_zip_sha = digest(self.zip.read_bytes())

    def make_ci(self, commit=None, status="success"):
        self.ci.write_text(json.dumps({"schema": "zee-ci-kaydi-1", "records": [{
            "provider": "fixture", "run_index": 30, "source_sha": commit or self.commit,
            "status": status, "evidence_level": "visible-commit-status"}]}))

    def run_tool(self, report=None, commit=None, expected_zip_sha=None):
        return subprocess.run([sys.executable, str(TOOL), "--depo", str(self.repo),
                               "--sha", commit or self.commit, "--zip", str(self.zip),
                               "--zip-sha256", expected_zip_sha or self.expected_zip_sha,
                               "--ci-kaydi", str(self.ci), "--rapor", str(report or self.report)],
                              capture_output=True, text=True, timeout=15)

    def assert_rejected(self, code):
        result = self.run_tool()
        self.assertEqual(result.returncode, 2, result.stderr + result.stdout)
        self.assertEqual(json.loads(self.report.read_text())["status"], "rejected")
        self.assertEqual(json.loads(self.report.read_text())["error_code"], code)

    def test_exact_archive_zip_manifest_and_ci_record_pass_without_input_mutation(self):
        inputs = {self.zip: self.zip.read_bytes(), self.ci: self.ci.read_bytes(),
                  self.repo / "README.md": (self.repo / "README.md").read_bytes()}
        self.assertEqual(self.run_tool().returncode, 0)
        report = json.loads(self.report.read_text())
        self.assertEqual(report["source_count"], 2)
        self.assertEqual(report["ci_acceptance"], "explicit-recorded-commit-status")
        self.assertFalse(report["network_queried"])
        self.assertFalse(report["runtime_release_acceptance"])
        first_report = self.report.read_bytes()
        self.assertEqual(self.run_tool().returncode, 0)
        self.assertEqual(self.report.read_bytes(), first_report)
        for path, value in inputs.items():
            self.assertEqual(path.read_bytes(), value)

    def test_workspace_change_does_not_replace_exact_archived_source(self):
        (self.repo / "README.md").write_bytes(b"uncommitted work\n")
        self.assertEqual(self.run_tool().returncode, 0)
        self.assertEqual((self.repo / "README.md").read_bytes(), b"uncommitted work\n")

    def test_changed_delivery_zip_sha_rejected(self):
        result = self.run_tool(expected_zip_sha="0" * 64)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(self.report.read_text())["error_code"], "ZIP_SHA256")

    def test_wrong_full_source_sha_rejected(self):
        result = self.run_tool(commit="0" * 40)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(self.report.read_text())["error_code"], "GIT_SOURCE")

    def test_short_source_sha_rejected(self):
        result = self.run_tool(commit=self.commit[:7])
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(self.report.read_text())["error_code"], "SOURCE_SHA")

    def test_wrong_zip_comment_rejected(self):
        self.make_zip(comment="0" * 40)
        self.assert_rejected("ZIP_COMMENT")

    def test_source_bytes_changed_even_with_updated_manifest_rejected(self):
        self.make_zip(sources={**self.sources, "README.md": b"changed source\n"})
        self.assert_rejected("SOURCE_HASH")

    def test_wrong_manifest_hash_rejected(self):
        self.make_zip(manifest=("0" * 64 + "  README.md\n" +
                               digest(self.sources["compiler/src/main.rs"]) +
                               "  compiler/src/main.rs\n").encode())
        self.assert_rejected("SOURCE_HASH")

    def test_missing_source_manifest_record_rejected(self):
        self.make_zip(manifest=(digest(self.sources["README.md"]) + "  README.md\n").encode())
        self.assert_rejected("SOURCE_SET")

    def test_unregistered_corpus_source_path_rejected(self):
        self.make_zip(extra={"zee/dogfood/unregistered.dil": b"\"extra\" yaz\n"})
        self.assert_rejected("SOURCE_SET")

    def test_path_traversal_rejected(self):
        self.make_zip(extra={"zee/../outside.dil": b"no extraction"})
        self.assert_rejected("SOURCE_PATH")

    def test_unknown_extra_metadata_rejected(self):
        self.make_zip(extra={"zee/UNTRUSTED-METADATA.txt": b"not an allowed metadata file"})
        self.assert_rejected("SOURCE_SET")

    def test_duplicate_zip_path_rejected(self):
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            with zipfile.ZipFile(self.zip, "a") as archive:
                archive.writestr("zee/README.md", self.sources["README.md"])
        self.expected_zip_sha = digest(self.zip.read_bytes())
        self.assert_rejected("ZIP_DUPLICATE")

    def test_stale_ci_success_with_identical_compiler_tree_is_not_head_acceptance(self):
        old = self.commit
        (self.repo / "README.md").write_bytes(b"source v2\n")
        self.git("add", "README.md")
        self.git("-c", "user.name=Zee test", "-c", "user.email=zee-test@example.invalid",
                 "commit", "-qm", "documentation-only fixture")
        self.commit = self.git("rev-parse", "HEAD").strip()
        self.sources["README.md"] = b"source v2\n"
        self.make_zip()
        self.make_ci(commit=old)
        self.assertEqual(self.git("rev-parse", old + ":compiler/src"),
                         self.git("rev-parse", self.commit + ":compiler/src"))
        self.assert_rejected("CI_EXACT_SOURCE")

    def test_pending_head_ci_is_not_success(self):
        self.make_ci(status="pending")
        self.assert_rejected("CI_SUCCESS")

    def test_observation_record_keeps_run29_and_exact_run30_separate(self):
        self.ci.write_text(json.dumps({"schema": "zee-source-ci-evidence-1", "ci": {
            "run29": {"evidence_level": "tracked-status", "record": {
                "provider": "fixture", "run_index": 29, "source_sha": "0" * 40,
                "status": "success"}},
            "run30": {"provider": "fixture", "run_index": 30,
                      "head_context_commit": self.commit, "status": "success",
                      "evidence_level": "visible-commit-status"},
            "limitations": ["No downloaded artifacts."]}}))
        self.assertEqual(self.run_tool().returncode, 0)
        records = json.loads(self.report.read_text())["ci_records"]
        self.assertEqual([record["exact_source"] for record in records], [False, True])

    def test_duplicate_ci_json_key_rejected(self):
        self.ci.write_text('{"schema":"zee-ci-kaydi-1","schema":"zee-ci-kaydi-1"}')
        self.assert_rejected("CI_DUPLICATE_KEY")

    def test_conflicting_duplicate_ci_run_rejected(self):
        data = json.loads(self.ci.read_text())
        data["records"].append({**data["records"][0], "status": "pending"})
        self.ci.write_text(json.dumps(data))
        self.assert_rejected("CI_DUPLICATE_RUN")

    def test_duplicate_manifest_path_rejected(self):
        manifest = "".join(digest(value) + "  " + name + "\n"
                           for name, value in sorted(self.sources.items()))
        manifest += digest(self.sources["README.md"]) + "  README.md\n"
        self.make_zip(manifest=manifest.encode())
        self.assert_rejected("MANIFEST_DUPLICATE")

    def test_zip_symlink_source_rejected(self):
        with zipfile.ZipFile(self.zip, "w") as archive:
            archive.comment = self.commit.encode()
            link = zipfile.ZipInfo("zee/README.md")
            link.create_system = 3
            link.external_attr = 0o120777 << 16
            archive.writestr(link, b"outside")
            archive.writestr("zee/KAYNAK-SHA256.txt", b"unused\n")
        self.expected_zip_sha = digest(self.zip.read_bytes())
        self.assert_rejected("ZIP_ENTRY")

    def test_invalid_ci_shape_replaces_previous_pass_with_rejection(self):
        self.assertEqual(self.run_tool().returncode, 0)
        self.ci.write_text("[]")
        self.assert_rejected("CI_SCHEMA")

    def test_report_cannot_overwrite_inputs_or_tracked_source(self):
        for destination in (self.zip, self.ci, self.repo / "README.md"):
            before = destination.read_bytes()
            self.assertEqual(self.run_tool(report=destination).returncode, 2)
            self.assertEqual(destination.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
