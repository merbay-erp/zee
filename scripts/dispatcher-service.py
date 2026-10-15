#!/usr/bin/env python3
"""K282: owned continuous dispatch; unknown effects never authorize replay."""
import argparse, contextlib, hashlib, importlib.util, json, math, os, re, signal, stat, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent

def load(name, path):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        s = os.fstat(fd)
        if not stat.S_ISREG(s.st_mode) or s.st_size > 262144: raise ValueError('module')
        raw = bytearray()
        while len(raw) <= 262144:
            block = os.read(fd, 4096)
            if not block: break
            raw.extend(block)
        if len(raw) != s.st_size or len(raw) > 262144: raise ValueError('module')
        after = os.fstat(fd)
        if (s.st_ino,s.st_dev,s.st_size,s.st_mtime_ns,s.st_ctime_ns) != (after.st_ino,after.st_dev,after.st_size,after.st_mtime_ns,after.st_ctime_ns): raise ValueError('module')
    finally: os.close(fd)
    spec = importlib.util.spec_from_loader(name, loader=None)
    module = importlib.util.module_from_spec(spec); module.__file__ = str(path)
    exec(compile(bytes(raw), str(path), 'exec'), module.__dict__)
    return module

T = None
try: T = load('dispatch_timer', HERE/'finite_timer.py')
except Exception: pass
STOP = False

def need(ok):
    if not ok: raise ValueError('service configuration rejected')

class Parser(argparse.ArgumentParser):
    def error(self, message): raise ValueError('arguments')
    def exit(self, status=0, message=None): raise ValueError('arguments')

def snapshot(path, maximum=134217728):
    path = T.canonical(path); before = path.lstat()
    need(stat.S_ISREG(before.st_mode) and before.st_uid in (0,os.geteuid()) and not before.st_mode & 0o022 and before.st_size <= maximum)
    fd = os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
    try:
        initial = os.fstat(fd); need(T.identity(before)==T.identity(initial)); h=hashlib.sha256(); count=0
        while count <= maximum:
            try: block=os.read(fd,min(65536,maximum+1-count))
            except InterruptedError: continue
            if not block: break
            count+=len(block); h.update(block)
        need(count==initial.st_size and count<=maximum and T.identity(initial)==T.identity(os.fstat(fd))==T.identity(path.lstat()))
        return h.hexdigest()
    finally: os.close(fd)

def configuration(path):
    raw=T.read_private(path); c=T.decode(raw)
    common={'schema','kind','repo','source_sha','interval_seconds','state'}
    need(type(c) is dict and c.get('schema')=='zee-dispatch-service-1' and c.get('kind') in ('sql-intent','application-adapter'))
    required=common|({'timer_config','timer_config_sha256'} if c['kind']=='sql-intent' else {'executable','executable_sha256','cwd','data','data_sha256','argv','timeout_seconds','contract'})
    need(set(c)==required and re.fullmatch('[0-9a-f]{40}',c['source_sha']) is not None)
    need(type(c['interval_seconds']) in (int,float) and math.isfinite(c['interval_seconds']) and 1<=c['interval_seconds']<=3600)
    for k in ('repo','state'): c[k]=T.canonical(c[k])
    T.private_directory(c['state'].parent.stat()); need(c['repo'].is_dir() and c['repo'] not in c['state'].parents and c['repo']!=c['state'])
    if c['kind']=='sql-intent':
        c['timer_config']=T.canonical(c['timer_config']); need(hashlib.sha256(T.read_private(c['timer_config'])).hexdigest()==c['timer_config_sha256'])
        c['_timer']=T.config(c['timer_config']); need(c['_timer']['source_sha']==c['source_sha'] and c['_timer']['repo']==c['repo'])
        T.state_path(c['state'],c['_timer'])
    else:
        for k in ('executable_sha256','data_sha256'): need(re.fullmatch('[0-9a-f]{64}',c[k]) is not None)
        for k in ('executable','cwd','data'): c[k]=T.canonical(c[k])
        need(c['contract']=='zee-app-dispatch-result-1' and c['cwd'].is_dir() and c['cwd'].stat().st_uid==os.geteuid() and not c['cwd'].stat().st_mode&0o022)
        need(os.access(c['executable'],os.X_OK) and c['state']!=c['cwd'] and c['state'] not in c['cwd'].parents and c['cwd'] not in c['state'].parents)
        need(type(c['argv']) is list and len(c['argv'])<=16 and all(type(x) is str and len(x.encode())<=256 and not any(ord(z)<32 or ord(z)==127 for z in x) for x in c['argv']))
        need(type(c['timeout_seconds']) in (int,float) and math.isfinite(c['timeout_seconds']) and 1<=c['timeout_seconds']<=165)
    c['_configuration']=T.canonical(path); c['_configuration_sha256']=hashlib.sha256(raw).hexdigest()
    return c

