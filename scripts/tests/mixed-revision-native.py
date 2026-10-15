#!/usr/bin/env python3
"""K291: bounded owned mixed-code revisions over one native listener and shared state."""
import argparse,builtins,concurrent.futures,hashlib,importlib.util,json,os,re,selectors,signal,socket,stat,subprocess,sys,threading,time
from pathlib import Path
HERE=Path(__file__).resolve().parents[1]

class Rejected(Exception):pass

def need(ok):
    if not ok:raise Rejected('mixed revision acceptance rejected')

def snapshot(path,limit=134217728):
    path=Path(path);before=path.lstat();need(stat.S_ISREG(before.st_mode) and before.st_size<=limit)
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
    try:
        opened=os.fstat(fd);key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns);need(key(before)==key(opened));h=hashlib.sha256();count=0
        while count<=limit:
            try:part=os.read(fd,min(65536,limit+1-count))
            except InterruptedError:continue
            if not part:break
            count+=len(part);need(count<=limit);h.update(part)
        need(count==opened.st_size and key(opened)==key(os.fstat(fd))==key(path.lstat()));return h.hexdigest()
    finally:os.close(fd)

def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

class ClientFailure(Exception):
    def __init__(self,kind):self.kind=kind

def http(port,path='/revision',client='127.2.0.1',method='GET',cookie=None,csrf=None,on_sent=None,deadline=None):
    end=min(time.monotonic()+5,deadline) if deadline is not None else time.monotonic()+5;raw=bytearray();body=''if csrf is None else '_csrf='+csrf
    headers='Host: fixture.example\r\nForwarded: for='+client+';proto=https;host=fixture.example\r\nConnection: close\r\n'
    if cookie:headers+='Cookie: '+cookie+'\r\n'
    if method=='POST':headers+='Origin: https://fixture.example\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: '+str(len(body))+'\r\n'
    try:
        left=end-time.monotonic()
        if left<=0:raise ClientFailure('deadline')
        with socket.create_connection(('127.0.0.1',port),timeout=left)as s:
            left=end-time.monotonic()
            if left<=0:raise ClientFailure('deadline')
            s.settimeout(left);s.sendall((method+' '+path+' HTTP/1.1\r\n'+headers+'\r\n'+body).encode())
            if on_sent is not None:on_sent()
            while True:
                left=end-time.monotonic()
                if left<=0:raise ClientFailure('deadline')
                s.settimeout(left);part=s.recv(4096)
                if not part:break
                raw.extend(part)
                if len(raw)>32768:raise ClientFailure('output_bound')
    except socket.timeout:raise ClientFailure('deadline') from None
    except ConnectionRefusedError:raise ClientFailure('refused') from None
    except OSError:raise ClientFailure('transport') from None
    try:
        header,payload=bytes(raw).split(b'\r\n\r\n',1);lines=header.decode('ascii').split('\r\n')
        if len(header)>16384 or re.fullmatch(r'HTTP/1\.[01] [1-5][0-9]{2} [ -~]+',lines[0])is None:raise ValueError()
        status=int(lines[0].split(' ')[1]);fields={}
        for line in lines[1:]:
            key,value=line.split(':',1);key=key.lower()
            if key in fields or re.fullmatch(r'[a-z0-9-]+',key)is None:raise ValueError()
            fields[key]=value.strip()
        if fields.get('content-length')!=str(len(payload))or fields.get('connection','').lower()!='close' or 'transfer-encoding'in fields:raise ValueError()
        return status,payload,fields.get('set-cookie','').split(';')[0]or None
    except (ValueError,UnicodeError):raise ClientFailure('partial_or_framing') from None

