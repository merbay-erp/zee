#!/usr/bin/env python3
"""K268/P09 preparation: one SQL-only tick, durable pause on every uncertain outcome."""
import argparse,contextlib,hashlib,importlib.util,json,os,re,signal,stat,sys,time
try:import fcntl
except ImportError:fcntl=None
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER='e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9'
D=None
try:
    raw=(HERE/'lifecycle_core.py').read_bytes()
    if hashlib.sha256(raw).hexdigest()==HELPER:
        spec=importlib.util.spec_from_file_location('timer_owner',HERE/'lifecycle_core.py');D=importlib.util.module_from_spec(spec);exec(compile(raw,str(HERE/'lifecycle_core.py'),'exec'),D.__dict__)
except Exception:D=None
class Paused(Exception):pass
class Busy(Exception):pass
class ClearVisible(Exception):
    def __init__(self,operation):self.outcome={"completion":"completion-visible-durability-unconfirmed","reconciliation":"reconciliation-visible-durability-unconfirmed"}[operation]
def report_clear(error):
    print(json.dumps({"schema":"zee-sql-timer-clear-publication-1","outcome":error.outcome},separators=(",",":")),file=sys.stderr)
class MaskedParser(argparse.ArgumentParser):
    def error(self,message):raise ValueError("arguments rejected")
    def exit(self,status=0,message=None):raise ValueError("arguments rejected")
def require(ok):
    if not ok:raise ValueError('fixed profile refused')
def canonical(path):
    p=Path(path);require(p.is_absolute() and str(p)==str(p.resolve()) and not any(c in str(p) for c in '\n\r\t\0'));return p
def private(info,mode):require(info.st_uid==os.geteuid() and stat.S_IMODE(info.st_mode)==mode and info.st_nlink==1 and stat.S_ISREG(info.st_mode))
def identity(info):
    return (info.st_dev,info.st_ino,info.st_mode,info.st_uid,info.st_gid,info.st_nlink,info.st_size,info.st_mtime_ns,info.st_ctime_ns)
def directory_identity(info):return (info.st_dev,info.st_ino,info.st_mode,info.st_uid,info.st_gid)
def snapshot_at(parent,name):
    before=os.stat(name,dir_fd=parent,follow_symlinks=False);private(before,0o600);require(before.st_size<=16384)
    fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC,dir_fd=parent)
    try:
        opened=os.fstat(fd);private(opened,0o600);require(identity(before)==identity(opened));chunks=[];size=0
        while size<=16384:
            try:block=os.read(fd,min(4096,16385-size))
            except InterruptedError:continue
            if not block:break
            size+=len(block);chunks.append(block)
        after=os.fstat(fd);current=os.stat(name,dir_fd=parent,follow_symlinks=False);private(current,0o600)
        require(identity(opened)==identity(after)==identity(current) and size==opened.st_size and size<=16384)
        return b''.join(chunks),identity(current)
    finally:os.close(fd)
def read_private(path):
    p=canonical(path);parent=os.open(p.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC)
    try:
        info=directory_identity(os.fstat(parent));raw,pin=snapshot_at(parent,p.name)
        require(info==directory_identity(os.lstat(p.parent)) and identity(os.lstat(p))==pin)
        return raw
    finally:os.close(parent)
def decode(raw):
    def pairs(rows):
        obj={}
        for k,v in rows:require(k not in obj);obj[k]=v
        return obj
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=lambda value:(_ for _ in ()).throw(ValueError("nonfinite JSON")))
def config(path):
    raw=read_private(path);data=decode(raw);keys={'schema','repo','source_sha','dil','binary_sha256','project','project_sha256','pg_port','ca','credential_root','credential_name','owner'};require(set(data)==keys and data['schema']=='zee-sql-timer-config-1')
    for k,n in [('source_sha',40),('binary_sha256',64),('project_sha256',64)]:require(re.fullmatch('[0-9a-f]{'+str(n)+'}',data[k]) is not None)
    for k in ('repo','dil','project','ca','credential_root'):data[k]=canonical(data[k]);require(data[k].exists())
    require(type(data['pg_port']) is int and 1<=data['pg_port']<=65535)
    require(re.fullmatch('[A-Za-z0-9_-]{1,64}',data['owner']) and re.fullmatch('[A-Za-z0-9_][A-Za-z0-9_.-]{0,63}',data['credential_name']))
    data['_config_path']=canonical(path);data['_config_sha256']=hashlib.sha256(raw).hexdigest()
    return data
