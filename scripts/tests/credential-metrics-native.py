#!/usr/bin/env python3
"""K260 actual native credential decrypt + two-worker metrics and HUP lifecycle."""
import argparse,hashlib,json,os,runpy,signal,socket,subprocess,sys,time
from pathlib import Path

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def require(v,m):
    if not v:raise AssertionError(m)
def main():
    a=argparse.ArgumentParser();a.add_argument('--dil',type=Path,required=True);a.add_argument('--output',type=Path,required=True);args=a.parse_args();root=Path(__file__).resolve().parents[2];binary=args.dil.resolve(strict=True);out=args.output.resolve();require(not out.exists(),'fresh proof');out.mkdir(mode=0o700,parents=True)
    sources=[root/'scripts/credential-baslat.py',root/'scripts/worker-supervisor.py',root/'scripts/native-metrics.py',root/'scripts/tests/worker-supervisor-test.py',Path(__file__).resolve(),root/'scripts/tests/credential-metrics-test.py'];hashes=[{'path':str(p.relative_to(root)),'sha256':sha(p)} for p in sources];binary_sha=sha(binary)
    W=runpy.run_path(str(root/'scripts/tests/worker-supervisor-test.py'));W['Fixture'].__init__.__globals__['NATIVE']=binary
    identity=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5))
    f=None;r=None;proof={'schema':'zee-k260-native-capability-1','status':'running','compiler_identity':identity,'binary_sha256':binary_sha,'sources':hashes,'cases':[]}
    try:
        f=W['Fixture'](native=True,credential=True,extra=['--runtime-metrics']);f.args[f.args.index('--startup-timeout')+1]='15';r=f.start();r.until(lambda x:x.get('event')=='state' and x.get('ready') is True,timeout=15)
        def request(port):
            with socket.create_connection(('127.0.0.1',port),timeout=5) as s:
                s.settimeout(5);s.sendall(b'GET /hazir HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n');data=b''
                while True:
                    b=s.recv(1024)
                    if not b:break
                    data+=b;require(len(data)<8192,'bounded oracle')
                require(data.startswith(b'HTTP/1.1 200 ') and data.endswith(b'ok'),'actual native response')
        for port in f.ports:request(port)
        metric=r.until(lambda x:x.get('event')=='native_metrics' and x.get('kind')==0 and x['v'][0]>=4,timeout=10);require(metric['channels']==2 and metric['unavailable']==0,'actual credential-owned metrics pipes')
        pids=[x['pid'] for x in r.events if x.get('event')=='worker_started'];proof['cases'].append({'name':'actual-credential-decrypt-two-native-workers-owned-metrics','status':'passed','request_prefix':metric['v'],'channels':metric['channels']})
        after=len(r.events);r.child.send_signal(signal.SIGHUP);r.until(lambda x:x.get('event')=='state' and x.get('ready') is True,after=after,timeout=15)
        for port in f.ports:request(port)
        metric=r.until(lambda x:x.get('event')=='native_metrics' and x.get('kind')==0 and x['v'][0]>=8,after=after,timeout=10);require(metric['channels']==2 and metric['unavailable']==0,'actual credential restart finite rollup')
        for pid in pids:
            try:os.kill(pid,0)
            except ProcessLookupError:continue
            raise AssertionError('old owned launcher leader still exists')
        proof['cases'].append({'name':'actual-credential-hup-fresh-native-children-rollup-and-old-group-cleanup','status':'passed','request_prefix':metric['v']})
        require(r.finish()==0,'TERM owner cleanup');serialized=json.dumps(r.events,ensure_ascii=False,separators=(',',':')).encode();require(W['CANARY'].encode() not in serialized and b'a5a5a5a5' not in serialized,'credential ciphertext/plaintext absent from metadata');(out/'host-metadata.jsonl').write_bytes(b''.join(json.dumps(x,separators=(',',':')).encode()+b'\n' for x in r.events));f.close();f=None;r=None
        require(sha(binary)==binary_sha and hashes==[{'path':str(p.relative_to(root)),'sha256':sha(p)} for p in sources],'source/binary unchanged');require(json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5))==identity,'build identity unchanged');proof['status']='passed'
    except BaseException as e:
        proof['status']='failed';proof['failure_type']=type(e).__name__;raise
    finally:
        if r and r.child.poll() is None:r.finish()
        if f:f.close()
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps({'status':proof['status'],'cases':len(proof['cases']),'proof_sha256':sha(out/'proof.json')}))
if __name__=='__main__':main()
