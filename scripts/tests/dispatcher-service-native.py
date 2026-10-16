#!/usr/bin/env python3
"""K282 dedicated: actual rendered Linux units, native SQL and application-owned adapter."""
import argparse,hashlib,importlib.util,json,os,runpy,shutil,sys,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
S=runpy.run_path(str(HERE/'service-native-support.py'));D=S['D'];Campaign=S['Campaign']

def main():
    p=argparse.ArgumentParser()
    for n in ('repo','dil','postgres-bin-dir','out'):p.add_argument('--'+n,type=Path,required=True)
    for n in ('sha','binary-sha256'):p.add_argument('--'+n,required=True)
    a=p.parse_args()
    with Campaign(a,'K282')as c:
        source=c.repo/'scripts';host=D.load('native_dispatch',source/'job-dispatcher.py');mod=runpy.run_path(str(source/'is-kuyrugu-ci.py'));tls=mod['TLS']
        temporary=c.temporary('z282-');tls['certificates'].__globals__['run']=c.borrowed_run;ca,_,cert,key=tls['certificates'](temporary)
        (temporary/'pg_hba.conf').write_text('local all all trust\nhostssl all all 127.0.0.1/32 trust\nhostnossl all all 0.0.0.0/0 reject\n')
        cluster=S['OwnedLocalCluster'](c,temporary,cert,key,a.postgres_bin_dir.resolve(strict=True));c.clusters.append(cluster);cluster.start();c.need(cluster.version=='postgres (PostgreSQL) 16.15');c.proof['postgres_version']=cluster.version
        probe=mod['Probe'](temporary,cluster,c.binary,ca,c.fixture('dogfood/zee-cekirdek/is-kuyrugu/kaynak/kuyruk.dil').decode());c.resources.append(probe)
        fixture=c.repo/'dogfood/zee-cekirdek/is-kuyrugu-dispatcher'
        probe.sql=lambda statement:c.command('SQL-observer',probe.psql_command()+['--command',statement],20).decode().strip()
        c.need(tuple(host.FILES)==S['QUEUE_FILES'])
        operation_count=0
        def owned_operation(operation,*args):
            nonlocal operation_count
            c.need(operation in mod['CALLS']);operation_count+=1
            target=c.out/('queue-operation-'+str(operation_count));target.mkdir(mode=0o700);(target/'gocler').mkdir(mode=0o700)
            (target/'proje.dil').write_text('proje "queue-ci" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "veritabanı" listesi olsun\nveritabanı_hedefi "postgresql://localhost:'+str(cluster.port)+'/postgres" olsun\nveritabanı_bağlantı_değişkeni "ZEE_JOB_CI_URL" olsun\nveritabanı_göçleri "gocler" olsun\n')
            values=[json.dumps(str(v),ensure_ascii=False)for v in args];call=((" ve ".join(values)+" ile ")if values else "")+mod['CALLS'][operation]
            (target/'ana.dil').write_text(probe.helper+mod['COUNTER']+'\nsonuc '+call+' olsun\nsonuc başarılıysa\n    sonucun değeri yaz\ndeğilse\n    "reddedildi" yaz\n')
            env=D.T.D.controlled_env();env.update(ZEE_JOB_CI_URL=f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require',ZEE_JOB_CI_URL_TLS_CA_PEM=ca.read_text())
            return c.command('native-queue-operation',[c.binary,'çalıştır',target],25,env).decode().strip()
        probe.op=owned_operation
        for name in ('0001_kuyruk.sql','0002_kira_sahibi.sql','0003_sabit_niyet.sql'):probe.sql(c.fixture('dogfood/zee-cekirdek/is-kuyrugu-dispatcher/gocler/'+name).decode())
        project=c.out/'project';project.mkdir(mode=0o700);(project/'gocler').mkdir(mode=0o700)
        for name in host.FILES:
            raw=c.fixture('dogfood/zee-cekirdek/is-kuyrugu-dispatcher/'+name)
            if name=='proje.dil':raw=raw.replace(b'localhost:18093',('localhost:'+str(cluster.port)).encode())
            (project/name).write_bytes(raw);(project/name).chmod(0o600)
        public=c.out/'ca.pem';public.write_bytes(ca.read_bytes());public.chmod(0o600);credentials=c.out/'credentials';credentials.mkdir(mode=0o700);secret=credentials/'db';secret.write_text(f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require');secret.chmod(0o400)
        timer=c.jsonfile(c.out/'timer.json',{'schema':'zee-sql-timer-config-1','repo':str(c.repo),'source_sha':a.sha,'dil':str(c.binary),'binary_sha256':a.binary_sha256,'project':str(project),'project_sha256':host.project_digest(host.project_snapshot(project)),'pg_port':cluster.port,'ca':str(public),'credential_root':str(credentials),'credential_name':'db','owner':'owned-service'})
        def config(label):
            return c.jsonfile(c.out/(label+'.json'),{'schema':'zee-dispatch-service-1','kind':'sql-intent','repo':str(c.repo),'source_sha':a.sha,'interval_seconds':1,'state':str(c.out/(label+'-state')),'timer_config':str(timer),'timer_config_sha256':D.snapshot(timer)})
        def rendered(config,label,calendar=None):
            out=c.out/(label+'-units');cmd=[sys.executable,'-I','-B',source/'dispatcher-service.py','--config',config,'--render',out]
            if calendar:cmd+=['--calendar',calendar]
            c.command('render-'+label,cmd,45);names=['zee-dispatch.service']+(['zee-dispatch.timer']if calendar else [])
            c.command('verify-'+label,['systemd-analyze','--user','verify',*[out/n for n in names]])
            if calendar:c.command('calendar-check',['systemd-analyze','calendar',calendar])
            c.link(out,names);return names[-1]
        jobs=[probe.enqueue('anonymous-one'),probe.enqueue('anonymous-two')];cfg=config('daemon');rendered(cfg,'daemon');c.start('zee-dispatch.service')
        c.wait(lambda:probe.scalar('SELECT count(*) FROM zee_job_intents')=='2',150)
        c.stop('zee-dispatch.service');c.need(all(probe.state(j)=='done' and probe.attempts(j)==1 for j in jobs));c.record('continuous-native-SQL-two-ticks-no-overlap',jobs=2,intents=2,attempts_each=1);c.unlink()
        probe.sql('TRUNCATE zee_job_intents,zee_job_leases,zee_jobs RESTART IDENTITY');job=probe.enqueue('calendar-job');cfg=config('calendar');rendered(cfg,'calendar','*-*-* *:*:00');c.start('zee-dispatch.timer');c.wait(lambda:probe.state(job)=='done',120);c.stop('zee-dispatch.timer');c.stop('zee-dispatch.service');c.need(probe.attempts(job)==1);c.record('actual-canonical-OnCalendar-unit-native-SQL',persistent_backfill=False,attempts=1);c.unlink()
        probe.sql('TRUNCATE zee_job_intents,zee_job_leases,zee_jobs RESTART IDENTITY');job=probe.enqueue('unknown-job')
        probe.sql('CREATE FUNCTION zee_service_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(282001); RETURN NEW; END $$; CREATE TRIGGER zee_service_pause BEFORE INSERT ON zee_job_intents FOR EACH ROW EXECUTE FUNCTION zee_service_pause();')
        cfg=config('unknown');rendered(cfg,'unknown')
        with S['OwnedSqlBlocker'](c,probe.psql_command(True),'SELECT pg_advisory_xact_lock(282001)'):
            c.start('zee-dispatch.service');c.wait(lambda:probe.scalar("SELECT count(*) FROM pg_stat_activity WHERE query LIKE 'INSERT INTO zee_job_intents%' AND wait_event_type='Lock'")=='1',60)
            c.stop('zee-dispatch.service')
        state=c.out/'unknown-state';c.need((state/'inflight.json').is_file());before=probe.attempts(job);c.start('zee-dispatch.service');c.wait(lambda:c.system('failed-paused','show','zee-dispatch.service','--property=ActiveState','--value').strip()==b'failed')
        c.need(probe.attempts(job)==before==1 and probe.scalar('SELECT count(*) FROM zee_job_intents')=='0');c.record('actual-TERM-unknown-explicit-restart-no-replay',attempts=1,intents=0,latch_retained=True);c.proof['retained_latch_sha256']=D.snapshot(state/'inflight.json');c.unlink()
        # Application-owned adapter invokes actual native interpreter. It is not a live provider.
        app=c.out/'application';app.mkdir(mode=0o700);data=app/'input.dil';data.write_text(json.dumps(json.dumps({'schema':'zee-app-dispatch-result-1','outcome':'completed','completed':1},separators=(',',':')))+' yaz\n');data.chmod(0o600)
        exe=app/'adapter';exe.write_text('#!'+str(Path(sys.executable).resolve())+'\nimport subprocess,sys\nr=subprocess.run([sys.argv[1],"çalıştır",sys.argv[-1]],capture_output=True,timeout=5)\nif r.returncode or r.stderr:raise SystemExit(3)\nsys.stdout.buffer.write(r.stdout)\n');exe.chmod(0o700)
        appcfg=c.jsonfile(c.out/'application.json',{'schema':'zee-dispatch-service-1','kind':'application-adapter','repo':str(c.repo),'source_sha':a.sha,'interval_seconds':1,'state':str(c.out/'application-state'),'executable':str(exe),'executable_sha256':D.snapshot(exe),'cwd':str(app),'data':str(data),'data_sha256':D.snapshot(data),'argv':[str(c.binary)],'timeout_seconds':15,'contract':'zee-app-dispatch-result-1'})
        c.command('application-adapter',[sys.executable,'-I','-B',source/'dispatcher-service.py','--config',appcfg,'--once'],60)
        c.need(not (c.out/'application-state/inflight.json').exists());c.record('actual-native-application-owned-adapter-ack',provider_delivery_claim=False)
        c.finish()
if __name__=='__main__':
    try:main()
    except (Exception,KeyboardInterrupt):sys.stderr.write('owned-dispatch-service-campaign-failed\n');raise SystemExit(2)