def source_pin(c):
    if '_config_path' in c:require(hashlib.sha256(read_private(c['_config_path'])).hexdigest()==c['_config_sha256'])
    require(D.digest(HERE/'lifecycle_core.py')==HELPER)
    D.clean_exact(c['repo'],c['source_sha']);require(D.digest(c['dil'])==c['binary_sha256'])
    paths=('job-dispatcher.py','worker-supervisor.py','credential-baslat.py','native-metrics.py')
    result={p:D.digest(c['repo']/'scripts'/p) for p in paths}
    result.update({'external/finite_timer.py':D.digest(Path(__file__)),'external/lifecycle_core.py':D.digest(HERE/'lifecycle_core.py')})
    optional=c['repo']/'scripts/native-capabilities.py'
    if optional.exists():result['native-capabilities.py']=D.digest(optional)
    return result
def private_directory(info):require(stat.S_ISDIR(info.st_mode) and info.st_uid==os.geteuid() and stat.S_IMODE(info.st_mode)==0o700)
def state_path(path,c):
    require(os.name=='posix' and os.geteuid()!=0);p=canonical(path);private_directory(p.parent.stat())
    for base in (c['repo'],c['project'],c['credential_root']):require(p!=base and base not in p.parents and p not in base.parents)
    if p.exists():private_directory(p.stat())
    return p
class Anchor:
    def __init__(self,path,parent,fd):
        self.path,self.parent,self.fd=path,parent,fd;self.parent_pin=directory_identity(os.fstat(parent));self.pin=directory_identity(os.fstat(fd));self.tmp=None;self.tmp_pin=None
    def check(self):
        private_directory(os.fstat(self.parent));private_directory(os.fstat(self.fd))
        require(self.parent_pin==directory_identity(os.fstat(self.parent))==directory_identity(os.lstat(self.path.parent)))
        require(self.pin==directory_identity(os.fstat(self.fd))==directory_identity(os.stat(self.path.name,dir_fd=self.parent,follow_symlinks=False))==directory_identity(os.lstat(self.path)))
        if self.tmp is not None:
            private_directory(os.fstat(self.tmp));require(self.tmp_pin==directory_identity(os.fstat(self.tmp))==directory_identity(os.stat('tmp',dir_fd=self.fd,follow_symlinks=False)))
    def temporary(self):
        self.check()
        try:os.mkdir('tmp',0o700,dir_fd=self.fd)
        except FileExistsError:pass
        self.tmp=os.open('tmp',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=self.fd);private_directory(os.fstat(self.tmp));self.tmp_pin=directory_identity(os.fstat(self.tmp));self.check()
@contextlib.contextmanager
def locked(state):
    # An owned0700 parent excludes another UID from swapping this namespace.
    parent=os.open(state.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC);fd=lock=None;anchor=None
    try:
        private_directory(os.fstat(parent));require(directory_identity(os.fstat(parent))==directory_identity(os.lstat(state.parent)))
        try:os.mkdir(state.name,0o700,dir_fd=parent)
        except FileExistsError:pass
        fd=os.open(state.name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=parent);private_directory(os.fstat(fd));anchor=Anchor(state,parent,fd);anchor.check()
        lock=os.open('lock',os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC,0o600,dir_fd=fd);private(os.fstat(lock),0o600)
        try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except BlockingIOError:raise Busy() from None
        anchor.check();yield anchor
    finally:
        if anchor is not None and anchor.tmp is not None:os.close(anchor.tmp)
        if lock is not None:os.close(lock)
        if fd is not None:os.close(fd)
        os.close(parent)
