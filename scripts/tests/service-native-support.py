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
        self.cleaning=False;self.cleanup_deadline=None;self.output_created=False;self.cleanup_owner=None;self.failed_body=False
        self.need(sys.platform=='linux' and os.geteuid()!=0 and re.fullmatch('[0-9a-f]{40}',a.sha) and re.fullmatch('[0-9a-f]{64}',a.binary_sha256))
        self.repo=D.T.canonical(a.repo);self.binary=D.T.canonical(a.dil);self.out=D.T.canonical(a.out)
        D.T.private_directory(self.out.parent.stat());self.need(not self.out.exists() and self.repo not in self.out.parents and self.out not in self.repo.parents)
        self.proof={'schema':'zee-owned-continuous-service-native-1','role':role,'source_sha':a.sha,'binary_sha256':a.binary_sha256,'cases':self.cases,'status':'running','boundaries':['owned Linux user-manager and synthetic local PG/application only','no real product adoption/provider exactly-once/physical-host acceptance']}
        try:
            for n in (signal.SIGTERM,signal.SIGINT):self.previous[n]=signal.signal(n,self.cancel)
            self.need(True);self.out.mkdir(mode=0o700);self.output_created=True
            # Same pinned owner code, separate cancellation state only for already-issued cleanup.
            # No process is created by loading it, and no native ownership gap is repaired here.
            helper=bounded_bytes(HERE/'lifecycle_core.py');self.need(hashlib.sha256(helper).hexdigest()==D.T.HELPER)
            self.cleanup_owner=importlib.util.module_from_spec(importlib.util.spec_from_loader('service_campaign_cleanup',loader=None))
            exec(compile(helper,str(HERE/'lifecycle_core.py'),'exec'),self.cleanup_owner.__dict__)
            D.T.D.clean_exact(self.repo,a.sha);self.need(D.snapshot(self.binary)==a.binary_sha256)
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
        cleaning=getattr(self,'cleaning',False)
        deadline=self.cleanup_deadline if cleaning else self.deadline
        if not ok or (not cleaning and (self.stopped or D.T.D.CANCELLED)) or time.monotonic()>=deadline:raise ValueError('owned campaign rejected')
    def command(self,label,argv,budget=25,env=None):
        self.need(True);self.command_sequence+=1;name=f'{self.command_sequence:04d}-{label}.log';log=self.out/name
        row={'sequence':self.command_sequence,'role':label,'log':name,'status':'failed'};self.commands.append(row)
        try:
            cleaning=getattr(self,'cleaning',False);owner=self.cleanup_owner if cleaning else D.T.D
            deadline=self.cleanup_deadline if cleaning else self.deadline
            raw=owner.run(argv,cwd=self.out,timeout=min(budget,max(.001,deadline-time.monotonic())),limit=262144,env=D.T.D.controlled_env() if env is None else env,log_path=log)
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
            p=row['path'];st=p.lstat();self.need((st.st_dev,st.st_ino)==row['identity'] and p.is_symlink() and p.resolve()==row['target'] and D.snapshot(row['target'],65536)==row['sha256'])
            self.need(p.name in self.units)
            state=self.system('state-'+p.name,'show',p.name,'--property=ActiveState','--value').strip()
            self.need(state in (b'inactive',b'failed'))
            if state==b'failed':self.system('reset-'+p.name,'reset-failed',p.name)
            self.need(self.system('inactive-'+p.name,'show',p.name,'--property=ActiveState','--value').strip()==b'inactive')
            st=p.lstat();self.need((st.st_dev,st.st_ino)==row['identity'] and p.is_symlink() and p.resolve()==row['target'] and D.snapshot(row['target'],65536)==row['sha256']);p.unlink()
        names=list(self.units);self.links.clear();self.units.clear();self.system('reload','daemon-reload')
        for name in names:self.need(self.system('released-'+name,'show',name,'--property=LoadState','--value').strip()==b'not-found')
    def finish(self):
        D.T.D.clean_exact(self.repo,self.a.sha);self.need(D.snapshot(self.binary)==self.a.binary_sha256 and all(D.snapshot(self.repo/n,262144)==v for n,v in self.sources.items()))
        self.need(all(D.snapshot(Path(n))==h for n,h in self.pg_pins.items()));self.proof['postgres_binary_pins_end_equal']=True
        self.proof['source_pins_end_equal']=True;self.proof['binary_end_equal']=True;self.finished=True
    def close(self):
        # Cleanup has a separate bounded budget; cancellation remains a sticky failed outcome.
        expired=time.monotonic()>=self.deadline
        previously_failed=self.proof.get('status')=='failed'
        self.stopped=self.stopped or D.T.D.CANCELLED
        self.cleaning=True;self.cleanup_deadline=time.monotonic()+300
        errors=[]
        def update_result():
            self.stopped=self.stopped or D.T.D.CANCELLED
            failed_body=getattr(self,'failed_body',False)
            self.proof.update(status='passed' if self.finished and not failed_body and not previously_failed and not errors and not self.stopped and not expired and time.monotonic()<self.cleanup_deadline else 'failed',cleanup_errors=errors,owned_units_released=not self.units,cancelled=self.stopped,failed_body=failed_body)
        def publish():
            update_result()
            if getattr(self,'output_created',False):(self.out/'proof.json').write_text(json.dumps(self.proof,indent=2)+'\n')
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
            publish()
            if self.proof['status']=='passed' and (self.stopped or D.T.D.CANCELLED or time.monotonic()>=self.cleanup_deadline):publish()
        except BaseException as error:
            errors.append({'resource':'cleanup-publication','class':type(error).__name__});publish();raise
        finally:
            self.cleaning=False
            restore_error=None
            for n,h in self.previous.items():
                try:signal.signal(n,h)
                except BaseException as error:
                    restore_error=restore_error or error;errors.append({'resource':'handler-restoration','class':type(error).__name__})
            if restore_error is not None:publish();raise restore_error
            if self.proof['status']=='passed' and (self.stopped or D.T.D.CANCELLED or time.monotonic()>=self.cleanup_deadline):publish()
    def __enter__(self):return self
    def __exit__(self,kind,value,traceback):
        if kind is not None:self.failed_body=True
        self.close()
        if kind is None:
            if self.proof['status']!='passed' or self.stopped or D.T.D.CANCELLED or time.monotonic()>=self.cleanup_deadline:
                if self.proof['status']=='passed':
                    self.proof.update(status='failed',cancelled=self.stopped or D.T.D.CANCELLED)
                    if getattr(self,'output_created',False):(self.out/'proof.json').write_text(json.dumps(self.proof,indent=2)+'\n')
                raise ValueError('owned cleanup failed')

