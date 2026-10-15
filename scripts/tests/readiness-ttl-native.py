#!/usr/bin/env python3
"""Owned two-worker TTL expiry and actual config replacement/HUP preparation."""
import argparse,hashlib,json,os,runpy,signal,socket,stat,time
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def require(value):
    if not value:raise AssertionError('owned acceptance predicate')
def sha(path):
    fd=os.open(path,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW)
    try:
        before=os.fstat(fd);require(stat.S_ISREG(before.st_mode) and before.st_nlink==1 and before.st_size<=134217728)
        digest=hashlib.sha256();size=0
        while True:
            data=os.read(fd,65536)
            if not data:break
            size+=len(data);require(size<=134217728);digest.update(data)
        after=os.fstat(fd);require((before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)==(after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns))
        return digest.hexdigest()
    finally:os.close(fd)
def request(port,path):
    deadline=time.monotonic()+3;data=bytearray()
    with socket.create_connection(('127.0.0.1',port),timeout=3) as s:
        s.sendall(('GET '+path+' HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n').encode())
        while True:
            s.settimeout(max(.001,deadline-time.monotonic()));chunk=s.recv(4096)
            require(time.monotonic()<deadline and len(data)+len(chunk)<=16384)
            if not chunk:break
            data.extend(chunk)
    return int(bytes(data).split(b' ',2)[1])
def main():
    p=argparse.ArgumentParser();p.add_argument('--dil',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(mode=0o700,parents=True,exist_ok=False)
    sources=[Path(__file__).resolve(),ROOT/'scripts/worker-supervisor.py',ROOT/'scripts/tests/worker-supervisor-test.py',ROOT/'scripts/native-capabilities.py',ROOT/'scripts/native-metrics.py'];before={str(f.relative_to(ROOT)):sha(f) for f in sources};binaryhash=sha(a.dil)
    W=runpy.run_path(str(ROOT/'scripts/tests/worker-supervisor-test.py'));W['Fixture'].__init__.__globals__['NATIVE']=a.dil;owner=None;proof={'schema':'zee-k274-native-1','status':'failed','cases':[],'boundaries':['short owned fixture, not 900s/clean final acceptance','memory rate counters reset on restart; no zero downtime promise'],'source_hashes':before,'binary_sha256':binaryhash}
    def event(r,ready,after=0):return r.until(lambda e:e.get('event')=='state' and e.get('ready') is ready and e.get('state') in {'ready','unready'},timeout=12,after=after)
    try:
        owner=W['Fixture'](native=True,ready=False,extra=('--readiness-ttl-seconds','8'))
        proof['compiler_identity']=owner.build
        owner.args[owner.args.index('--startup-timeout')+1]='15';r=owner.start();event(r,False);n=len(r.events);W['gate'](owner.gate,True);event(r,True,n)
        require(all(request(port,'/hazir')==200 for port in owner.ports));proof['cases'].append('fresh-receipt-two-workers-ready')
        n=len(r.events);event(r,False,n);require(all(request(port,'/hazir')==200 for port in owner.ports));proof['cases'].append('expired-receipt-unready-with-two-live-200-workers')
        n=len(r.events);W['gate'](owner.gate,True);event(r,True,n);proof['cases'].append('fresh-receipt-restores-readiness');owner.close();owner=None
        owner=W['Fixture'](native=True,ready=True);require(owner.build==proof['compiler_identity']);owner.args[owner.args.index('--startup-timeout')+1]='15';owner.source.write_text(owner.source.read_text()+'\nGET "/limit" adresine istek geldiğinde\n    "ok" yanıtını gönder\n')
        config=owner.project/'zee.ayar'
        def configure(limit):
            value={'sema':'zee-ayar-1','surum':1,'ayarlar':{},'guvenlik':{'web-modu':'proxy','proxy-origin':'https://fixture.example','oran-deposu':'bellek','oran-siniri':limit,'oran-kapasitesi':120,'oran-doluluk':'reddet'}}
            tmp=config.with_suffix('.next');tmp.write_text(json.dumps(value));tmp.chmod(0o600);os.replace(tmp,config)
        configure(100);r=owner.start();event(r,True)
        oldpids=[e['pid'] for e in r.events if e.get('event')=='worker_started'];require(len(oldpids)==2)
        for port in owner.ports:require([request(port,'/limit') for _ in range(101)]==[200]*100+[429])
        proof['cases'].append('actual-config-initial-limit100');configure(101);n=len(r.events);r.child.send_signal(signal.SIGHUP);r.until(lambda e:e.get('state')=='restarting' and e.get('ready') is False,timeout=8,after=n);event(r,True,n)
        newpids=[e['pid'] for e in r.events[n:] if e.get('event')=='worker_started'];require(len(newpids)==2 and set(oldpids).isdisjoint(newpids))
        for port in owner.ports:require([request(port,'/limit') for _ in range(102)]==[200]*101+[429])
        proof['cases'].append('actual-config-replacement-hup-new-workers-limit101');owner.close();owner=None
        require(before=={str(f.relative_to(ROOT)):sha(f) for f in sources} and sha(a.dil)==binaryhash);proof.update(status='passed',case_count=len(proof['cases']),source_unchanged=True,binary_unchanged=True,owned_workers_closed=True)
    except BaseException as error:proof['failure_class']=type(error).__name__;raise
    finally:
        try:
            if owner is not None:owner.close()
        finally:
            (a.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps({'status':proof['status'],'cases':len(proof['cases'])}))
if __name__=='__main__':main()
