#!/usr/bin/env python3
"""Offline selector gates with synthetic authenticated packages and real Ed25519."""
from datetime import datetime, timezone
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

DIRECTORY = Path(__file__).resolve().parent
SELECTOR = DIRECTORY / "derleyici-secici.py"
if not SELECTOR.is_file():
    SELECTOR = DIRECTORY.parent / "derleyici-secici.py"
MODULE = runpy.run_path(str(SELECTOR))
COMPILER_DIR = SELECTOR.parent.parent / "compiler" / "target" / "debug"
EPOCH = 1700000000
DATE = "2023-11-14"
SOURCE = MODULE["SOURCE_URI"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


class Fixture:
    def __init__(self):
        self.temp = tempfile.TemporaryDirectory(prefix="zee-selector-test-")
        self.root = Path(self.temp.name)
        self.store = self.root / "store"
        self.store.mkdir()
        self.tool = self.root / "verifier"
        shutil.copyfile(COMPILER_DIR / "surum_artefakti", self.tool)
        self.tool.chmod(0o755)
        self.tool_sha = sha(self.tool.read_bytes())
        self.key = self.root / "synthetic.zee-anahtar"
        r = subprocess.run([str(COMPILER_DIR / "dil"), "anahtar", "üret", str(self.key)], capture_output=True)
        assert r.returncode == 0, r.stderr
        self.catalog = self.root / "guven.json"
        self.trust = {"schema": "zee-derleyici-guven-1", "publishers": [], "releases": []}
        self.projects = []
        self.platform = MODULE["host_platform"]()

    def sign(self, path):
        result = subprocess.run([str(self.tool), "imzala", "--anahtar", str(self.key), str(path)], capture_output=True)
        assert result.returncode == 0, result.stderr
        signature = json.loads(result.stdout)
        Path(str(path) + ".zee-imza").write_bytes(result.stdout)
        return signature["anahtar_kimligi"]

    def save_catalog(self):
        self.catalog.write_text(json.dumps(self.trust) + "\n")
        self.publisher = self.sign(self.catalog)

    def project(self, name, pin):
        p = self.root / name
        p.mkdir()
        (p / MODULE["PIN_FILE"]).write_text(json.dumps(pin) + "\n")
        self.projects.append(p)
        return p

    def package(self, tag, version="1.0.0", channel="stable", unknown=False, dirty=False, capability="safe",
                source_sha=None, epoch=EPOCH, source_date=DATE):
        commit = source_sha or ("a" if tag == "A" else "b") * 40
        pin = {"schema": "zee-derleyici-pin-1", "version": version, "source_sha": commit,
               "channel": channel, "platform": self.platform}
        bundle = self.store / commit / self.platform
        bundle.mkdir(parents=True)
        build = {"schema": "zee-build-v1", "version": version, "source_sha": "unknown" if unknown else commit,
                 "source_date": "unknown" if unknown else source_date, "channel": channel, "dirty": dirty}
        caps = {"schema": "zee-config-capability-1", "config_schemas": ["zee-ayar-1"],
                "unknown_security": "reject", "version": version}
        if capability == "unsafe":
            caps["unknown_security"] = "warn"
        marker = self.root / f"{tag}-executed"
        main_marker = self.root / f"{tag}-main"
        code = (f"#!{sys.executable}\nimport json,sys\nfrom pathlib import Path\n"
                f"Path({str(marker)!r}).write_text('authenticated code executed')\n"
                f"if sys.argv[1:]==['sürüm','--json']: print(json.dumps({build!r}))\n"
                "elif sys.argv[1:]==['sürüm','--ayar-semasi']:\n")
        if capability == "missing":
            code += "    sys.exit(2)\n"
        else:
            code += f"    print(json.dumps({caps!r}))\n"
        code += f"else:\n    Path({str(main_marker)!r}).write_text('main launched')\n    print({tag!r}+':'+Path.cwd().name+':'+json.dumps(sys.argv[1:]))\n"
        for name in MODULE["binary_names"](pin):
            (bundle / name).write_text(code)
            (bundle / name).chmod(0o755)
        names = sorted(MODULE["binary_names"](pin))
        sums = {name: sha((bundle / name).read_bytes()) for name in names}
        (bundle / "SHA256SUMS").write_text("".join(f"{sums[name]}  {name}\n" for name in names))
        publisher = self.sign(bundle / "SHA256SUMS")
        (bundle / "GIT_SHA").write_text(commit + "\n")
        graph = [{"type": "software_Package", "name": "dil", "software_packageVersion": version,
                  "software_downloadLocation": SOURCE + commit}]
        graph += [{"type": "software_File", "name": name,
                   "verifiedUsing": [{"type": "Hash", "algorithm": "sha256", "hashValue": sums[name]}]} for name in names]
        (bundle / "sbom.spdx.json").write_text(json.dumps({"@context": "https://spdx.org/rdf/3.0.1/spdx-context.jsonld", "@graph": graph}) + "\n")
        provenance = {"_type": "https://in-toto.io/Statement/v1", "predicateType": "https://slsa.dev/provenance/v1",
                      "subject": [{"name": name, "digest": {"sha256": sums[name]}} for name in names],
                      "predicate": {"buildDefinition": {"buildType": "https://zee-lang.dev/surum/tekrar-uretilebilir/v1",
                          "externalParameters": {"gitCommit": commit, "surum": version, "platform": self.platform, "sourceDateEpoch": epoch},
                          "resolvedDependencies": [{"uri": SOURCE + commit, "digest": {"gitCommit": commit}}]},
                        "runDetails": {"byproducts": [{"name": name, "digest": {"sha256": sha((bundle / "SHA256SUMS").read_bytes())}}
                                                     for name in ("SHA256SUMS", "SHA256SUMS.ikinci-klon")]},
                        "zeeTekrarUretim": {"ikiTemizKlonEsit": True}}}
        (bundle / "provenance.intoto.json").write_text(json.dumps(provenance) + "\n")
        record = {key: value for key, value in pin.items() if key != "schema"}
        record.update(publisher=publisher, file_hashes={name: sha((bundle / name).read_bytes()) for name in MODULE["META"]})
        if publisher not in self.trust["publishers"]:
            self.trust["publishers"].append(publisher)
        self.trust["releases"].append(record)
        self.save_catalog()
        return pin, bundle, record

    def allow_sidecar(self, bundle, record, name):
        record["file_hashes"][name] = sha((bundle / name).read_bytes())
        self.save_catalog()

    def args(self, project, command="seç", pilot=False):
        result = [sys.executable, "-I", str(SELECTOR), command, "--proje", str(project), "--depo", str(self.store),
                  "--guven", str(self.catalog), "--guven-anahtari", self.publisher,
                  "--dogrulayici", str(self.tool), "--dogrulayici-sha256", self.tool_sha]
        if pilot:
            result.append("--pilot")
        return result

    def execute(self, project, command="seç", pilot=False, extra=()):
        return subprocess.run(self.args(project, command, pilot) + list(extra), capture_output=True, text=True)


class SelectorTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()

    def tearDown(self):
        self.f.temp.cleanup()

    def rejected(self, result, text):
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn(text, result.stderr)

    def test_two_projects_exact_routing_and_explicit_rollback(self):
        a, _, _ = self.f.package("A")
        b, _, _ = self.f.package("B")
        left, right = self.f.project("left", a), self.f.project("right", b)
        for project, label in [(left, "A:left:"), (right, "B:right:")]:
            result = self.f.execute(project, "çalıştır", extra=("--", "denetle"))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(result.stdout.startswith(label), result.stdout)
            self.assertIn(str(project), result.stdout)
        extra = ("--surum", b["version"], "--sha", b["source_sha"], "--kanal", b["channel"])
        self.rejected(self.f.execute(left, "pinle", extra=extra), "--degistir")
        self.assertEqual(self.f.execute(left, "pinle", extra=extra + ("--degistir",)).returncode, 0)
        self.assertTrue(self.f.execute(left, "çalıştır", extra=("--", "dene")).stdout.startswith("B:left:"))

    def test_missing_pin_and_missing_exact_bundle_have_no_fallback(self):
        a, _, _ = self.f.package("A")
        b, bundle, _ = self.f.package("B")
        project = self.f.project("project", b)
        shutil.rmtree(bundle)
        self.rejected(self.f.execute(project), "exact paket yok")
        self.assertFalse((self.f.root / "A-executed").exists())
        (project / MODULE["PIN_FILE"]).unlink()
        self.rejected(self.f.execute(project), "fallback yok")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_metadata_cannot_supply_executable_or_publisher(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        for field in ("path", "command", "publisher", "verifier"):
            invalid = dict(pin, **{field: "/usr/bin/anything"})
            (project / MODULE["PIN_FILE"]).write_text(json.dumps(invalid))
            self.rejected(self.f.execute(project), "exact beş alan")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_binary_checksum_rejects_before_any_candidate_code(self):
        pin, bundle, _ = self.f.package("A")
        project = self.f.project("project", pin)
        (bundle / "dil").write_bytes((bundle / "dil").read_bytes() + b"\n# modified\n")
        self.rejected(self.f.execute(project), "binary SHA-256 farklı")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_sidecar_catalog_binding_and_metadata_consistency(self):
        pin, bundle, record = self.f.package("A")
        project = self.f.project("project", pin)
        sbom = json.loads((bundle / "sbom.spdx.json").read_text())
        sbom["@graph"][0]["software_packageVersion"] = "9.9.9"
        (bundle / "sbom.spdx.json").write_text(json.dumps(sbom))
        self.rejected(self.f.execute(project), "trusted sidecar hash farklı")
        self.f.allow_sidecar(bundle, record, "sbom.spdx.json")
        self.rejected(self.f.execute(project), "SBOM sürüm/kaynak pin ile farklı")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_corrupt_crypto_signature_even_with_authenticated_catalog(self):
        pin, bundle, record = self.f.package("A")
        project = self.f.project("project", pin)
        signature = json.loads((bundle / "SHA256SUMS.zee-imza").read_text())
        encoded = signature["ed25519"]
        signature["ed25519"] = f"{int(encoded[:2],16) ^ 255:02x}" + encoded[2:]
        (bundle / "SHA256SUMS.zee-imza").write_text(json.dumps(signature))
        self.f.allow_sidecar(bundle, record, "SHA256SUMS.zee-imza")
        self.rejected(self.f.execute(project), "Ed25519 imza doğrulanamadı")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_untrusted_catalog_key_or_modified_verifier_rejected(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        command = self.f.args(project)
        command[command.index("--guven-anahtari") + 1] = "sha256:" + "0" * 64
        self.rejected(subprocess.run(command, capture_output=True, text=True), "imza içerik/yayıncı hash farklı")
        self.f.tool.write_bytes(self.f.tool.read_bytes() + b"changed")
        self.rejected(self.f.execute(project), "verifier hash farklı")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_duplicate_keys_and_release_records_rejected(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        raw = json.dumps(pin)[:-1] + ',"channel":"stable"}'
        (project / MODULE["PIN_FILE"]).write_text(raw)
        self.rejected(self.f.execute(project), "yinelenen JSON alanı")
        (project / MODULE["PIN_FILE"]).write_text(json.dumps(pin))
        self.f.trust["releases"].append(self.f.trust["releases"][0])
        self.f.save_catalog()
        self.rejected(self.f.execute(project), "yinelenen exact release")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_symlink_and_checksum_path_escape_rejected(self):
        pin, bundle, record = self.f.package("A")
        project = self.f.project("project", pin)
        saved = self.f.root / "saved"
        (bundle / "dil").rename(saved)
        (bundle / "dil").symlink_to(saved)
        self.rejected(self.f.execute(project), "düzenli gerçek dosya")
        (bundle / "dil").unlink()
        saved.rename(bundle / "dil")
        (bundle / "SHA256SUMS").write_text("0" * 64 + "  ../escape\n")
        self.f.sign(bundle / "SHA256SUMS")
        for name in ("SHA256SUMS", "SHA256SUMS.zee-imza"):
            record["file_hashes"][name] = sha((bundle / name).read_bytes())
        self.f.save_catalog()
        self.rejected(self.f.execute(project), "SHA256SUMS yolu/hash")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_wrong_platform_and_false_channel_rejected(self):
        pin, _, record = self.f.package("A")
        project = self.f.project("project", pin)
        wrong = dict(pin, platform=next(value for value in MODULE["PLATFORMS"] if value != pin["platform"]))
        (project / MODULE["PIN_FILE"]).write_text(json.dumps(wrong))
        self.rejected(self.f.execute(project), "başka platform")
        pin["channel"] = record["channel"] = "beta"
        self.f.save_catalog()
        (project / MODULE["PIN_FILE"]).write_text(json.dumps(pin))
        self.rejected(self.f.execute(project), "compiler sürüm/kanal pin ile farklı")
        self.assertFalse((self.f.root / "A-main").exists())

    def test_development_unknown_requires_explicit_pilot(self):
        pin, _, _ = self.f.package("A", version="0.8.0-dev", channel="development", unknown=True, dirty=True)
        project = self.f.project("project", pin)
        self.rejected(self.f.execute(project), "explicit --pilot")
        self.assertFalse((self.f.root / "A-executed").exists())
        result = self.f.execute(project, pilot=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["authenticated_build"]["source_sha"], "unknown")

    def test_pilot_does_not_promote_unknown_or_dirty_to_stable(self):
        pin, _, _ = self.f.package("A", unknown=True, dirty=True)
        project = self.f.project("project", pin)
        self.rejected(self.f.execute(project, pilot=True), "kararlı kanal unknown/kirli")
        self.assertFalse((self.f.root / "A-main").exists())

    def test_config_security_capability_gate_cannot_be_bypassed_by_pilot(self):
        pin, _, _ = self.f.package("A", capability="missing")
        project = self.f.project("project", pin)
        (project / "zee.ayar").write_text('schema = "zee-ayar-1"\n')
        for pilot in (False, True):
            self.rejected(self.f.execute(project, "çalıştır", pilot, extra=("--", "denetle")), "capability sorgusu başarısız")
        self.assertFalse((self.f.root / "A-main").exists())

    def test_safe_capability_accepts_and_security_warn_capability_rejects(self):
        good, _, _ = self.f.package("A", capability="safe")
        bad, _, _ = self.f.package("B", capability="unsafe")
        a, b = self.f.project("safe", good), self.f.project("unsafe", bad)
        for p in (a, b):
            (p / "zee.ayar").write_text('schema = "zee-ayar-1"\n')
        self.assertEqual(self.f.execute(a, "çalıştır", extra=("--", "denetle")).returncode, 0)
        self.rejected(self.f.execute(b, "çalıştır", extra=("--", "denetle")), "fail-closed güvenlik yetkinliği")
        self.assertFalse((self.f.root / "B-main").exists())

    def test_original_bundle_swap_cannot_change_authenticated_snapshot(self):
        pin, bundle, _ = self.f.package("A")
        project = self.f.project("project", pin)
        with MODULE["authenticated_snapshot"](pin, self.f.store, self.f.catalog, self.f.publisher,
                self.f.tool, self.f.tool_sha, False, project) as (snapshot, _):
            before = (snapshot / "dil").read_bytes()
            (bundle / "dil").write_text("untrusted replacement")
            self.assertEqual((snapshot / "dil").read_bytes(), before)
            result = subprocess.run([str(snapshot / "dil"), "denetle", str(project)], cwd=project, env=MODULE["safe_environment"](), capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(result.stdout.startswith("A:project:"))

    def test_cannot_target_another_project_or_use_duplicate_selector_option(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        self.rejected(self.f.execute(project, "çalıştır", extra=("--", "denetle", str(self.f.root))), "başka proje yolu")
        self.rejected(self.f.execute(project, extra=("--proje", str(project))), "yinelenen seçici")
        self.assertFalse((self.f.root / "A-main").exists())

    def test_real_local_debug_pair_authenticates_and_checks_its_own_project(self):
        compiler = COMPILER_DIR / "dil"
        probe = subprocess.run([str(compiler), "sürüm", "--json"], capture_output=True, text=True)
        self.assertEqual(probe.returncode, 0, probe.stderr)
        build = json.loads(probe.stdout)
        self.assertEqual(build["schema"], "zee-build-v1")
        date = build["source_date"] if build["source_date"] != "unknown" else DATE
        epoch = int(datetime.strptime(date, "%Y-%m-%d").replace(tzinfo=timezone.utc).timestamp())
        commit = build["source_sha"] if build["source_sha"] != "unknown" else "a" * 40
        pin, bundle, record = self.f.package("A", version=build["version"], channel=build["channel"],
                source_sha=commit, source_date=date, epoch=epoch)
        for name in MODULE["binary_names"](pin):
            shutil.copyfile(COMPILER_DIR / name, bundle / name)
        sums = {name: sha((bundle / name).read_bytes()) for name in MODULE["binary_names"](pin)}
        (bundle / "SHA256SUMS").write_text("".join(f"{sums[name]}  {name}\n" for name in sorted(sums)))
        self.f.sign(bundle / "SHA256SUMS")
        sbom = json.loads((bundle / "sbom.spdx.json").read_text())
        for node in sbom["@graph"]:
            if node["type"] == "software_File":
                node["verifiedUsing"][0]["hashValue"] = sums[node["name"]]
        (bundle / "sbom.spdx.json").write_text(json.dumps(sbom))
        provenance = json.loads((bundle / "provenance.intoto.json").read_text())
        for subject in provenance["subject"]:
            subject["digest"] = {"sha256": sums[subject["name"]]}
        for byproduct in provenance["predicate"]["runDetails"]["byproducts"]:
            byproduct["digest"] = {"sha256": sha((bundle / "SHA256SUMS").read_bytes())}
        (bundle / "provenance.intoto.json").write_text(json.dumps(provenance))
        record["file_hashes"] = {name: sha((bundle / name).read_bytes()) for name in MODULE["META"]}
        self.f.save_catalog()
        created = subprocess.run([str(compiler), "yeni", "real-project"], cwd=self.f.root, capture_output=True, text=True)
        self.assertEqual(created.returncode, 0, created.stderr)
        project = self.f.root / "real-project"
        (project / MODULE["PIN_FILE"]).write_text(json.dumps(pin))
        result = self.f.execute(project, "çalıştır", pilot=True, extra=("--", "denetle"))
        self.assertEqual(result.returncode, 0, result.stderr)
        # A real local debug copy is not a two-clean-clone released artifact.
        # Synthetic signed sidecars test the protocol, not release provenance truth.

    def test_hardening_deep_pin_json_masks_recursion_and_executes_no_candidate(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        (project / MODULE["PIN_FILE"]).write_bytes(b'[' * 10000 + b'0' + b']' * 10000)
        result = self.f.execute(project)
        self.rejected(result, "geçersiz UTF-8/JSON")
        self.assertFalse("Traceback" in result.stderr)
        self.assertFalse("RecursionError" in result.stderr)
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_hardening_value_errors_duplicate_key_and_wrong_type_masks(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        path = project / MODULE["PIN_FILE"]
        for raw in [b'{"n":' + b'9' * 5000 + b'}',
                    b'{"SENTINEL_SECRET_KEY":1,"SENTINEL_SECRET_KEY":2}']:
            path.write_bytes(raw)
            result = self.f.execute(project)
            self.assertEqual(result.returncode, 2)
            self.assertFalse("Traceback" in result.stderr)
            self.assertFalse("SENTINEL_SECRET" in result.stderr)
        for field in ["channel", "platform"]:
            for value in [{}, [], None, True]:
                path.write_text(json.dumps(dict(pin, **{field: value})))
                result = self.f.execute(project)
                self.rejected(result, "pin kanal/platform")
                self.assertFalse("Traceback" in result.stderr)
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_hardening_trusted_json_container_types_reject_before_candidate(self):
        pin, bundle, record = self.f.package("A")
        project = self.f.project("project", pin)
        publisher = list(self.f.trust["publishers"])
        for value in [None, {}, [{}], [publisher]]:
            self.f.trust["publishers"] = value
            self.f.save_catalog()
            result = self.f.execute(project)
            self.assertEqual(result.returncode, 2)
            self.assertFalse("Traceback" in result.stderr)
        self.f.trust["publishers"] = publisher
        self.f.save_catalog()
        original = (bundle / "provenance.intoto.json").read_text()
        for field, value in [("predicate", []), ("subject", [[], {}])]:
            data = json.loads(original)
            data[field] = value
            (bundle / "provenance.intoto.json").write_text(json.dumps(data))
            self.f.allow_sidecar(bundle, record, "provenance.intoto.json")
            result = self.f.execute(project)
            self.assertEqual(result.returncode, 2)
            self.assertFalse("Traceback" in result.stderr)
        self.assertFalse((self.f.root / "A-executed").exists())

    @unittest.skipUnless(hasattr(os, "mkfifo"), "POSIX no-writer FIFO probe")
    def test_hardening_post_lstat_fifo_swap_is_nonblocking_and_never_read(self):
        path = self.f.root / "regular.bin"
        path.write_bytes(b"ordinary")
        code = """import runpy,sys,os
from pathlib import Path
m=runpy.run_path(sys.argv[1]);p=Path(sys.argv[2]);original=os.open
def swap(path,flags,*a,**k):
    p.unlink();os.mkfifo(p);print('before-open',flush=True)
    return original(path,flags,*a,**k)
os.open=swap
try:
    m['regular_bytes'](p)
except m['Reject']:
    print('rejected-regular',flush=True);sys.exit(2)
sys.exit(7)
"""
        result = subprocess.run([sys.executable, "-I", "-c", code, str(SELECTOR), str(path)],
                                capture_output=True, text=True, timeout=3)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(result.stdout.splitlines(), ["before-open", "rejected-regular"])
        self.assertEqual(result.stderr, "")

    def test_hardening_role_limits_reject_oversize_before_open_and_accept_boundary(self):
        path = self.f.root / "bounded.bin"
        expected = {"pin": 65536, "config": 65536, "signature": 65536, "catalog": 2 * 1024 * 1024,
                    "sbom": 4 * 1024 * 1024, "provenance": 16 * 1024 * 1024, "executable": 64 * 1024 * 1024}
        for role, limit in expected.items():
            self.assertEqual(MODULE["LIMITS"][role], limit)
            with path.open("wb") as stream:
                stream.truncate(limit + 1)  # sparse synthetic file, no huge allocation
            with mock.patch.object(MODULE["os"], "open", side_effect=AssertionError("oversize was opened")):
                with self.assertRaises(MODULE["Reject"]):
                    MODULE["regular_bytes"](path, limit)
        path.write_bytes(b'x' * 65536)
        self.assertEqual(len(MODULE["regular_bytes"](path, MODULE["LIMITS"]["pin"])), 65536)
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        (project / MODULE["PIN_FILE"]).write_bytes(b' ' * 65537)
        self.rejected(self.f.execute(project), "dosya sınırı")
        self.assertFalse((self.f.root / "A-executed").exists())

    def test_hardening_loader_environment_controls_removed_application_values_preserved(self):
        controls = ["LD_PRELOAD", "LD_LIBRARY_PATH", "LD_AUDIT", "LD_DEBUG", "LD_DEBUG_OUTPUT",
                    "DYLD_INSERT_LIBRARIES", "DYLD_LIBRARY_PATH", "PYTHONPATH", "PYTHONHOME",
                    "PYTHONINSPECT", "PYTHONSTARTUP", "PYTHONUSERBASE", "PYTHONSAFEPATH", "PYTHONWARNINGS"]
        additions = {key: "synthetic-control" for key in controls}
        additions["ZEE_SYNTHETIC_APP_SETTING"] = "ordinary-value"
        with mock.patch.dict(os.environ, additions):
            clean = MODULE["safe_environment"]()
            for key in controls:
                self.assertFalse(key in clean, "loader control retained: " + key)
            self.assertEqual(clean["ZEE_SYNTHETIC_APP_SETTING"], "ordinary-value")
            self.assertTrue(clean.get("PATH") == os.environ.get("PATH"), "operator PATH is an explicit residual trust prerequisite")
        # This is environment filtering evidence; no Linux LD_AUDIT exploit ran.

    def test_hardening_global_host_flags_cannot_be_program_tail_ordinary_args_survive(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        flags = ["--güvenli", "--guvenli", "--deneysel-web", "--web-proxy", "--güvenli-web-proxy",
                 "--web-worker-port", "--web-oran-deposu", "--web-oran-siniri", "--web-oran-kapasitesi",
                 "--web-oran-doluluk", "--web-oran-future", "--web-proxy=https://synthetic.invalid"]
        for flag in flags:
            with self.assertRaises(MODULE["Reject"]):
                MODULE["scoped_arguments"](["çalıştır", flag], project)
        self.rejected(self.f.execute(project, "çalıştır", extra=("--", "çalıştır", "--deneysel-web")), "global host/güvenlik")
        self.assertFalse((self.f.root / "A-main").exists())
        ordinary = ["--foo", "--json", "ordinary-value", "--uygulama-secimi"]
        self.assertEqual(MODULE["scoped_arguments"](["çalıştır"] + ordinary, project), ["çalıştır", str(project)] + ordinary)
        result = self.f.execute(project, "çalıştır", extra=("--", "çalıştır", *ordinary))
        self.assertEqual(result.returncode, 0, result.stderr)
        for value in ordinary:
            self.assertTrue(value in result.stdout)

    def test_hardening_fsync_failure_cleans_pin_temp_and_preserves_current_pin(self):
        pin, _, _ = self.f.package("A")
        project = self.f.project("project", pin)
        before = (project / MODULE["PIN_FILE"]).read_bytes()
        with mock.patch.object(MODULE["os"], "fsync", side_effect=OSError("synthetic fsync failure")):
            with self.assertRaises(OSError):
                MODULE["write_pin"](project, pin, True)
        self.assertEqual(list(project.glob(".zee-pin-*")), [])
        self.assertEqual((project / MODULE["PIN_FILE"]).read_bytes(), before)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--compiler-dir", type=Path, default=COMPILER_DIR)
    options, tests = parser.parse_known_args()
    COMPILER_DIR = options.compiler_dir.resolve()
    unittest.main(argv=[sys.argv[0]] + tests, verbosity=2)
