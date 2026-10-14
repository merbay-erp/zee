#!/usr/bin/env python3
"""K-223: real compiled Zee login helper on an owned ephemeral PostgreSQL.

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
import subprocess
import tempfile
import threading
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
TLS = runpy.run_path(str(ROOT / "scripts/postgresql-tls-ci.py"))
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
CREATE FUNCTION zee_rehash_update_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 PERFORM nextval('zee_rehash_update_attempts');
 IF NEW.yazma_reddi THEN RAISE EXCEPTION 'synthetic update refusal' USING ERRCODE='23514'; END IF;
 INSERT INTO zee_rehash_audit VALUES (NEW.kimlik);
 RETURN NEW;
END $$;
CREATE TRIGGER zee_rehash_update_guard BEFORE UPDATE ON zee_rehash_kullanicilar
 FOR EACH ROW EXECUTE FUNCTION zee_rehash_update_guard();
CREATE FUNCTION zee_rehash_bind_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 PERFORM nextval('zee_rehash_bind_attempts');
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
            "native runner requires exactly seven cases")
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

    def start(self, account, old=None, password=None, application=None, verify_only=False):
        project = self.directory / ("project-" + uuid.uuid4().hex)
        project.mkdir()
        (project / "gocler").mkdir()
        target = f"postgresql://localhost:{self.cluster.port}/postgres"
        (project / "proje.dil").write_text(
            'proje "rehash-ci" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\n'
            'giriş "ana.dil" olsun\n'
            'yetkinlikler "veritabanı", "kriptografi" listesi olsun\n'
            f'veritabanı_hedefi "{target}" olsun\n'
            'veritabanı_bağlantı_değişkeni "ZEE_REHASH_CI_URL" olsun\n'
            'veritabanı_göçleri "gocler" olsun\n')
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
        (project / "ana.dil").write_text(source)
        env = dict(self.environment)
        application = application or "zee-rehash-ci-login"
        env["ZEE_REHASH_CI_URL"] = (f"host=localhost port={self.cluster.port} user=postgres "
                                     f"dbname=postgres sslmode=require application_name={application}")
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
    report = {"schema": "zee-parola-goc-ci-1", "started_utc": datetime.now(timezone.utc).isoformat(),
              "status": "failed", "phase": "initializing", "cases": [], "ignored": 0,
              "commit_response_loss_probed": False,
              "boundary": "Owned synthetic PostgreSQL and anonymous helper; SQL session row only. No live login, web cookie, product migration, C027 commit network fault or signed release."}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    run = TLS["run"]
    source = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
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
        report["build_identity"] = build
        paths = [Path(__file__), ROOT / "scripts/postgresql-tls-ci.py",
                 ROOT / "dogfood/zee-cekirdek/parola-goc/giriste-yukselt.dil",
                 ROOT / "compiler/tests/zee_cekirdek_urun_kabulu_testi.rs"]
        inputs = {str(p.relative_to(ROOT)): p.read_bytes() for p in paths}
        report["fixture_hashes"] = {name: digest(value) for name, value in inputs.items()}
        ca, _wrong_ca, cert, key = TLS["certificates"](directory)
        (directory / "pg_hba.conf").write_text("local all all trust\nhostssl all all 0.0.0.0/0 trust\nhostnossl all all 0.0.0.0/0 reject\n")
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
            report["phase"] = "native-cases"
            native_cases(probe, report["cases"])
            report["status"] = "passed"
            report["phase"] = "completed"
            report["server"] = cluster.version
            report["server_identity"] = cluster.identity
        finally:
            cluster.close()
            report["source_clean"] = not run(["git", "-C", str(ROOT), "status", "--porcelain"]).strip()
            report["source_end_sha"] = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
            report["source_unchanged"] = report["source_end_sha"] == source
            report["source_acceptance"] = ("clean-exact" if report["source_clean"] and not build["dirty"]
                                             and report["source_unchanged"]
                                             else "working-tree-only")
            report["elapsed_seconds"] = round(time.monotonic() - started, 3)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print("Native password migration gate: 7/7 passed, 0 ignored; report=" + str(args.output))


if __name__ == "__main__":
    main()
