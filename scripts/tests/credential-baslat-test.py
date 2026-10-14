#!/usr/bin/env python3
"""K-245: gerçek POSIX dosya/çocuk ve mevcut native Zee yetki kampanyası."""
import base64
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import threading
import unittest
import signal
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/credential-baslat.py"
sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("credential_baslat", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
PUBLIC_A = "d3-public-fixture-a"
PUBLIC_B = "d3-public-fixture-b"
ENV = "ZEE_CREDENTIAL_FIXTURE"
SECOND_ENV = "ZEE_SECOND"


class CredentialTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="zee-credential-", dir=Path(tempfile.gettempdir()).resolve())
        self.root = Path(self.temp.name)
        self.credentials = self.root / "credentials"
        self.credentials.mkdir(mode=0o700)
        self.value = self.credentials / "secret"
        self.write_value(PUBLIC_A)
        self.marker = self.root / "child.json"
        self.child = self.root / "child"
        self.child.write_text("#!" + sys.executable + "\nimport json,os,sys\nfrom pathlib import Path\n"
            "Path(sys.argv[2]).write_text(json.dumps({'argv':sys.argv[1:],'value':os.environ['ZEE_CREDENTIAL_FIXTURE']}))\n"
            "print(os.environ['ZEE_CREDENTIAL_FIXTURE'])\nprint(os.environ['ZEE_CREDENTIAL_FIXTURE'],file=sys.stderr)\n"
            "sys.exit(int(sys.argv[3]) if len(sys.argv)>3 else 0)\n")
        self.child.chmod(0o700)
        self.environment = {key: value for key, value in os.environ.items() if key != ENV}
        self.original_fixture = (ENV in os.environ, os.environ.get(ENV))
        self.dil = Path(os.environ.get("ZEE_CREDENTIAL_TEST_DIL", str(ROOT / "compiler/target/debug/dil"))).resolve()

    def tearDown(self):
        self.temp.cleanup()
        self.assertFalse(self.root.exists())

    def write_value(self, value):
        self.value.write_bytes(value.encode("utf-8") if isinstance(value, str) else value)
        self.value.chmod(0o600)

    def arguments(self, bindings=None, child=None, command=None, root=None):
        result = ["--credential-root", str(root or self.credentials), "--dil", str(child or self.child)]
        for binding in bindings or [ENV + "=secret"]:
            result += ["--env", binding]
        return result + ["--"] + (command or ["calistir", str(self.marker)])

    def invoke(self, **options):
        environment = options.pop("environment", self.environment)
        process = subprocess.Popen([sys.executable, "-I", "-B", str(SCRIPT)] + self.arguments(**options),
            env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
        try:
            stdout, stderr = process.communicate(timeout=15)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
            self.fail("Sahipli child-group 15 saniyelik fixture hang guard'ını aştı")
        result = subprocess.CompletedProcess(process.args, process.returncode, stdout, stderr)
        metadata = json.loads(result.stdout)
        self.assertEqual(set(metadata), {"schema", "source", "exit"})
        self.assertEqual(metadata["schema"], "zee-credential-baslat-1")
        self.assertEqual(metadata["exit"], result.returncode)
        self.assertEqual(result.stderr, "")
        for secret in [PUBLIC_A, PUBLIC_B, str(self.credentials), str(self.marker)]:
            self.assertNotIn(secret, result.stdout + result.stderr)
        return result, metadata

    def rejected(self, **options):
        result, metadata = self.invoke(**options)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(metadata["source"], "launcher")
        self.assertFalse(self.marker.exists(), "Hiçbir başarısız credential çocuğu başlatamaz")

    def test_actual_child_value_argv_exit_and_echo_are_metadata_only(self):
        before = self.value.stat()
        result, metadata = self.invoke(command=["calistir", str(self.marker), "7", "Türkçe", "[literal]", "--uygulama-arg"])
        self.assertEqual(result.returncode, 7)
        self.assertEqual(metadata["source"], "child")
        actual = json.loads(self.marker.read_text())
        self.assertEqual(actual["value"], PUBLIC_A)
        self.assertEqual(actual["argv"], ["calistir", str(self.marker), "7", "Türkçe", "[literal]", "--uygulama-arg"])
        after = self.value.stat()
        self.assertEqual((before.st_mode, before.st_mtime_ns, before.st_size), (after.st_mode, after.st_mtime_ns, after.st_size))
        self.assertEqual(self.value.read_text(), PUBLIC_A)
        self.assertTrue((ENV in os.environ, os.environ.get(ENV)) == self.original_fixture,
                        "Caller ortamı değiştirilmemeli")

    def test_missing_file_and_ambient_collision_have_no_fallback_or_child(self):
        self.value.unlink()
        self.rejected()
        self.write_value(PUBLIC_A)
        self.rejected(environment=dict(self.environment, **{ENV: PUBLIC_B}))

    def test_root_and_file_modes_and_owner_fail_before_child(self):
        for mode in [0o644, 0o640, 0o666, 0o700, 0o000]:
            with self.subTest(file_mode=mode):
                self.value.chmod(mode)
                self.rejected()
        self.value.chmod(0o600)
        for mode in [0o755, 0o750, 0o777]:
            with self.subTest(root_mode=mode):
                self.credentials.chmod(mode)
                self.rejected()
        self.credentials.chmod(0o700)
        wrong = self.value.stat()
        values = list(wrong)
        values[4] = os.geteuid() + 1 if os.geteuid() else 12345
        with self.assertRaises(MODULE.Rejected):
            MODULE.private_file(os.stat_result(values))
        for file_mode, root_mode in [(0o400, 0o500), (0o600, 0o700)]:
            self.value.chmod(file_mode)
            self.credentials.chmod(root_mode)
            result, _ = self.invoke()
            self.assertEqual(result.returncode, 0)
            self.marker.unlink()
        self.credentials.chmod(0o700)

    def test_symlink_component_final_file_hardlink_and_fifo_fail_closed(self):
        original = self.credentials / "original"
        self.value.rename(original)
        self.value.symlink_to(original)
        self.rejected()
        self.value.unlink()
        os.link(original, self.value)
        self.rejected()
        self.value.unlink()
        os.mkfifo(self.value, 0o600)
        self.rejected()
        self.value.unlink()
        original.rename(self.value)
        alias = self.root / "alias"
        alias.symlink_to(self.credentials, target_is_directory=True)
        self.rejected(root=alias)
        nested = self.root / "nested"
        nested.mkdir(mode=0o700)
        ancestor = self.root / "ancestor"
        ancestor.symlink_to(self.root, target_is_directory=True)
        self.rejected(root=ancestor / "nested")

    def test_text_and_file_limits_are_bounded_without_trimming(self):
        for value in [b"", b"x" * (MODULE.MAX_FILE_BYTES + 1), b"\xff", b"\xef\xbb\xbfx", b"x\x00y", b"x\ny", b"x\r\ny"]:
            with self.subTest(size=len(value)):
                self.write_value(value)
                self.rejected()
        self.write_value(" " + PUBLIC_A + " ")
        result, _ = self.invoke()
        self.assertEqual(result.returncode, 0)
        self.assertEqual(json.loads(self.marker.read_text())["value"], " " + PUBLIC_A + " ")

    def test_exact_file_and_aggregate_size_boundaries(self):
        self.write_value(b"x" * MODULE.MAX_FILE_BYTES)
        result, _ = self.invoke()
        self.assertEqual(result.returncode, 0)
        self.marker.unlink()
        extra = self.credentials / "extra"
        extra.write_text("x")
        extra.chmod(0o600)
        self.rejected(bindings=[ENV + "=secret", "ZEE_CREDENTIAL_EXTRA=extra"])

    def test_mapping_limit_duplicates_reserved_names_and_traversal(self):
        for binding in ["PATH=secret", "HOME=secret", "LD_PRELOAD=secret", "DYLD_INSERT_LIBRARIES=secret", "PYTHONPATH=secret", "CREDENTIALS_DIRECTORY=secret", "CODEX_HOME=secret", "RUSTUP_TOOLCHAIN=secret", "ZEE_OK=../secret", "ZEE_OK=/secret", "ZEE_OK=secret/name", "ZEE_OK=..", "bad=secret", "ZEE_OK=secret=extra"]:
            with self.subTest(binding=binding):
                self.rejected(bindings=[binding])
        for bindings in [[ENV + "=secret", ENV + "=other"], [ENV + "=secret", SECOND_ENV + "=secret"], [f"ZEE_{i}=file{i}" for i in range(17)]]:
            self.rejected(bindings=bindings)
        bindings = []
        for i in range(16):
            file = self.credentials / ("file" + str(i))
            file.write_text("x")
            file.chmod(0o600)
            bindings.append(f"ZEE_{i}=file{i}")
        with mock.patch.dict(os.environ, self.environment, clear=True):
            root, actual, command = MODULE.parse_arguments(self.arguments(bindings=bindings))
            environment = MODULE.child_environment(root, actual)
            self.assertEqual([environment[f"ZEE_{i}"] for i in range(16)], ["x"] * 16)
            self.assertEqual(command[0], str(self.child))

    def test_cli_errors_and_unsupported_platform_never_print_arguments(self):
        for args in [[], ["--secret-value", PUBLIC_A], self.arguments()[:-1], self.arguments() + ["--unknown-application-option"]]:
            if args and args[-1] == "--unknown-application-option":
                # Uygulama argv'si aynen taşınır; bilinmeyen launcher seçeneği ayraç öncesinde reddedilir.
                args = ["--unknown", PUBLIC_A] + self.arguments()
            result = subprocess.run([sys.executable, "-I", str(SCRIPT)] + args, env=self.environment, capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stderr, "")
            self.assertNotIn(PUBLIC_A, result.stdout)
            self.assertFalse(self.marker.exists())
        with mock.patch.object(MODULE.os, "name", "nt"), mock.patch.object(MODULE, "parse_arguments") as parse, contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(MODULE.main(self.arguments()), 2)
            parse.assert_not_called()
            self.assertEqual(json.loads(output.getvalue())["source"], "launcher")

    def test_replace_by_fifo_between_stat_and_open_is_nonblocking(self):
        # Önce gerçek regular stat, sonra writersız gerçek FIFO; ayrı süreç hang guard'ı.
        probe = '''import contextlib,importlib.util,io,json,os,sys
from pathlib import Path
from unittest import mock
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location("credential_fifo_probe",sys.argv[1])
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
value=Path(sys.argv[2]);args=json.loads(sys.argv[3]);actual_open=os.open;changed=False
def swap(path,flags,**kwargs):
 global changed
 if path=="secret" and not changed:
  changed=True;value.unlink();os.mkfifo(value,0o600)
 return actual_open(path,flags,**kwargs)
with mock.patch.object(module.os,"open",side_effect=swap),mock.patch.object(module.subprocess,"Popen") as child,contextlib.redirect_stdout(io.StringIO()):
 assert module.main(args)==2
 child.assert_not_called()
assert changed
'''
        result = subprocess.run([sys.executable, "-I", "-B", "-c", probe, str(SCRIPT), str(self.value), json.dumps(self.arguments())], env=self.environment, capture_output=True, text=True, timeout=3)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertFalse(self.marker.exists())

    def test_read_mutation_and_later_binding_mutation_reject_snapshot(self):
        actual_read = os.read
        changed = False
        def truncate(fd, size):
            nonlocal changed
            data = actual_read(fd, size)
            if not changed:
                changed = True
                self.value.write_text("x")
            return data
        with mock.patch.object(MODULE.os, "read", side_effect=truncate), mock.patch.dict(os.environ, self.environment, clear=True), mock.patch.object(MODULE.subprocess, "Popen") as child, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(MODULE.main(self.arguments()), 2)
            child.assert_not_called()
        self.assertTrue(changed)
        self.write_value(PUBLIC_A)
        second = self.credentials / "second"
        second.write_text(PUBLIC_B)
        second.chmod(0o600)
        actual_load = MODULE.read_credential
        def later(root_fd, name):
            if name == "second":
                self.value.write_text(PUBLIC_B)
            return actual_load(root_fd, name)
        with mock.patch.object(MODULE, "read_credential", side_effect=later), mock.patch.dict(os.environ, self.environment, clear=True), mock.patch.object(MODULE.subprocess, "Popen") as child, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(MODULE.main(self.arguments(bindings=[ENV + "=secret", SECOND_ENV + "=second"])), 2)
            child.assert_not_called()

    def test_root_replacement_and_all_before_child_close_owned_descriptors(self):
        old = self.root / "old"
        actual_load = MODULE.read_credential
        def replace(root_fd, name):
            value = actual_load(root_fd, name)
            self.credentials.rename(old)
            self.credentials.mkdir(mode=0o700)
            self.write_value(PUBLIC_B)
            return value
        actual_open, actual_close = os.open, os.close
        owned = set()
        def opened(*args, **kwargs):
            fd = actual_open(*args, **kwargs)
            owned.add(fd)
            return fd
        def closed(fd):
            actual_close(fd)
            owned.remove(fd)
        with mock.patch.object(MODULE, "read_credential", side_effect=replace), mock.patch.object(MODULE.os, "open", side_effect=opened), mock.patch.object(MODULE.os, "close", side_effect=closed), mock.patch.dict(os.environ, self.environment, clear=True), mock.patch.object(MODULE.subprocess, "Popen") as child, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(MODULE.main(self.arguments()), 2)
            child.assert_not_called()
        self.assertEqual(owned, set())

    def test_restart_reads_new_file_while_existing_child_keeps_original_value(self):
        ready, release, old_value = [self.root / name for name in ["ready", "release", "old-value"]]
        self.child.write_text("#!" + sys.executable + "\nimport os,sys,time\nfrom pathlib import Path\n"
            "value=os.environ['ZEE_CREDENTIAL_FIXTURE']\nPath(sys.argv[2]).write_text('ready')\n"
            "until=time.monotonic()+10\n"
            "while not Path(sys.argv[3]).exists():\n    if time.monotonic()>until: sys.exit(9)\n    time.sleep(.01)\n"
            "Path(sys.argv[4]).write_text(value)\n")
        process = subprocess.Popen([sys.executable, "-I", str(SCRIPT)] + self.arguments(command=["calistir", str(ready), str(release), str(old_value)]), env=self.environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            import time
            until = time.monotonic() + 5
            while not ready.exists():
                self.assertIsNone(process.poll())
                self.assertLess(time.monotonic(), until, "Çocuk hazır bariyeri gelmedi")
                time.sleep(.01)
            self.write_value(PUBLIC_B)
            release.write_text("release")
            stdout, stderr = process.communicate(timeout=15)
            self.assertEqual(process.returncode, 0)
            self.assertEqual(stderr, "")
            self.assertEqual(json.loads(stdout)["source"], "child")
            self.assertEqual(old_value.read_text(), PUBLIC_A)
            self.assertNotIn(PUBLIC_A, stdout)
            # İkinci gerçek launch yeni dosyayı okur, ilk çocuğun ortamı değiştirilmez.
            self.child.write_text("#!" + sys.executable + "\nimport os,sys\nfrom pathlib import Path\nPath(sys.argv[2]).write_text(os.environ['ZEE_CREDENTIAL_FIXTURE'])\n")
            result, _ = self.invoke()
            self.assertEqual(result.returncode, 0)
            self.assertEqual(self.marker.read_text(), PUBLIC_B)
        finally:
            if process.poll() is None:
                release.write_text("cleanup")
                process.communicate(timeout=15)

    def native_project(self, source, permitted=True):
        project = self.root / "project"
        project.mkdir(exist_ok=True)
        (project / "ana.dil").write_text(source)
        (project / "proje.dil").write_text('proje "credential-fixture" olsun\nsürüm "0.1.0" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ" listesi olsun\nağ_hedefleri "https://service.example" listesi olsun\n' + (f'http_gizli_değişkenleri "{ENV}" listesi olsun\n' if permitted else ""))
        self.assertTrue(self.dil.is_file(), "Gerçek native dil ikilisi gereklidir; bu vaka skip/canned değildir")
        return project

    def test_actual_native_dil_preserves_main_project_env_authority(self):
        source = f'sonuc "{ENV}" gizli ortam değerini okumayı dene olsun\nsonuc başarılıysa\n    deger sonucun değeri olsun\n    deger "{PUBLIC_A}" e eşitse\n        deger yaz\n        programı 0 ile bitir\nprogramı 7 ile bitir\n'
        project = self.native_project(source)
        result, metadata = self.invoke(child=self.dil, command=["çalıştır", str(project)])
        self.assertEqual(result.returncode, 0)
        self.assertEqual(metadata["source"], "child")
        self.native_project(source, permitted=False)
        result, metadata = self.invoke(child=self.dil, command=["çalıştır", str(project)])
        self.assertEqual(result.returncode, 7, "Host köprüsü Zee izin listesini açamaz")
        self.assertEqual(metadata["source"], "child")

    def test_actual_native_smtp_consumes_file_url_and_auth_once(self):
        listener = socket.socket()
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(10)
        port = listener.getsockname()[1]
        state = {"connections": 0, "auth": 0, "data": 0, "error": None}
        def server():
            try:
                connection, _ = listener.accept()
                state["connections"] += 1
                with connection:
                    connection.settimeout(5)
                    stream = connection.makefile("rb")
                    connection.sendall(b"220 fixture\r\n")
                    for prefix, response in [(b"EHLO ", b"250 fixture\r\n"), (b"AUTH PLAIN ", b"235 ok\r\n"), (b"MAIL FROM:", b"250 ok\r\n"), (b"RCPT TO:", b"250 ok\r\n"), (b"DATA", b"354 data\r\n")]:
                        line = stream.readline(8192)
                        self.assertTrue(line.startswith(prefix))
                        if prefix == b"AUTH PLAIN ":
                            self.assertEqual(base64.b64decode(line[len(prefix):].strip()), b"\x00fixture\x00d3-public-password")
                            state["auth"] += 1
                        connection.sendall(response)
                    total = 0
                    while True:
                        line = stream.readline(8192)
                        self.assertTrue(line)
                        total += len(line)
                        self.assertLess(total, 65536)
                        if line == b".\r\n":
                            break
                    state["data"] += 1
                    connection.sendall(b"250 queued\r\n")
                    self.assertEqual(stream.readline(8192), b"QUIT\r\n")
                    connection.sendall(b"221 bye\r\n")
                    stream.close()
            except Exception as error:
                state["error"] = error
            finally:
                listener.close()
        worker = threading.Thread(target=server)
        worker.start()
        try:
            self.write_value(f"smtp://fixture:d3-public-password@127.0.0.1:{port}?from=sender@example.invalid")
            project = self.native_project('sonuc "recipient@example.invalid" "Konu" "Gövde" ile ileti göndermeyi dene olsun\nsonuc başarılıysa\n    programı 0 ile bitir\nprogramı 7 ile bitir\n')
            (project / "proje.dil").write_text(f'proje "credential-smtp-fixture" olsun\nsürüm "0.1.0" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ", "yerel-ağ" listesi olsun\nağ_hedefleri "smtp://127.0.0.1:{port}" listesi olsun\ne_posta_bağlantı_değişkeni "{ENV}" olsun\n')
            result, metadata = self.invoke(child=self.dil, command=["çalıştır", str(project)])
            worker.join(timeout=12)
            self.assertFalse(worker.is_alive())
            self.assertIsNone(state["error"])
            self.assertEqual(result.returncode, 0)
            self.assertEqual(metadata["source"], "child")
            self.assertEqual({k: state[k] for k in ["connections", "auth", "data"]}, {"connections": 1, "auth": 1, "data": 1})
            self.assertNotIn("d3-public-password", result.stdout + result.stderr)
        finally:
            listener.close()
            worker.join(timeout=12)


if __name__ == "__main__":
    unittest.main(verbosity=2)
