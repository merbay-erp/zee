"""K282/K283 dedicated Linux campaign ownership; never install or enable services."""
import hashlib,importlib.util,json,os,re,selectors,shutil,signal,stat,subprocess,sys,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('service_driver_dispatch',HERE/'dispatcher-service.py');D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D)


COMMON_SOURCES=('scripts/dispatcher-service.py','scripts/finite_timer.py','scripts/lifecycle_core.py','scripts/worker-supervisor.py','scripts/credential-baslat.py','scripts/native-capabilities.py','scripts/native-metrics.py','scripts/postgresql-tls-ci.py','scripts/postgresql-tls11-control.py','scripts/tests/service-native-support.py')
QUEUE_FILES=('ana.dil','proje.dil','gocler/0001_kuyruk.sql','gocler/0002_kira_sahibi.sql','gocler/0003_sabit_niyet.sql')
WORKER_FILES=('ana.dil','proje.dil')
PROBE_FILES=('proje.dil','kaynak/hazirlik.dil','gocler/0001_hazirlik.sql')
DISPATCH_SOURCES=('scripts/job-dispatcher.py','scripts/is-kuyrugu-ci.py','scripts/parola-goc-ci.py','scripts/postgresql-commit-proxy.py','scripts/tests/dispatcher-service-native.py','dogfood/zee-cekirdek/is-kuyrugu/kaynak/kuyruk.dil',*('dogfood/zee-cekirdek/is-kuyrugu-dispatcher/'+n for n in QUEUE_FILES))
HTTP_SOURCES=('scripts/http-service.py','scripts/db-hazirlik.py','scripts/tests/http-service-native.py',*('compiler/tests/fixtures/db-readiness/worker/'+n for n in WORKER_FILES),*('compiler/tests/fixtures/db-readiness/probe/'+n for n in PROBE_FILES))

def bounded_bytes(path,limit=262144):
    before=path.lstat()
    if path.resolve()!=path or not stat.S_ISREG(before.st_mode) or before.st_size>limit:raise ValueError('fixed consumed file')
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
    try:
        key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
        opened=os.fstat(fd)
        if key(before)!=key(opened):raise ValueError('fixed consumed identity')
        raw=bytearray()
        while len(raw)<=limit:
            part=os.read(fd,min(65536,limit+1-len(raw)))
            if not part:break
            raw.extend(part)
        if len(raw)>limit or len(raw)!=opened.st_size or key(opened)!=key(os.fstat(fd)) or key(opened)!=key(path.lstat()):raise ValueError('fixed consumed bytes')
        return bytes(raw)
    finally:os.close(fd)

class OwnedSqlBlocker:
    """One issued interactive psql leader; WNOWAIT authority retained through group cleanup."""
    def __init__(self,campaign,argv,sql):self.c=campaign;self.argv=argv;self.sql=sql;self.child=None;self.closed=False;self.registered=False
    def __enter__(self):
        self.c.need(len(self.sql)<4096 and not self.registered);self.c.resources.append(self);self.registered=True
        self.child=subprocess.Popen(self.argv,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True,close_fds=True,env=D.T.D.controlled_env())
        try:
            os.set_blocking(self.child.stdin.fileno(),False);raw=('BEGIN;\n'+self.sql+";\nSELECT 'BARRIER_READY';\n").encode();self.c.need(os.write(self.child.stdin.fileno(),raw)==len(raw))
            end=min(self.c.deadline,time.monotonic()+20);captured=bytearray()
            with selectors.DefaultSelector()as selector:
                selector.register(self.child.stdout,selectors.EVENT_READ)
                while b'BARRIER_READY'not in captured:
                    self.c.need(time.monotonic()<end)
                    for key,_ in selector.select(min(.05,max(0,end-time.monotonic()))):
                        part=os.read(key.fileobj.fileno(),4096);self.c.need(part and len(captured)+len(part)<=8192);captured.extend(part)
            return self
        except BaseException:self.__exit__(None,None,None);raise
    def close(self):
        if self.closed:return
        if self.child is not None:
            D.T.D.stop_owned(self.child,grace=.2)
            self.child.stdin.close();self.child.stdout.close()
        self.closed=True
    def __exit__(self,*exc):self.close()

