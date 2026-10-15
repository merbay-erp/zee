from pathlib import Path
import argparse, datetime, hashlib, importlib.util, json, os, runpy, signal, shutil, subprocess, tempfile, time
parser=argparse.ArgumentParser(description='Actual owned-PG finite SQL dispatcher acceptance')
parser.add_argument('--framework-root',type=Path,required=True);parser.add_argument('--dil',type=Path,required=True);parser.add_argument('--postgres-bin-dir',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
ARGS=parser.parse_args();ROOT=ARGS.framework_root.resolve(strict=True);OUTPUT=ARGS.output.resolve()
if OUTPUT.exists():parser.error('--output must be a fresh directory')
OUTPUT.mkdir(parents=True,mode=0o700)
FIXTURE=ROOT/'dogfood/zee-cekirdek/is-kuyrugu-dispatcher';HELPERS=ROOT/'scripts/tests/fixtures/job-dispatcher';DISPATCHER=ROOT/'scripts/job-dispatcher.py'
MOD=runpy.run_path(str(ROOT/'scripts/is-kuyrugu-ci.py'));TLS=MOD['TLS'];require=MOD['require']
spec=importlib.util.spec_from_file_location('dispatcher',DISPATCHER);host=importlib.util.module_from_spec(spec);spec.loader.exec_module(host)
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
 r={'schema':'zee-sql-intent-dispatcher-native-1','started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_acceptance':'actual-given-binary-and-candidate-fixture','cases':[],'boundaries':['SQL durable intent only; not catalog SMTP/PDF service','finite100tick maximum prototype; K246 ownedgroups/nonblockinglog reused','no calendar cron or live product adoption; K245 credential reader reused inline for fixed stdout protocol, not CLIrun_child','no clean final-source acceptance','COMMIT proxy tests invoke host component only with scoped owned plaintext fixture; CLI default stays TLSrequire']}
 with tempfile.TemporaryDirectory(prefix='zee-dispatcher-owned-pg-') as tmp:
  d=Path(tmp);binary=d/'dil';shutil.copy2(ARGS.dil.resolve(strict=True),binary);binary.chmod(0o500)
  r['binary_sha256']=digest(binary);r['build_identity']=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],text=True));r['fixture_hashes']={str(f.relative_to(ROOT)):digest(f) for f in [*FIXTURE.rglob('*.dil'),*FIXTURE.rglob('*.sql'),DISPATCHER,HELPERS/'backpressure-driver.py',HELPERS/'leader-driver.py',ROOT/'scripts/worker-supervisor.py',ROOT/'scripts/credential-baslat.py',ROOT/'scripts/is-kuyrugu-ci.py',ROOT/'scripts/postgresql-tls-ci.py',ROOT/'scripts/postgresql-tls11-control.py',Path(__file__).resolve()]}
  ca,_,cert,key=TLS['certificates'](d);(d/'pg_hba.conf').write_text('local all all trust\nhostssl all all 0.0.0.0/0 trust\nhostnossl all all 127.0.0.1/32 trust\nhostnossl all all 0.0.0.0/0 reject\n')
  cluster=TLS['LocalCluster'](d,cert,key,str(ARGS.postgres_bin_dir.resolve(strict=True)));probe=None;children=[]
  def passed(name,obs):r['cases'].append({'name':name,'passed':True,'ignored':False,'observations':obs});print(name+' ... passed',flush=True)
  try:
   cluster.start();r['server']=cluster.version
   probe=MOD['Probe'](d,cluster,binary,ca,(ROOT/'dogfood/zee-cekirdek/is-kuyrugu/kaynak/kuyruk.dil').read_text())
   for name in ['0001_kuyruk.sql','0002_kira_sahibi.sql','0003_sabit_niyet.sql']:probe.sql((FIXTURE/'gocler'/name).read_text())
   def project(port):
    p=d/('project-'+str(port));p.mkdir(exist_ok=True);(p/'gocler').mkdir(exist_ok=True)
    for name in host.FILES:
     f=p/name;value=(FIXTURE/name).read_bytes()
     if name=='proje.dil':value=value.replace(b'localhost:18093',('localhost:'+str(port)).encode())
     f.write_bytes(value);f.chmod(0o600)
    return p
   def reset():probe.sql('TRUNCATE zee_job_intents,zee_job_leases,zee_jobs RESTART IDENTITY')
   credentials=d/'credentials';credentials.mkdir(mode=0o700)
   for name,value in [('db',f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require'),('sentinel','PRIVATE-K258-CREDENTIAL-SENTINEL')]:
    target=credentials/name;target.write_text(value);target.chmod(0o400)
   def launch(p,owner,ticks):
    argv=['python3','-I','-B',str(DISPATCHER),'--dil',str(binary),'--project',str(p),'--expected-source',r['build_identity']['source_sha'],'--expected-binary-hash',r['binary_sha256'],'--expected-project-hash',host.project_digest(host.project_snapshot(p)),'--owned-pg-port',str(cluster.port),'--owned-ca',str(ca),'--credential-root',str(credentials.resolve()),'--credential-env','ZEE_JOB_CI_URL=db','--credential-env','APP_SENTINEL=sentinel','--owner',owner,'--ticks',str(ticks)]
    child=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=True);children.append(child);return child
   p=project(cluster.port);reset()
   for n in range(1,5):probe.enqueue('job-'+str(n),payload="opaque '); DROP TABLE zee_jobs; --")
   actors=[launch(p,'actor-a',12),launch(p,'actor-b',12)]
   records=[]
   for actor in actors:
    out,err=actor.communicate(timeout=30);r.setdefault('actor_failure_metadata',[]).append({'exit':actor.returncode,'out':out,'err':err});require(actor.returncode==0 and not err,'actual host dispatcher completion');records.extend(json.loads(line) for line in out.splitlines())
   count=int(probe.scalar('SELECT count(*) FROM zee_job_intents'));done=int(probe.scalar("SELECT count(*) FROM zee_jobs WHERE state='done'"));attempts=int(probe.scalar('SELECT sum(attempts) FROM zee_jobs'))
   require((count,done,attempts)==(4,4,4),'two actual dispatchers four fixed SQL intents once');require(int(probe.scalar('SELECT count(*) FROM zee_job_intents i JOIN zee_jobs j ON i.job_id=j.id AND i.payload=j.payload AND i.approval_revision=j.approval_revision'))==4,'payload data preserved')
   passed('two_owned_dispatchers_fixed_sql_intents_once',{'actors':2,'intents':count,'done':done,'attempts':attempts,'finished_events':sum(e['event']=='finished' for e in records),'payload_data_preserved':True})
   reset();job=probe.enqueue();probe.sql("CREATE FUNCTION zee_stage_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(254002); RETURN NEW; END $$; CREATE TRIGGER zee_stage_pause BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_stage_pause();")
   with MOD['PAROLA']['SqlBlocker'](probe,'SELECT pg_advisory_xact_lock(254002)'):
    child=launch(p,'long-handler',1);limit=time.monotonic()+10
    while probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'")!='1':
     require(time.monotonic()<limit,'actual fixed SQL intent host reached barrier');time.sleep(.1)
    held=time.monotonic()
    while time.monotonic()-held<35:
     require(child.poll() is None,'host stays alive while actual handler runs');time.sleep(.1)
    renewals=int(probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job));original_expired=probe.scalar('SELECT lease_expires<clock_timestamp() FROM zee_jobs WHERE id='+job)=='true'
    require(renewals>=6 and original_expired,'actual host heartbeats beyond unchanged original30s')
    require(probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'")=='1','handler still blocked at heartbeat observation')
   out,err=child.communicate(timeout=20);events=[json.loads(line) for line in out.splitlines()];require(child.returncode==0 and not err,'long host completes after owned barrier release')
   require(probe.state(job)=='done' and probe.attempts(job)==1 and int(probe.scalar('SELECT count(*) FROM zee_job_intents'))==1,'actual long host durable intent done once')
   require(events[-1]['heartbeat_acks']==renewals,'hostack equals independent DB counter')
   passed('actual_dispatcher_35second_fixed_handler_with_independent_heartbeats',{'held_seconds':round(time.monotonic()-held,3),'renewals':renewals,'heartbeat_acks':events[-1]['heartbeat_acks'],'original30s_expired':True,'attempts':1,'intents':1,'done':True})
   reset();job=probe.enqueue()
   with MOD['PAROLA']['SqlBlocker'](probe,'SELECT pg_advisory_xact_lock(254002)'):
    child=launch(p,'cancelled-handler',1);limit=time.monotonic()+10
    while probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'")!='1':
     require(time.monotonic()<limit,'cancelled actual host reached barrier');time.sleep(.1)
    pid=int(probe.scalar("SELECT pid FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'"));child.terminate();out,err=child.communicate(timeout=10)
    require(child.returncode==3 and not err and any(json.loads(line)['event']=='paused_uncertain' for line in out.splitlines()),'TERM actual host pauses no retry write')
   limit=time.monotonic()+10
   while probe.scalar('SELECT count(*) FROM pg_stat_activity WHERE pid='+str(pid))!='0':
    require(time.monotonic()<limit,'actual old backend gone after owned barrier release');time.sleep(.1)
   require(probe.attempts(job)==1 and probe.state(job)=='leased' and int(probe.scalar('SELECT count(*) FROM zee_job_intents'))==0,'cancelled handler did not persist intent or false retry outcome')
   passed('actual_term_pauses_admission_no_retry_or_uncertain_effect',{'exit':3,'backend_gone':True,'attempts':1,'state_leased':True,'intents':0})
   probe.sql('DROP TRIGGER zee_stage_pause ON zee_job_intents; DROP FUNCTION zee_stage_pause()')
   reset();job=probe.enqueue();probe.sql("CREATE FUNCTION zee_stage_refuse() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'synthetic refusal' USING ERRCODE='23514'; END $$; CREATE TRIGGER zee_stage_refuse BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_stage_refuse();")
   child=launch(p,'refusal-actor',40);out,err=child.communicate(timeout=30);events=[json.loads(line) for line in out.splitlines()]
   require(child.returncode==0 and not err and probe.attempts(job)==3 and probe.state(job)=='dead_letter' and int(probe.scalar('SELECT count(*) FROM zee_job_intents'))==0,'actual three knownSQL failures reach deadletter')
   require(sum(e['event']=='known_sql_failure' for e in events)==3,'three explicit acknowledgements no fourth')
   passed('known_sql_refusals_three_attempts_deadletter',{'attempts':3,'known_failures':3,'intents':0,'dead_letter':True});probe.sql('DROP TRIGGER zee_stage_refuse ON zee_job_intents; DROP FUNCTION zee_stage_refuse()')
   for mode in ['before','after']:
    reset();probe.enqueue();app='dispatcher-fault-'+mode;proxy=MOD['PROXY']['CommitProxy']('127.0.0.1',cluster.port,mode,application=app,deadline_seconds=30).start()
    pp=project(proxy.port);env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','LC_ALL':'C.UTF-8','TZ':'UTC','ZEE_JOB_CI_URL':f'host=localhost port={proxy.port} user=postgres dbname=postgres sslmode=disable application_name={app}'}
    dispatcher=host.Dispatcher(binary,pp,env,'fault-actor');paused=False
    try:
     try:dispatcher.tick()
     except host.Uncertain:paused=True
     wire=proxy.finish();probe.wait_gone(app)
     require(paused and wire['commits_seen']==1 and int(probe.scalar('SELECT count(*) FROM zee_job_intents'))==0 and probe.attempts('1')==0,'actual COMMIT uncertainty pauses before further writes/admission')
     passed('actual_commit_'+mode+'_pauses_host_without_replay',{'paused':True,'commits_seen':1,'commit_forwarded':wire['commit_forwarded'],'commit_complete':wire['commit_complete'],'attempts':0,'intents':0,'owned_children_remaining':len(dispatcher.children)})
    finally:dispatcher.cleanup();proxy.close()
   reset();probe.enqueue();os.environ['ZEE_JOB_CI_URL']='AMBIENT-K258-FORBIDDEN';child=launch(p,'ambient-reject',1)
   try:
    out,err=child.communicate(timeout=10);require(child.returncode==2 and not err and 'AMBIENT-K258-FORBIDDEN' not in out and 'PRIVATE-K258-CREDENTIAL-SENTINEL' not in out,'ambient credential is rejected masked')
    require(probe.attempts('1')==0 and int(probe.scalar('SELECT count(*) FROM zee_job_intents'))==0,'no operation after rejected credential')
   finally:del os.environ['ZEE_JOB_CI_URL']
   passed('ambient_credential_rejected_no_fallback_or_secret_log',{'exit':2,'attempts':0,'intents':0,'secret_in_log':False})
   reset();job=probe.enqueue();cfg=d/'component-env.json';cfg.write_text(json.dumps({'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','LC_ALL':'C.UTF-8','TZ':'UTC','ZEE_JOB_CI_URL':f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require','ZEE_JOB_CI_URL_TLS_CA_PEM':ca.read_text()}));cfg.chmod(0o600)
   receipt=d/'backpressure.json';actor=subprocess.Popen(['python3','-I','-B',str(HELPERS/'backpressure-driver.py'),str(binary),str(p),str(cfg),str(receipt)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True);children.append(actor)
   # Deliberately do not read stdout before wait: actual pipe reaches capacity.
   actor.wait(timeout=15);out,err=actor.communicate(timeout=2);obs=json.loads(receipt.read_text())
   require(actor.returncode==0 and not err and obs['dropped_before_sql']>0 and obs['dropped_after_sql']>=obs['dropped_before_sql'] and obs['applied']==1 and obs['owned_children_remaining']==0,'actual log backpressure bounded while SQL tick completes')
   require(probe.state(job)=='done' and probe.attempts(job)==1 and probe.scalar('SELECT count(*) FROM zee_job_intents')=='1','durable intent while unread log pipe full')
   lines=out.splitlines();require(all(json.loads(line)['schema']=='zee-sql-intent-dispatcher-1' for line in lines),'atomic complete log records under saturation')
   obs.update({'log_records_received':len(lines),'intents':1,'attempts':1,'done':True,'stdout_unread_until_child_exit':True});passed('unread_full_log_pipe_drops_records_actual_sql_still_completes',obs)
   reset();job=probe.enqueue();env=json.loads(cfg.read_text());dispatcher=host.Dispatcher(binary,p,env,'owned-leader-loss');token='lease-owned-leader-loss';claim=dispatcher.query('claim',token=token);jid,generation=claim.split(':');lease=(jid,token,generation);require(dispatcher.query('begin',lease)=='1','leader loss handler started')
   probe.sql("CREATE FUNCTION zee_stage_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(258002); RETURN NEW; END $$; CREATE TRIGGER zee_stage_pause BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_stage_pause();")
   foreign=subprocess.Popen(['python3','-I','-c','import time; time.sleep(30)'],start_new_session=True);children.append(foreign);pidfile=d/'native-descendant.pid'
   try:
    with MOD['PAROLA']['SqlBlocker'](probe,'SELECT pg_advisory_xact_lock(258002)'):
     leader=dispatcher.group_owner.spawn(['python3','-I','-B',str(HELPERS/'leader-driver.py'),str(pidfile),str(binary),'çalıştır',str(p),'apply',jid,token,generation,'owned-leader-loss'],capture=True,environment=env);os.set_blocking(leader.stdout.fileno(),False);limit=time.monotonic()+10
     while probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'")!='1':
      require(time.monotonic()<limit,'actual native descendant reached SQL barrier');time.sleep(.1)
     backend=int(probe.scalar("SELECT pid FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'"));native_pid=int(pidfile.read_text());require(os.getpgid(native_pid)==leader.pid,'native belongs to issued leader group')
     os.kill(leader.pid,9);paused=False;started=time.monotonic()
     try:dispatcher.finish(leader,5,heartbeat=lease)
     except host.Uncertain:paused=True
     elapsed=time.monotonic()-started;require(paused and elapsed<3 and not dispatcher.children and foreign.poll() is None,'leader loss pauses immediately and cleans owned group without foreign kill')
    limit=time.monotonic()+10
    while probe.scalar('SELECT count(*) FROM pg_stat_activity WHERE pid='+str(backend))!='0':
     require(time.monotonic()<limit,'descendant backend gone after barrier release');time.sleep(.1)
    require(probe.state(job)=='leased' and probe.attempts(job)==1 and probe.scalar('SELECT count(*) FROM zee_job_intents')=='0','no false intent/fail write after lost owned leader')
    passed('killed_owned_leader_native_descendant_group_cleanup_no_foreign_adoption',{'paused':True,'pause_cleanup_seconds':round(elapsed,3),'native_same_owned_group':True,'backend_gone':True,'owned_children_remaining':0,'foreign_process_survived':True,'attempts':1,'intents':0,'renewals':int(probe.scalar('SELECT renewals FROM zee_job_leases WHERE job_id='+job))})
   finally:
    dispatcher.cleanup();foreign.terminate();foreign.wait(timeout=3);probe.sql('DROP TRIGGER zee_stage_pause ON zee_job_intents; DROP FUNCTION zee_stage_pause()')
   reset();probe.enqueue()
   # Real FIFO rejection uses no writer; fails before a SQL child can execute.
   db_file=credentials/'db';db_file.unlink();os.mkfifo(db_file,0o600);child=launch(p,'fifo-reject',1)
   try:
    out,err=child.communicate(timeout=10);require(child.returncode==2 and not err,'FIFO credential rejected without waiting for writer');require(probe.attempts('1')==0,'no worker write after FIFO refusal')
   finally:db_file.unlink();db_file.write_text(f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require');db_file.chmod(0o400)
   passed('fifo_credential_rejected_without_blocking_or_sql_effect',{'exit':2,'attempts':0})
   canary='PRIVATE_K258_REVIEW_CANARY'
   for argv in [['--ticks',canary],['--unknown-'+canary]]:
    result=subprocess.run(['python3','-I','-B',str(DISPATCHER),*argv],capture_output=True,timeout=5)
    require(result.returncode==2 and result.stderr==b'' and canary.encode() not in result.stdout and result.stdout.count(b'\n')==1,'argparse invalidargument masked fixed protocol')
    value=json.loads(result.stdout);require(value['event']=='rejected' and value['schema']=='zee-sql-intent-dispatcher-1' and set(value)=={'schema','event','elapsed_ms'},'argument rejection only fixed JSON metadata')
   passed('invalid_argument_canary_masked_no_argv_usage_stderr_leak',{'variants':2,'exit':2,'stderr_bytes':0,'canary_in_output':False,'fixed_json_only':True})
   r['status']='passed'
  finally:
   for c in children:
    if c.poll() is None:
     # Own direct Popen handle only. Give host TERM path time to clean its groups.
     c.terminate()
     try:c.communicate(timeout=3)
     except subprocess.TimeoutExpired:
      os.killpg(c.pid,signal.SIGKILL);c.communicate(timeout=3)
   if probe:probe.close()
   cluster.close();r['owned_cluster_closed']=True;r['finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();(OUTPUT/'proof.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
if __name__=='__main__':main()
