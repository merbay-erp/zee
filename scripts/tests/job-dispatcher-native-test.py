from pathlib import Path
import argparse, datetime, hashlib, importlib.util, json, os, runpy, signal, shutil, subprocess, tempfile, time
import sys, types, queue, threading
parser = argparse.ArgumentParser(description='Actual owned-PG finite SQL dispatcher acceptance')
parser.add_argument('--framework-root', type=Path, required=True)
parser.add_argument('--dil', type=Path, required=True)
parser.add_argument('--postgres-bin-dir', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
ARGS = parser.parse_args()
ROOT = ARGS.framework_root.resolve(strict=True)
OUTPUT = ARGS.output.resolve()
OUTPUT.mkdir(parents=True, mode=448)
FIXTURE = ROOT / 'dogfood/zee-cekirdek/is-kuyrugu-dispatcher'
HELPERS = ROOT / 'scripts/tests/fixtures/job-dispatcher'
DISPATCHER = ROOT / 'scripts/job-dispatcher.py'
MOD = runpy.run_path(str(ROOT / 'scripts/is-kuyrugu-ci.py'))
TLS = MOD['TLS']
require = MOD['require']
spec = importlib.util.spec_from_file_location('dispatcher', DISPATCHER)
host = importlib.util.module_from_spec(spec)
spec.loader.exec_module(host)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
EXPECTED_CASES = ['two_owned_dispatchers_fixed_sql_intents_once', 'actual_dispatcher_35second_fixed_handler_with_independent_heartbeats', 'actual_term_pauses_admission_no_retry_or_uncertain_effect', 'known_sql_refusals_three_attempts_deadletter', 'actual_commit_before_pauses_host_without_replay', 'actual_commit_after_pauses_host_without_replay', 'ambient_credential_rejected_no_fallback_or_secret_log', 'unread_full_log_pipe_drops_records_actual_sql_still_completes', 'killed_owned_leader_native_descendant_group_cleanup_no_foreign_adoption', 'fifo_credential_rejected_without_blocking_or_sql_effect', 'invalid_argument_canary_masked_no_argv_usage_stderr_leak']
H = runpy.run_path(str(ROOT / 'scripts/lifecycle_core.py'))
# runpy returns a shallow copy; the retained owner is the functions' actual globals.
H = H['run'].__globals__
assert all(H[name].__globals__ is H for name in ('run', 'stop_owned', 'held_status'))
PG = runpy.run_path(str(ROOT / 'scripts/tests/native-pg-owner.py'))
started = time.monotonic()
deadline = started + 600
cleanup_limit = None
cleaning = False
cancelled = False
oldsignals = {}
r = {}
work = None
work_identity = None
cluster = None
probe = None
children = []
components = []
cleanup_errors = []
private_uncertain = False
public_commands = []
raw_metadata = []

def need(value, message='owned job harness rejected'):
    if not value:
        raise ValueError(message)

def active():
    need(not cancelled and (not H['CANCELLED']) and (time.monotonic() < deadline), 'job work cancelled/deadline')

def cancel(*_):
    global cancelled
    cancelled = True
    if not cleaning:
        H['CANCELLED'] = True

def cleanup_end():
    global cleanup_limit
    if cleanup_limit is None:
        cleanup_limit = time.monotonic() + 90
    return cleanup_limit

def timeout(limit):
    active()
    return min(limit, max(0.001, deadline - time.monotonic()))

def require(value, message):
    active()
    MOD['require'](value, message)

def safe_command(role, argv, limit=20, env=None):
    global private_uncertain
    active()
    permitted = {'certificate-setup': {'openssl'}, 'owned-PG-initdb': {'initdb'}, 'owned-PG-version': {'postgres'}, 'binary-version': {'dil'}, 'source-head': {'git'}}
    tool = Path(str(argv[0])).name
    need(role in permitted and tool in permitted[role], 'explicit public tool role')
    index = len(public_commands) + 1
    path = OUTPUT / f'command-{index:03d}.log'
    primary = None
    try:
        result = H['run'](argv, timeout=timeout(limit), limit=1048576, env=env, log_path=path)
        active()
        raw_metadata.append({'role': role, 'tool': tool, 'stdout_sha256': hashlib.sha256(result).hexdigest(), 'bytes': len(result)})
        return result
    except BaseException as error:
        private_uncertain = True
        primary = error
        raise
    finally:
        try:
            data = path.read_bytes()
            need(len(data) <= 1048576, 'bounded public command output')
            public_commands.append({'role': role, 'tool': tool, 'name': path.name, 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)})
        except BaseException:
            if primary is None:
                raise