class Campaign:
    def __init__(self,a,role):
        self.tempdirs=[];self.commands=[];self.command_sequence=0;self.stopped=False;self.deadline=time.monotonic()+1200;self.units={};self.links=[];self.clusters=[];self.resources=[];self.previous={};self.a=a;self.cases=[];self.finished=False
        self.need(sys.platform=='linux' and os.geteuid()!=0 and re.fullmatch('[0-9a-f]{40}',a.sha) and re.fullmatch('[0-9a-f]{64}',a.binary_sha256))
        self.repo=D.T.canonical(a.repo);self.binary=D.T.canonical(a.dil);self.out=D.T.canonical(a.out)
        D.T.private_directory(self.out.parent.stat());self.need(not self.out.exists() and self.repo not in self.out.parents and self.out not in self.repo.parents)
        D.T.D.clean_exact(self.repo,a.sha);self.need(D.snapshot(self.binary)==a.binary_sha256)
        self.out.mkdir(mode=0o700)
        self.proof={'schema':'zee-owned-continuous-service-native-1','role':role,'source_sha':a.sha,'binary_sha256':a.binary_sha256,'cases':self.cases,'status':'running','boundaries':['owned Linux user-manager and synthetic local PG/application only','no real product adoption/provider exactly-once/physical-host acceptance']}
        for n in (signal.SIGTERM,signal.SIGINT):self.previous[n]=signal.signal(n,self.cancel)
        D.T.D.CANCELLED=False
        try:
            self.pg_pins={str(a.postgres_bin_dir.resolve()/n):D.snapshot(a.postgres_bin_dir.resolve()/n) for n in ('postgres','initdb','pg_ctl','psql')}
            self.proof['postgres_binary_pins_start']=self.pg_pins
            self.sources={}
            self.pin_files(COMMON_SOURCES + (DISPATCH_SOURCES if role=='K282' else HTTP_SOURCES))
            self.proof['source_pins_start']=self.sources;self.proof['commands']=self.commands
            build=json.loads(self.command('build',[self.binary,'sürüm','--json'],15));self.need(build.get('schema')=='zee-build-v1' and build['source_sha']==a.sha and build['dirty']is False);self.proof['build']=build
            self.need(self.system('manager','is-system-running').strip()==b'running')
        except BaseException:self.close();raise
    def cancel(self,n,f):self.stopped=True;D.T.D.CANCELLED=True
    def need(self,ok):
        if not ok or self.stopped or time.monotonic()>=self.deadline:raise ValueError('owned campaign rejected')
    def command(self,label,argv,budget=25,env=None):
        self.need(True);self.command_sequence+=1;name=f'{self.command_sequence:04d}-{label}.log';log=self.out/name
        row={'sequence':self.command_sequence,'role':label,'log':name,'status':'failed'};self.commands.append(row)
        try:
            raw=D.T.D.run(argv,cwd=self.out,timeout=min(budget,max(.001,self.deadline-time.monotonic())),limit=262144,env=D.T.D.controlled_env() if env is None else env,log_path=log)
            self.need(len(raw)<=262144);row['status']='passed';return raw
        finally:
            if log.exists():row.update(bytes=log.stat().st_size,sha256=D.snapshot(log,262144))
    def temporary(self,prefix):
        directory=Path(tempfile.mkdtemp(prefix=prefix,dir="/tmp")).resolve();st=directory.stat();self.tempdirs.append((directory,(st.st_dev,st.st_ino)));return directory
    def borrowed_run(self,argv,**kwargs):
        budget=kwargs.pop('timeout',60);env=kwargs.pop('env',None);cwd=kwargs.pop('cwd',None)
        self.need(not kwargs and (cwd is None or Path(cwd)==self.out))
        return self.command('owned-PG-tool',argv,budget,env).decode()
    def pin_files(self,names):
        self.need(len(names)<=64 and len(set(names))==len(names))
        for name in names:
            raw=bounded_bytes(self.repo/name);self.sources[name]=hashlib.sha256(raw).hexdigest()
        tracked=self.command('tracked-consumed-files',['git','-C',self.repo,'ls-files','--error-unmatch','--',*names],30).decode().splitlines()
        self.need(set(tracked)==set(names))
    def fixture(self,name):
        self.need(name in self.sources);raw=bounded_bytes(self.repo/name);self.need(hashlib.sha256(raw).hexdigest()==self.sources[name]);return raw
    def copy_fixture(self,prefix,target,names):
        self.need(not target.exists());target.mkdir(mode=0o700)
        for name in names:
            destination=target/name;destination.parent.mkdir(mode=0o700,parents=True,exist_ok=True);destination.write_bytes(self.fixture(prefix+'/'+name));destination.chmod(0o600)

    def system(self,label,*args,budget=25):return self.command(label,['systemctl','--user',*args],budget)
    def wait(self,condition,seconds=45):
        end=time.monotonic()+seconds
        while True:
            self.need(True)
            if condition():return
            self.need(time.monotonic()<end);time.sleep(.1)
    def record(self,name,**fields):self.cases.append({'name':name,'passed':True,**fields})
    def jsonfile(self,path,value):path.write_text(json.dumps(value,separators=(',',':'))+'\n');path.chmod(0o600);return path
    def empty(self,name):
        raw=self.system('cgroup-'+name,'show',name,'--property=ControlGroup','--value').decode().strip()
        if not raw:return True
        self.need(raw.startswith('/user.slice/') and '..' not in Path(raw).parts);base=Path('/sys/fs/cgroup')/raw.lstrip('/')
        if not base.exists():return True
        # Cap total visited entries, not merely matched cgroup.procs files.
        pending=[base];count=0
        while pending:
            directory=pending.pop()
            with os.scandir(directory)as entries:
                for entry in entries:
                    count+=1;self.need(count<=256)
                    if entry.is_dir(follow_symlinks=False):pending.append(Path(entry.path))
                    elif entry.name=='cgroup.procs':
                        fd=os.open(entry.path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
                        try:data=os.read(fd,8193)
                        finally:os.close(fd)
                        self.need(len(data)<=8192)
                        if data.strip():return False
        return True
    def link(self,directory,names):
        runtime=Path(os.environ.get('XDG_RUNTIME_DIR',''));self.need(runtime==Path('/run/user')/str(os.geteuid()) and runtime.resolve()==runtime and runtime.stat().st_uid==os.geteuid())
        targetdir=runtime/'systemd/user';self.need(targetdir.parent.resolve()==targetdir.parent)
        for name in names:
            self.need(self.system('absent-'+name,'show',name,'--property=LoadState','--value').strip()==b'not-found');self.need(not (targetdir/name).exists() and not (targetdir/name).is_symlink())
        for name in names:
            source=directory/name;pin=D.snapshot(source,65536);link=targetdir/name
            try:self.system('link-'+name,'link','--runtime',str(source))
            finally:
                if link.is_symlink() and link.resolve()==source:
                    st=link.lstat();self.links.append({'path':link,'target':source,'identity':(st.st_dev,st.st_ino),'sha256':pin});self.units[name]=True
            self.need(any(x['path']==link for x in self.links))
    def start(self,name):self.need(name in self.units);self.system('start-'+name,'start',name,'--no-block')
    def stop(self,name):self.need(name in self.units);self.system('stop-'+name,'stop',name,budget=240);self.need(self.empty(name))
    def unlink(self):
        if not self.units and not self.links:return
        errors=[]
        for name in reversed(list(self.units)):
            try:self.stop(name)
            except BaseException as e:errors.append(type(e).__name__)
        self.need(not errors)
        for row in reversed(self.links):
            p=row['path'];st=p.lstat();self.need((st.st_dev,st.st_ino)==row['identity'] and p.is_symlink() and p.resolve()==row['target'] and D.snapshot(row['target'],65536)==row['sha256']);p.unlink()
        names=list(self.units);self.links.clear();self.units.clear();self.system('reload','daemon-reload')
        for name in names:self.need(self.system('released-'+name,'show',name,'--property=LoadState','--value').strip()==b'not-found')
    def finish(self):
        D.T.D.clean_exact(self.repo,self.a.sha);self.need(D.snapshot(self.binary)==self.a.binary_sha256 and all(D.snapshot(self.repo/n,262144)==v for n,v in self.sources.items()))
        self.need(all(D.snapshot(Path(n))==h for n,h in self.pg_pins.items()));self.proof['postgres_binary_pins_end_equal']=True
        self.proof['source_pins_end_equal']=True;self.proof['binary_end_equal']=True;self.finished=True
    def close(self):
        # Cancellation cannot interrupt bounded cleanup; only issued handles/names.
        self.stopped=False;self.deadline=time.monotonic()+300;D.T.D.CANCELLED=False
        for n in self.previous:signal.signal(n,signal.SIG_IGN)
        errors=[]
        try:
            try:self.unlink()
            except BaseException as e:errors.append({'resource':'owned-unit-links','class':type(e).__name__})
            for resource in reversed(self.clusters+self.resources):
                try:resource.close()
                except BaseException as e:errors.append({'resource':'owned-handle','class':type(e).__name__})
            if not errors:
                for temporary,identity in reversed(self.tempdirs):
                    try:
                        st=temporary.lstat();self.need(stat.S_ISDIR(st.st_mode) and (st.st_dev,st.st_ino)==identity);shutil.rmtree(temporary)
                    except BaseException as e:errors.append({'resource':'owned-temporary','class':type(e).__name__})
            self.proof['temporary_cleanup_complete']=not errors
            self.proof['commands']=self.commands
            self.proof.update(status='passed' if self.finished and not errors else 'failed',cleanup_errors=errors,owned_units_released=not self.units)
            (self.out/'proof.json').write_text(json.dumps(self.proof,indent=2)+'\n')
        finally:
            for n,h in self.previous.items():signal.signal(n,h)
    def __enter__(self):return self
    def __exit__(self,kind,value,traceback):
        self.close()
        if kind is None and self.proof['status']!='passed':raise ValueError('owned cleanup failed')