class Run:
    def __init__(self,args):
        self.a=args;self.stop=False;self.end=time.monotonic()+300;self.host=None;self.foreign=None;self.gens=[];self.oldhandlers={};self.cases=[];self.revisions=[]
        need(os.name=='posix' and os.geteuid()!=0 and re.fullmatch('[0-9a-f]{40}',args.sha)and re.fullmatch('[0-9a-f]{64}',args.binary_sha256))
        self.repo=Path(args.repo);self.binary=Path(args.dil);self.out=Path(args.out)
        for p in (self.repo,self.binary,self.out):need(p.is_absolute()and p.resolve()==p)
        need(not self.out.exists()and self.out.parent.stat().st_uid==os.geteuid()and stat.S_IMODE(self.out.parent.stat().st_mode)==0o700 and self.repo not in self.out.parents and self.out not in self.repo.parents)
        self.H=load('revision_lifecycle',self.repo/'scripts/lifecycle_core.py');self.H.clean_exact(self.repo,args.sha);need(snapshot(self.binary)==args.binary_sha256)
        self.out.mkdir(mode=0o700);self.proof={'schema':'zee-mixed-revision-native-1','source_sha':args.sha,'binary_sha256':args.binary_sha256,'cases':self.cases,'revisions':self.revisions,'status':'running','boundaries':['anonymous same-contract code revisions only','not universal artifact/schema deployment, saturation or zero-downtime SLA','cookie/CSRF/argv/source values not logged']}
        self.source_pins={str(p.relative_to(self.repo)):snapshot(p,262144)for p in [Path(__file__).resolve(),self.repo/'scripts/native-listener-host.py',self.repo/'scripts/worker-supervisor.py',self.repo/'scripts/native-capabilities.py',self.repo/'scripts/lifecycle_core.py',self.repo/'scripts/tests/native-listener-deadline-test.py']}
        self.proof['source_pins_start']=self.source_pins
        for n in (signal.SIGTERM,signal.SIGINT):self.oldhandlers[n]=signal.signal(n,self.cancel)
    def cancel(self,n,f):self.stop=True;self.H.CANCELLED=True
    def check(self):need(not self.stop and time.monotonic()<self.end)
    def request(self,*args,**kwargs):
        self.check();return http(*args,**kwargs,deadline=self.end)
    def publish(self,number):
        revision=hashlib.sha1(('K291-anonymous-'+str(number)).encode()).hexdigest();marker='revision-'+str(number)
        receipt=json.dumps({'schema':'zee-worker-readiness-1','application_revision':revision,'ready':True},separators=(',',':'))
        program='argümanlar komut satırından gelenler olsun\nreceipt argümanların ilki olsun\nreceipt dosyasına '+json.dumps(receipt)+' yaz\n8080 kapısında sunucu başlat\nGET "/revision" adresine istek geldiğinde\n    '+json.dumps(marker)+' yanıtını gönder\nGET "/slow" adresine istek geldiğinde\n    "entered" yaz\n    3 saniye bekle\n    '+json.dumps('slow-'+marker)+' yanıtını gönder\nGET "/csrf" adresine istek geldiğinde\n    csrf belirteci yanıtını gönder\nPOST "/login" adresine istek geldiğinde\n    herkese açık\n    "owned" kullanıcısını "test" rolüyle oturuma al\n    "logged" yanıtını gönder\nGET "/panel" adresine istek geldiğinde\n    oturum gerekli\n    "panel" yanıtını gönder\nPOST "/logout" adresine istek geldiğinde\n    oturum gerekli\n    oturumu kapat\n    "revoked" yanıtını gönder\nGET "/limited" adresine istek geldiğinde\n    "limited" yanıtını gönder\n'
        raw=program.encode();target=self.project/'ana.dil';temporary=self.project/'ana.next';temporary.write_bytes(raw);temporary.chmod(0o600);temporary.replace(target)
        (self.out/('revision-'+str(number)+'.source')).write_bytes(raw)
        row={'application_revision':revision,'response_marker':marker,'source_sha256':snapshot(target),'manifest_sha256':snapshot(self.project/'proje.dil')};self.revisions.append(row)
        if self.host is not None:self.host.application_hashes={name:snapshot(self.project/name)for name in ('ana.dil','proje.dil')};self.host.revision=revision
        return revision,marker
    def spawn(self):
        self.check();g=self.host.spawn();self.gens.append(g);self.host.ready(g);return g
    def entered(self,g):
        os.set_blocking(g['child'].stdout.fileno(),False);raw=bytearray();end=time.monotonic()+5
        with selectors.DefaultSelector()as selector:
            selector.register(g['child'].stdout,selectors.EVENT_READ)
            while b'entered\n'not in raw:
                self.check();need(time.monotonic()<end)
                for key,_ in selector.select(.05):
                    part=os.read(key.fd,4096);need(part and len(raw)+len(part)<=4096);raw.extend(part)
    def legacy_deadline(self,path,output):
        # Private module namespace only: legacy source remains byte-identical and its
        # own Supervisor/finally retains native process ownership. No global monkeypatch.
        self.check();need(self.end-time.monotonic()>=25);original_argv=sys.argv;original_import=builtins.__import__;campaign=self
        class Queries:
            def check_output(self,argv,**options):
                campaign.check();need(not options and [str(v)for v in argv]==[str(campaign.binary),'sürüm','--json'])
                return campaign.H.run(argv,timeout=min(5,campaign.end-time.monotonic()),limit=8192)
            def __getattr__(self,name):return getattr(subprocess,name)
        proxy=Queries()
        class Timings:
            def monotonic(self):campaign.check();return time.monotonic()
            def sleep(self,seconds):
                end=time.monotonic()+seconds
                while time.monotonic()<end:
                    campaign.check();time.sleep(min(.02,max(0,end-time.monotonic())))
            def __getattr__(self,name):return getattr(time,name)
        timings=Timings()
        def private_import(name,globals=None,locals=None,fromlist=(),level=0):
            if name=='subprocess'and level==0:return proxy
            if name=='time'and level==0:return timings
            return original_import(name,globals,locals,fromlist,level)
        namespace={'__name__':'__main__','__file__':str(path),'__builtins__':dict(vars(builtins),__import__=private_import)}
        before=snapshot(path,262144);fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
        try:
            need(stat.S_ISREG(os.fstat(fd).st_mode));raw=bytearray()
            while len(raw)<=262144:
                part=os.read(fd,min(65536,262145-len(raw)))
                if not part:break
                raw.extend(part)
            need(len(raw)<=262144 and hashlib.sha256(raw).hexdigest()==before);raw=bytes(raw)
        finally:os.close(fd)
        try:
            sys.argv=[str(path),'--framework-root',str(self.repo),'--dil',str(self.binary),'--output',str(output),'--expect','rejected']
            exec(compile(raw,str(path),'exec'),namespace)
        finally:sys.argv=original_argv
        need(snapshot(path,262144)==before)
    def main(self):
        self.check();build=json.loads(self.H.run([self.binary,'sürüm','--json'],timeout=min(10,self.end-time.monotonic()),limit=8192));need(build['source_sha']==self.a.sha and build['dirty']is False);self.proof['build']=build
        module=load('mixed_listener_host',self.repo/'scripts/native-listener-host.py');self.project=self.out/'project';self.project.mkdir(mode=0o700)
        (self.project/'proje.dil').write_text('proje "mixed-owned-fixture" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu", "dosya-yazma", "web-oturumu" listesi olsun\n')
        revision,marker=self.publish(0);self.host=module.Host(self.binary,self.project,build,revision,self.a.binary_sha256);self.foreign=subprocess.Popen([sys.executable,'-c','import time;time.sleep(300)'],start_new_session=True)
        listener=os.fstat(self.host.listener.fileno());g=self.spawn();self.host.activate(g);need(self.request(self.host.port)[1]==marker.encode())
        status,csrf,guest=self.request(self.host.port,'/csrf');need(status==200 and len(csrf)==64 and guest);status,_,cookie=self.request(self.host.port,'/login',method='POST',cookie=guest,csrf=csrf.decode());need(status==200 and cookie)
        state=self.project/'.zee/web-durumu-v1.json';need(state.exists());self.proof['shared_state_path']='project/.zee/web-durumu-v1.json'
        outcomes=[];failures={};expected_bodies=[];total=0;counter_lock=threading.Lock()
        partial={'requests':0,'status200':0,'status5xx':0,'failures':{},'body_mismatches':0,'retries':0,'incomplete':True};self.proof['bounded_load']=partial
        for wave in range(1,5):
            oldmarker=marker;revision,marker=self.publish(wave);new=self.spawn()
            with concurrent.futures.ThreadPoolExecutor(max_workers=3)as pool:
                admitted=pool.submit(http,self.host.port,'/slow','127.3.0.'+str(wave));self.entered(g)
                queued=[threading.Event(),threading.Event()]
                def load_client(client):
                    rows=[]
                    for index in range(20):
                        self.check();started=time.monotonic()
                        try:
                            answer=self.request(self.host.port,client='127.'+str(10+wave)+'.'+str(client)+'.'+str(index+1),on_sent=queued[client-1].set if index==0 else None)
                            rows.append((answer[0],answer[1],None,started,time.monotonic()))
                        except ClientFailure as e:rows.append((None,None,e.kind,started,time.monotonic()))
                        status,body,error,_,_=rows[-1]
                        with counter_lock:
                            partial['requests']+=1;partial['status200']+=int(status==200);partial['status5xx']+=int(status is not None and status>=500)
                            if error:partial['failures'][error]=partial['failures'].get(error,0)+1
                            else:partial['body_mismatches']+=int(body!=marker.encode())
                        time.sleep(.01)
                    return rows
                futures=[pool.submit(load_client,n)for n in (1,2)]
                need(all(event.wait(1)for event in queued));activated=time.monotonic();self.host.activate(new)
                need(self.request(self.host.port,client='127.4.0.'+str(wave))[1]==marker.encode());need(not admitted.done())
                term=time.monotonic();os.kill(g['child'].pid,signal.SIGTERM)
                slow=admitted.result(timeout=5);need(slow[0]==200 and slow[1]==('slow-'+oldmarker).encode());self.host.stop(g);g=new
                completed_after_term=0
                for f in futures:
                    for status,body,error,started,ended in f.result(timeout=105):
                        total+=1;outcomes.append(status);completed_after_term+=int(ended>=term)
                        if error:failures[error]=failures.get(error,0)+1
                        else:expected_bodies.append(body==marker.encode())
                self.proof.setdefault('wave_ingress',[]).append({'wave':wave,'queued_before_ACTIVATE':2,'responses_completed_after_old_TERM':completed_after_term,'requests':40,'load_clients':2,'additional_old_admitted_request':1,'additional_new_revision_probe':1})
            need(self.request(self.host.port,'/panel',cookie=cookie)[0]==200);need(os.fstat(self.host.listener.fileno()).st_ino==listener.st_ino and self.H.held_status(self.foreign)is None)
            self.cases.append({'name':'mixed-code-wave-'+str(wave),'passed':True,'old_inflight_complete_after_TERM':True,'old_marker':oldmarker,'new_marker':marker,'new_generation_session200':True,'listener_inode_unchanged':True})
        self.proof['bounded_load']={'requests':total,'concurrency':2,'additional_old_admitted_request':1,'additional_revision_probe':1,'rollovers':4,'status200':outcomes.count(200),'status5xx':sum(1 for x in outcomes if x is not None and x>=500),'failures':failures,'retries':0,'quota_profile':'distinct synthetic loopback client identities; no state reset or quota increase','max_request_seconds':5,'body_mismatches':sum(not x for x in expected_bodies)}
        need(total==160 and outcomes==[200]*160 and all(expected_bodies)and not failures)
        status,csrf,_=self.request(self.host.port,'/csrf',cookie=cookie);need(status==200);need(self.request(self.host.port,'/logout',method='POST',cookie=cookie,csrf=csrf.decode())[0]==200);need(self.request(self.host.port,'/panel',cookie=cookie)[0]==401)
        # Another real revision must retain the revoked session and exhausted shared quota.
        for _ in range(200):
            self.check();status,_,_=self.request(self.host.port,'/limited',client='127.9.9.9')
            if status==429:break
            need(status==200)
        need(status==429);self.publish(5);new=self.spawn();self.host.activate(new);self.host.stop(g);g=new
        need(self.request(self.host.port,'/panel',cookie=cookie)[0]==401 and self.request(self.host.port,'/limited',client='127.9.9.9')[0]==429);self.cases.append({'name':'mixed-revision-shared-revoke-and-quota','passed':True,'revoked401':True,'quota429':True,'state_reset':False})
        # Wrong expected app revision cannot borrow another generation's receipt.
        actual=self.host.revision;self.host.revision='f'*40;bad=self.host.spawn();self.gens.append(bad)
        try:self.host.ready(bad)
        except module.OWNER.Reject:pass
        else:raise Rejected('wrong revision admitted')
        self.host.stop(bad);self.host.revision=actual;need(self.request(self.host.port)[1]==b'revision-5');self.cases.append({'name':'wrong-app-revision-no-admission-old-still-usable','passed':True})
        bad=self.spawn();self.host.activate(bad,b'ACTIVATE '+b'0'*32+b'\n');need(self.host.await_exit(bad,5)==0);self.host.stop(bad);need(self.request(self.host.port)[1]==b'revision-5');self.cases.append({'name':'wrong-activation-no-admission-old-still-usable','passed':True})
        before=(self.project/'ana.dil').read_bytes();invalid=b'K291_PRIVATE_SOURCE_CANARY !!!\n';(self.project/'ana.dil').write_bytes(invalid);self.host.application_hashes['ana.dil']=snapshot(self.project/'ana.dil');bad=self.host.spawn();self.gens.append(bad)
        try:self.host.ready(bad)
        except (module.OWNER.Reject,AssertionError):pass
        else:raise Rejected('invalid source admitted')
        need(snapshot(self.project/'ana.dil')==hashlib.sha256(invalid).hexdigest());(self.project/'ana.dil').write_bytes(before);self.host.application_hashes['ana.dil']=snapshot(self.project/'ana.dil');need(self.request(self.host.port)[1]==b'revision-5');self.cases.append({'name':'new-invalid-source-never-replaces-active-revision','passed':True,'private_diagnostic_forwarded':False,'boundary':'host stderr is DEVNULL; existing quietFD gate remains separate'})
        need(all(snapshot(self.project/n)==h for n,h in self.host.application_hashes.items()));self.proof['published_final_source_hashes']=dict(self.host.application_hashes)
        self.host.stop(g);self.host.cleanup();need(all(x['child'].returncode is not None for x in self.gens));self.proof['owned_native_generations_reaped']=True;self.host=None
        self.check();deadline=self.out/'deadline.json';self.legacy_deadline(self.repo/'scripts/tests/native-listener-deadline-test.py',deadline)
        value=json.loads(deadline.read_text());need(value['passed']and value['response_bytes']==0);self.cases.append({'name':'same-source-activation-deadline-single-generation-control','passed':True,'separate_from_mixed_load':True})
        self.H.clean_exact(self.repo,self.a.sha);need(snapshot(self.binary)==self.a.binary_sha256 and all(snapshot(self.repo/p,262144)==h for p,h in self.source_pins.items()));self.proof.update(status='cases-passed-pending-cleanup',source_pins_end_equal=True,binary_end_equal=True)
    def close(self):
        for n in self.oldhandlers:signal.signal(n,signal.SIG_IGN)
        self.H.CANCELLED=False;errors=[]
        try:
            if self.host is not None:
                for g in reversed(self.gens):
                    try:self.host.stop(g)
                    except BaseException as e:errors.append(type(e).__name__)
                try:self.host.cleanup()
                except BaseException as e:errors.append(type(e).__name__)
            if self.foreign is not None:
                try:self.H.stop_owned(self.foreign,grace=.2)
                except BaseException as e:errors.append(type(e).__name__)
            self.proof.update(status='passed'if self.proof['status']=='cases-passed-pending-cleanup'and not errors else 'failed',cleanup_errors=errors,foreign_owned_sentinel_reaped=self.foreign is None or self.foreign.returncode is not None)
            (self.out/'proof.json').write_text(json.dumps(self.proof,indent=2)+'\n')
        finally:
            for n,h in self.oldhandlers.items():signal.signal(n,h)

class Parser(argparse.ArgumentParser):
    def error(self,message):raise Rejected('arguments')

def main():
    p=Parser()
    for name in ('repo','dil','out'):p.add_argument('--'+name,type=Path,required=True)
    for name in ('sha','binary-sha256'):p.add_argument('--'+name,required=True)
    a=p.parse_args();run=Run(a)
    try:run.main()
    finally:run.close()
    need(run.proof['status']=='passed')
if __name__=='__main__':
    try:main()
    except (Exception,KeyboardInterrupt):sys.stderr.write('owned-mixed-revision-campaign-failed\n');raise SystemExit(2)