class JobPgCampaign:

    def __init__(self, directory):
        self.out = OUTPUT
        self.deadline = deadline
        self._cleanup_deadline = None
        self.clusters = []
        info = directory.stat()
        self.tempdirs = [(directory, (info.st_dev, info.st_ino))]
        self.proof = r
        self.pg_pins = {str(ARGS.postgres_bin_dir.resolve(strict=True) / n): digest(ARGS.postgres_bin_dir.resolve(strict=True) / n) for n in ('postgres', 'initdb', 'pg_ctl', 'psql')}

    @property
    def stopped(self):
        return cancelled or H['CANCELLED']

    @property
    def cleaning(self):
        return cleaning

    @property
    def cleanup_deadline(self):
        if cleaning and self._cleanup_deadline is None:
            self._cleanup_deadline = cleanup_end()
        return self._cleanup_deadline

    def need(self, value):
        end = self.cleanup_deadline if cleaning else self.deadline
        need(value and time.monotonic() < end and (cleaning or not self.stopped), 'job PG fixture acceptance/cleanup budget')

    def command(self, label, argv, budget=20, env=None):
        return safe_command(label, argv, budget, env)

def register_component(role, value, close):
    row = {'role': role, 'value': value, 'close': close, 'closed': False}
    components.append(row)
    return row

def close_component(row):
    try:
        row['close']()
        row['closed'] = True
    except BaseException:
        row['closed'] = False
        raise

