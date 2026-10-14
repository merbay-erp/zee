#!/usr/bin/env python3
"""K-223/K-242: real anonymous login helper on an owned PostgreSQL.

No existing cluster, product database, live login or real credential is used.
The SQL row stands for the anonymous model's session binding, not a web cookie.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import runpy
import shutil
import socket
import subprocess
import tempfile
import threading
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
TLS = runpy.run_path(str(ROOT / "scripts/postgresql-tls-ci.py"))
PROXY = runpy.run_path(str(ROOT / "scripts/postgresql-commit-proxy.py"))
EXPECTED = {
    "ilk_bcrypt_girisi_argon2_ve_oturum_commit":
        {"variants": 3, "updates": 3, "sessions": 3, "modern_verified": True},
    "ikinci_modern_giriste_update_yok":
        {"accepted": True, "hash_unchanged": True, "additional_updates": 0,
         "additional_update_attempts": 0, "sessions": 2},
    "yanlis_parola_yazisiz_reddedilir":
        {"rejected": True, "hash_unchanged": True, "updates": 0, "sessions": 0,
         "update_attempts": 0, "bind_attempts": 0},
    "ayni_eski_snapshot_gercek_sql_cas_yarisi":
        {"waiting_clients": 2, "accepted": 1, "rejected": 1, "updates": 1,
         "sessions": 1, "modern_hash": True, "idle_transactions": 0},
    "sql_update_hatasi_rollback":
        {"rejected": True, "legacy_preserved": True, "update_attempts": 1,
         "bind_attempts": 0, "updates": 0, "sessions": 0},
    "oturum_bind_hatasi_dis_eylemi_rollback":
        {"rejected": True, "legacy_preserved": True, "update_attempts": 1,
         "bind_attempts": 1, "updates": 0, "sessions": 0},
    "oturum_ve_hash_yalniz_commit_sonrasi_gorunur":
        {"waiting_clients": 1, "update_attempts_before_commit": 1,
         "bind_attempts_before_commit": 1, "legacy_visible_before_commit": True,
         "sessions_before_commit": 0, "updates_before_commit": 0,
         "accepted": True, "modern_hash_after_commit": True,
         "sessions_after_commit": 1, "updates_after_commit": 1},
}
for name, mode, applied, refusal in [
        ("commit_olumlu_kontrol_bildirim_bir_kez", "observe", True, False),
        ("commit_iletilmeden_kayip_c027_kayit_yok", "before", False, False),
        ("commit_uygulandi_yanit_kayip_c027_kayit_var", "after", True, False),
        ("commit_deferred_ret_c027_gozlemci_rollback", "observe", False, True)]:
    control = applied and mode == "observe"
    EXPECTED[name] = {
        "client_exit": 0 if control else 1, "c027": not control,
        "stdout_empty": not control, "continuation_completed": control,
        "smtp_connections": int(control), "smtp_messages": int(control),
        "update_attempts": 1, "bind_attempts": 1, "commits_seen": 1,
        "commit_forwarded": mode != "before", "commit_complete": applied,
        "commit_ready_idle": mode != "before", "completion_delivered": control,
        "server_error": refusal, "error_delivered": refusal,
        "error_sqlstate": "23514" if refusal else "",
        "error_severity": "ERROR" if refusal else "",
        "actual_db_applied": applied, "modern_hash": applied,
        "modern_verified": applied, "sessions": int(applied), "updates": int(applied),
        "session_hash_consistent": True, "remaining_client_backends": 0,
    }
SQL = r"""
CREATE TABLE zee_rehash_kullanicilar (
 kimlik text PRIMARY KEY, parola_ozeti text NOT NULL,
 yazma_reddi boolean NOT NULL DEFAULT false,
 baglama_reddi boolean NOT NULL DEFAULT false,
 baglama_bariyeri boolean NOT NULL DEFAULT false);
CREATE TABLE zee_rehash_oturumlar (kimlik text NOT NULL);
CREATE TABLE zee_rehash_audit (kimlik text NOT NULL);
CREATE SEQUENCE zee_rehash_update_attempts;
CREATE SEQUENCE zee_rehash_bind_attempts;
CREATE FUNCTION zee_rehash_statement_attempt() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 PERFORM nextval(TG_ARGV[0]::regclass);
 RETURN NULL;
