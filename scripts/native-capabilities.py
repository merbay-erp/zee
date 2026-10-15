"""K267 borrowed named roles; no close, dup, ownership, secret read or PID adoption."""
import ctypes,hashlib,json,os,re,signal,socket,stat,sys
try:import fcntl
except ImportError:fcntl=None
from typing import NamedTuple
class Rejected(ValueError):pass
def require(value):
    if not value:raise Rejected()
class Listener(NamedTuple):
    listener:int
    ready:int
    activate:int
    startup:int
    generation:str
    port:int
    binary_sha256:str
    metric:object=None
    def native_fds(self):return (self.listener,self.ready,self.activate)+(() if self.metric is None else (self.metric,))
_PIPE_QUERY=None
def pipe_pair(fd,closed=False):
    info=os.fstat(fd)
    if not stat.S_ISFIFO(info.st_mode):return None
    if sys.platform!='darwin':return (info.st_dev,info.st_ino)
    # Apple SDK sys/proc_info.h pipe_fdinfo:184 bytes, handles160/168.
    # Own PID only, exact returned size; no handle is logged or adopted.
    global _PIPE_QUERY
    if _PIPE_QUERY is None:
        lib=ctypes.CDLL('/usr/lib/libproc.dylib',use_errno=True);_PIPE_QUERY=lib.proc_pidfdinfo
        _PIPE_QUERY.argtypes=(ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_void_p,ctypes.c_int);_PIPE_QUERY.restype=ctypes.c_int
    data=ctypes.create_string_buffer(184);require(_PIPE_QUERY(os.getpid(),fd,6,data,184)==184)
    handle=ctypes.c_uint64.from_buffer(data,160).value;peer=ctypes.c_uint64.from_buffer(data,168).value
    require(handle and (peer or closed));return tuple(sorted((handle,peer or handle)))
def stdio_alias(fd):
    info=os.fstat(fd);key=(info.st_dev,info.st_ino);pair=pipe_pair(fd,True)
    for standard in range(3):
        try:other=os.fstat(standard)
        except OSError:continue
        if key==(other.st_dev,other.st_ino) or (pair is not None and pair==pipe_pair(standard,True)):return True
    return False
def quiet_stdio_alias(argv):
    if not supported():return False
    options=argv[:argv.index('--')] if '--' in argv else argv
    roles=('--native-listener-fd','--native-ready-fd','--native-activate-fd','--native-startup-fd','--native-metrics-fd')
    # Quiet detection accepts malformed '=' spelling; extract still grants only
    # the complete spaced typed profile. Each occurrence is checked independently.
    for i,option in enumerate(options):
        key,separator,text=option.partition('=')
        if key not in roles:continue
        if not separator:
            if i+1>=len(options):continue
            text=options[i+1]
        try:
            if not (text.isascii() and text.isdecimal()):continue
            digits=text.lstrip('0') or '0'
            if len(digits)>10:continue
            descriptor=int(digits)
            if 0<=descriptor<=2147483647 and stdio_alias(descriptor):return True
        except (OSError,ValueError,OverflowError):continue
    return False
