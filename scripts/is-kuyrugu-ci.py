#!/usr/bin/env python3
"""K-247: anonymous durable SQL-only queue on a newly owned PostgreSQL.

The fixed idempotent counter is generated only in temporary test projects.
No customer URL, existing cluster, SMTP dispatch or executable job data exists.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import runpy
import shutil
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
TLS = runpy.run_path(str(ROOT / "scripts/postgresql-tls-ci.py"))
QUEUE = ROOT / "dogfood/zee-cekirdek/is-kuyrugu"
PAROLA = runpy.run_path(str(ROOT / "scripts/parola-goc-ci.py"))
PROXY = runpy.run_path(str(ROOT / "scripts/postgresql-commit-proxy.py"))
EXPECTED = {
    "ayni_onay_tek_is_icerik_farki_reddedilir":
        {"jobs": 1, "same_id": True, "changed_payload_rejected": True, "payload_is_data": True},
    "iki_worker_sirasiz_ayri_kira_tek_sahip_ve_limit":
        {"waiting_workers": 2, "distinct_jobs": 2, "active_leases": 2,
         "third_worker_empty": True, "same_owner_empty": True},
    "suresi_dolan_kira_eski_nesli_reddeder_tek_etki":
        {"expired_token_reuse_selects_current_job": True,
         "generation_increased": True, "stale_rejected": True, "effects_before": 0,
         "effects_after": 1, "repeat_rejected": True, "state_done": True},
    "iki_ret_sonrasi_ucuncu_deneme_bir_etki":
        {"first_backoff": 1, "second_backoff": 2, "early_claim_empty": True,
         "attempts": 3, "handler_statements": 3, "effects": 1, "state_done": True},
    "uc_ret_olu_mektup_dorduncu_deneme_yok":
        {"attempts": 3, "handler_statements": 3, "effects": 0,
         "dead_letter": True, "fourth_claim_empty": True},
    "sigkill_commit_oncesi_yeniden_kira_tek_etki":
        {"actual_lock_wait": True, "uncommitted_invisible": True,
         "killed": True, "effects_after_kill": 0, "effects_after_recovery": 1,
         "attempts": 2, "state_done": True},
    "sigkill_commit_sonrasi_uzlastirma_tekrar_yok":
        {"ack_observed": True, "killed": True, "reconciled_done": True,
         "effects": 1, "handler_statements": 1},
    "handler_commit_iletilmeden_c027_uzlastirma":
        {"c027": True, "stdout_empty": True, "commits_seen": 1,
         "commit_forwarded": False, "commit_complete": False, "commit_ready": False,
         "state_leased": True, "effects": 0, "reconcile_no_replay": True},
    "handler_commit_yanit_kayip_c027_uzlastirma":
        {"c027": True, "stdout_empty": True, "commits_seen": 1,
         "commit_forwarded": True, "commit_complete": True, "commit_ready": True,
         "state_done": True, "effects": 1, "reconcile_no_replay": True},
    "enqueue_commit_iletilmeden_c027_anahtar_uzlastirma":
        {"c027": True, "stdout_empty": True, "jobs_before_reconcile": 0,
         "reconciled_missing": True, "jobs_after_explicit_decision": 1},
    "enqueue_commit_yanit_kayip_c027_anahtar_uzlastirma":
        {"c027": True, "stdout_empty": True, "jobs_before_reconcile": 1,
         "reconciled_ready": True, "jobs_after_reconcile": 1},
}
HEARTBEAT_EXPECTED = {'separate_lease_heartbeat_commits_while_handler_row_locked': {'handler_still_blocked_at_renewal_ack': True,
                                                               'renewals_before': 0,
                                                               'renewals_after': 7,
                                                               'heartbeat_acks': 7,
                                                               'held_seconds': 35.0,
                                                               'original30s_expired': True,
                                                               'effects': 1,
                                                               'done': True},
 'done_job_renewal_rejected': {'renewals': 7},
 'early_revoked_lease_reused_token_does_not_resurrect_old_row': {'old_authoritative_expired': True,
                                                                 'old_compatibility_deadline_future': True,
                                                                 'returned_current_job': True,
                                                                 'active_leases': 1},
 'wrong_token_generation_owner_no_mutation': {'rejections': 3, 'renewals': 0},
 'renewal_keeps_120second_server_clock_cap': {'renew_until_unchanged': True,
                                              'renewals': 1,
                                              'renew_window_seconds': 120},
 'expired_old_generation_no_resurrection_or_replacement_mutation': {'generation_increased': True,
                                                                    'new_renewals': 0},
 'absolute_lifetime_cap_rejected': {'renewals': 0},
 'expired_locked_dead_worker_skiplocked_reaper_then_fenced_replacement': {'natural_lease_expiry': True,
                                                                          'reaper_while_locked': 0,
                                                                          'reaper_after_backend_gone': 1,
                                                                          'old_generation_rejected': True,
                                                                          'effects': 1,
                                                                          'attempts': 2,
                                                                          'no_heartbeat_resurrection': True,
                                                                          'backend_gone': True,
                                                                          'killed': True},
 'heartbeat_commit_before_c027_no_replay': {'c027': True,
                                            'renewals': 0,
                                            'effects': 0,
                                            'commits_seen': 1,
                                            'commit_forwarded': False,
                                            'commit_complete': False,
                                            'commit_ready': False},
 'heartbeat_commit_after_c027_no_replay': {'c027': True,
                                           'renewals': 1,
                                           'effects': 0,
                                           'commits_seen': 1,
                                           'commit_forwarded': True,
                                           'commit_complete': True,
                                           'commit_ready': True}}
EXPECTED.update(HEARTBEAT_EXPECTED)

FIXTURE_SQL = """
CREATE TABLE zee_job_effects(job_id bigint PRIMARY KEY REFERENCES zee_jobs(id),value integer NOT NULL CHECK(value=1));
CREATE TABLE zee_job_fixture_failures(job_id bigint PRIMARY KEY REFERENCES zee_jobs(id));
CREATE SEQUENCE zee_job_handler_statements;
CREATE FUNCTION zee_job_handler_attempt() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN PERFORM nextval('zee_job_handler_statements'); RETURN NULL; END $$;
CREATE TRIGGER zee_job_handler_attempt BEFORE INSERT ON zee_job_effects FOR EACH STATEMENT EXECUTE FUNCTION zee_job_handler_attempt();
CREATE FUNCTION zee_job_fixture_refusal() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN IF EXISTS(SELECT 1 FROM zee_job_fixture_failures WHERE job_id=NEW.job_id)
THEN RAISE EXCEPTION 'synthetic idempotent SQL handler refusal' USING ERRCODE='23514'; END IF; RETURN NEW; END $$;
CREATE TRIGGER zee_job_fixture_refusal BEFORE INSERT ON zee_job_effects FOR EACH ROW EXECUTE FUNCTION zee_job_fixture_refusal();
"""
# This counter is an acceptance instrument, not a transferred product source.
COUNTER = '''
eylem sabitSqlHandler
    kimlik Metin olarak al
    jeton Metin olarak al
    nesil Metin olarak al
    TamSayı sonucu döndürür
    hak kimlik ve jeton ve nesil ile isHakkiniKilitle olsun
    hak başarılıysa
        parametreler kimlik listesi olsun
        etki "INSERT INTO zee_job_effects(job_id,value) VALUES($1::bigint,1) ON CONFLICT(job_id) DO NOTHING" sorgusunu parametreler ile değiştirmeyi dene olsun
        etki başarılıysa
            # fixture-barrier
            tamamlama kimlik ve jeton ve nesil ile isiTamamla olsun
            tamamlama başarılıysa
                tamamlamanın değeri döndür
            "handler-tamamlama-hatasi" hatasını döndür
        "handler-depo-hatasi" hatasını döndür
    "handler-kirasi-gecersiz" hatasını döndür
'''
CALLS = {
    "enqueue": "isKuyrugaEkle", "claim": "isKirala", "begin": "isDenemesiniBaslat",
    "apply": "sabitSqlHandler", "fail": "isDenemesiniReddet",
    "reap": "suresiDolanIsleriBirak", "read": "isDurumunuOku", "renew": "isKirasiniYenile",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def exact_types(actual, expected):
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict):
        return actual.keys() == expected.keys() and all(exact_types(actual[k], v) for k, v in expected.items())
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(exact_types(a, b) for a, b in zip(actual, expected))
    return True


def check_results(results, exit_code):
    require(type(exit_code) is int and exit_code == 0, "native queue command failed")
    require(isinstance(results, list) and len(results) == len(EXPECTED), "all twenty-one queue cases are required")
    require(all(type(r) is dict and r.keys() == {"name", "status", "ignored", "observations"}
                for r in results), "queue case result shape is not exact")
    require([r.get("name") for r in results] == list(EXPECTED), "queue names/order are missing or duplicated")
    heartbeat = None
    for result in results:
        require(result.get("status") == "passed" and result.get("ignored") is False,
                "queue case failed, skipped or lacks an explicit result")
        name, actual = result["name"], result["observations"]
        expected = EXPECTED[name]
        require(exact_types(actual, expected), "queue observation names or types differ")
        if name == "separate_lease_heartbeat_commits_while_handler_row_locked":
            varying = {"held_seconds", "heartbeat_acks", "renewals_after"}
            require(all(actual[k] == expected[k] for k in actual.keys() - varying),
                    "actual locked handler did not prove its invariant")
            require(math.isfinite(actual["held_seconds"]) and 35.0 <= actual["held_seconds"] <= 60.0 and
                    2 <= actual["heartbeat_acks"] <= 24 and actual["renewals_after"] == actual["heartbeat_acks"],
                    "actual long handler and independent committed heartbeats are required")
            heartbeat = actual
        elif name == "done_job_renewal_rejected":
            require(heartbeat is not None and actual["renewals"] == heartbeat["heartbeat_acks"],
                    "done job renewal must not change the prior actual lease")
        else:
            require(actual == expected, "actual queue observations do not prove the invariant")


def sql_string(value):
    return "'" + str(value).replace("'", "''") + "'"


class Probe:
    def __init__(self, directory, cluster, binary, ca, helper):
        self.directory, self.cluster, self.binary, self.helper = directory, cluster, binary, helper
        self.env = {k: v for k, v in os.environ.items() if not k.startswith("ZEE_JOB_CI_")}
        self.env["ZEE_JOB_CI_URL_TLS_CA_PEM"] = ca.read_text()
        self.children = []

    def psql_command(self, interactive=False):
        command = list(self.cluster.psql)
        if interactive and command[:2] == ["docker", "exec"]:
            command.insert(2, "--interactive")
        return command + ["--no-psqlrc", "--dbname",
            f"host=127.0.0.1 port={self.cluster.psql_port} user=postgres dbname=postgres sslmode=require",
            "--quiet", "--no-align", "--tuples-only", "--set", "ON_ERROR_STOP=1"]

    def sql(self, statement):
        r = subprocess.run(self.psql_command() + ["--command", statement], capture_output=True, text=True, timeout=20)
        require(r.returncode == 0, "owned queue fixture SQL command failed")
        return r.stdout.strip()

    def reset(self):
        self.sql("TRUNCATE zee_job_leases,zee_job_effects,zee_job_fixture_failures,zee_jobs RESTART IDENTITY; ALTER SEQUENCE zee_job_handler_statements RESTART WITH 1")

    def scalar(self, expression):
        return self.sql("SELECT (" + expression + ")::text")

    def effects(self):
        return int(self.scalar("SELECT count(*) FROM zee_job_effects"))

    def statements(self):
        return int(self.sql("SELECT CASE WHEN is_called THEN last_value ELSE 0 END FROM zee_job_handler_statements"))

    def state(self, job):
        return self.sql("SELECT state FROM zee_jobs WHERE id=" + str(int(job)))

    def attempts(self, job):
        return int(self.sql("SELECT attempts FROM zee_jobs WHERE id=" + str(int(job))))

    def start(self, operation, args=(), application=None, proxy=None, barrier=False, pause=False):
        require(operation in CALLS, "only fixed queue fixture operations exist")
        project = self.directory / ("project-" + uuid.uuid4().hex)
        project.mkdir(); (project / "gocler").mkdir()
        port = proxy.port if proxy else self.cluster.port
        (project / "proje.dil").write_text(
            'proje "queue-ci" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\n'
            'giriş "ana.dil" olsun\nyetkinlikler "veritabanı" listesi olsun\n'
            f'veritabanı_hedefi "postgresql://localhost:{port}/postgres" olsun\n'
            'veritabanı_bağlantı_değişkeni "ZEE_JOB_CI_URL" olsun\nveritabanı_göçleri "gocler" olsun\n')
        counter = COUNTER
        if barrier:
            counter = counter.replace('# fixture-barrier',
                'bariyerParametreleri boş liste olsun\n            bariyer "SELECT pg_advisory_xact_lock(247001)::text AS wait" sorgusunu bariyerParametreleri ile okumayı dene olsun')
        args = [json.dumps(str(v), ensure_ascii=False) for v in args]
        call = ((" ve ".join(args) + " ile ") if args else "") + CALLS[operation]
        source = self.helper + counter + '\nsonuc ' + call + ' olsun\nsonuc başarılıysa\n    sonucun değeri yaz\ndeğilse\n    "reddedildi" yaz\n'
        if pause:
            source += '60 saniye bekle\n'
        (project / "ana.dil").write_text(source)
        application = application or "zee-job-ci-" + uuid.uuid4().hex[:12]
        env = dict(self.env)
        env["ZEE_JOB_CI_URL"] = (f"host=localhost port={port} user=postgres dbname=postgres "
                               f"sslmode={'disable' if proxy else 'require'} application_name={application}")
        child = subprocess.Popen([str(self.binary), "çalıştır", str(project)], env=env,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.children.append(child)
        return child

    def finish(self, child, fault=False):
        stdout, stderr = child.communicate(timeout=25)
        if fault:
            require(child.returncode == 1 and stdout == "" and
                    re.findall(r"^HATA (C\d{3})$", stderr, re.MULTILINE) == ["C027"],
                    "COMMIT fault must be actual C027 with no success continuation")
            return stdout
        if child.returncode:
            safe = re.sub(r'"[^"\n]*"', '"[fixture]"', stderr)
            print("Queue fixture diagnostic: " + safe[:1200], flush=True)
        require(child.returncode == 0 and stderr == "", "actual Zee queue fixture failed")
        return stdout.strip()

    def op(self, operation, *args):
        return self.finish(self.start(operation, args))

    def enqueue(self, key="one", payload="synthetic-ok"):
        value = self.op("enqueue", key, "1", payload)
        require(value.isdigit(), "enqueue did not return an actual job identity")
        return value

    def claim(self, owner="worker-one", token=None):
        token = token or "lease-" + uuid.uuid4().hex
        value = self.op("claim", owner, token)
        if value == "bos":
            return None
        require(re.fullmatch(r"\d+:\d+", value) is not None, "claim did not return id:generation")
        job, generation = value.split(":")
        return job, token, generation

    def renew(self, lease, owner="worker-one"):
        return self.op("renew", *lease, owner)

    def begin(self, lease):
        require(self.op("begin", *lease) == "1", "attempt reservation failed")

    def expire(self, job):
        self.sql("UPDATE zee_job_leases SET expires_at=clock_timestamp() WHERE job_id=" + str(int(job)))
        require(self.op("reap").isdigit(), "expired lease recovery failed")

    def due(self, job):
        self.sql("UPDATE zee_jobs SET available_at=clock_timestamp()-interval '1 second' WHERE id=" + str(int(job)))

    def wait_for_locks(self, applications, query_prefix):
        deadline = time.monotonic() + 15
        names = ",".join(sql_string(v) for v in applications)
        while time.monotonic() < deadline:
            n = int(self.sql("SELECT count(*) FROM pg_stat_activity WHERE application_name IN (" + names +
                             ") AND wait_event_type='Lock' AND query LIKE " + sql_string(query_prefix + "%")))
            if n == len(applications):
                return n
            time.sleep(0.02)
        raise ValueError("actual queue processes did not reach the database lock barrier")

    def wait_gone(self, application):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if self.scalar("SELECT count(*) FROM pg_stat_activity WHERE application_name=" + sql_string(application)) == "0":
                return
            time.sleep(0.02)
        raise ValueError("owned killed/finished queue backend remained")

    def close(self):
        for child in self.children:
            if child.poll() is None:
                child.kill(); child.communicate(timeout=10)


def native_cases(probe, results):
    def passed(name, observations):
        require(observations == EXPECTED[name] and exact_types(observations, EXPECTED[name]),
                "native queue invariant failed: " + name)
        results.append({"name": name, "status": "passed", "ignored": False, "observations": observations})
        print("native queue case: " + name + " ... passed", flush=True)
    probe.reset()
    payload = "synthetic '); DROP TABLE zee_jobs; --"
    first = probe.enqueue(payload=payload)
    second = probe.enqueue(payload=payload)
    rejected = probe.op("enqueue", "one", "1", "different") == "reddedildi"
    passed("ayni_onay_tek_is_icerik_farki_reddedilir", {
        "jobs": int(probe.scalar("SELECT count(*) FROM zee_jobs")), "same_id": first == second,
        "changed_payload_rejected": rejected,
        "payload_is_data": probe.scalar("SELECT payload FROM zee_jobs WHERE id=" + first) == payload})
    probe.reset()
    for key in ("a", "b", "c"):
        probe.enqueue(key)
    apps = ["zee-job-ci-race-a", "zee-job-ci-race-b"]
    with PAROLA["SqlBlocker"](probe, "SELECT 1 FROM zee_job_limits FOR UPDATE"):
        children = [probe.start("claim", [app, "token-" + app], application=app) for app in apps]
        waiting = probe.wait_for_locks(apps, "UPDATE zee_job_limits")
    ids = [probe.finish(c).split(":")[0] for c in children]
    active = int(probe.scalar("SELECT count(*) FROM zee_jobs WHERE state='leased'"))
    passed("iki_worker_sirasiz_ayri_kira_tek_sahip_ve_limit", {
        "waiting_workers": waiting, "distinct_jobs": len(set(ids)), "active_leases": active,
        "third_worker_empty": probe.claim("third-worker") is None,
        "same_owner_empty": probe.claim(apps[0]) is None})
    probe.reset(); expired_job = probe.enqueue("expired"); current_job = probe.enqueue("current")
    require(probe.claim("reused-owner", "reused-token")[0] == expired_job, "old token lease missing")
    probe.sql("UPDATE zee_job_leases SET expires_at=clock_timestamp() WHERE job_id=" + expired_job)
    current = probe.claim("reused-owner", "reused-token")
    reuse_current = (current is not None and current[0] == current_job and probe.state(expired_job) == "leased" and
                     int(probe.scalar("SELECT count(*) FROM zee_jobs WHERE state='leased' AND lease_owner='reused-owner' AND lease_token='reused-token'")) == 2)
    probe.reset(); job = probe.enqueue(); old = probe.claim(); require(old is not None, "old lease missing")
    probe.expire(job); new = probe.claim("worker-two"); require(new is not None, "new lease missing")
    stale = probe.op("apply", *old) == "reddedildi"
    before = probe.effects(); probe.begin(new); require(probe.op("apply", *new) == "1", "valid SQL handler failed")
    repeated = probe.op("apply", *new) == "reddedildi"
    passed("suresi_dolan_kira_eski_nesli_reddeder_tek_etki", {
        "expired_token_reuse_selects_current_job": reuse_current,
        "generation_increased": new[0] == old[0] and int(new[2]) > int(old[2]),
        "stale_rejected": stale, "effects_before": before, "effects_after": probe.effects(),
        "repeat_rejected": repeated, "state_done": probe.state(job) == "done"})
    for recover, name in [(True, "iki_ret_sonrasi_ucuncu_deneme_bir_etki"),
                           (False, "uc_ret_olu_mektup_dorduncu_deneme_yok")]:
        probe.reset(); job = probe.enqueue(); delays = []; early = True
        probe.sql("INSERT INTO zee_job_fixture_failures(job_id) VALUES(" + job + ")")
        for attempt in (1, 2, 3):
            lease = probe.claim(); require(lease is not None, "due retry did not acquire a lease")
            probe.begin(lease)
            if recover and attempt == 3:
                probe.sql("DELETE FROM zee_job_fixture_failures WHERE job_id=" + job)
            marker = probe.op("apply", *lease)
            if marker == "reddedildi":
                require(probe.op("fail", *lease) == "1", "failed SQL attempt was not persisted")
                if attempt < 3:
                    delays.append(int(probe.sql("SELECT extract(epoch FROM available_at-transitioned_at)::integer FROM zee_jobs WHERE id=" + job)))
                    early = early and probe.claim("too-early") is None
                    probe.due(job)
            else:
                require(recover and attempt == 3 and marker == "1", "unexpected handler success")
        common = {"attempts": probe.attempts(job), "handler_statements": probe.statements(), "effects": probe.effects()}
        if recover:
            common.update(first_backoff=delays[0], second_backoff=delays[1], early_claim_empty=early,
                          state_done=probe.state(job) == "done")
        else:
            common.update(dead_letter=probe.state(job) == "dead_letter", fourth_claim_empty=probe.claim() is None)
        passed(name, common)
    probe.reset(); job = probe.enqueue(); lease = probe.claim(); probe.begin(lease)
    app = "zee-job-ci-kill-before"
    with PAROLA["SqlBlocker"](probe, "SELECT pg_advisory_xact_lock(247001)"):
        child = probe.start("apply", lease, application=app, barrier=True)
        waited = probe.wait_for_locks([app], "SELECT pg_advisory_xact_lock") == 1
        invisible = probe.effects() == 0 and probe.state(job) == "leased"
        child.kill(); child.communicate(timeout=10); killed = child.returncode == -9
    probe.wait_gone(app)
    after = probe.effects(); probe.expire(job); recovery = probe.claim("replacement"); probe.begin(recovery)
    require(probe.op("apply", *recovery) == "1", "killed job did not recover")
    passed("sigkill_commit_oncesi_yeniden_kira_tek_etki", {
        "actual_lock_wait": waited, "uncommitted_invisible": invisible, "killed": killed,
        "effects_after_kill": after, "effects_after_recovery": probe.effects(),
        "attempts": probe.attempts(job), "state_done": probe.state(job) == "done"})
    probe.reset(); job = probe.enqueue(); lease = probe.claim(); probe.begin(lease)
    app = "zee-job-ci-kill-after"; child = probe.start("apply", lease, application=app, pause=True)
    import select
    ready, _, _ = select.select([child.stdout], [], [], 15)
    require(bool(ready), "worker did not emit its post-COMMIT marker")
    ack = child.stdout.readline() == "1\n"
    child.kill(); child.communicate(timeout=10); probe.wait_gone(app)
    passed("sigkill_commit_sonrasi_uzlastirma_tekrar_yok", {
        "ack_observed": ack, "killed": child.returncode == -9,
        "reconciled_done": probe.op("read", "one", "1") == "done",
        "effects": probe.effects(), "handler_statements": probe.statements()})
    for mode in ("before", "after"):
        probe.reset(); job = probe.enqueue(); lease = probe.claim(); probe.begin(lease)
        app = "zee-job-ci-handler-" + mode
        proxy = PROXY["CommitProxy"]("127.0.0.1", probe.cluster.port, mode,
                                    application=app, deadline_seconds=30).start()
        try:
            child = probe.start("apply", lease, application=app, proxy=proxy)
            probe.finish(child, fault=True); wire = proxy.finish(); probe.wait_gone(app)
            before = probe.statements(); state = probe.op("read", "one", "1"); no_replay = probe.statements() == before == 1
            observations = {"c027": True, "stdout_empty": True, "commits_seen": wire["commits_seen"],
                "commit_forwarded": wire["commit_forwarded"], "commit_complete": wire["commit_complete"],
                "commit_ready": wire["commit_ready"], "effects": probe.effects(), "reconcile_no_replay": no_replay}
            if mode == "before": observations["state_leased"] = state == "leased"
            else: observations["state_done"] = state == "done"
            passed("handler_commit_iletilmeden_c027_uzlastirma" if mode == "before" else
                   "handler_commit_yanit_kayip_c027_uzlastirma", observations)
        finally: proxy.close()
    for mode in ("before", "after"):
        probe.reset(); app = "zee-job-ci-enqueue-" + mode
        proxy = PROXY["CommitProxy"]("127.0.0.1", probe.cluster.port, mode,
                                    application=app, deadline_seconds=30).start()
        try:
            child = probe.start("enqueue", ["one", "1", "synthetic-ok"], application=app, proxy=proxy)
            probe.finish(child, fault=True); proxy.finish(); probe.wait_gone(app)
            count = int(probe.scalar("SELECT count(*) FROM zee_jobs")); state = probe.op("read", "one", "1")
            if mode == "before":
                require(state == "yok", "missing enqueue did not reconcile")
                probe.enqueue()
                passed("enqueue_commit_iletilmeden_c027_anahtar_uzlastirma", {
                    "c027": True, "stdout_empty": True, "jobs_before_reconcile": count,
                    "reconciled_missing": state == "yok", "jobs_after_explicit_decision": int(probe.scalar("SELECT count(*) FROM zee_jobs"))})
            else:
                passed("enqueue_commit_yanit_kayip_c027_anahtar_uzlastirma", {
                    "c027": True, "stdout_empty": True, "jobs_before_reconcile": count,
                    "reconciled_ready": state == "ready", "jobs_after_reconcile": int(probe.scalar("SELECT count(*) FROM zee_jobs"))})
        finally: proxy.close()
    heartbeat_native_cases(probe, results)
    check_results(results, 0)


def heartbeat_native_cases(probe, results):
    """K-254: separate lease authority and real long-handler/fault observations."""
    def passed(name, observations):
        results.append({"name": name, "status": "passed", "ignored": False,
                        "observations": observations})
        print("native queue case: " + name + " ... passed", flush=True)
    probe.reset();job=probe.enqueue();lease=probe.claim();probe.begin(lease);app='zee-v2-heartbeat-locked-handler'
    with PAROLA['SqlBlocker'](probe,'SELECT pg_advisory_xact_lock(247001)'):
        child=probe.start('apply',lease,application=app,barrier=True)
        require(probe.wait_for_locks([app],'SELECT pg_advisory_xact_lock')==1,'real handler blocked')
        before=probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job)
        require(probe.renew(lease)=='1','independent renewal commit while handler row lock held')
        require(probe.wait_for_locks([app],'SELECT pg_advisory_xact_lock')==1,'old handler still blocked until control release')
        after=probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job)
        require((before,after)==('0','1'),'one actual committed renewal')
        held_started=time.monotonic();next_renew=time.monotonic()+5; renew_acks=1
        # Real unchanged30s lease budget; wait until server observes its original deadline passed.
        while probe.scalar('SELECT clock_timestamp()>lease_expires FROM zee_jobs WHERE id='+job)!='true' or time.monotonic()-held_started<35:
            require(time.monotonic()-held_started<40,'server clock reached original deadline within bounded fixture')
            if time.monotonic()>=next_renew:
                require(probe.renew(lease)=='1','actual periodic heartbeat committed');renew_acks+=1;next_renew=time.monotonic()+5
            time.sleep(.1)
        require(probe.wait_for_locks([app],'SELECT pg_advisory_xact_lock')==1,'handler still alive beyond original lease')
        require(probe.scalar('SELECT expires_at>clock_timestamp() FROM zee_job_leases WHERE job_id='+job)=='true','authoritative lease remains live beyond original30s')
        held_seconds=round(time.monotonic()-held_started,3)
    require(probe.finish(child)=='1','renewed handler completion');require(probe.effects()==1 and probe.state(job)=='done','durable one effect/done')
    passed('separate_lease_heartbeat_commits_while_handler_row_locked',{'handler_still_blocked_at_renewal_ack':True,'renewals_before':0,'renewals_after':int(probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job)),'heartbeat_acks':renew_acks,'held_seconds':held_seconds,'original30s_expired':True,'effects':1,'done':True})
    require(probe.renew(lease)=='reddedildi','done job cannot renew');passed('done_job_renewal_rejected',{'renewals':renew_acks})
    # Adversarial early revocation while compatibility expiry is still future.
    probe.reset();old_job=probe.enqueue('old');current_job=probe.enqueue('current');old=probe.claim('reuse-owner','reuse-token')
    probe.sql('UPDATE zee_job_leases SET expires_at=clock_timestamp() WHERE job_id='+old_job)
    require(probe.scalar('SELECT lease_expires>clock_timestamp() FROM zee_jobs WHERE id='+old_job)=='true','legacy placeholder deliberately still future')
    current=probe.claim('reuse-owner','reuse-token');require(current is not None and current[0]==current_job,'new exact generation upsert cannot revive old expired lease')
    require(probe.scalar('SELECT expires_at<=clock_timestamp() FROM zee_job_leases WHERE job_id='+old_job)=='true','old authoritative lease remains expired')
    require(probe.scalar("SELECT count(*) FROM zee_jobs j JOIN zee_job_leases l ON l.job_id=j.id WHERE j.state='leased' AND l.expires_at>clock_timestamp()")=='1','exact one live lease after token reuse')
    passed('early_revoked_lease_reused_token_does_not_resurrect_old_row',{'old_authoritative_expired':True,'old_compatibility_deadline_future':True,'returned_current_job':True,'active_leases':1})
    probe.reset();job=probe.enqueue();lease=probe.claim()
    rejected=[probe.renew((lease[0],'wrong-token',lease[2])),probe.renew((lease[0],lease[1],str(int(lease[2])+1))),probe.renew(lease,'wrong-owner')]
    require(rejected==['reddedildi']*3 and probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job)=='0','wrong ownership/fence rejected')
    passed('wrong_token_generation_owner_no_mutation',{'rejections':3,'renewals':0})
    cap=probe.scalar('SELECT renew_until::text FROM zee_job_leases WHERE job_id='+job);require(probe.renew(lease)=='1','valid renewal')
    require(cap==probe.scalar('SELECT renew_until::text FROM zee_job_leases WHERE job_id='+job),'renewal cannot move hard cap')
    require(probe.scalar('SELECT round(extract(epoch FROM renew_until-issued_at))::integer FROM zee_job_leases WHERE job_id='+job)=='120','actual default renewal window is120seconds')
    passed('renewal_keeps_120second_server_clock_cap',{'renew_until_unchanged':True,'renewals':1,'renew_window_seconds':120})
    probe.expire(job);require(probe.renew(lease)=='reddedildi','expired lease no resurrection');new=probe.claim('replacement')
    require(new is not None and int(new[2])>int(lease[2]),'fresh generation')
    require(probe.renew(lease)=='reddedildi' and probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job)=='0','old renewal cannot mutate replacement')
    passed('expired_old_generation_no_resurrection_or_replacement_mutation',{'generation_increased':True,'new_renewals':0})
    probe.sql('WITH an AS (SELECT clock_timestamp() AS t) UPDATE zee_job_leases SET expires_at=an.t,renew_until=an.t FROM an WHERE job_id='+job)
    require(probe.renew(new,'replacement')=='reddedildi','hard lifetime cap refusal');passed('absolute_lifetime_cap_rejected',{'renewals':0})
    # An abandoned heartbeat with a still-locked handler: actual unchanged30s expiration.
    probe.reset();job=probe.enqueue();lease=probe.claim();probe.begin(lease);app='zee-v2-expired-locked-handler'
    with PAROLA['SqlBlocker'](probe,'SELECT pg_advisory_xact_lock(247001)'):
        child=probe.start('apply',lease,application=app,barrier=True)
        require(probe.wait_for_locks([app],'SELECT pg_advisory_xact_lock')==1,'expired fixture actual SQL barrier')
        started=time.monotonic()
        while probe.scalar('SELECT expires_at<=clock_timestamp() FROM zee_job_leases WHERE job_id='+job)!='true':
            require(time.monotonic()-started<40,'actual original30s expiry bounded');time.sleep(.1)
        require(probe.op('reap')=='0','SKIP LOCKED reaper returns without blocked handler row')
        require(probe.wait_for_locks([app],'SELECT pg_advisory_xact_lock')==1,'old handler remains blocked after reaper ack')
        require(probe.renew(lease)=='reddedildi','expired heartbeat cannot revive locked handler')
        child.kill();child.communicate(timeout=10)
        require(child.returncode == -9,'owned expired handler was actually killed')
    # PostgreSQL may not observe TLS disconnect until blocked SQL is released.
    probe.wait_gone(app)
    require(probe.effects()==0 and probe.op('reap')=='1','actual killed expired backend freed; reaper recovers one')
    replacement=probe.claim('replacement');require(replacement is not None and int(replacement[2])>int(lease[2]),'replacement fenced')
    require(probe.op('apply',*lease)=='reddedildi' and probe.renew(lease)=='reddedildi','old generation revoked')
    probe.begin(replacement);require(probe.op('apply',*replacement)=='1','replacement actual SQL handler')
    require(probe.effects()==1 and probe.attempts(job)==2,'two reserved attempts one durable effect')
    passed('expired_locked_dead_worker_skiplocked_reaper_then_fenced_replacement',{'natural_lease_expiry':True,'reaper_while_locked':0,'reaper_after_backend_gone':1,'old_generation_rejected':True,'effects':1,'attempts':2,'no_heartbeat_resurrection':True,'backend_gone':True,'killed':True})
    for mode in ('before','after'):
        probe.reset();job=probe.enqueue();lease=probe.claim();app='zee-v2-heartbeat-commit-'+mode
        proxy=PROXY['CommitProxy']('127.0.0.1',probe.cluster.port,mode,application=app,deadline_seconds=30).start()
        try:
            child=probe.start('renew',(*lease,'worker-one'),application=app,proxy=proxy);probe.finish(child,fault=True);wire=proxy.finish();probe.wait_gone(app)
            actual=int(probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job));require(actual==(0 if mode=='before' else 1),'actual commit effect observed')
            require(probe.state(job)=='leased' and probe.effects()==0,'no SQL handler/admission continuation')
            passed('heartbeat_commit_'+mode+'_c027_no_replay',{'c027':True,'renewals':actual,'effects':0,'commits_seen':wire['commits_seen'],'commit_forwarded':wire['commit_forwarded'],'commit_complete':wire['commit_complete'],'commit_ready':wire['commit_ready']})
        finally:proxy.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", choices=("docker", "local"), default="docker")
    parser.add_argument("--postgres-bin-dir")
    parser.add_argument("--dil", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "compiler/target/is-kuyrugu-ci.json")
    args = parser.parse_args()
    if args.engine == "local" and not args.postgres_bin_dir:
        parser.error("local engine requires owned initdb/pg_ctl tools, never an existing URL")
    report = {"schema": "zee-is-kuyrugu-ci-1", "status": "failed", "phase": "initializing",
              "started_utc": datetime.now(timezone.utc).isoformat(), "cases": [], "ignored": 0,
              "host": platform.platform(),
              "boundary": "Owned anonymous SQL-only queue; generated fixed counter is a test instrument. No SMTP retry, customer cluster, cron grammar, new CLI command, product adoption, Python removal or live deployment acceptance."}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    run = TLS["run"]; source = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
    report["source_start_sha"] = source
    report["source_clean_start"] = not run(["git", "-C", str(ROOT), "status", "--porcelain"]).strip()
    if args.dil is None:
        run(["cargo", "build", "--locked", "--bin", "dil"], cwd=ROOT / "compiler", timeout=900)
    original = (args.dil or ROOT / "compiler/target/debug/dil").resolve()
    paths = [Path(__file__), ROOT / "scripts/tests/is-kuyrugu-ci-test.py",
             QUEUE / "proje.dil", QUEUE / "README.md", QUEUE / "kaynak/kuyruk.dil", QUEUE / "gocler/0001_kuyruk.sql", QUEUE / "gocler/0002_kira_sahibi.sql",
             ROOT / "scripts/postgresql-tls-ci.py", ROOT / "scripts/postgresql-commit-proxy.py",
             ROOT / "scripts/parola-goc-ci.py"]
    inputs = {str(path.relative_to(ROOT)): path.read_bytes() for path in paths}
    report["fixture_hashes"] = [{"path": path, "sha256": digest(value)} for path, value in sorted(inputs.items())]
    started = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="zee-job-ci-") as name:
        directory = Path(name); binary = directory / "dil"; shutil.copy2(original, binary)
        identity = json.loads(subprocess.check_output([str(binary), "sürüm", "--json"], text=True))
        require(identity.get("schema") == "zee-build-v1" and identity.get("source_sha") == source and
                identity.get("dirty") is False, "queue requires an actual clean-source binary for selected HEAD")
        report["build_identity"] = identity; report["binary_sha256"] = digest(binary.read_bytes())
        ca, _, cert, key = TLS["certificates"](directory)
        peer = "samenet" if args.engine == "docker" else "127.0.0.1/32"
        (directory / "pg_hba.conf").write_text("local all all trust\nhostssl all all 0.0.0.0/0 trust\n"+
                                             f"hostnossl all all {peer} trust\nhostnossl all all 0.0.0.0/0 reject\n")
        cluster = (TLS["DockerCluster"](directory, cert, key) if args.engine == "docker" else
                   TLS["LocalCluster"](directory, cert, key, args.postgres_bin_dir))
        probe = None
        try:
            cluster.start(); report["server"] = cluster.version; report["engine"] = args.engine
            if args.engine == "local":
                report["runtime_hashes"] = [
                    {"path": name, "sha256": digest((cluster.binaries / name).read_bytes())}
                    for name in ("postgres", "initdb", "pg_ctl", "psql")]
            else:
                report["runtime_identity"] = cluster.identity
            probe = Probe(directory, cluster, binary, ca, inputs[str((QUEUE / "kaynak/kuyruk.dil").relative_to(ROOT))].decode())
            probe.sql(inputs[str((QUEUE / "gocler/0001_kuyruk.sql").relative_to(ROOT))].decode())
            probe.sql(inputs[str((QUEUE / "gocler/0002_kira_sahibi.sql").relative_to(ROOT))].decode()); probe.sql(FIXTURE_SQL)
            report["phase"] = "native-cases"; native_cases(probe, report["cases"])
            report["status"] = "passed"; report["phase"] = "completed"
        finally:
            if probe: probe.close()
            cluster.close(); report["owned_cluster_closed"] = True
            report["source_end_sha"] = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
            report["source_clean_end"] = not run(["git", "-C", str(ROOT), "status", "--porcelain"]).strip()
            report["source_unchanged"] = report["source_end_sha"] == source
            report["fixture_unchanged"] = all((ROOT / path).read_bytes() == value for path, value in inputs.items())
            if not report["source_unchanged"] or not report["fixture_unchanged"]:
                report["status"] = "failed"; report["phase"] = "source-or-fixture-mutated"
            report["source_acceptance"] = ("clean-exact" if report["source_clean_start"] and report["source_clean_end"]
                                             and report["source_unchanged"] else "working-tree-only")
            report["elapsed_seconds"] = round(time.monotonic() - started, 3)
            args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    require(report["status"] == "passed", "queue source or fixture changed during the campaign")
    print("Native queue gate: 21/21 passed, 0 ignored; report=" + str(args.output))


if __name__ == "__main__":
    main()