END $$;
CREATE TRIGGER zee_rehash_update_attempt BEFORE UPDATE ON zee_rehash_kullanicilar
 FOR EACH STATEMENT EXECUTE FUNCTION zee_rehash_statement_attempt('zee_rehash_update_attempts');
CREATE TRIGGER zee_rehash_bind_attempt BEFORE INSERT ON zee_rehash_oturumlar
 FOR EACH STATEMENT EXECUTE FUNCTION zee_rehash_statement_attempt('zee_rehash_bind_attempts');
CREATE FUNCTION zee_rehash_update_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF NEW.yazma_reddi THEN RAISE EXCEPTION 'synthetic update refusal' USING ERRCODE='23514'; END IF;
 INSERT INTO zee_rehash_audit VALUES (NEW.kimlik);
 RETURN NEW;
END $$;
CREATE TRIGGER zee_rehash_update_guard BEFORE UPDATE ON zee_rehash_kullanicilar
 FOR EACH ROW EXECUTE FUNCTION zee_rehash_update_guard();
CREATE FUNCTION zee_rehash_bind_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF (SELECT baglama_reddi FROM zee_rehash_kullanicilar WHERE kimlik=NEW.kimlik)
 THEN RAISE EXCEPTION 'synthetic bind refusal' USING ERRCODE='23514'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER zee_rehash_bind_guard BEFORE INSERT ON zee_rehash_oturumlar
 FOR EACH ROW EXECUTE FUNCTION zee_rehash_bind_guard();
CREATE FUNCTION zee_rehash_bind_barrier() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF (SELECT baglama_bariyeri FROM zee_rehash_kullanicilar WHERE kimlik=NEW.kimlik)
 THEN PERFORM pg_advisory_xact_lock(426001); END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER zee_rehash_bind_barrier AFTER INSERT ON zee_rehash_oturumlar
 FOR EACH ROW EXECUTE FUNCTION zee_rehash_bind_barrier();
