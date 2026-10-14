#!/usr/bin/env python3
"""Actual native SIGTERM drain fixture; run unchanged against before/after binaries."""
import argparse,hashlib,json,os,selectors,signal,socket,subprocess,time
from pathlib import Path
import tempfile


def require(ok,label):
    if not ok:raise AssertionError(label)


def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(65536),b''):h.update(b)
    return h.hexdigest()


def main():
    p=argparse.ArgumentParser();p.add_argument('--dil',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--expect-drain',action='store_true');p.add_argument('--fixture-project',type=Path,required=True);a=p.parse_args()
    binary=a.dil.resolve(strict=True);out=a.output.resolve();require(not out.exists(),'fresh output');out.mkdir(parents=True,mode=0o700)
    identity=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5))
    proof={'schema':'zee-native-sigterm-drain-1','status':'running','compiler_identity':identity,'binary_sha256':sha(binary),'source_sha256':sha(Path(__file__).resolve()),'expected_drain':a.expect_drain,'cases':[],'scope':'single admitted native handler; no queued/backlog drain, socket transfer or full P14 acceptance'}
    process=None;s=None;queued=None;selector=None
    try:
        with tempfile.TemporaryDirectory(prefix='zee-drain-') as t:
            project=Path(t).resolve()
            fixture=a.fixture_project.resolve(strict=True)
            for name in ['proje.dil','ana.dil']:
                original=fixture/name
                (project/name).write_bytes(original.read_bytes())
                proof.setdefault('fixture_sources',[]).append({'path':name,'sha256':sha(original)})
            check=subprocess.run([str(binary),'denetle',str(project)],capture_output=True,timeout=10);require(check.returncode==0,'anonymous fixture denetle')
            with socket.socket() as free:free.bind(('127.0.0.1',0));port=free.getsockname()[1]
            def launch():
                child=subprocess.Popen([str(binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(port),str(project)],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
                sel=selectors.DefaultSelector();os.set_blocking(child.stdout.fileno(),False);sel.register(child.stdout,selectors.EVENT_READ)
                return child,sel
            def line(expected):
                deadline=time.monotonic()+5;data=bytearray()
                while True:
                    require(time.monotonic()<deadline and len(data)<=4096,'bounded actual entry marker')
                    for key,_ in selector.select(min(.1,max(0,deadline-time.monotonic()))):
                        b=os.read(key.fileobj.fileno(),4096);require(bool(b),'stdout EOF');data.extend(b)
                    if expected in data:return
            def request(path):
                s=socket.create_connection(('127.0.0.1',port),timeout=5);s.settimeout(5);s.sendall(b'GET '+path+b' HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n');return s
            def receive(s):
                reply=bytearray()
                try:
                    while True:
                        b=s.recv(1024)
                        if not b:break
                        reply.extend(b);require(len(reply)<=16384,'bounded response')
                except ConnectionResetError:pass
                return bytes(reply)
            process,selector=launch();line(b'Sunucu dinliyor:')
            s=request(b'/slow');line(b'entered\n');start=time.monotonic();reply=receive(s);s.close();s=None
            require(reply.startswith(b'HTTP/1.1 200 ') and reply.endswith(b'completed'),'baseline response')
            proof['cases'].append({'name':'baseline-actual-handler-completed','http_status':200,'elapsed_seconds':round(time.monotonic()-start,3)})
            s=request(b'/slow');line(b'entered\n');start=time.monotonic();process.send_signal(signal.SIGTERM)
            if a.expect_drain:
                time.sleep(.03);queued=request(b'/hazir')
            reply=receive(s);s.close();s=None
            code=process.wait(timeout=5);duration=time.monotonic()-start;stderr=process.stderr.read();(out/'synthetic-stderr.log').write_bytes(stderr)
            completed=reply.startswith(b'HTTP/1.1 200 ') and reply.endswith(b'completed')
            require(completed is a.expect_drain,'expected actual drain behavior')
            if a.expect_drain:require(code==0 and 2<=duration<=4.5,'completed handler then exit0')
            else:require(code==-signal.SIGTERM and duration<1,'actual before default SIGTERM')
            proof['cases'].append({'name':'sigterm-after-actual-entry','http200_completed':completed,'response_bytes':len(reply),'exit':code,'elapsed_seconds':round(duration,3)})
            if queued is not None:
                refused=receive(queued);queued.close();queued=None;require(not refused.startswith(b'HTTP/1.1 200 '),'no new admitted handler after SIGTERM');proof['cases'].append({'name':'no-new-handler-after-sigterm','http200_completed':False,'response_bytes':len(refused)})
            process.stdout.close();process.stderr.close();selector.close();selector=None;process=None
            if a.expect_drain:
                process,selector=launch();line(b'Sunucu dinliyor:');start=time.monotonic();process.send_signal(signal.SIGTERM);code=process.wait(timeout=2);require(code==0 and time.monotonic()-start<1,'idle bounded stop')
                proof['cases'].append({'name':'idle-accept-bounded-stop','exit':code,'elapsed_seconds':round(time.monotonic()-start,3)})
            require(sha(binary)==proof['binary_sha256'],'binary unchanged')
            proof.update(status='passed' if a.expect_drain else 'confirmed-missing-drain',owned_worker_closed=True)
    except BaseException as e:
        proof['status']='failed';proof['failure_class']=type(e).__name__;proof['failure_message']=str(e);raise
    finally:
        if s is not None:s.close()
        if queued is not None:queued.close()
        if process is not None:
            if process.poll() is None:os.killpg(process.pid,signal.SIGKILL)
            process.wait(timeout=5);process.stdout.close();process.stderr.close()
        if selector is not None:selector.close()
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps({'status':proof['status'],'proof_sha256':sha(out/'proof.json')}))

if __name__=='__main__':main()