def pins(c):
    need(hashlib.sha256(T.read_private(c['_configuration'])).hexdigest()==c['_configuration_sha256'])
    T.D.clean_exact(c['repo'],c['source_sha'])
    result={name:snapshot(HERE/name,262144) for name in ('dispatcher-service.py','finite_timer.py','lifecycle_core.py')}
    if c['kind']=='sql-intent':
        need(hashlib.sha256(T.read_private(c['timer_config'])).hexdigest()==c['timer_config_sha256']); result.update(T.source_pin(c['_timer']))
    else:
        result['executable']=snapshot(c['executable']); result['data']=snapshot(c['data'],1048576)
        need(result['executable']==c['executable_sha256'] and result['data']==c['data_sha256'])
    return result

def copy_snapshot(source,directory,name,expected,maximum,mode):
    before=source.lstat();need(stat.S_ISREG(before.st_mode) and before.st_uid in (0,os.geteuid()) and not before.st_mode&0o022 and before.st_size<=maximum)
    readfd=os.open(source,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC);writefd=None
    try:
        opened=os.fstat(readfd);need(T.identity(before)==T.identity(opened))
        writefd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600,dir_fd=directory)
        size=0;h=hashlib.sha256()
        while size<=maximum:
            try:block=os.read(readfd,min(65536,maximum+1-size))
            except InterruptedError:continue
            if not block:break
            size+=len(block);need(size<=maximum);h.update(block);view=memoryview(block)
            while view:
                n=os.write(writefd,view);need(n>0);view=view[n:]
        need(size==opened.st_size and h.hexdigest()==expected and T.identity(opened)==T.identity(os.fstat(readfd))==T.identity(source.lstat()))
        os.fchmod(writefd,mode)
    finally:
        if writefd is not None:os.close(writefd)
        os.close(readfd)

def adapter_cwd(fd):
    need(sys.platform=='linux')
    path=Path('/proc')/str(os.getpid())/'fd'/str(fd)
    need(T.directory_identity(path.stat())==T.directory_identity(os.fstat(fd)))
    return path