"""


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def check_results(results, returncode):
    require(type(returncode) is int and returncode == 0, "native runner exit was not successful")
    require(isinstance(results, list) and len(results) == len(EXPECTED),
            "native runner requires exactly eleven cases")
    require([case.get("name") for case in results] == list(EXPECTED),
            "native runner case names/order are missing or duplicated")
    for case in results:
        require(case.get("status") == "passed" and case.get("ignored") is False,
                "native case failed, was skipped or has no explicit result")
        require(case.get("observations") == EXPECTED[case["name"]],
                "native SQL observations do not establish the required invariant")
        require(all(type(case["observations"][key]) is type(value)
                    for key, value in EXPECTED[case["name"]].items()),
                "native counts and boolean checks have incorrect types")


def extract_model(source):
    legacy = re.search(r'const LEGACY: &str = "([^"]+)";', source)
    password = re.search(r'const PAROLA: &str = "([^"]+)";', source)
    atomic = re.search(r'const ATOMIK_GIRIS: &str = r#"(.*?)"#;', source, re.S)
    require(legacy is not None and password is not None and atomic is not None,
            "existing anonymous login model constants were not found")
    return legacy.group(1), password.group(1), atomic.group(1)


def sql_string(value):
    return "'" + value.replace("'", "''") + "'"


class Probe:
    def __init__(self, directory, cluster, binary, ca, helper, model):
        self.directory, self.cluster, self.binary = directory, cluster, binary
        self.helper = helper
        self.legacy, self.password, self.atomic = extract_model(model)
        self.environment = {key: value for key, value in os.environ.items()
                            if not key.startswith("ZEE_REHASH_CI_")}
        self.environment["ZEE_REHASH_CI_URL_TLS_CA_PEM"] = ca.read_text()
        self.children = []
        self.projects = []

    def psql_command(self, interactive=False):
        command = list(self.cluster.psql)
        if interactive and command[:2] == ["docker", "exec"]:
            command.insert(2, "--interactive")
        url = f"host=127.0.0.1 port={self.cluster.psql_port} user=postgres dbname=postgres sslmode=require"
        return command + ["--no-psqlrc", "--dbname", url, "--no-align", "--tuples-only",
                          "--quiet", "--set", "ON_ERROR_STOP=1"]

    def sql(self, statement):
        result = subprocess.run(self.psql_command() + ["--command", statement],
                                capture_output=True, text=True, timeout=30)
        require(result.returncode == 0, "owned fixture SQL command failed")
        return result.stdout.strip()

    def count(self, table, account):
        return int(self.sql(f"SELECT count(*)::text FROM {table} WHERE kimlik={sql_string(account)}"))

    def hash(self, account):
        return self.sql("SELECT parola_ozeti FROM zee_rehash_kullanicilar WHERE kimlik=" + sql_string(account))

    def sequence(self, name):
        return int(self.sql("SELECT CASE WHEN is_called THEN last_value ELSE 0 END::text FROM " + name))

    def account(self, name, legacy=None, write=False, bind=False, barrier=False):
        self.sql("INSERT INTO zee_rehash_kullanicilar VALUES (" + sql_string(name) + "," +
                 sql_string(legacy or self.legacy) + "," + str(write).lower() + "," +
                 str(bind).lower() + "," + str(barrier).lower() + ")")

    def start(self, account, old=None, password=None, application=None, verify_only=False,
              fault_proxy=None, notification=None):
        project = self.directory / ("project-" + uuid.uuid4().hex)
        project.mkdir()
        (project / "gocler").mkdir()
        port = fault_proxy.port if fault_proxy else self.cluster.port
        target = f"postgresql://localhost:{port}/postgres"
        (project / "proje.dil").write_text(
            'proje "rehash-ci" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\n'
            'giriş "ana.dil" olsun\n' +
            ('yetkinlikler "veritabanı", "kriptografi", "ağ", "yerel-ağ" listesi olsun\n'
             if notification else 'yetkinlikler "veritabanı", "kriptografi" listesi olsun\n') +
            f'veritabanı_hedefi "{target}" olsun\n'
            'veritabanı_bağlantı_değişkeni "ZEE_REHASH_CI_URL" olsun\n'
            'veritabanı_göçleri "gocler" olsun\n')
        if notification:
            with (project / "proje.dil").open("a") as manifest:
                manifest.write(f'ağ_hedefleri "smtp://127.0.0.1:{notification.port}" listesi olsun\n'
                               'e_posta_bağlantı_değişkeni "ZEE_REHASH_CI_SMTP" olsun\n')
        password, old = password or self.password, old or self.legacy
        if verify_only:
            source = ("parola " + json.dumps(password) + " olsun\nozet " + json.dumps(old) +
                      ' olsun\nparola ozet ile doğrulanıyorsa\n    "dogrulandi" yaz\n'
                      'değilse\n    "reddedildi" yaz\n')
        else:
            source = (self.helper + "\n" + self.atomic + "\nsonuc " + json.dumps(account) +
                      " ve " + json.dumps(password) + " ve " + json.dumps(old) +
                      ' ile anonimAtomikGiris olsun\nsonuc başarılıysa\n    "gecti" yaz\n'
                      'değilse\n    "reddedildi" yaz\n')
            if notification:
                # This tail executes only after the real outer COMMIT returns.
                source += ('bildirim "receiver@example.invalid" "Fixture" "Sentetik" ile ileti göndermeyi dene olsun\n'
                           'bildirim başarılıysa\n    "bildirildi" yaz\n'
                           'değilse\n    "bildirim-reddi" yaz\n')
        (project / "ana.dil").write_text(source)
        env = dict(self.environment)
        application = application or "zee-rehash-ci-login"
        sslmode = "disable" if fault_proxy else "require"
        env["ZEE_REHASH_CI_URL"] = (f"host=localhost port={port} user=postgres "
                                     f"dbname=postgres sslmode={sslmode} application_name={application}")
        if notification:
            env["ZEE_REHASH_CI_SMTP"] = (f"smtp://fixture:synthetic-password@127.0.0.1:{notification.port}"
                                         "?from=sender@example.invalid")
        child = subprocess.Popen([str(self.binary), "çalıştır", str(project)], env=env,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.children.append(child)
        self.projects.append(project)
        return child

    def finish(self, child):
        stdout, stderr = child.communicate(timeout=90)
        if child.returncode:
            # Generated sources contain only fixture values. Still mask quoted
            # values and PHC strings before surfacing a diagnostic for repair.
            safe = re.sub(r'"[^"\n]*"', '"[fixture]"', stderr)
            safe = re.sub(r'\$(?:2[aby]|argon2id)\$[^\s"<>]+', '[fixture-hash]', safe)
            print("Zee fixture diagnostic: " + safe[:1600], flush=True)
        require(child.returncode == 0, "compiled Zee fixture execution failed")
        require(stdout.strip() in ("gecti", "reddedildi", "dogrulandi"),
                "compiled Zee fixture did not return its exact expected marker")
        return stdout.strip()

    def login(self, *args, **kwargs):
        return self.finish(self.start(*args, **kwargs))

    def wait_for_locks(self, applications, query_prefix):
        names = ",".join(sql_string(name) for name in applications)
        until = time.monotonic() + 45
        while time.monotonic() < until:
            count = int(self.sql("SELECT count(*)::text FROM pg_stat_activity WHERE "
                                f"application_name IN ({names}) AND wait_event_type='Lock' "
                                "AND query LIKE " + sql_string(query_prefix + "%")))
            if count == len(applications):
                return count
            require(all(child.poll() is None for child in self.children[-len(applications):]),
                    "Zee client ended before reaching the SQL concurrency barrier")
            time.sleep(0.05)
        raise ValueError("Zee clients did not reach the actual SQL lock barrier")

    def close(self):
        for child in self.children:
            if child.poll() is None:
                child.terminate()
                try:
                    child.communicate(timeout=10)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.communicate(timeout=10)


class SqlBlocker:
    def __init__(self, probe, lock_sql):
        self.probe, self.lock_sql = probe, lock_sql
        self.process = None

    def __enter__(self):
        self.process = subprocess.Popen(self.probe.psql_command(True), stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.process.stdin.write("BEGIN;\n" + self.lock_sql + ";\nSELECT 'BARRIER_READY';\n")
        self.process.stdin.flush()
        signal = queue.Queue()

        def read_ready():
            for line in self.process.stdout:
                if line.strip() == "BARRIER_READY":
                    signal.put(True)
                    return
            signal.put(False)

        reader = threading.Thread(target=read_ready, daemon=True)
        reader.start()
        try:
            require(signal.get(timeout=20), "SQL blocker did not acquire its own fixture lock")
        except Exception:
            self.__exit__(None, None, None)
            raise
        reader.join(timeout=1)
        return self

    def __exit__(self, *exc):
        if self.process is not None and self.process.poll() is None:
            try:
                self.process.stdin.write("COMMIT;\n\\q\n")
                self.process.stdin.flush()
                self.process.communicate(timeout=20)
            except (BrokenPipeError, subprocess.TimeoutExpired):
                self.process.kill()
                self.process.communicate(timeout=10)


class SmtpSink:
    """Real synthetic loopback SMTP, reachable positive control for the tail."""
    def __init__(self):
        self.listener = socket.socket()
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen(2)
        self.listener.settimeout(0.05)
        self.port = self.listener.getsockname()[1]
        self.deadline = time.monotonic() + 45
        self.connections = self.messages = 0
        self.done = threading.Event()
        self.error = None
        self.client = None
        self.thread = threading.Thread(target=self._run, daemon=True)
        self.thread.start()

    def _run(self):
        try:
            while not self.done.is_set():
                require(time.monotonic() < self.deadline, "SMTP fixture deadline exceeded")
                try:
                    self.client, _ = self.listener.accept()
                except socket.timeout:
                    continue
                self.connections += 1
                require(self.connections == 1, "SMTP notification was repeated")
                self.client.settimeout(5)
                reader = self.client.makefile("rb")
                def line():
                    value = reader.readline(16385)
                    require(value and len(value) <= 16384 and value.endswith(b"\r\n"),
                            "invalid synthetic SMTP command framing")
                    return value
                def expect(prefix, response):
                    require(line().startswith(prefix), "unexpected synthetic SMTP command")
                    self.client.sendall(response)
                with reader:
                    self.client.sendall(b"220 fixture\r\n")
                    expect(b"EHLO ", b"250 fixture\r\n")
                    expect(b"AUTH PLAIN ", b"235 accepted\r\n")
                    expect(b"MAIL FROM:", b"250 accepted\r\n")
                    expect(b"RCPT TO:", b"250 accepted\r\n")
                    expect(b"DATA\r\n", b"354 continue\r\n")
                    total = 0
                    while True:
                        value = line()
                        total += len(value)
                        require(total <= 65536, "synthetic SMTP DATA limit exceeded")
                        if value == b".\r\n":
                            break
                    self.messages += 1
                    self.client.sendall(b"250 accepted\r\n")
                    expect(b"QUIT\r\n", b"221 bye\r\n")
                self.client.close()
                self.client = None
        except (OSError, ValueError) as error:
            if not self.done.is_set():
                self.error = type(error).__name__

    def close(self):
        self.done.set()
        self.listener.close()
        if self.client is not None:
            self.client.close()
        self.thread.join(timeout=6)
        require(not self.thread.is_alive(), "SMTP fixture did not stop")
        require(self.error is None, "synthetic SMTP protocol failed")


def commit_fault_cases(probe, passed):
    profiles = [
        ("fault-control", "observe", True, False, "commit_olumlu_kontrol_bildirim_bir_kez"),
        ("fault-before", "before", False, False, "commit_iletilmeden_kayip_c027_kayit_yok"),
        ("fault-after", "after", True, False, "commit_uygulandi_yanit_kayip_c027_kayit_var"),
        ("fault-reject", "observe", False, True, "commit_deferred_ret_c027_gozlemci_rollback"),
    ]
    for account, mode, applied, refusal, name in profiles:
        probe.account(account)
        if refusal:
            # This stock fixture rejects before durability. Its outcome is
            # independently observed, not inferred by a SQLSTATE allowlist.
            probe.sql("CREATE FUNCTION zee_rehash_commit_refusal() RETURNS trigger LANGUAGE plpgsql AS $$ "
                      "BEGIN IF NEW.kimlik='fault-reject' THEN RAISE EXCEPTION 'synthetic deferred refusal' "
                      "USING ERRCODE='23514'; END IF; RETURN NEW; END $$; "
                      "CREATE CONSTRAINT TRIGGER zee_rehash_commit_refusal AFTER INSERT ON zee_rehash_oturumlar "
                      "DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION zee_rehash_commit_refusal()")
        update_before = probe.sequence("zee_rehash_update_attempts")
        bind_before = probe.sequence("zee_rehash_bind_attempts")
        application = "zee-rehash-ci-" + account
        proxy = PROXY["CommitProxy"]("127.0.0.1", probe.cluster.port, mode,
                                    application=application, deadline_seconds=45).start()
        sink = SmtpSink()
        try:
            child = probe.start(account, application=application, fault_proxy=proxy, notification=sink)
            stdout, stderr = child.communicate(timeout=35)
            control = mode == "observe" and applied
            require(child.returncode == (0 if control else 1), "COMMIT fixture returned wrong CLI exit")
            diagnostics = re.findall(r"^HATA (C\d{3})$", stderr, re.MULTILINE)
            if control:
                require(stdout == "gecti\nbildirildi\n" and stderr == "", "positive continuation control failed")
            else:
                require(stdout == "" and diagnostics == ["C027"],
                        "COMMIT fault must stop actual runtime before success/SMTP continuation with C027")
            wire = proxy.finish()
            sink.close()
            current = probe.hash(account)
            modern = current.startswith("$argon2id$v=19$")
            sessions = probe.count("zee_rehash_oturumlar", account)
            updates = probe.count("zee_rehash_audit", account)
            verified = modern and probe.login(account, old=current, verify_only=True) == "dogrulandi"
            backend_count = int(probe.sql("SELECT count(*)::text FROM pg_stat_activity WHERE application_name=" +
                                          sql_string(application)))
            error = wire["commit_error"] or {}
            passed(name, {
                "client_exit": child.returncode, "c027": diagnostics == ["C027"],
                "stdout_empty": stdout == "", "continuation_completed": stdout == "gecti\nbildirildi\n",
                "smtp_connections": sink.connections, "smtp_messages": sink.messages,
                "update_attempts": probe.sequence("zee_rehash_update_attempts") - update_before,
                "bind_attempts": probe.sequence("zee_rehash_bind_attempts") - bind_before,
                "commits_seen": wire["commits_seen"], "commit_forwarded": wire["commit_forwarded"],
                "commit_complete": wire["commit_complete"], "commit_ready_idle": wire["commit_ready"],
                "completion_delivered": wire["completion_delivered"],
                "server_error": bool(error), "error_delivered": wire["error_delivered"],
                "error_sqlstate": error.get("sqlstate", ""), "error_severity": error.get("severity", ""),
                "actual_db_applied": modern and sessions == updates == 1,
                "modern_hash": modern, "modern_verified": bool(verified),
                "sessions": sessions, "updates": updates,
                "session_hash_consistent": ((modern and sessions == 1) or
                                             (current == probe.legacy and sessions == 0)),
                "remaining_client_backends": backend_count,
            })
        finally:
            proxy.close()
            sink.close()


def native_cases(probe, results=None):
    results = [] if results is None else results

    def passed(name, observations):
        require(observations == EXPECTED[name], "native invariant failed: " + name)
        results.append({"name": name, "status": "passed", "ignored": False,
                        "observations": observations})
        print("native case: " + name + " ... passed", flush=True)

    try:
        verified = True
        for prefix in ("2a", "2b", "2y"):
            legacy = "$" + prefix + probe.legacy[3:]
            account = "prefix-" + prefix
            probe.account(account, legacy)
            require(probe.login(account, old=legacy) == "gecti", "first legacy login failed")
            modern = probe.hash(account)
            require(modern.startswith("$argon2id$v=19$"), "first login did not produce modern Argon2id")
            verified &= probe.login(account, old=modern, verify_only=True) == "dogrulandi"
        passed("ilk_bcrypt_girisi_argon2_ve_oturum_commit", {
            "variants": 3, "updates": int(probe.sql("SELECT count(*)::text FROM zee_rehash_audit")),
            "sessions": int(probe.sql("SELECT count(*)::text FROM zee_rehash_oturumlar")),
            "modern_verified": verified})
        modern = probe.hash("prefix-2b")
        updates = probe.count("zee_rehash_audit", "prefix-2b")
        attempts = probe.sequence("zee_rehash_update_attempts")
        accepted = probe.login("prefix-2b", old=modern) == "gecti"
        passed("ikinci_modern_giriste_update_yok", {
            "accepted": accepted, "hash_unchanged": probe.hash("prefix-2b") == modern,
            "additional_updates": probe.count("zee_rehash_audit", "prefix-2b") - updates,
            "additional_update_attempts": probe.sequence("zee_rehash_update_attempts") - attempts,
            "sessions": probe.count("zee_rehash_oturumlar", "prefix-2b")})
        probe.account("wrong")
        update_before = probe.sequence("zee_rehash_update_attempts")
        bind_before = probe.sequence("zee_rehash_bind_attempts")
        rejected = probe.login("wrong", password="synthetic-wrong") == "reddedildi"
        passed("yanlis_parola_yazisiz_reddedilir", {
            "rejected": rejected, "hash_unchanged": probe.hash("wrong") == probe.legacy,
            "updates": probe.count("zee_rehash_audit", "wrong"),
            "update_attempts": probe.sequence("zee_rehash_update_attempts") - update_before,
            "bind_attempts": probe.sequence("zee_rehash_bind_attempts") - bind_before,
            "sessions": probe.count("zee_rehash_oturumlar", "wrong")})
        probe.account("race")
        applications = ["zee-rehash-ci-cas-a", "zee-rehash-ci-cas-b"]
        with SqlBlocker(probe, "SELECT 1 FROM zee_rehash_kullanicilar WHERE kimlik='race' FOR UPDATE"):
            children = [probe.start("race", application=app) for app in applications]
            waiting = probe.wait_for_locks(applications, "UPDATE zee_rehash_kullanicilar")
            require(probe.hash("race") == probe.legacy and probe.count("zee_rehash_oturumlar", "race") == 0,
                    "uncommitted CAS clients exposed a session or password update")
        markers = [probe.finish(child) for child in children]
        idle = int(probe.sql("SELECT count(*)::text FROM pg_stat_activity WHERE "
                            "application_name IN ('zee-rehash-ci-cas-a','zee-rehash-ci-cas-b') "
                            "AND state='idle in transaction'"))
        passed("ayni_eski_snapshot_gercek_sql_cas_yarisi", {
            "waiting_clients": waiting, "accepted": markers.count("gecti"),
            "rejected": markers.count("reddedildi"), "updates": probe.count("zee_rehash_audit", "race"),
            "sessions": probe.count("zee_rehash_oturumlar", "race"),
            "modern_hash": probe.hash("race").startswith("$argon2id$v=19$"), "idle_transactions": idle})
        for account, bind, case in [("write-fault", False, "sql_update_hatasi_rollback"),
                                    ("bind-fault", True, "oturum_bind_hatasi_dis_eylemi_rollback")]:
            probe.account(account, write=not bind, bind=bind)
            update_before = probe.sequence("zee_rehash_update_attempts")
            bind_before = probe.sequence("zee_rehash_bind_attempts")
            rejected = probe.login(account) == "reddedildi"
            passed(case, {"rejected": rejected, "legacy_preserved": probe.hash(account) == probe.legacy,
                          "update_attempts": probe.sequence("zee_rehash_update_attempts") - update_before,
                          "bind_attempts": probe.sequence("zee_rehash_bind_attempts") - bind_before,
                          "updates": probe.count("zee_rehash_audit", account),
                          "sessions": probe.count("zee_rehash_oturumlar", account)})
        probe.account("visibility", barrier=True)
        update_before = probe.sequence("zee_rehash_update_attempts")
        bind_before = probe.sequence("zee_rehash_bind_attempts")
        with SqlBlocker(probe, "SELECT pg_advisory_xact_lock(426001)"):
            child = probe.start("visibility", application="zee-rehash-ci-visibility")
            waiting = probe.wait_for_locks(["zee-rehash-ci-visibility"], "INSERT INTO zee_rehash_oturumlar")
            observation = {"waiting_clients": waiting,
                           "update_attempts_before_commit": probe.sequence("zee_rehash_update_attempts") - update_before,
                           "bind_attempts_before_commit": probe.sequence("zee_rehash_bind_attempts") - bind_before,
                           "legacy_visible_before_commit": probe.hash("visibility") == probe.legacy,
                           "sessions_before_commit": probe.count("zee_rehash_oturumlar", "visibility"),
                           "updates_before_commit": probe.count("zee_rehash_audit", "visibility")}
        observation.update({"accepted": probe.finish(child) == "gecti",
                            "modern_hash_after_commit": probe.hash("visibility").startswith("$argon2id$v=19$"),
                            "sessions_after_commit": probe.count("zee_rehash_oturumlar", "visibility"),
                            "updates_after_commit": probe.count("zee_rehash_audit", "visibility")})
        passed("oturum_ve_hash_yalniz_commit_sonrasi_gorunur", observation)
        commit_fault_cases(probe, passed)
        check_results(results, 0)
        return results
    finally:
        probe.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", choices=("docker", "local"), default="docker")
    parser.add_argument("--postgres-bin-dir")
    parser.add_argument("--dil", type=Path, help="Explicit built binary; default builds locked compiler/bin dil")
    parser.add_argument("--output", type=Path, default=ROOT / "compiler/target/parola-goc-ci.json")
    args = parser.parse_args()
    if args.engine == "local" and not args.postgres_bin_dir:
        parser.error("local engine requires own initdb/pg_ctl tools, not an existing database URL")
    report = {"schema": "zee-parola-goc-ci-2", "started_utc": datetime.now(timezone.utc).isoformat(),
              "status": "failed", "phase": "initializing", "cases": [], "ignored": 0,
              "commit_response_loss_probed": False,
              "tls_cases": 7, "plaintext_fault_cases": 4,
              "boundary": "Owned synthetic PostgreSQL and anonymous helper; SQL session row only. Seven TLS cases plus four loopback plaintext COMMIT/SMTP controls. No live login, web cookie, product migration, TLS fault interception, postcommit extension audit or signed release."}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    run = TLS["run"]
    source = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
    report["source_start_sha"] = source
    report["source_clean_start"] = not run(["git", "-C", str(ROOT), "status", "--porcelain"]).strip()
    if args.dil is None:
        print("Building actual locked Zee compiler for native fixture...", flush=True)
        run(["cargo", "build", "--locked", "--bin", "dil"], cwd=ROOT / "compiler", timeout=900,
            env={**os.environ, "CARGO_TARGET_DIR": str(ROOT / "compiler/target")})
    original = (args.dil or ROOT / "compiler/target/debug/dil").resolve()
    started = time.monotonic()
    report["source_sha"] = source
    with tempfile.TemporaryDirectory(prefix="zee-rehash-ci-") as temp:
        directory = Path(temp)
        binary = directory / ("dil.exe" if os.name == "nt" else "dil")
        shutil.copy2(original, binary)
        report["binary_sha256"] = digest(binary.read_bytes())
        version = subprocess.run([str(binary), "sürüm", "--json"], capture_output=True, text=True, timeout=30)
        require(version.returncode == 0, "compiled binary must expose explicit JSON build identity")
        build = json.loads(version.stdout)
        require(build.get("schema") == "zee-build-v1" and build.get("source_sha") == source,
                "compiled binary does not identify the selected source HEAD")
        require(build.get("dirty") is False, "native fixture requires an actual clean-source binary")
        report["build_identity"] = build
        paths = [Path(__file__), ROOT / "scripts/postgresql-tls-ci.py", ROOT / "scripts/postgresql-commit-proxy.py",
                 ROOT / "dogfood/zee-cekirdek/parola-goc/giriste-yukselt.dil",
                 ROOT / "compiler/tests/zee_cekirdek_urun_kabulu_testi.rs"]
        inputs = {str(p.relative_to(ROOT)): p.read_bytes() for p in paths}
        report["fixture_hashes"] = [{"path": name, "sha256": digest(value)}
                                     for name, value in sorted(inputs.items())]
        ca, _wrong_ca, cert, key = TLS["certificates"](directory)
        # The owned Docker backend sees its bridge peer behind the host's
        # 127.0.0.1-only published port. Local initdb admits only 127/32;
        # Docker admits only its own interface subnet, never a customer DSN.
        plaintext_peer = "samenet" if args.engine == "docker" else "127.0.0.1/32"
        (directory / "pg_hba.conf").write_text("local all all trust\nhostssl all all 0.0.0.0/0 trust\n"
                                               f"hostnossl all all {plaintext_peer} trust\n"
                                               "hostnossl all all 0.0.0.0/0 reject\n")
        cluster = (TLS["DockerCluster"](directory, cert, key) if args.engine == "docker" else
                   TLS["LocalCluster"](directory, cert, key, args.postgres_bin_dir))
        if args.engine == "docker":
            cluster.name = "zee-rehash-ci-" + uuid.uuid4().hex
        try:
            cluster.start()
            probe = Probe(directory, cluster, binary, ca,
                          inputs["dogfood/zee-cekirdek/parola-goc/giriste-yukselt.dil"].decode(),
                          inputs["compiler/tests/zee_cekirdek_urun_kabulu_testi.rs"].decode())
            probe.sql(SQL)
            # Zero-row writes still count as executed SQL statements. A stale
            # CAS replay cannot evade the observer merely by matching no rows.
            probe.sql("UPDATE zee_rehash_kullanicilar SET parola_ozeti=parola_ozeti WHERE false; "
                      "INSERT INTO zee_rehash_oturumlar SELECT kimlik FROM zee_rehash_kullanicilar WHERE false")
            require(probe.sequence("zee_rehash_update_attempts") == 1 and
                    probe.sequence("zee_rehash_bind_attempts") == 1,
                    "SQL statement observer failed to count zero-row writes")
            report["statement_attempt_observer_verified"] = True
            report["phase"] = "native-cases"
            native_cases(probe, report["cases"])
            report["commit_response_loss_probed"] = True
            report["status"] = "passed"
            report["phase"] = "completed"
            report["server"] = cluster.version
            report["server_identity"] = cluster.identity
            if args.engine == "local":
                report["runtime_binaries"] = [
                    {"path": str(cluster.binaries / name),
                     "sha256": digest((cluster.binaries / name).read_bytes())}
                    for name in ("postgres", "initdb", "pg_ctl", "psql")]
        finally:
            cluster.close()
            report["owned_cluster_closed"] = True
            report["source_clean"] = not run(["git", "-C", str(ROOT), "status", "--porcelain"]).strip()
            report["source_end_sha"] = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
            report["source_unchanged"] = report["source_end_sha"] == source
            report["source_acceptance"] = ("clean-exact" if report["source_clean_start"] and report["source_clean"] and not build["dirty"]
                                             and report["source_unchanged"]
                                             else "working-tree-only")
            report["elapsed_seconds"] = round(time.monotonic() - started, 3)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print("Native password migration gate: 11/11 passed, 0 ignored; report=" + str(args.output))


if __name__ == "__main__":
    main()