class OwnedContext:

    def __init__(self, value):
        self.value = value
        self.row = register_component('SQL-blocker', value, self.close)

    def close(self):
        child = self.value.process
        first = None
        end = cleanup_end() if cleaning else deadline
        try:
            if child is not None and child.returncode is None:
                child.communicate(input="COMMIT;\n\\q\n", timeout=min(20, max(0.001, end - time.monotonic())))
        except BaseException as error:
            first = error
        if child is not None:
            try:
                close_direct(child)
            except BaseException as error:
                if first is None:
                    first = error
        reader = getattr(self, 'reader', None)
        if reader is not None:
            try:
                reader.join(timeout=min(1, max(0.001, end - time.monotonic())))
                need(not reader.is_alive(), 'original SQL blocker reader remains')
            except BaseException as error:
                if first is None:
                    first = error
        if first is not None:
            raise first

    def __enter__(self):
        try:
            active()
            # Original direct child, deliberately no process-group ownership claim.
            self.value.process = subprocess.Popen(self.value.probe.psql_command(True),
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            child = self.value.process
            child.stdin.write("BEGIN;\n" + self.value.lock_sql + ";\nSELECT 'BARRIER_READY';\n")
            child.stdin.flush()
            ready = queue.Queue()
            def read_ready():
                try:
                    for line in child.stdout:
                        if line.strip() == 'BARRIER_READY':
                            ready.put(True)
                            return
                    ready.put(False)
                except BaseException:
                    ready.put(False)
            self.reader = threading.Thread(target=read_ready, daemon=True)
            self.reader.start()
            need(ready.get(timeout=timeout(20)), 'SQL blocker did not acquire its own fixture lock')
            self.reader.join(timeout=timeout(1))
            need(not self.reader.is_alive(), 'SQL blocker barrier reader remains')
            active()
            return self.value
        except BaseException as primary:
            try:
                close_component(self.row)
            except BaseException:
                pass
            raise primary

    def __exit__(self, kind, value, tb):
        try:
            close_component(self.row)
        except BaseException:
            if value is not None:
                raise value.with_traceback(tb)
            raise
        return False

def own_dispatcher(value):
    register_component('component-dispatcher', value, value.cleanup)
    return value

def own_proxy(value):
    row = register_component('loopback-COMMIT-proxy', value, value.close)
    value.start()
    active()
    return value

def cleanup_all(*callbacks, terminal=False):
    global cleaning
    primary = sys.exc_info()[1]
    first = primary
    previous = cleaning
    cleaning = previous or terminal
    if terminal:
        cleanup_end()
    for callback in callbacks:
        try:
            callback()
        except BaseException as error:
            cleanup_errors.append(type(error).__name__)
            if first is None:
                first = error
    cleaning = previous
    if first is not None:
        raise first

class OwnedProbe(MOD['Probe']):

    def sql(self, statement):
        global private_uncertain
        active()
        try:
            result = H['run'](self.psql_command() + ['--command', statement], timeout=timeout(20), limit=1048576)
            active()
            return result.decode().strip()
        except BaseException:
            private_uncertain = True
            raise

    def close(self):
        first = None
        for child in self.children:
            try:
                close_direct(child)
            except BaseException as error:
                if first is None:
                    first = error
        if first is not None:
            raise first

def close_direct(child):
    end = cleanup_end() if cleaning else deadline
    first = None
    try:
        if child.returncode is None:
            child.terminate()
            try:
                child.communicate(timeout=min(3, max(0.001, end - time.monotonic())))
            except subprocess.TimeoutExpired:
                child.kill()
                child.communicate(timeout=min(3, max(0.001, end - time.monotonic())))
        need(child.returncode is not None, 'original direct child unreaped')
    except BaseException as error:
        first = error
    for stream in (child.stdin, child.stdout, child.stderr):
        if stream is not None:
            try:
                stream.close()
            except BaseException as error:
                if first is None:
                    first = error
    if first is not None:
        raise first

def close_children():
    first = None
    for child in children:
        try:
            if child.returncode is None:
                H['stop_owned'](child, grace=min(3, max(0, cleanup_end() - time.monotonic())))
            need(child.returncode is not None, 'original session child unreaped')
            for stream in (child.stdin, child.stdout, child.stderr):
                if stream is not None:
                    stream.close()
        except BaseException as error:
            if first is None:
                first = error
    if first is not None:
        raise first

def close_components():
    first = None
    for row in reversed(components):
        if row['closed']:
            continue
        try:
            close_component(row)
        except BaseException as error:
            if first is None:
                first = error
    if first is not None:
        raise first

def close_probe():
    if probe is not None:
        probe.close()

def close_pg():
    previous = H['CANCELLED']
    H['CANCELLED'] = False
    try:
        if cluster is not None:
            cluster.close()
    finally:
        H['CANCELLED'] = previous or cancelled or H['CANCELLED']

def close_work():
    if work is None:
        return
    outstanding = private_uncertain or cleanup_errors or any((c.returncode is None for c in children)) or any((not row['closed'] for row in components)) or (probe is not None and any((c.returncode is None for c in probe.children))) or (cluster is not None and (not cluster.closed or (cluster.child is not None and cluster.child.returncode is None) or any((row['child'] is not None and row['child'].returncode is None for row in cluster.probes))))
    if outstanding:
        r['private_root_preserved_after_uncertain_owner'] = True
        raise ValueError('owned job resources uncertain; private root retained')
    info = work.lstat()
    need(work.resolve(strict=True) == work and (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid) == work_identity, 'private root identity changed')
    PG['private_directory'](info)
    need(time.monotonic() < cleanup_end(), 'private root cleanup deadline')
    shutil.rmtree(work)
    need(not work.exists(), 'private root remains')

def publish(primary):

    def verdict():
        pg = r.get('postgres_clusters', [])
        removed = work is not None and (not work.exists())
        r.update(cancelled=cancelled or H['CANCELLED'], deadline_expired=time.monotonic() >= deadline, private_root_removed=removed, cleanup_deadline_expired=cleanup_limit is not None and time.monotonic() >= cleanup_limit, cleanup_errors=list(cleanup_errors), owned_cluster_closed=bool(cluster is not None and cluster.closed), all_session_children_reaped=all((c.returncode is not None for c in children)), all_probe_children_reaped=probe is None or all((c.returncode is not None for c in probe.children)), all_components_closed=all((row['closed'] for row in components)), elapsed_seconds=time.monotonic() - started, command_logs=public_commands)
        good = r.get('body_completed') is True and primary is None and (not cleanup_errors) and (not private_uncertain) and removed and (not r['cancelled']) and (not r['deadline_expired']) and (not r['cleanup_deadline_expired']) and (len(pg) == 1) and ([x.get('name') for x in r['cases']] == EXPECTED_CASES) and all((x.get('passed') is True and x.get('ignored') is False for x in r['cases'])) and (pg[0].get('normal_shutdown') is True) and (pg[0].get('reaped') is True) and (pg[0].get('fallback_used') is False) and r['all_session_children_reaped'] and r['all_probe_children_reaped'] and r['all_components_closed']
        r['status'] = 'passed' if good else 'failed'

    def write():
        (OUTPUT / 'proof.json').write_text(json.dumps(r, ensure_ascii=False, indent=2) + '\n')
    try:
        verdict()
        (OUTPUT / 'raw-metadata.json').write_text(json.dumps(raw_metadata, indent=2) + '\n')
        verdict()
        write()
        verdict()
        if r['status'] != 'passed':
            write()
    except BaseException as error:
        r['status'] = 'failed'
        r['publication_failure'] = type(error).__name__
        if primary is None:
            primary = error
        try:
            write()
        except BaseException as error:
            r['failed_proof_write_error'] = type(error).__name__
    finally:
        errors = []
        for number, handler in oldsignals.items():
            try:
                signal.signal(number, handler)
            except BaseException as error:
                errors.append(type(error).__name__)
                if primary is None:
                    primary = error
        if errors:
            r['handler_restore_errors'] = errors
    try:
        verdict()
    except BaseException as error:
        r['status'] = 'failed'
        r['finalization_failure'] = type(error).__name__
        if primary is None:
            primary = error
    if primary is not None or r['status'] != 'passed' or r.get('handler_restore_errors'):
        r['status'] = 'failed'
        try:
            write()
        except BaseException as error:
            if primary is None:
                primary = error
        if primary is not None:
            raise primary
        raise ValueError('owned job acceptance failed')
    return None
TLS = dict(TLS)
_cert_env = dict(TLS)
_cert_env['run'] = lambda argv, **kwargs: safe_command('certificate-setup', argv, kwargs.pop('timeout', 20)).decode()
TLS['certificates'] = types.FunctionType(TLS['certificates'].__code__, _cert_env)

def install_handlers():
    for number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        # Retain restoration authority before the mutator can partially fail.
        oldsignals[number] = signal.getsignal(number)
        signal.signal(number, cancel)

def main():
    global r, work, work_identity, cluster, probe, children
    primary = None
    r = {'schema': 'zee-sql-intent-dispatcher-native-1', 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'source_acceptance': 'actual-given-binary-and-candidate-fixture', 'cases': [], 'boundaries': ['SQL durable intent only; not catalog SMTP/PDF service', 'finite100tick maximum prototype; K246 ownedgroups/nonblockinglog reused', 'no calendar cron or live product adoption; K245 credential reader reused inline for fixed stdout protocol, not CLIrun_child', 'no clean final-source acceptance', 'COMMIT proxy tests invoke host component only with scoped owned plaintext fixture; CLI default stays TLSrequire']}
    r['status'] = 'running'
    r['source_acceptance'] = 'exact-clean-source-and-pinned-binary-required'
    r['boundaries'] = [x for x in r['boundaries'] if x != 'no clean final-source acceptance']
    try:
        install_handlers()
        active()
        work = Path(tempfile.mkdtemp(prefix='zjd-', dir=Path('/tmp').resolve(strict=True))).resolve(strict=True)
        PG['private_directory'](work.lstat())
        info = work.lstat()
        work_identity = (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid)
        r['framework_head'] = safe_command('source-head', ['git', '-C', ROOT, 'rev-parse', 'HEAD']).decode().strip()
        H['clean_exact'](ROOT, r['framework_head'])
        d = work
        binary = d / 'dil'
        shutil.copy2(ARGS.dil.resolve(strict=True), binary)
        binary.chmod(320)
        r['binary_sha256'] = digest(binary)
        r['build_identity'] = json.loads(safe_command('binary-version', [str(binary), 'sürüm', '--json']))
        need(r['build_identity']['source_sha'] == r['framework_head'] and r['build_identity']['dirty'] is False, 'same clean release source')
        r['fixture_hashes'] = {str(f.relative_to(ROOT)): digest(f) for f in [*FIXTURE.rglob('*.dil'), *FIXTURE.rglob('*.sql'), DISPATCHER, HELPERS / 'backpressure-driver.py', HELPERS / 'leader-driver.py', ROOT / 'scripts/worker-supervisor.py', ROOT / 'scripts/credential-baslat.py', ROOT / 'scripts/native-capabilities.py', ROOT / 'scripts/is-kuyrugu-ci.py', ROOT / 'scripts/postgresql-tls-ci.py', ROOT / 'scripts/postgresql-tls11-control.py', ROOT / 'dogfood/zee-cekirdek/is-kuyrugu/kaynak/kuyruk.dil', ROOT / 'scripts/tests/native-pg-owner.py', ROOT / 'scripts/lifecycle_core.py', ROOT / 'scripts/parola-goc-ci.py', ROOT / 'scripts/postgresql-commit-proxy.py', Path(__file__).resolve()]}
        ca, _, cert, key = TLS['certificates'](d)
        (d / 'pg_hba.conf').write_text('local all all trust\nhostssl all all 0.0.0.0/0 trust\nhostnossl all all 127.0.0.1/32 trust\nhostnossl all all 0.0.0.0/0 reject\n')
        campaign = JobPgCampaign(d)
        cluster = PG['OwnedLocalCluster'](campaign, d, cert, key, ARGS.postgres_bin_dir.resolve(strict=True), H)
        campaign.clusters.append(cluster)
        probe = None

        def passed(name, obs):
            r['cases'].append({'name': name, 'passed': True, 'ignored': False, 'observations': obs})
            print(name + ' ... passed', flush=True)
            (OUTPUT / 'progress.json').write_text(json.dumps({'status': 'running', 'cases': r['cases']}, ensure_ascii=False) + '\n')
        try:
            cluster.start()
            r['server'] = cluster.version
            probe = OwnedProbe(d, cluster, binary, ca, (ROOT / 'dogfood/zee-cekirdek/is-kuyrugu/kaynak/kuyruk.dil').read_text())
            for name in ['0001_kuyruk.sql', '0002_kira_sahibi.sql', '0003_sabit_niyet.sql']:
                probe.sql((FIXTURE / 'gocler' / name).read_text())

            def project(port):
                p = d / ('project-' + str(port))
                p.mkdir(exist_ok=True)
                (p / 'gocler').mkdir(exist_ok=True)
                for name in host.FILES:
                    f = p / name
                    value = (FIXTURE / name).read_bytes()
                    if name == 'proje.dil':
                        value = value.replace(b'localhost:18093', ('localhost:' + str(port)).encode())
                    f.write_bytes(value)
                    f.chmod(384)
                return p

            def reset():
                probe.sql('TRUNCATE zee_job_intents,zee_job_leases,zee_jobs RESTART IDENTITY')
            credentials = d / 'credentials'
            credentials.mkdir(mode=448)
            for name, value in [('db', f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require'), ('sentinel', 'PRIVATE-K258-CREDENTIAL-SENTINEL')]:
                target = credentials / name
                target.write_text(value)
                target.chmod(256)

            def launch(p, owner, ticks):
                argv = ['python3', '-I', '-B', str(DISPATCHER), '--dil', str(binary), '--project', str(p), '--expected-source', r['build_identity']['source_sha'], '--expected-binary-hash', r['binary_sha256'], '--expected-project-hash', host.project_digest(host.project_snapshot(p)), '--owned-pg-port', str(cluster.port), '--owned-ca', str(ca), '--credential-root', str(credentials.resolve()), '--credential-env', 'ZEE_JOB_CI_URL=db', '--credential-env', 'APP_SENTINEL=sentinel', '--owner', owner, '--ticks', str(ticks)]
                child = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
                children.append(child)
                return child
            p = project(cluster.port)
            reset()
            for n in range(1, 5):
                probe.enqueue('job-' + str(n), payload="opaque '); DROP TABLE zee_jobs; --")
            actors = [launch(p, 'actor-a', 12), launch(p, 'actor-b', 12)]
            records = []
            for actor in actors:
                out, err = actor.communicate(timeout=30)
                r.setdefault('host_receipts', []).append({'role': 'canonical-host-actor', 'exit': actor.returncode, 'reaped': actor.returncode is not None, 'stdout_bytes': len(out.encode()), 'stderr_bytes': len(err.encode())})
                require(actor.returncode == 0 and (not err), 'actual host dispatcher completion')
                records.extend((json.loads(line) for line in out.splitlines()))
            count = int(probe.scalar('SELECT count(*) FROM zee_job_intents'))
            done = int(probe.scalar("SELECT count(*) FROM zee_jobs WHERE state='done'"))
            attempts = int(probe.scalar('SELECT sum(attempts) FROM zee_jobs'))
            require((count, done, attempts) == (4, 4, 4), 'two actual dispatchers four fixed SQL intents once')
            require(int(probe.scalar('SELECT count(*) FROM zee_job_intents i JOIN zee_jobs j ON i.job_id=j.id AND i.payload=j.payload AND i.approval_revision=j.approval_revision')) == 4, 'payload data preserved')
            passed('two_owned_dispatchers_fixed_sql_intents_once', {'actors': 2, 'intents': count, 'done': done, 'attempts': attempts, 'finished_events': sum((e['event'] == 'finished' for e in records)), 'payload_data_preserved': True})
            reset()
            job = probe.enqueue()
            probe.sql('CREATE FUNCTION zee_stage_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(254002); RETURN NEW; END $$; CREATE TRIGGER zee_stage_pause BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_stage_pause();')
            with OwnedContext(MOD['PAROLA']['SqlBlocker'](probe, 'SELECT pg_advisory_xact_lock(254002)')):
                child = launch(p, 'long-handler', 1)
                limit = time.monotonic() + 10
                while probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'") != '1':
                    require(time.monotonic() < limit, 'actual fixed SQL intent host reached barrier')
                    time.sleep(0.1)
                held = time.monotonic()
                while time.monotonic() - held < 35:
                    require(child.poll() is None, 'host stays alive while actual handler runs')
                    time.sleep(0.1)
                renewals = int(probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id=' + job))
                original_expired = probe.scalar('SELECT lease_expires<clock_timestamp() FROM zee_jobs WHERE id=' + job) == 'true'
                require(renewals >= 6 and original_expired, 'actual host heartbeats beyond unchanged original30s')
                require(probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'") == '1', 'handler still blocked at heartbeat observation')
            out, err = child.communicate(timeout=20)
            events = [json.loads(line) for line in out.splitlines()]
            require(child.returncode == 0 and (not err), 'long host completes after owned barrier release')
            require(probe.state(job) == 'done' and probe.attempts(job) == 1 and (int(probe.scalar('SELECT count(*) FROM zee_job_intents')) == 1), 'actual long host durable intent done once')
            require(events[-1]['heartbeat_acks'] == renewals, 'hostack equals independent DB counter')
            passed('actual_dispatcher_35second_fixed_handler_with_independent_heartbeats', {'held_seconds': round(time.monotonic() - held, 3), 'renewals': renewals, 'heartbeat_acks': events[-1]['heartbeat_acks'], 'original30s_expired': True, 'attempts': 1, 'intents': 1, 'done': True})
            reset()
            job = probe.enqueue()
            with OwnedContext(MOD['PAROLA']['SqlBlocker'](probe, 'SELECT pg_advisory_xact_lock(254002)')):
                child = launch(p, 'cancelled-handler', 1)
                limit = time.monotonic() + 10
                while probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'") != '1':
                    require(time.monotonic() < limit, 'cancelled actual host reached barrier')
                    time.sleep(0.1)
                pid = int(probe.scalar("SELECT pid FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'"))
                child.terminate()
                out, err = child.communicate(timeout=10)
                require(child.returncode == 3 and (not err) and any((json.loads(line)['event'] == 'paused_uncertain' for line in out.splitlines())), 'TERM actual host pauses no retry write')
            limit = time.monotonic() + 10
            while probe.scalar('SELECT count(*) FROM pg_stat_activity WHERE pid=' + str(pid)) != '0':
                require(time.monotonic() < limit, 'actual old backend gone after owned barrier release')
                time.sleep(0.1)
            require(probe.attempts(job) == 1 and probe.state(job) == 'leased' and (int(probe.scalar('SELECT count(*) FROM zee_job_intents')) == 0), 'cancelled handler did not persist intent or false retry outcome')
            passed('actual_term_pauses_admission_no_retry_or_uncertain_effect', {'exit': 3, 'backend_gone': True, 'attempts': 1, 'state_leased': True, 'intents': 0})
            probe.sql('DROP TRIGGER zee_stage_pause ON zee_job_intents; DROP FUNCTION zee_stage_pause()')
            reset()
            job = probe.enqueue()
            probe.sql("CREATE FUNCTION zee_stage_refuse() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'synthetic refusal' USING ERRCODE='23514'; END $$; CREATE TRIGGER zee_stage_refuse BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_stage_refuse();")
            child = launch(p, 'refusal-actor', 40)
            out, err = child.communicate(timeout=30)
            events = [json.loads(line) for line in out.splitlines()]
            require(child.returncode == 0 and (not err) and (probe.attempts(job) == 3) and (probe.state(job) == 'dead_letter') and (int(probe.scalar('SELECT count(*) FROM zee_job_intents')) == 0), 'actual three knownSQL failures reach deadletter')
            require(sum((e['event'] == 'known_sql_failure' for e in events)) == 3, 'three explicit acknowledgements no fourth')
            passed('known_sql_refusals_three_attempts_deadletter', {'attempts': 3, 'known_failures': 3, 'intents': 0, 'dead_letter': True})
            probe.sql('DROP TRIGGER zee_stage_refuse ON zee_job_intents; DROP FUNCTION zee_stage_refuse()')
            for mode in ['before', 'after']:
                reset()
                probe.enqueue()
                app = 'dispatcher-fault-' + mode
                proxy = own_proxy(MOD['PROXY']['CommitProxy']('127.0.0.1', cluster.port, mode, application=app, deadline_seconds=30))
                pp = project(proxy.port)
                env = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8', 'TZ': 'UTC', 'ZEE_JOB_CI_URL': f'host=localhost port={proxy.port} user=postgres dbname=postgres sslmode=disable application_name={app}'}
                dispatcher = own_dispatcher(host.Dispatcher(binary, pp, env, 'fault-actor'))
                paused = False
                try:
                    try:
                        dispatcher.tick()
                    except host.Uncertain:
                        paused = True
                    wire = proxy.finish()
                    probe.wait_gone(app)
                    require(paused and wire['commits_seen'] == 1 and (int(probe.scalar('SELECT count(*) FROM zee_job_intents')) == 0) and (probe.attempts('1') == 0), 'actual COMMIT uncertainty pauses before further writes/admission')
                    passed('actual_commit_' + mode + '_pauses_host_without_replay', {'paused': True, 'commits_seen': 1, 'commit_forwarded': wire['commit_forwarded'], 'commit_complete': wire['commit_complete'], 'attempts': 0, 'intents': 0, 'owned_children_remaining': len(dispatcher.children)})
                finally:
                    cleanup_all(dispatcher.cleanup, proxy.close)
            reset()
            probe.enqueue()
            os.environ['ZEE_JOB_CI_URL'] = 'AMBIENT-K258-FORBIDDEN'
            child = launch(p, 'ambient-reject', 1)
            try:
                out, err = child.communicate(timeout=10)
                require(child.returncode == 2 and (not err) and ('AMBIENT-K258-FORBIDDEN' not in out) and ('PRIVATE-K258-CREDENTIAL-SENTINEL' not in out), 'ambient credential is rejected masked')
                require(probe.attempts('1') == 0 and int(probe.scalar('SELECT count(*) FROM zee_job_intents')) == 0, 'no operation after rejected credential')
            finally:
                del os.environ['ZEE_JOB_CI_URL']
            passed('ambient_credential_rejected_no_fallback_or_secret_log', {'exit': 2, 'attempts': 0, 'intents': 0, 'secret_in_log': False})
            reset()
            job = probe.enqueue()
            cfg = d / 'component-env.json'
            cfg.write_text(json.dumps({'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8', 'TZ': 'UTC', 'ZEE_JOB_CI_URL': f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require', 'ZEE_JOB_CI_URL_TLS_CA_PEM': ca.read_text()}))
            cfg.chmod(384)
            receipt = d / 'backpressure.json'
            actor = subprocess.Popen(['python3', '-I', '-B', str(HELPERS / 'backpressure-driver.py'), str(binary), str(p), str(cfg), str(receipt)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            children.append(actor)
            actor.wait(timeout=15)
            out, err = actor.communicate(timeout=2)
            obs = json.loads(receipt.read_text())
            require(actor.returncode == 0 and (not err) and (obs['dropped_before_sql'] > 0) and (obs['dropped_after_sql'] >= obs['dropped_before_sql']) and (obs['applied'] == 1) and (obs['owned_children_remaining'] == 0), 'actual log backpressure bounded while SQL tick completes')
            require(probe.state(job) == 'done' and probe.attempts(job) == 1 and (probe.scalar('SELECT count(*) FROM zee_job_intents') == '1'), 'durable intent while unread log pipe full')
            lines = out.splitlines()
            require(all((json.loads(line)['schema'] == 'zee-sql-intent-dispatcher-1' for line in lines)), 'atomic complete log records under saturation')
            obs.update({'log_records_received': len(lines), 'intents': 1, 'attempts': 1, 'done': True, 'stdout_unread_until_child_exit': True})
            passed('unread_full_log_pipe_drops_records_actual_sql_still_completes', obs)
            reset()
            job = probe.enqueue()
            env = json.loads(cfg.read_text())
            dispatcher = own_dispatcher(host.Dispatcher(binary, p, env, 'owned-leader-loss'))
            token = 'lease-owned-leader-loss'
            claim = dispatcher.query('claim', token=token)
            jid, generation = claim.split(':')
            lease = (jid, token, generation)
            require(dispatcher.query('begin', lease) == '1', 'leader loss handler started')
            probe.sql('CREATE FUNCTION zee_stage_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(258002); RETURN NEW; END $$; CREATE TRIGGER zee_stage_pause BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_stage_pause();')
            foreign = subprocess.Popen(['python3', '-I', '-c', 'import time; time.sleep(30)'], start_new_session=True)
            children.append(foreign)
            pidfile = d / 'native-descendant.pid'
            try:
                with OwnedContext(MOD['PAROLA']['SqlBlocker'](probe, 'SELECT pg_advisory_xact_lock(258002)')):
                    leader = dispatcher.group_owner.spawn(['python3', '-I', '-B', str(HELPERS / 'leader-driver.py'), str(pidfile), str(binary), 'çalıştır', str(p), 'apply', jid, token, generation, 'owned-leader-loss'], capture=True, environment=env)
                    os.set_blocking(leader.stdout.fileno(), False)
                    limit = time.monotonic() + 10
                    while probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'") != '1':
                        require(time.monotonic() < limit, 'actual native descendant reached SQL barrier')
                        time.sleep(0.1)
                    backend = int(probe.scalar("SELECT pid FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'"))
                    native_pid = int(pidfile.read_text())
                    require(os.getpgid(native_pid) == leader.pid, 'native belongs to issued leader group')
                    os.kill(leader.pid, 9)
                    paused = False
                    started = time.monotonic()
                    try:
                        dispatcher.finish(leader, 5, heartbeat=lease)
                    except host.Uncertain:
                        paused = True
                    elapsed = time.monotonic() - started
                    require(paused and elapsed < 3 and (not dispatcher.children) and (foreign.poll() is None), 'leader loss pauses immediately and cleans owned group without foreign kill')
                limit = time.monotonic() + 10
                while probe.scalar('SELECT count(*) FROM pg_stat_activity WHERE pid=' + str(backend)) != '0':
                    require(time.monotonic() < limit, 'descendant backend gone after barrier release')
                    time.sleep(0.1)
                require(probe.state(job) == 'leased' and probe.attempts(job) == 1 and (probe.scalar('SELECT count(*) FROM zee_job_intents') == '0'), 'no false intent/fail write after lost owned leader')
                passed('killed_owned_leader_native_descendant_group_cleanup_no_foreign_adoption', {'paused': True, 'pause_cleanup_seconds': round(elapsed, 3), 'native_same_owned_group': True, 'backend_gone': True, 'owned_children_remaining': 0, 'foreign_process_survived': True, 'attempts': 1, 'intents': 0, 'renewals': int(probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id=' + job))})
            finally:
                cleanup_all(dispatcher.cleanup, lambda: close_direct(foreign), lambda: probe.sql('DROP TRIGGER zee_stage_pause ON zee_job_intents; DROP FUNCTION zee_stage_pause()'))
            reset()
            probe.enqueue()
            db_file = credentials / 'db'
            db_file.unlink()
            os.mkfifo(db_file, 384)
            child = launch(p, 'fifo-reject', 1)
            try:
                out, err = child.communicate(timeout=10)
                require(child.returncode == 2 and (not err), 'FIFO credential rejected without waiting for writer')
                require(probe.attempts('1') == 0, 'no worker write after FIFO refusal')
            finally:
                db_file.unlink()
                db_file.write_text(f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require')
                db_file.chmod(256)
            passed('fifo_credential_rejected_without_blocking_or_sql_effect', {'exit': 2, 'attempts': 0})
            canary = 'PRIVATE_K258_REVIEW_CANARY'
            for argv in [['--ticks', canary], ['--unknown-' + canary]]:
                result = subprocess.run(['python3', '-I', '-B', str(DISPATCHER), *argv], capture_output=True, timeout=5)
                require(result.returncode == 2 and result.stderr == b'' and (canary.encode() not in result.stdout) and (result.stdout.count(b'\n') == 1), 'argparse invalidargument masked fixed protocol')
                value = json.loads(result.stdout)
                require(value['event'] == 'rejected' and value['schema'] == 'zee-sql-intent-dispatcher-1' and (set(value) == {'schema', 'event', 'elapsed_ms'}), 'argument rejection only fixed JSON metadata')
            passed('invalid_argument_canary_masked_no_argv_usage_stderr_leak', {'variants': 2, 'exit': 2, 'stderr_bytes': 0, 'canary_in_output': False, 'fixed_json_only': True})
            r['body_completed'] = True
        finally:
            cleanup_all(close_components, close_children, close_probe, close_pg, close_work, terminal=True)
        active()
        need(digest(ARGS.dil.resolve(strict=True)) == r['binary_sha256'] and all((digest(ROOT / name) == value for name, value in r['fixture_hashes'].items())), 'binary/consumed source bytes unchanged')
        need(safe_command('source-head', ['git', '-C', ROOT, 'rev-parse', 'HEAD']).decode().strip() == r['framework_head'], 'source unchanged')
        H['clean_exact'](ROOT, r['framework_head'])
        r['source_end_unchanged'] = True
    except BaseException as error:
        primary = error
        r['failure_type'] = type(error).__name__
        r['status'] = 'failed'
        try:
            cleanup_all(close_components, close_children, close_probe, close_pg, close_work, terminal=True)
        except BaseException:
            pass
    finally:
        publish(primary)
if __name__ == '__main__':
    main()