def bytes_at(fd,name):return snapshot_at(fd,name)[0]
def runtime_supported():return sys.platform=='linux'
def execution_paths(anchor):
    # Linux only: derive this process and its own held descriptors, never config PID/FD.
    require(runtime_supported());anchor.check();require(anchor.tmp is not None)
    root=Path('/proc')/str(os.getpid())/'fd';cwd=root/str(anchor.fd);tmp=root/str(anchor.tmp)
    require(directory_identity(os.stat(cwd))==anchor.pin and directory_identity(os.stat(tmp))==anchor.tmp_pin)
    return cwd,tmp
def store(fd,name,value,exclusive=False):
    raw=(json.dumps(value,separators=(',',':'),sort_keys=True)+'\n').encode();require(len(raw)<=16384)
    if exclusive:
        f=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=fd)
    else:
        # Never truncate a preexisting symlink or foreign file. Atomic fixed-name replace.
        try:private(os.stat(name,dir_fd=fd,follow_symlinks=False),0o600)
        except FileNotFoundError:pass
        f=os.open(name+'.new',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=fd)
    try:
        require(os.write(f,raw)==len(raw));os.fsync(f)
    finally:os.close(f)
    if not exclusive:os.replace(name+'.new',name,src_dir_fd=fd,dst_dir_fd=fd)
    os.fsync(fd)
def argv(c):
    return [sys.executable,'-I','-B',str(c['repo']/'scripts/job-dispatcher.py'),'--dil',str(c['dil']),'--project',str(c['project']),'--expected-source',c['source_sha'],'--expected-binary-hash',c['binary_sha256'],'--expected-project-hash',c['project_sha256'],'--owned-pg-port',str(c['pg_port']),'--owned-ca',str(c['ca']),'--credential-root',str(c['credential_root']),'--credential-env','ZEE_JOB_CI_URL='+c['credential_name'],'--owner',c['owner'],'--ticks','1']
def execute(c,state):
    pins=source_pin(c)
    with locked(state) as anchor:
        fd=anchor.fd;anchor.check()
        try:bytes_at(fd,'inflight.json')
        except FileNotFoundError:pass
        else:raise Paused()
        latch={'schema':'zee-sql-timer-inflight-1','source_sha':c['source_sha'],'binary_sha256':c['binary_sha256'],'created_ns':time.time_ns()};store(fd,'inflight.json',latch,True)
        # Durable BEFORE Popen. Pre-clear uncertainty leaves admission paused; post-clear is explicit.
        anchor.temporary();cwd,tmp=execution_paths(anchor)
        env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TMPDIR':str(tmp)};start=time.monotonic()
        try:
            anchor.check();raw=D.run(argv(c),cwd=cwd,env=env,timeout=165,limit=1024*1024)
            anchor.check()
            require(source_pin(c)==pins)
            # Persist only fixed metadata and raw-output hash; never argv/output/credentials.
            store(fd,'last.json',{'schema':'zee-sql-timer-result-1','outcome':'completed','source_sha':c['source_sha'],'elapsed_seconds':time.monotonic()-start,'output_sha256':hashlib.sha256(raw).hexdigest()})
            clear_latch(anchor,(json.dumps(latch,separators=(',',':'),sort_keys=True)+'\n').encode(),'completion')
        except ClearVisible:raise
        except BaseException:
            # No exception text, child streams, identifiers or source paths are recorded.
            raise Paused() from None
    return 0
def validate_latch(raw,c):
    value=decode(raw);require(type(value) is dict and set(value)=={'schema','source_sha','binary_sha256','created_ns'})
    require(value['schema']=='zee-sql-timer-inflight-1' and value['source_sha']==c['source_sha'] and value['binary_sha256']==c['binary_sha256'])
    require(type(value['created_ns']) is int and 0<value['created_ns']<=2**63-1)
    return value
def clear_latch(anchor,expected,operation):
    anchor.check();require(bytes_at(anchor.fd,'inflight.json')==expected)
    os.unlink('inflight.json',dir_fd=anchor.fd)
    try:os.fsync(anchor.fd)
    except OSError:raise ClearVisible(operation) from None
