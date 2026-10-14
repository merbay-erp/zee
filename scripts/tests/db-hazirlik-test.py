#!/usr/bin/env python3
"""K-251 owned PostgreSQL + two real native workers; no existing DB URL."""
import argparse
from datetime import datetime, timezone
import hashlib,json,os
from pathlib import Path
import runpy,shutil,signal,socket,subprocess,tempfile,time


def require(ok, message):
    if not ok: raise ValueError(message)

def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(65536),b''):h.update(b)
    return h.hexdigest()

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--dil',required=True,type=Path)
    p.add_argument('--framework-root',required=True,type=Path)
    p.add_argument('--postgres-bin-dir',required=True,type=Path)
    p.add_argument('--output',required=True,type=Path)
    a=p.parse_args()
    stage=Path(__file__).resolve().parents[2]
    fixture=stage/'compiler/tests/fixtures/db-readiness'
    framework=a.framework_root.resolve(strict=True)
    binary=a.dil.resolve(strict=True)
    output=a.output.resolve()
    require(not output.exists(),'Never overwrite earlier raw campaign')
    output.mkdir(parents=True,mode=0o700)
    proof={'schema':'zee-k251-native-candidate-1','status':'running','cases':[],
           'source_identity':'external uncommitted candidate; actual existing compiler identity recorded',
           'started_utc':datetime.now(timezone.utc).isoformat(),
           'boundaries':['once helper must be reinvoked; no TTL or SIGKILL invalidation','receipt alone does not prove actual DDL','no full P13/P14/drain/zero downtime','owned local anonymous fixtures, no live product or platform acceptance']}
    source_paths=[stage/'scripts/db-hazirlik.py',Path(__file__).resolve(),stage/'scripts/worker-supervisor.py',*sorted(fixture.rglob('*.dil')),*sorted(fixture.rglob('*.sql'))]
    source_hashes=[{'path':str(f.relative_to(stage)),'sha256':sha(f)} for f in source_paths]
    before_binary=sha(binary)
    identity=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=10))
    require(identity['dirty'] is False,'Actual clean compiler identity required')
    proof['compiler_identity']=identity;proof['compiler_binary_sha256']=before_binary;proof['source_files']=source_hashes
    TLS=runpy.run_path(str(framework/'scripts/postgresql-tls-ci.py'))
    workers=runpy.run_path(str(framework/'scripts/tests/worker-supervisor-test.py'))
    workers['Fixture'].__init__.__globals__['NATIVE']=binary
    REVISION=workers['REVISION']
    clusters=[];supervisor=None;owner=None
    def command(args,env=None):
        return subprocess.run(args,capture_output=True,env=env,timeout=30,check=True).stdout
    try:
        # PostgreSQL's Unix socket path must fit sockaddr_un; macOS default TMPDIR is long.
        with tempfile.TemporaryDirectory(prefix='zee-k251-', dir='/private/tmp') as temporary:
            owned=Path(temporary).resolve()
            def cluster(number):
                root=owned/('pg'+str(number));root.mkdir(mode=0o700)
                _, _, cert, key = TLS['certificates'](root)
                (root/'pg_hba.conf').write_text('local all all trust\nhost all all 127.0.0.1/32 trust\n')
                c=TLS['LocalCluster'](root,cert,key,a.postgres_bin_dir)
                clusters.append(c)
                try:
                    c.start()
                except subprocess.SubprocessError as error:
                    for name in ['server.log']:
                        if (root/name).exists():
                            shutil.copyfile(root/name, output/('pg'+str(number)+'-'+name))
                    (output/('pg'+str(number)+'-start-stderr.log')).write_bytes(error.stderr or b'')
                    raise
                def sql(text,dbname='postgres'):
                    return command(c.psql+['--host','127.0.0.1','--port',str(c.psql_port),'--username','postgres','--dbname',dbname,'--no-psqlrc','--tuples-only','--no-align','--set','ON_ERROR_STOP=1','--command',text]).decode().strip()
                sql('CREATE ROLE zee_ready LOGIN; ALTER ROLE zee_ready SET default_transaction_read_only=on;')
                sql('CREATE DATABASE zee_readiness_fixture;')
                c.sql=lambda text:sql(text,'zee_readiness_fixture')
                c.root=root
                return c
            c=cluster(1)
            require(c.version=='postgres (PostgreSQL) 16.15','Expected owned PG16.15')
            proof['postgres_version']=c.version
            probe=owned/'probe';shutil.copytree(fixture/'probe',probe)
            def configure(c):
                text=(fixture/'probe/proje.dil').read_text().replace('127.0.0.1:5432','localhost:'+str(c.port))
                (probe/'proje.dil').write_text(text)
                env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',ZEE_READINESS_DATABASE_URL=f'host=localhost port={c.port} user=zee_ready dbname=zee_readiness_fixture sslmode=require',ZEE_READINESS_DATABASE_URL_TLS_CA_PEM=(c.root/'ca.crt').read_text())
                return env
            env=configure(c)
            contract=owned/'contract.json'
            def receipt():
                files=[{'path':str(f.relative_to(probe)),'sha256':sha(f)} for f in sorted(probe.rglob('*')) if f.is_file()]
                migrations=[{'version':int(f.name.split('_')[0]),'name':f.name,'sha256':sha(f)} for f in sorted((probe/'gocler').glob('*.sql'))]
                contract.write_text(json.dumps({'schema':'zee-db-readiness-contract-1','application_revision':REVISION,'files':files,'migrations':migrations},separators=(',',':')))
            owner=workers['Fixture'](native=True,ready=False)
            # Use this campaign's explicit liveness-only source, never app self-ready.
            owner.source.unlink()
            for source in (fixture/'worker').iterdir():
                shutil.copyfile(source, owner.project/source.name)
            owner.args[owner.args.index('--startup-timeout')+1] = '15'
            def invoke(label,ready,reason=None):
                receipt()
                argv=['python3','-I','-B',str(stage/'scripts/db-hazirlik.py'),'--dil',str(binary),'--project',str(probe),'--worker-project',str(owner.project),'--gate',str(owner.gate),'--expected-source',identity['source_sha'],'--application-revision',REVISION,'--contract',str(contract),'--contract-sha256',sha(contract),'--deadline','8']
                start=time.monotonic();r=subprocess.run(argv,env=env,capture_output=True,timeout=25)
                (output/(label+'-stdout.json')).write_bytes(r.stdout);(output/(label+'-stderr.log')).write_bytes(r.stderr)
                value=json.loads(r.stdout)
                require(not r.stderr and value['ready'] is ready and value['application_revision']==REVISION and (r.returncode==0 if ready else r.returncode!=0),'Fixed metadata/exit/ready mismatch: '+label)
                if reason:require(value['reason']==reason,'Expected helper rejection: '+label)
                gate=json.loads(owner.gate.read_text())
                require(set(gate)=={'schema','application_revision','ready'} and gate['schema']=='zee-worker-readiness-1' and gate['application_revision']==REVISION and gate['ready'] is ready,'Gate did not fail closed: '+label)
                require(b'ZEE_K251_CANARY' not in r.stdout+r.stderr,'Secret/error leaked')
                return {'name':label,'status':'passed','ready':ready,'reason':value['reason'],'exit':r.returncode,'elapsed_seconds':round(time.monotonic()-start,3)}
            def apply(c):
                admin=env.copy();admin['ZEE_READINESS_DATABASE_URL']=f'host=localhost port={c.port} user=postgres dbname=zee_readiness_fixture sslmode=require'
                result=subprocess.run([str(binary),'göçür',str(probe)],env=admin,capture_output=True,timeout=20)
                require(result.returncode==0,'Owned migration apply failed')
                c.sql('GRANT SELECT ON ALL TABLES IN SCHEMA public TO zee_ready;')
            def state_event(ready,after):
                return supervisor.until(lambda x:x.get('event')=='state' and x.get('ready') is ready and x.get('state') in {'ready','unready'},timeout=10,after=after)
            def health200():
                for port in owner.ports:
                    deadline = time.monotonic()+3
                    with socket.create_connection(('127.0.0.1',port),timeout=3) as s:
                        s.sendall(b'GET /hazir HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n')
                        reply = bytearray()
                        while b'\r\n\r\n' not in reply:
                            require(time.monotonic()<deadline and len(reply)<=16384,'Bounded health header')
                            s.settimeout(deadline-time.monotonic())
                            chunk=s.recv(1024)
                            require(chunk,'Health response EOF')
                            reply.extend(chunk)
                        reply=bytes(reply)
                        (output/('health-'+str(port)+'.log')).write_bytes(reply)
                        require(reply.startswith(b'HTTP/1.1 200 '),'Liveness200 changed')
            proof['cases'].append(invoke('missing-ledger',False,'migration'))
            apply(c)
            proof['cases'].append(invoke('applied-before-start-stale',True,'ready'))
            supervisor=owner.start()
            initial=state_event(False,0)
            require(initial['application_ready'] is False,'Old pre-start gate incorrectly adopted')
            after=len(supervisor.events);proof['cases'].append(invoke('applied-ready-two-workers',True,'ready'))
            state_event(True,after);health200()
            before_state=c.sql("SELECT revision FROM public.zee_app_state; SELECT surum||':'||ad||':'||sha256 FROM _zee_gocleri ORDER BY surum;")
            pending=probe/'gocler/0002_pending.sql';pending.write_text('SELECT 1;\n')
            after=len(supervisor.events);proof['cases'].append(invoke('pending-migration',False,'migration'));state_event(False,after);health200();pending.unlink()
            proof['cases'].append(invoke('restored-after-pending',True,'ready'));state_event(True,len(supervisor.events)-1)
            c.sql("UPDATE _zee_gocleri SET sha256=repeat('0',64);")
            proof['cases'].append(invoke('ledger-checksum-drift',False,'migration'))
            c.sql("UPDATE _zee_gocleri SET sha256='"+sha(probe/'gocler/0001_hazirlik.sql')+"';")
            c.sql('ALTER TABLE public.zee_app_state ALTER COLUMN revision DROP NOT NULL;')
            proof['cases'].append(invoke('actual-schema-drift',False,'schema'));health200()
            c.sql('ALTER TABLE public.zee_app_state ALTER COLUMN revision SET NOT NULL;')
            c.sql("UPDATE public.zee_app_state SET revision='wrong';")
            proof['cases'].append(invoke('revision-mismatch',False,'schema'))
            c.sql("UPDATE public.zee_app_state SET revision='readiness-v1';")
            proof['cases'].append(invoke('schema-restored',True,'ready'))
            require(c.sql("SELECT revision FROM public.zee_app_state; SELECT surum||':'||ad||':'||sha256 FROM _zee_gocleri ORDER BY surum;")==before_state,'Read probes changed committed state')
            dump=owned/'dump.sql'
            command([str(a.postgres_bin_dir/'pg_dump'),'--host','127.0.0.1','--port',str(c.port),'--username','postgres','--dbname','zee_readiness_fixture','--no-owner','--no-privileges','--file',str(dump)])
            after=len(supervisor.events);c.close();proof['cases'].append(invoke('pg-stopped',False));state_event(False,after);health200()
            fresh=cluster(2);env=configure(fresh)
            command(fresh.psql+['--host','127.0.0.1','--port',str(fresh.port),'--username','postgres','--dbname','zee_readiness_fixture','--no-psqlrc','--set','ON_ERROR_STOP=1','--file',str(dump)])
            fresh.sql('GRANT SELECT ON ALL TABLES IN SCHEMA public TO zee_ready;')
            after=len(supervisor.events);proof['cases'].append(invoke('fresh-logical-restore-ready',True,'ready'));state_event(True,after);health200()
            after=len(supervisor.events);supervisor.child.send_signal(signal.SIGHUP)
            event=supervisor.until(lambda x:x.get('event')=='state' and x.get('state')=='restarting',timeout=8,after=after)
            require(event['ready'] is False,'Restart did not invalidate aggregate readiness')
            proof['cases'].append(invoke('restart-fresh-app-gate',True,'ready'));state_event(True,after)
            fresh.sql('CREATE FUNCTION public.k251_error() RETURNS text LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION \'ZEE_K251_CANARY\'; END $$;')
            original=(probe/'kaynak/hazirlik.dil').read_text()
            (probe/'kaynak/hazirlik.dil').write_text('parametreler boş liste olsun\nsonuc "SELECT public.k251_error()::text" sorgusunu parametreler ile okumayı dene olsun\nprogramı 1 ile bitir\n')
            proof['cases'].append(invoke('sql-error-masked-unready',False,'schema'))
            (probe/'kaynak/hazirlik.dil').write_text(original)
            fresh.sql('DROP FUNCTION public.k251_error();')
            proof['supervisor_events'] = list(supervisor.events)
            (output/'supervisor-events.json').write_text(json.dumps(supervisor.events,indent=2)+'\n')
            owner.close();owner=None;supervisor=None
            for c in reversed(clusters):c.close()
            require(all(not (c.data/'postmaster.pid').exists() for c in clusters),'Owned PostgreSQL still running')
            require(source_hashes==[{'path':str(f.relative_to(stage)),'sha256':sha(f)} for f in source_paths] and sha(binary)==before_binary,'Source/binary changed')
            proof.update(status='passed',case_count=len(proof['cases']),failed=0,ignored=0,owned_clusters_closed=True,owned_workers_closed=True,source_unchanged=True,binary_unchanged=True,readonly_probe_state_unchanged=True)
    except BaseException as error:
        proof['status']='failed';proof['failure_class']=type(error).__name__
        raise
    finally:
        if supervisor is not None:
            proof['supervisor_events'] = supervisor.events
            (output/'supervisor-events.json').write_text(json.dumps(supervisor.events,indent=2)+'\n')
            if supervisor.child.poll() is not None:
                (output/'supervisor-final-stderr.log').write_bytes(supervisor.child.stderr.read())
        if owner is not None:owner.close()
        for c in reversed(clusters):c.close()
        proof['finished_utc']=datetime.now(timezone.utc).isoformat()
        proof['raw_files']=[{'path':f.name,'sha256':sha(f)} for f in sorted(output.iterdir()) if f.is_file()]
        (output/'proof.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':proof['status'],'cases':proof['case_count'],'proof_sha256':sha(output/'proof.json')}))

if __name__=='__main__':main()