def supported():return os.name=="posix" and fcntl is not None and all(hasattr(os,n) for n in ("O_NONBLOCK","O_CLOEXEC","geteuid"))
def borrowed(profile):
    require(supported())
    roles=profile.native_fds()+(profile.startup,);require(len(set(roles))==len(roles))
    identities=[];pairs=[]
    for fd in roles:
        require(type(fd) is int and 3<=fd<=255);info=os.fstat(fd);key=(info.st_dev,info.st_ino);require(key not in identities);identities.append(key)
        require(info.st_uid in (0,os.geteuid()));pair=pipe_pair(fd,profile.metric==fd)
        if pair is not None:require(pair not in pairs);pairs.append(pair)
        require(not stdio_alias(fd))
    for fd,access in ((profile.ready,os.O_WRONLY),(profile.activate,os.O_RDONLY),(profile.startup,os.O_WRONLY)):
        require(stat.S_ISFIFO(os.fstat(fd).st_mode));flags=fcntl.fcntl(fd,fcntl.F_GETFL);require(flags&os.O_ACCMODE==access)
    require(fcntl.fcntl(profile.startup,fcntl.F_GETFL)&os.O_NONBLOCK and os.fpathconf(profile.startup,'PC_PIPE_BUF')>=512)
    if profile.metric is not None:
        fd=profile.metric;flags=fcntl.fcntl(fd,fcntl.F_GETFL);require(stat.S_ISFIFO(os.fstat(fd).st_mode) and flags&os.O_ACCMODE==os.O_WRONLY and flags&os.O_NONBLOCK and os.fpathconf(fd,'PC_PIPE_BUF')>=512)
    require(signal.getsignal(signal.SIGPIPE)==signal.SIG_IGN)
    require(fcntl.fcntl(profile.listener,fcntl.F_GETFL)&os.O_NONBLOCK)
    sock=socket.socket(fileno=profile.listener)
    try:
        require(sock.family==socket.AF_INET and sock.getsockopt(socket.SOL_SOCKET,socket.SO_TYPE)==socket.SOCK_STREAM)
        sock.getsockopt(socket.IPPROTO_TCP,socket.TCP_NODELAY)
        if sys.platform=='darwin':
            raw=sock.getsockopt(socket.IPPROTO_TCP,socket.TCP_CONNECTION_INFO,256);require(0<len(raw)<=256 and raw[0]==1)
        else:require(sock.getsockopt(socket.SOL_SOCKET,socket.SO_ACCEPTCONN)==1)
        require(sock.getsockname()==('127.0.0.1',profile.port) and 0<profile.port<=65535)
    finally:sock.detach()
    require(re.fullmatch('[0-9a-f]{32}',profile.generation) and re.fullmatch('[0-9a-f]{64}',profile.binary_sha256))
    return profile
HOST_FLAGS=('--native-listener-fd','--native-ready-fd','--native-activate-fd','--native-startup-fd','--native-generation','--native-worker-port','--native-binary-sha256')
CLI_FLAGS=('--web-listener-fd','--web-ready-fd','--web-activate-fd','--web-generation','--web-worker-port','--web-metrik-fd')
VALUED=frozenset(CLI_FLAGS+('--web-proxy','--güvenli-web-proxy','--web-oran-deposu','--web-oran-siniri','--web-oran-kapasitesi','--web-oran-doluluk','--ayar','--profil'))
def command_values(command,strict=False):
    require(command and command[0] in ('çalıştır','calistir'));values={};i=1
    while i<len(command):
        value=command[i]
        if not value.startswith('-'):break
        if value in VALUED:
            require(i+1<len(command) and value not in values);values[value]=command[i+1];i+=2
        else:require(not strict);i+=1
    require(i<len(command));return values
def extract(argv,metric=None):
    require('--' in argv);split=argv.index('--');options=argv[:split];command=argv[split+1:];named={};kept=[];i=0
    while i<len(options):
        if options[i] in HOST_FLAGS:
            key=options[i];require(key not in named and i+1<len(options));named[key]=options[i+1];i+=2
        else:kept.append(options[i]);i+=1
    values=command_values(command,strict=bool(named))
    if not named:
        require(not any(flag in values for flag in CLI_FLAGS[:4]));return argv,None
    require(set(named)==set(HOST_FLAGS))
    def fd(key):
        text=named[key];require(text.isascii() and text.isdecimal());return int(text)
    profile=Listener(fd(HOST_FLAGS[0]),fd(HOST_FLAGS[1]),fd(HOST_FLAGS[2]),fd(HOST_FLAGS[3]),named[HOST_FLAGS[4]],fd(HOST_FLAGS[5]),named[HOST_FLAGS[6]],metric)
    expected={CLI_FLAGS[0]:str(profile.listener),CLI_FLAGS[1]:str(profile.ready),CLI_FLAGS[2]:str(profile.activate),CLI_FLAGS[3]:profile.generation,CLI_FLAGS[4]:str(profile.port)}
    if metric is not None:expected[CLI_FLAGS[5]]=str(metric)
    else:require(CLI_FLAGS[5] not in values)
    require(all(values.get(k)==v for k,v in expected.items()) and any(k in values for k in ('--web-proxy','--güvenli-web-proxy')))
    borrowed(profile);return kept+['--']+command,profile
def announce(profile,pid):
    raw=(json.dumps({'schema':'zee-credential-native-start-1','pid':pid,'generation':profile.generation,'binary_sha256':profile.binary_sha256},separators=(',',':'))+'\n').encode();require(len(raw)<=512);require(os.write(profile.startup,raw)==len(raw))
