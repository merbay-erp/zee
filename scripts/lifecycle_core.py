"""K268 narrow lifecycle core; reviewed509106d7 lineage v1 + K-319 Linux zombie-leader delta in lineage v2."""
import hashlib,os,selectors,signal,subprocess,sys,time
from pathlib import Path
MAX_CAPTURE=8*1024*1024
CANCELLED=False
class Reject(Exception):pass
def require(value,reason):
    if not value:raise Reject(reason)
def digest(path):
    h=hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda:stream.read(1024*1024),b''):h.update(block)
    return h.hexdigest()
def held_status(child):
    require(child.returncode is None,'lost child authority')
    status=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
    return None if status is None else (status.si_status if status.si_code==os.CLD_EXITED else -status.si_status)
def linux_group_live(pgid):
    # K-319: a held (unreaped) zombie leader keeps killpg(pgid,0) succeeding on Linux.
    try:entries=os.listdir('/proc')
    except OSError:return True
    for entry in entries:
        if not entry.isdigit():continue
        try:
            with open('/proc/'+entry+'/stat','rb') as stream:stat=stream.read()
        except OSError:continue
        fields=stat[stat.rfind(b')')+2:].split()
        if len(fields)>2 and int(fields[2])==pgid and fields[0] not in (b'Z',b'X'):return True
    return False
def stop_owned(child,grace=3):
    deadline=time.monotonic()+grace
    held_status(child)
    linux=sys.platform.startswith('linux')
    def retry():
        left=deadline-time.monotonic();require(left>0,'owned cleanup permission deadline');time.sleep(min(.001,left))
    def alive():
        while True:
            try:
                os.killpg(child.pid,0)
                return not(linux and held_status(child) is not None and not linux_group_live(child.pid))
            except ProcessLookupError:return False
            except PermissionError:
                if held_status(child) is not None:return False
                retry()
    def send(number):
        while True:
            try:os.killpg(child.pid,number);return
            except ProcessLookupError:return
            except PermissionError:
                if not alive():return
                retry()
    if alive():send(signal.SIGTERM)
    while alive() and time.monotonic()<deadline:time.sleep(min(.02,max(0,deadline-time.monotonic())))
    # The held leader still reserves the PGID: on Linux sweep any member a /proc scan raced.
    if linux or alive():send(signal.SIGKILL)
    return child.wait(timeout=2)
def controlled_env():
    env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
    env.pop('ZEE_SURUM_TANI_DIZINI',None)
    env.update({'GIT_TERMINAL_PROMPT':'0','GIT_CONFIG_GLOBAL':'/dev/null','GIT_CONFIG_NOSYSTEM':'1'})
    return env
def run(argv,cwd=None,timeout=30,limit=MAX_CAPTURE,env=None,log_path=None):
    require(not CANCELLED,'cancelled')
    child=subprocess.Popen([str(x) for x in argv],cwd=cwd,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True,close_fds=True,env=controlled_env() if env is None else env)
    sel=None;out=bytearray();deadline=time.monotonic()+timeout
    try:
        sel=selectors.DefaultSelector();sel.register(child.stdout,selectors.EVENT_READ)
        while sel.get_map():
            require(not CANCELLED and time.monotonic()<deadline,'command deadline/cancelled')
            for key,_ in sel.select(min(.1,max(0,deadline-time.monotonic()))):
                block=os.read(key.fileobj.fileno(),4096)
                if not block:sel.unregister(key.fileobj)
                else:require(len(out)+len(block)<=limit,'command capture budget');out.extend(block)
        status=held_status(child)
        while status is None:
            require(not CANCELLED and time.monotonic()<deadline,'terminal command deadline/cancelled');time.sleep(.01);status=held_status(child)
        require(status==0,'command nonzero')
        return bytes(out)
    finally:
        try:
            if sel is not None:sel.close()
            if log_path is not None:Path(log_path).write_bytes(out)
        finally:
            try:stop_owned(child)
            finally:child.stdout.close()
def git(repo,*args):return run(['git','-C',repo,*args]).decode().strip()
def clean_exact(repo,sha):
    require(git(repo,'rev-parse','HEAD')==sha,'HEAD mismatch')
    require(not git(repo,'status','--porcelain','--untracked-files=all'),'dirty repository')