class OwnedLocalCluster:
    """Linux service fixtures only; retain the actual foreground PostgreSQL child."""
    LOG_LIMIT = 8 * 1024 * 1024
    READY_LIMIT = 64 * 1024

    def __init__(self, campaign, directory, cert, key, binaries):
        self.c = campaign
        self.directory, self.cert, self.key = Path(directory), Path(cert), Path(key)
        self.binaries = Path(binaries)
        self.data = self.directory / 'data'
        self.protocol = 'TLSv1.2'
        self.child = None
        self.stream = None
        self.closed = False
        self.started = False
        self.failed = False
        self.probes = []
        self.close_errors = []
        self.row = {'schema': 'zee-owned-foreground-postgresql-1', 'status': 'not-started',
                    'readiness': [], 'fallback_used': False, 'normal_shutdown': False}

    @staticmethod
    def require(value, reason):
        if not value:
            raise ValueError(reason)

    def active(self):
        self.c.need(True)
        self.require(not self.c.stopped and not D.T.D.CANCELLED and not getattr(self.c, 'cleaning', False),
                     'PostgreSQL fixture cancelled')
        if self.child is not None:
            self.require(D.T.D.held_status(self.child) is None, 'PostgreSQL fixture exited early')
            self.log_size()

    def log_size(self):
        opened, named = os.fstat(self.stream.fileno()), self.log.lstat()
        self.require(stat.S_ISREG(opened.st_mode) and stat.S_ISREG(named.st_mode)
                     and (opened.st_dev, opened.st_ino) == (named.st_dev, named.st_ino)
                     and opened.st_size <= self.LOG_LIMIT, 'PostgreSQL server log identity/budget')
        return opened.st_size

    def remaining_cleanup(self):
        deadline = getattr(self.c, 'cleanup_deadline', None)
        if deadline is None:
            deadline = self.c.deadline
        return max(0.0, deadline - time.monotonic())

    def readiness_command(self, attempt, deadline):
        self.active()
        log = self.c.out / f'postgres-ready-{attempt:03d}.log'
        argv = self.psql + ['--no-psqlrc', '--host', str(self.directory), '--port', str(self.port),
                           '--username', 'postgres', '--dbname', 'postgres', '--tuples-only',
                           '--no-align', '--set', 'ON_ERROR_STOP=1', '--command',
                           'SHOW data_directory; SHOW server_version;']
        row = {'argv': argv, 'log': log.name, 'status': 'failed', 'timeout_seconds': 3}
        self.row['readiness'].append(row)
        retained = {'child': None, 'row': row}
        self.probes.append(retained)
        child = None
        terminal = None
        start = time.monotonic()
        end = min(deadline, start + 3, self.c.deadline)
        row['timeout_seconds'] = max(0, end - start)
        try:
            with log.open('xb') as stream:
                log.chmod(0o600)
                self.active()
                self.require(time.monotonic() < end, 'PostgreSQL readiness command deadline')
                retained['child'] = subprocess.Popen(argv, cwd=self.c.out, env=self.env,
                                                    stdin=subprocess.DEVNULL, stdout=stream,
                                                    stderr=subprocess.STDOUT, start_new_session=True, close_fds=True)
                child = retained['child']
                try:
                    while True:
                        self.active()
                        self.require(time.monotonic() < end and os.fstat(stream.fileno()).st_size <= self.READY_LIMIT,
                                     'PostgreSQL readiness command budget')
                        terminal = D.T.D.held_status(child)
                        if terminal is not None:
                            break
                        time.sleep(min(.05, max(0, end - time.monotonic())))
                    row['observed_exit'] = terminal
                    self.require(terminal in (0, 2), 'PostgreSQL readiness command exit')
                finally:
                    # No cancellation/deadline predicate may skip this original handle.
                    if child.returncode is None:
                        grace = min(.2 if terminal in (0, 2) else 3,
                                    max(0, end - time.monotonic()), self.remaining_cleanup())
                        D.T.D.stop_owned(child, grace=grace)
                    row.update(exit=child.returncode, reaped=child.returncode is not None)
            self.active()
            self.require(time.monotonic() < end and child.returncode == terminal,
                         'PostgreSQL readiness cancelled/deadline after reap')
            raw = bounded_bytes(log, self.READY_LIMIT)
            row.update(status='passed' if terminal == 0 else 'readiness-retry',
                       sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))
            return terminal, raw
        finally:
            # A failed first cleanup retains the same child for one bounded retry.
            if child is not None and child.returncode is None:
                D.T.D.stop_owned(child, grace=min(.2, self.remaining_cleanup()))
                row.update(exit=child.returncode, reaped=child.returncode is not None)
            row['seconds'] = time.monotonic() - start
            if log.exists():
                raw = bounded_bytes(log, self.READY_LIMIT)
                row.update(sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))

    def start(self):
        self.active()
        self.require(self in self.c.clusters and self.child is None and not self.closed
                     and not self.started and self.row['status'] == 'not-started', 'PostgreSQL registration')
        self.c.proof.setdefault('postgres_clusters', []).append(self.row)
        self.row['status'] = 'starting'
        try:
            self.require(self.directory.resolve(strict=True) == self.directory
                         and any(path == self.directory and identity == (self.directory.stat().st_dev, self.directory.stat().st_ino)
                                 for path, identity in self.c.tempdirs), 'registered PostgreSQL private directory')
            D.T.private_directory(self.directory.stat())
            self.require(self.cert.parent == self.directory and self.key.parent == self.directory
                         and self.binaries.resolve(strict=True) == self.binaries, 'PostgreSQL fixture paths')
            self.require(all(stat.S_ISREG(path.lstat().st_mode) for path in (self.cert, self.key)),
                         'PostgreSQL certificate/key fixture shape')
            for name in ('initdb', 'postgres', 'psql'):
                path = self.binaries / name
                self.require(D.snapshot(path) == self.c.pg_pins[str(path)], 'PostgreSQL runtime pin')
            self.env = {k: v for k, v in D.T.D.controlled_env().items() if not k.startswith('PG')}
            self.env['LC_ALL'] = 'C'
            self.c.command('owned-PG-initdb', [self.binaries / 'initdb', '--no-locale', '--encoding=UTF8',
                           '--auth=trust', '--username=postgres', '--pgdata', self.data], 60, self.env)
            self.active()
            self.version = self.c.command('owned-PG-version', [self.binaries / 'postgres', '--version'], 15, self.env).decode().strip()
            self.require(self.version == 'postgres (PostgreSQL) 16.15', 'PostgreSQL fixture version')
            self.psql = [str(self.binaries / 'psql')]
            import socket
            with socket.socket() as listener:
                listener.bind(('127.0.0.1', 0))
                self.port = listener.getsockname()[1]
            self.psql_port = self.port
            self.identity = {'binaries': str(self.binaries), 'platform': sys.platform, 'ownership': 'original-foreground-Popen'}
            config = (f"listen_addresses='127.0.0.1'\nport={self.port}\n"
                      f"unix_socket_directories='{self.directory}'\nssl=on\n"
                      f"ssl_min_protocol_version='{self.protocol}'\nssl_max_protocol_version='{self.protocol}'\n"
                      f"ssl_cert_file='{self.cert}'\nssl_key_file='{self.key}'\n"
                      f"hba_file='{self.directory / 'pg_hba.conf'}'\nlogging_collector=off\n")
            (self.data / 'postgresql.conf').write_text(config)
            self.log = self.c.out / 'postgres-server.log'
            argv = [str(self.binaries / 'postgres'), '-D', str(self.data)]
            self.row.update(argv=argv, log=self.log.name, data_directory=str(self.data),
                            socket_directory=str(self.directory), port=self.port, version=self.version,
                            log_limit_bytes=self.LOG_LIMIT, log_limit_semantics='observed-during-start-readiness-and-close',
                            postgres_sha256=self.c.pg_pins[str(self.binaries / 'postgres')])
            self.stream = self.log.open('xb')
            self.log.chmod(0o600)
            self.active()
            self.child = subprocess.Popen(argv, cwd=self.c.out, env=self.env, stdin=subprocess.DEVNULL,
                                          stdout=self.stream, stderr=subprocess.STDOUT,
                                          start_new_session=True, close_fds=True)
            self.row['pid'] = self.child.pid
            end = min(time.monotonic() + 30, self.c.deadline)
            attempt = 0
            while time.monotonic() < end:
                self.active()
                attempt += 1
                code, raw = self.readiness_command(attempt, end)
                if code == 0:
                    self.require(raw.decode().strip().splitlines() == [str(self.data), '16.15'],
                                 'PostgreSQL data-directory/version identity')
                    self.active()
                    self.require(time.monotonic() < end, 'PostgreSQL terminal readiness deadline')
                    self.started = True
                    self.row.update(status='ready', readiness_identity_verified=True)
                    return
                time.sleep(min(.1, max(0, end - time.monotonic())))
            raise ValueError('PostgreSQL readiness deadline')
        except BaseException as error:
            self.failed = True
            self.row.update(status='failed', failure_type=type(error).__name__)
            raise

    def close(self):
        if self.closed:
            self.require(not self.close_errors, 'previous PostgreSQL cleanup failed')
            return
        errors = self.close_errors
        terminal = None
        end = time.monotonic() + min(30, self.remaining_cleanup())
        self.row['status'] = 'closing'
        try:
            for retained in reversed(self.probes):
                child = retained['child']
                if child is not None and child.returncode is None:
                    try:
                        D.T.D.stop_owned(child, grace=min(.2, self.remaining_cleanup()))
                        retained['row'].update(exit=child.returncode, reaped=child.returncode is not None)
                    except BaseException as error:
                        errors.append('owned PostgreSQL readiness cleanup: ' + type(error).__name__)
            if self.child is not None:
                terminal = (D.T.D.held_status(self.child) if self.child.returncode is None
                            else self.row.get('observed_exit'))
                if terminal is None and time.monotonic() < end:
                    os.kill(self.child.pid, signal.SIGINT)
                    self.row['SIGINT_requested'] = True
                    while terminal is None and time.monotonic() < end:
                        self.log_size()
                        time.sleep(min(.05, max(0, end - time.monotonic())))
                        terminal = D.T.D.held_status(self.child)
                self.row['observed_exit'] = terminal
                if terminal != 0 or not self.row.get('SIGINT_requested') or time.monotonic() >= end:
                    self.row['fallback_used'] = True
                    errors.append('non-normal PostgreSQL terminal status')
        except BaseException as error:
            errors.append(type(error).__name__)
        finally:
            try:
                if self.child is not None:
                    if self.child.returncode is None:
                        if terminal != 0:
                            self.row['fallback_used'] = True
                        # No fresh shutdown budget, and no PID-derived adoption.
                        D.T.D.stop_owned(self.child, grace=min(.2, max(0, end - time.monotonic()), self.remaining_cleanup()))
                    self.row.update(exit=self.child.returncode, reaped=self.child.returncode is not None)
            except BaseException as error:
                errors.append('owned PostgreSQL cleanup: ' + type(error).__name__)
            finally:
                for retained in reversed(self.probes):
                    child = retained['child']
                    if child is not None and child.returncode is None:
                        try:
                            D.T.D.stop_owned(child, grace=min(.2, self.remaining_cleanup()))
                            retained['row'].update(exit=child.returncode, reaped=child.returncode is not None)
                        except BaseException as error:
                            errors.append('final owned readiness cleanup: ' + type(error).__name__)
                try:
                    if self.stream is not None:
                        self.stream.close()
                except BaseException as error:
                    errors.append('PostgreSQL log close: ' + type(error).__name__)
            self.closed = ((self.child is None or self.child.returncode is not None)
                           and all(p['child'] is None or p['child'].returncode is not None for p in self.probes)
                           and (self.stream is None or self.stream.closed))
            if hasattr(self, 'log') and self.log.exists() and self.closed:
                try:
                    raw = bounded_bytes(self.log, self.LOG_LIMIT)
                    self.row.update(log_sha256=hashlib.sha256(raw).hexdigest(), log_bytes=len(raw))
                    self.row['normal_shutdown'] = (self.child is not None and terminal == 0
                        and self.child.returncode == 0 and self.row.get('SIGINT_requested') is True
                        and not self.row['fallback_used'] and not errors
                        and b'database system is shut down' in raw)
                except BaseException as error:
                    errors.append('PostgreSQL log verification: ' + type(error).__name__)
            if self.child is not None and not self.row['normal_shutdown'] and not errors:
                errors.append('normal PostgreSQL shutdown not observed')
            self.row.update(cleanup_errors=errors, cancelled=self.c.stopped or D.T.D.CANCELLED,
                            owned_child_cleanup_complete=self.closed)
            self.row['status'] = ('closed' if self.closed and not errors and not self.failed
                                 and not self.row['cancelled'] else 'failed')
        self.require(not errors and self.closed, 'PostgreSQL owned cleanup failed')