def reconcile(c,state,path,pin):
    raw=read_private(path);require(re.fullmatch('[0-9a-f]{64}',pin or '') and hashlib.sha256(raw).hexdigest()==pin);review=decode(raw)
    require(set(review)=={'schema','source_sha','binary_sha256','inflight_sha256','service_inactive_reviewed','owned_cgroup_empty_reviewed','sql_outcome_reviewed'} and review['schema']=='zee-sql-timer-reconcile-1')
    require(review['source_sha']==c['source_sha'] and review['binary_sha256']==c['binary_sha256'] and review['service_inactive_reviewed'] is True and review['owned_cgroup_empty_reviewed'] is True and review['sql_outcome_reviewed'] in ('commit-verified','rollback-verified','no-child-started-verified'))
    with locked(state) as anchor:
        fd=anchor.fd;anchor.check()
        raw_latch=bytes_at(fd,'inflight.json');require(hashlib.sha256(raw_latch).hexdigest()==review['inflight_sha256']);validate_latch(raw_latch,c)
        anchor.check();store(fd,'last-reconciliation.json',review);clear_latch(anchor,raw_latch,'reconciliation')
    return 0
def unit_text(runner,configuration,state):
    paths=[canonical(p) for p in (Path(sys.executable).resolve(),runner,configuration,state)]
    require(all(re.fullmatch('/[A-Za-z0-9_./-]+',str(p)) for p in paths));python,runner,configuration,state=paths
    service='[Unit]\nDescription=Zee owned SQL-only finite dispatcher\n[Service]\nType=oneshot\nUMask=0077\nExecStart='+str(python)+' -I -B '+str(runner)+' --config '+str(configuration)+' --state '+str(state)+' --run\nRestart=no\nKillMode=mixed\nTimeoutStartSec=180\nTimeoutStopSec=10\nSendSIGKILL=yes\nStandardInput=null\nStandardOutput=null\nStandardError=null\nNoNewPrivileges=yes\n'
    timer='[Unit]\nDescription=Zee owned SQL-only finite dispatcher timer\n[Timer]\nOnActiveSec=30s\nOnUnitInactiveSec=60s\nAccuracySec=1s\nPersistent=false\nUnit=zee-sql-intent.service\n[Install]\nWantedBy=timers.target\n'
    return service,timer
def main(args=None):
    p=MaskedParser(add_help=False,allow_abbrev=False);p.add_argument('--config',required=True,type=Path);p.add_argument('--state',required=True,type=Path);g=p.add_mutually_exclusive_group();g.add_argument('--run',action='store_true');g.add_argument('--reconcile',type=Path);g.add_argument('--render',type=Path);p.add_argument('--reconciliation-sha256');a=p.parse_args(args)
    require((a.reconcile is None)==(a.reconciliation_sha256 is None))
    require(fcntl is not None and D is not None and os.name=='posix' and hasattr(os,'WNOWAIT'))
    D.CANCELLED=False
    old={n:signal.getsignal(n) for n in (signal.SIGTERM,signal.SIGINT)}
    def cancel(number,frame):D.CANCELLED=True
    try:
        for n in old:signal.signal(n,cancel)
        c=config(a.config);state=state_path(a.state,c);source_pin(c)
        if a.render:
            out=state_path(a.render,c);require(not out.exists());service,timer=unit_text(Path(__file__),a.config,state);out.mkdir(mode=0o700);(out/'zee-sql-intent.service').write_text(service);(out/'zee-sql-intent.timer').write_text(timer);return 0
        if a.reconcile:return reconcile(c,state,a.reconcile,a.reconciliation_sha256)
        if not a.run:return 0
        require(runtime_supported())
        return execute(c,state)
    finally:
        for n,h in old.items():signal.signal(n,h)
if __name__=='__main__':
    try:raise SystemExit(main())
    except ClearVisible as error:report_clear(error);raise SystemExit(4)
    except Paused:raise SystemExit(3)
    except Busy:raise SystemExit(75)
    except (Exception,KeyboardInterrupt):raise SystemExit(2)
