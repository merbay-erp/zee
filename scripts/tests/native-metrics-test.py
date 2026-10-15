#!/usr/bin/env python3
"""K-257 actual owned PG16.15, native request counters and host composition."""
import argparse,hashlib,json,os,runpy,signal,socket,subprocess,tempfile,time
from pathlib import Path


def require(ok,label):
    if not ok:raise AssertionError(label)


def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(65536),b''):h.update(b)
    return h.hexdigest()


def main():
    p=argparse.ArgumentParser();p.add_argument('--dil',type=Path,required=True);p.add_argument('--framework-root',type=Path,required=True);p.add_argument('--postgres-bin-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    root=a.framework_root.resolve(strict=True);binary=a.dil.resolve(strict=True);out=a.output.resolve();require(not out.exists(),'fresh output');out.mkdir(parents=True,mode=0o700)
    TLS=runpy.run_path(str(root/'scripts/postgresql-tls-ci.py'));API=runpy.run_path(str(root/'scripts/native-metrics.py'))
    source=[root/'scripts/native-metrics.py',root/'scripts/native-capabilities.py',root/'scripts/worker-supervisor.py',root/'scripts/postgresql-tls-ci.py',root/'scripts/postgresql-tls11-control.py',root/'scripts/credential-baslat.py',root/'scripts/tests/worker-supervisor-test.py',Path(__file__).resolve(),root/'compiler/src/web_metrikleri.rs',root/'compiler/src/postgresql/havuz.rs',root/'compiler/src/postgresql/havuz/metrik.rs',root/'compiler/src/postgresql.rs',root/'compiler/src/main.rs',root/'dogfood/zee-cekirdek/metrik/ana.dil',root/'dogfood/zee-cekirdek/metrik/proje.dil']
    hashes=[{'path':str(x.relative_to(root)),'sha256':sha(x)} for x in source]
    proof={'schema':'zee-k257-actual-native-1','status':'running','cases':[],'compiler_identity':json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5)),'binary_sha256':sha(binary),'sources':hashes,'scope':'native exact transport/status/rate decisions and pool boundary samples; no Prometheus endpoint/clientdelivery/fullP13 acceptance'}
    cluster=None;children=[];fds=[];owner=None;running=None
    try:
        short_tmp = '/private/tmp' if Path('/private/tmp').is_dir() else '/tmp'
        with tempfile.TemporaryDirectory(prefix='zee-k257-',dir=short_tmp) as t:
            owned=Path(t).resolve();_,_,cert,key=TLS['certificates'](owned)
            (owned/'pg_hba.conf').write_text('local all all trust\nhost all all 127.0.0.1/32 trust\n')
            cluster=TLS['LocalCluster'](owned,cert,key,a.postgres_bin_dir);cluster.start();require(cluster.version=='postgres (PostgreSQL) 16.15','owned PG16.15')
            proof['postgres_version']=cluster.version
            env=os.environ.copy();env['RUSTUP_TOOLCHAIN']='1.93.1';env['ZEE_K257_FIXTURE_HEDEFI']=f'postgresql://localhost:{cluster.port}/postgres';env['ZEE_K257_FIXTURE_URL']=f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require';env['ZEE_K257_FIXTURE_URL_TLS_CA_PEM']=(owned/'ca.crt').read_text()
            command=['cargo','test','--locked','--offline','--lib','postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar','--','--exact','--ignored','--nocapture','--test-threads=1']
            result=subprocess.run(command,cwd=root/'compiler',env=env,capture_output=True,timeout=120)
            (out/'owned-pg-pool-stdout.log').write_bytes(result.stdout);(out/'owned-pg-pool-stderr.log').write_bytes(result.stderr)
            require(result.returncode==0 and b'1 passed; 0 failed; 0 ignored' in result.stdout,'actual concurrent4lease/5thwait/timeout/recovery')
            rows=json.loads(next(x.split('K257_OWNED_POOL_METRICS ',1)[1] for x in result.stdout.decode().splitlines() if 'K257_OWNED_POOL_METRICS ' in x))
            require(any(r[:4]==[4,0,4,1] for r in rows) and rows[-1][:4]==[4,4,0,0] and rows[-1][5]==1,'actual owner state records')
            proof['cases'].append({'name':'actual-pg-pool-four-leases-wait-timeout-recovery','status':'passed','snapshots':rows})
            project=owned/'project';project.mkdir(mode=0o700)
            fixture=root/'dogfood/zee-cekirdek/metrik'
            (project/'proje.dil').write_text((fixture/'proje.dil').read_text().replace('postgresql://localhost:5432/postgres',env['ZEE_K257_FIXTURE_HEDEFI']))
            (project/'ana.dil').write_bytes((fixture/'ana.dil').read_bytes())
            check=subprocess.run([str(binary),'denetle',str(project)],env=env,capture_output=True,timeout=10);require(check.returncode==0,'anonymous native source denetle')
            def start(mode='consume'):
                with socket.socket() as free:free.bind(('127.0.0.1',0));port=free.getsockname()[1]
                rd,wr=os.pipe();os.set_blocking(rd,False);os.set_blocking(wr,False)
                if mode=='full':
                    while True:
                        try:os.write(wr,b'x'*512)
                        except BlockingIOError:break
                if mode=='closed':os.close(rd);rd=None
                child=subprocess.Popen([str(binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(port),'--web-oran-deposu','bellek','--web-oran-siniri','2','--web-metrik-fd',str(wr),str(project)],env=env,stdin=subprocess.DEVNULL,stdout=wr if mode=='alias' else subprocess.DEVNULL,stderr=wr if mode=='stderr-alias' else subprocess.PIPE,pass_fds=(wr,),start_new_session=True)
                children.append(child);os.close(wr)
                if rd is not None:fds.append(rd)
                if mode in ('alias','stderr-alias'):
                    code=child.wait(timeout=10);error=child.stderr.read() if child.stderr else b''
                    channel=os.read(rd,4096)
                    require(code==2 and error==b'' and channel==b'','stdio alias quiet rejection')
                    require(b'ZEE_K257_PRIVATE_SOURCE_CANARY' not in channel and b'zee-native-metrics-1' not in channel,'early alias rejection before private source diagnostics')
                    return child,port,rd
                deadline=time.monotonic()+5
                while True:
                    require(child.poll() is None and time.monotonic()<deadline,'actual worker start')
                    try:
                        with socket.create_connection(('127.0.0.1',port),timeout=.1):break
                    except OSError:time.sleep(.01)
                # The startup connection was EOF-only; it is a native no-response attempt.
                return child,port,rd
            def request(port,path,forward=True):
                with socket.create_connection(('127.0.0.1',port),timeout=5) as s:
                    s.settimeout(5);s.sendall(b'GET '+path+b' HTTP/1.1\r\nHost: fixture.example\r\n'+(b'Forwarded: for=127.0.0.1;proto=https;host=fixture.example\r\n' if forward else b'')+b'Connection: close\r\n\r\n');reply=bytearray()
                    while True:
                        b=s.recv(1024)
                        if not b:break
                        reply.extend(b);require(len(reply)<=16384,'bounded native response')
                    return int(reply.split(b' ',2)[1]),bytes(reply)
            child,port,rd=start();channel=API['Channel'](rd)
            results=[request(port,b'/ok'),request(port,b'/ok'),request(port,b'/ok'),request(port,b'/missing-ZEE_K257_PRIVATE_SENTINEL'),request(port,b'/ok',False),request(port,b'/db')]
            require([r[0] for r in results]==[200,200,429,404,426,200] and results[-1][1].endswith(b'db-ok'),'actual response oracle')
            child.send_signal(signal.SIGTERM);child.wait(timeout=5)
            raw=bytearray()
            while True:
                try:part=os.read(rd,4096)
                except BlockingIOError:break
                if not part:break
                raw.extend(part);require(len(raw)<=32768,'bounded actual native metadata')
            require(b'ZEE_K257_PRIVATE_SENTINEL' not in raw and b'SELECT' not in raw and b'fixture.example' not in raw,'actual content absent from pipe')
            for line in raw.splitlines():API['decode'](line)
            (out/'native-metadata.jsonl').write_bytes(raw)
            channel.buffer.extend(raw);channel.receive();require(channel.valid,'finite native metadata')
            row,db=channel.rows[0],channel.rows[1]
            require(row[0]==row[1]==7 and row[4]==1 and row[5]==1 and row[6]==3 and row[11]==1 and row[15]==1 and row[16]==1,'native accept/EOF/prehandler429/finalstatus counters')
            require(db[0]>=1 and db[1]==db[0] and db[2]==db[3]==0 and db[4]>=2 and db[7]>=50000,'actual checkout and idle safe boundary')
            metadata=json.dumps({'request':row,'pool':db});require('ZEE_K257_PRIVATE_SENTINEL' not in metadata,'metadata secret absence')
            proof['cases'].append({'name':'native-request-status-rate429-db-boundary-no-content','status':'passed','request':row,'pool':db,'body_path_sql_absent':True})
            if rd in fds:fds.remove(rd)
            channel.close()
            child,port,rd=start('full');begin=time.monotonic();codes=[request(port,b'/hazir')[0] for _ in range(2)];require(codes==[200,200] and time.monotonic()-begin<2,'full pipe never delays requests')
            while True:
                try:
                    if not os.read(rd,4096):break
                except BlockingIOError:break
            require(request(port,b'/ok')[0]==200,'full pipe recovery');child.send_signal(signal.SIGTERM);child.wait(timeout=5);channel=API['Channel'](rd);channel.receive();require(channel.valid and channel.dropped>=5,'actual dropped snapshots recovery');proof['cases'].append({'name':'actual-native-pipe-backpressure-recovery','status':'passed','codes':codes,'dropped':channel.dropped});fds.remove(rd);channel.close()
            child,port,rd=start('closed');require(request(port,b'/hazir')[0]==200 and child.poll() is None,'closed pipe not SIGPIPE/request failure');child.send_signal(signal.SIGTERM);require(child.wait(timeout=5)==0,'closed pipe graceful exit');proof['cases'].append({'name':'actual-native-closed-reader-no-sigpipe','status':'passed'})
            original_source=(project/'ana.dil').read_bytes()
            (project/'ana.dil').write_text('ZEE_K257_PRIVATE_SOURCE_CANARY !!! malformed native source\n')
            for alias in ('alias','stderr-alias'):
                child,port,rd=start(alias);proof['cases'].append({'name':'actual-native-malformed-source-'+alias+'-early-rejection','status':'passed','private_source_canary_absent':True});os.close(rd);fds.remove(rd)
            (project/'ana.dil').write_bytes(original_source)
            W=runpy.run_path(str(root/'scripts/tests/worker-supervisor-test.py'));W['Fixture'].__init__.__globals__['NATIVE']=binary
            owner=W['Fixture'](native=True,app_signal=True,extra=['--runtime-metrics']);owner.args[owner.args.index('--startup-timeout')+1]='15';running=owner.start();running.until(lambda x:x.get('event')=='state' and x.get('ready') is True,timeout=15)
            for port in owner.ports:require(request(port,b'/hazir')[0]==200,'two host workers')
            metrics=running.until(lambda x:x.get('event')=='native_metrics' and x.get('kind')==0 and x['v'][0]>=4,timeout=10)
            require(len(metrics['v'])==11 and metrics['unavailable']==0 and metrics['channels']==2,'owned host finite rollup')
            proof['cases'].append({'name':'actual-two-worker-supervisor-collector','status':'passed','request_prefix':metrics['v'],'channels':2})
            running.finish();owner.close();owner=None;running=None
            # K260 now supports precisely the designated metadata capability.
            with W['Fixture'](native=True,credential=True,extra=['--runtime-metrics']) as f:
                f.args[f.args.index('--startup-timeout')+1]='15'
                r=f.start();r.until(lambda x:x.get('event')=='state' and x.get('ready') is True,timeout=15)
                for port in f.ports:require(request(port,b'/hazir')[0]==200,'credential native workers healthy')
                rollup=r.until(lambda x:x.get('event')=='native_metrics' and x.get('kind')==0 and x['v'][0]>=4,timeout=10)
                require(rollup['channels']==2 and rollup['unavailable']==0,'credential designated capability rollup')
                require(r.finish()==0,'credential metric worker groups gracefully closed')
                require(W['CANARY'] not in json.dumps(r.events),'credential plaintext absent from host metadata')
            proof['cases'].append({'name':'credential-designated-metadata-composition-two-workers','status':'passed','channels':2})
            cluster.close();require(not (cluster.data/'postmaster.pid').exists(),'owned cluster stopped');cluster=None
            require(hashes==[{'path':str(x.relative_to(root)),'sha256':sha(x)} for x in source] and sha(binary)==proof['binary_sha256'],'source/binary unchanged')
            proof.update(status='passed',case_count=len(proof['cases']),failed=0,ignored=0,owned_workers_closed=True,owned_pg_closed=True)
    except BaseException as e:
        proof.update(status='failed',failure_class=type(e).__name__,failure_message=str(e));raise
    finally:
        if owner is not None:owner.close()
        for child in children:
            if child.poll() is None:os.killpg(child.pid,signal.SIGKILL)
            child.wait(timeout=5)
            if child.stderr is not None:child.stderr.close()
        for fd in fds:os.close(fd)
        if cluster is not None:cluster.close()
        proof['raw_files']=[{'path':p.name,'sha256':sha(p)} for p in sorted(out.iterdir()) if p.is_file()]
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps({'status':proof['status'],'cases':proof['case_count'],'proof_sha256':sha(out/'proof.json')}))


if __name__=='__main__':main()