def adapter_tick(c):
    before=pins(c)
    with T.locked(c['state']) as anchor:
        try: T.bytes_at(anchor.fd,'inflight.json')
        except FileNotFoundError: pass
        else: raise T.Paused()
        latch={'schema':'zee-app-dispatch-inflight-1','source_sha':c['source_sha'],'executable_sha256':c['executable_sha256'],'data_sha256':c['data_sha256'],'created_ns':time.time_ns()}
        encoded=(json.dumps(latch,sort_keys=True,separators=(',',':'))+'\n').encode()
        T.store(anchor.fd,'inflight.json',latch,True)
        cwd_fd=None
        try:
            anchor.temporary(); _,tmp=T.execution_paths(anchor)
            copy_snapshot(c['executable'],anchor.tmp,'adapter.snapshot',c['executable_sha256'],134217728,0o500)
            copy_snapshot(c['data'],anchor.tmp,'input.snapshot',c['data_sha256'],1048576,0o400)
            cwd_fd=os.open(c['cwd'],os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC); cwd_pin=T.directory_identity(os.fstat(cwd_fd))
            need(cwd_pin==T.directory_identity(c['cwd'].lstat()))
            cwd=adapter_cwd(cwd_fd)
            anchor.check(); raw=T.D.run([tmp/'adapter.snapshot',*c['argv'],'--input',tmp/'input.snapshot'],cwd=cwd,timeout=c['timeout_seconds'],limit=4096,env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TMPDIR':str(tmp)})
            result=T.decode(raw); need(type(result) is dict and set(result)=={'schema','outcome','completed'} and result['schema']=='zee-app-dispatch-result-1' and result['outcome'] in ('completed','idle'))
            need(type(result['completed']) is int and 0<=result['completed']<=100 and (result['completed']==0 if result['outcome']=='idle' else result['completed']>0))
            anchor.check(); need(T.directory_identity(c['cwd'].lstat())==cwd_pin and pins(c)==before)
            need(snapshot(anchor.path/'tmp/adapter.snapshot')==c['executable_sha256'] and snapshot(anchor.path/'tmp/input.snapshot',1048576)==c['data_sha256'])
            os.unlink('adapter.snapshot',dir_fd=anchor.tmp);os.unlink('input.snapshot',dir_fd=anchor.tmp)
            T.store(anchor.fd,'last.json',{'schema':'zee-dispatch-service-result-1','outcome':result['outcome'],'completed':result['completed'],'source_sha':c['source_sha']})
            T.clear_latch(anchor,encoded,'completion')
        except T.ClearVisible: raise
        except BaseException: raise T.Paused() from None
        finally:
            if cwd_fd is not None: os.close(cwd_fd)
    return 0

def tick(c):
    return T.execute(c['_timer'],c['state']) if c['kind']=='sql-intent' else adapter_tick(c)

@contextlib.contextmanager
def service_lock(state):
    # Separate lifetime lock: keep admission exclusive even between completed ticks.
    with T.locked(state) as original:
        parent=os.dup(original.parent);directory=os.dup(original.fd)
        held=None
        try:
            anchor=T.Anchor(state,parent,directory)
            held=os.open('service.lock',os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC,0o600,dir_fd=directory)
            T.private(os.fstat(held),0o600)
            try:T.fcntl.flock(held,T.fcntl.LOCK_EX|T.fcntl.LOCK_NB)
            except BlockingIOError:raise T.Busy() from None
        except BaseException:
            if held is not None:os.close(held)
            os.close(directory);os.close(parent);raise
    try:
        anchor.check();yield anchor
    finally:os.close(held);os.close(directory);os.close(parent)

def run(c, once=False):
    with service_lock(c['state']) as anchor:
        while not STOP:
            anchor.check();pins(c);tick(c);anchor.check()
            if once:return 0
            # No catch-up/backfill: next interval starts only after known completion.
            deadline=time.monotonic()+c['interval_seconds']
            while not STOP and time.monotonic()<deadline:time.sleep(min(.1,max(0,deadline-time.monotonic())))
    return 0

def render(config, directory, calendar=None):
    values=[Path(sys.executable).resolve(),Path(__file__).resolve(),T.canonical(config)]
    need(all(re.fullmatch('/[A-Za-z0-9_./-]+',str(p)) for p in values))
    service='[Unit]\nDescription=Zee owned dispatcher service\n[Service]\nType=simple\nUMask=0077\nExecStart='+str(values[0])+' -I -B '+str(values[1])+' --config '+str(values[2])+(' --once' if calendar else '')+'\nRestart=on-failure\nRestartPreventExitStatus=2 3 4 75\nRestartSec=5\nKillMode=mixed\nTimeoutStopSec=10\nSendSIGKILL=yes\nNoNewPrivileges=yes\nStandardInput=null\nStandardOutput=null\nStandardError=null\n'
    files={'zee-dispatch.service':service}
    if calendar is not None:
        need(type(calendar) is str and 1<=len(calendar)<=128 and re.fullmatch('[A-Za-z0-9*,:./ +~-]+',calendar) is not None)
        files['zee-dispatch.timer']='[Unit]\nDescription=Zee owned calendar dispatcher\n[Timer]\nOnCalendar='+calendar+'\nPersistent=false\nAccuracySec=1s\nUnit=zee-dispatch.service\n'
    directory=T.canonical(directory); T.private_directory(directory.parent.stat()); need(not directory.exists()); directory.mkdir(mode=0o700)
    for name,data in files.items(): (directory/name).write_text(data)

def main(argv=None):
    global STOP
    need(T is not None and T.D is not None and T.fcntl is not None and os.name=='posix' and os.geteuid()!=0)
    p=Parser(add_help=False,allow_abbrev=False); p.add_argument('--config',type=Path,required=True); g=p.add_mutually_exclusive_group(); g.add_argument('--run',action='store_true');g.add_argument('--once',action='store_true');g.add_argument('--render',type=Path);p.add_argument('--calendar');a=p.parse_args(argv)
    need(a.calendar is None or a.render is not None); STOP=False;T.D.CANCELLED=False
    old={n:signal.getsignal(n) for n in (signal.SIGTERM,signal.SIGINT)}
    def stop(n,f):
        global STOP
        STOP=True; T.D.CANCELLED=True
    try:
        for n in old:signal.signal(n,stop)
        c=configuration(a.config); pins(c)
        if a.render: render(a.config,a.render,a.calendar); return 0
        if not (a.run or a.once): return 0
        need(T.runtime_supported());return run(c,a.once)
    finally:
        for n,h in old.items(): signal.signal(n,h)

if __name__=='__main__':
    try:raise SystemExit(main())
    except (Exception,KeyboardInterrupt) as error:
        code=2
        if T is not None:
            for kind,value in ((T.ClearVisible,4),(T.Paused,3),(T.Busy,75)):
                if isinstance(error,kind):code=value;break
        raise SystemExit(code)
