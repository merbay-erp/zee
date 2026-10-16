#!/usr/bin/env python3
"""K283 dedicated owned Linux user service, real PG/native HTTP, no public ingress claim."""
import argparse,hashlib,importlib.util,json,os,runpy,shutil,signal,socket,sys,tempfile,threading,time
from pathlib import Path
from unittest import mock
HERE=Path(__file__).resolve().parent;SUP=runpy.run_path(str(HERE/'service-native-support.py'));D=SUP['D'];Campaign=SUP['Campaign']

def request(port,path):
    deadline=time.monotonic()+5
    with socket.create_connection(('127.0.0.1',port),timeout=5)as s:
        def remaining():
            left=deadline-time.monotonic()
            if left<=0:raise ValueError('request deadline')
            s.settimeout(left)
        remaining();s.sendall(('GET '+path+' HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n').encode())
        data=bytearray()
        while True:
            remaining();block=s.recv(4096)
            if not block:break
            data.extend(block)
            if len(data)>65536:raise ValueError('request output')
    header,body=bytes(data).split(b'\r\n\r\n',1);status=int(header.split(b' ',2)[1]);return status,body

def port():
    with socket.socket()as s:s.bind(('127.0.0.1',0));return s.getsockname()[1]

def main():
    p=argparse.ArgumentParser()
    for n in ('repo','dil','postgres-bin-dir','out'):p.add_argument('--'+n,type=Path,required=True)
    for n in ('sha','binary-sha256'):p.add_argument('--'+n,required=True)
    a=p.parse_args()
    with Campaign(a,'K283')as c:
        source=c.repo/'scripts';http=D.load('native_http_host',source/'http-service.py');tls=runpy.run_path(str(source/'postgresql-tls-ci.py'))
        temporary=c.temporary('z283-');tls['certificates'].__globals__['run']=c.borrowed_run;ca,_,cert,key=tls['certificates'](temporary)
        (temporary/'pg_hba.conf').write_text('local all all trust\nhostssl all all 127.0.0.1/32 trust\nhostnossl all all 0.0.0.0/0 reject\n')
        cluster=SUP['OwnedLocalCluster'](c,temporary,cert,key,a.postgres_bin_dir.resolve(strict=True));c.clusters.append(cluster);cluster.start();c.need(cluster.version=='postgres (PostgreSQL) 16.15');c.proof['postgres_version']=cluster.version
        def sql(text,database='postgres'):
            return c.command('sql-observer',cluster.psql+['--host','127.0.0.1','--port',str(cluster.port),'--username','postgres','--dbname',database,'--no-psqlrc','--tuples-only','--no-align','--set','ON_ERROR_STOP=1','--command',text]).decode().strip()
        sql('CREATE ROLE zee_ready LOGIN; ALTER ROLE zee_ready SET default_transaction_read_only=on;');sql('CREATE DATABASE zee_readiness_fixture;')
        fixture=c.repo/'compiler/tests/fixtures/db-readiness';project=c.out/'worker';probe=c.out/'probe';c.copy_fixture('compiler/tests/fixtures/db-readiness/worker',project,SUP['WORKER_FILES']);c.copy_fixture('compiler/tests/fixtures/db-readiness/probe',probe,SUP['PROBE_FILES'])
        (probe/'proje.dil').write_text((probe/'proje.dil').read_text().replace('127.0.0.1:5432','localhost:'+str(cluster.port)))
        # Anonymous actual long response used only for drain oracle; no customer source.
        with (project/'ana.dil').open('a')as f:f.write('\nGET "/limit" adresine istek geldiğinde\n    "ok" yanıtını gönder\n\nGET "/yavas" adresine istek geldiğinde\n    3 saniye bekle\n    "bitti" yanıtını gönder\n')
        credentials=c.out/'credentials';credentials.mkdir(mode=0o700);secret=credentials/'db';secret.write_text(f'host=localhost port={cluster.port} user=zee_ready dbname=zee_readiness_fixture sslmode=require');secret.chmod(0o400)
        public=c.out/'ca.pem';public.write_bytes(ca.read_bytes());public.chmod(0o600)
        env=D.T.D.controlled_env();env.update(ZEE_READINESS_DATABASE_URL=f'host=localhost port={cluster.port} user=postgres dbname=zee_readiness_fixture sslmode=require',ZEE_READINESS_DATABASE_URL_TLS_CA_PEM=ca.read_text())
        c.command('native-readiness-migration',[c.binary,'göçür',probe],30,env);env.clear();sql('GRANT SELECT ON ALL TABLES IN SCHEMA public TO zee_ready;','zee_readiness_fixture')
        inventory=lambda p:{n:D.snapshot(p/n)for n in (SUP['WORKER_FILES'] if p==project else SUP['PROBE_FILES'])}
        revision='2'*40;contract=c.jsonfile(c.out/'contract.json',{'schema':'zee-db-readiness-contract-1','application_revision':revision,'files':[{'path':n,'sha256':h}for n,h in inventory(probe).items()],'migrations':[{'version':int(q.name.split('_')[0]),'name':q.name,'sha256':D.snapshot(q)}for q in [probe/'gocler/0001_hazirlik.sql']]})
        ports=[port(),port()]
        while ports[0]==ports[1]:ports[1]=port()
        gate=project/'ready.json'
        c.proof['allow_development']=True;c.proof['development_boundary']='explicit pilot development channel only; dirtyfalse/exactSHA/hash still independently required'
        value={'schema':'zee-http-service-1','allow_development':True,'repo':str(c.repo),'source_sha':a.sha,'dil':str(c.binary),'binary_sha256':a.binary_sha256,'project':str(project),'project_files':inventory(project),'probe_project':str(probe),'probe_files':inventory(probe),'contract':str(contract),'contract_sha256':D.snapshot(contract),'application_revision':revision,'origin':'https://fixture.example','worker_ports':ports,'readiness_file':str(gate),'health_path':'/hazir','health_body':'ok','state':str(c.out/'service-state'),'watch_interval':2,'probe_deadline':8,'readiness_ttl':12,'credential_root':None,'credential_env':[],'probe_credential_root':str(credentials),'probe_credential_name':'db','probe_ca':str(public),'probe_ca_sha256':D.snapshot(public)}
        cfg=c.jsonfile(c.out/'http.json',value)
        def configure(limit):
            new=project/'zee.ayar.new';new.write_text(json.dumps({'sema':'zee-ayar-1','surum':1,'ayarlar':{},'guvenlik':{'web-modu':'proxy','proxy-origin':'https://fixture.example','oran-deposu':'bellek','oran-siniri':limit,'oran-kapasitesi':120,'oran-doluluk':'reddet'}}));new.chmod(0o600);new.replace(project/'zee.ayar')
        configure(100)
        def ready(expected):
            try:return json.loads(gate.read_text())['ready']is expected
            except (FileNotFoundError,ValueError):return False
        units=c.out/'units';c.command('render',[sys.executable,'-I','-B',source/'http-service.py','--config',cfg,'--render',units],45);c.command('verify',['systemd-analyze','--user','verify',units/'zee-http.service']);c.link(units,['zee-http.service']);c.start('zee-http.service')
        def healthy():
            try:return ready(True) and all(request(p0,'/hazir')==(200,b'ok') for p0 in ports)
            except (OSError,ValueError):return False
        c.wait(healthy,180)
        for p0 in ports:c.need(request(p0,'/hazir')==(200,b'ok'))
        c.record('actual-rendered-continuous-user-service-nativeHTTP-PG',workers=2)
        sql("UPDATE public.zee_app_state SET revision='wrong';",'zee_readiness_fixture');c.wait(lambda:ready(False),40)
        for p0 in ports:c.need(request(p0,'/hazir')[0]==200)
        c.record('actual-DB-drift-unready-liveness-still200',ingress_rejection_claim=False)
        sql("UPDATE public.zee_app_state SET revision='readiness-v1';",'zee_readiness_fixture');c.wait(lambda:ready(True),40);c.record('actual-DB-restore-ready')
        # Quota key is path-local. /hazir polling does not spend /limit budget.
        for p0 in ports:c.need([request(p0,'/limit')[0]for _ in range(101)]==[200]*100+[429])
        configure(101);c.system('reload-http','reload','zee-http.service')
        # First observed200 after the old exhausted quota proves replacement; it spends one of101.
        def successful(p0,path):
            try:return request(p0,path)[0]==200
            except (OSError,ValueError):return False
        for p0 in ports:
            c.wait(lambda:successful(p0,'/limit'),180)
            c.need([request(p0,'/limit')[0]for _ in range(101)]==[200]*100+[429])
        c.wait(lambda:ready(True),60)
        c.record('actual-HUP-config100-to101-native-quota',zero_downtime_claim=False)
        restart_ns=time.time_ns();c.system('kill-owned-http','kill','--kill-who=all','--signal=SIGKILL','zee-http.service');c.wait(lambda:ready(True) and gate.stat().st_mtime_ns>restart_ns,90)
        c.wait(lambda:all(successful(p0,'/hazir')for p0 in ports),90);c.record('actual-owned-unit-failure-systemd-restart',pid_adoption=False)
        result=[]
        def slow():
            try:result.append(request(ports[0],'/yavas'))
            except Exception:result.append(None)
        thread=threading.Thread(target=slow);thread.start();time.sleep(.5);c.stop('zee-http.service');thread.join(8);c.need(not thread.is_alive() and result==[(200,b'bitti')]);c.need(ready(False));c.record('actual-unit-TERM-finishes-inflight-and-final-false',client_response_received=True);c.unlink()
        # Exact own Popen watcher loss inside the production host class, separate from userunit claims.
        owned=[];original=http.Service.live
        def live(self,workers):
            original(self,workers)
            if len(workers)==2 and self.application_ready() and not owned:
                owned.append(self);os.kill(self.watcher.pid,signal.SIGKILL)
        try:
            with mock.patch.object(http.Service,'live',live):http.execute(http.configuration(cfg))
        except http.W.Reject:pass
        c.need(len(owned)==1 and owned[0].watcher.returncode is not None and not owned[0].children);c.record('actual-native-host-component-owned-watcher-loss-cleans-workers',userunit_case=False)
        c.finish()
if __name__=='__main__':
    try:main()
    except (Exception,KeyboardInterrupt):sys.stderr.write('owned-http-service-campaign-failed\n');raise SystemExit(2)
