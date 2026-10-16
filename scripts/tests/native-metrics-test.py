#!/usr/bin/env python3
"""K310 metrics-only lifecycle proposal; no work at import."""
import argparse,hashlib,json,os,runpy,signal,socket,subprocess,tempfile,time
import selectors,shutil,stat,sys,types,re
from pathlib import Path

EXPECTED_CASES=['actual-pg-pool-four-leases-wait-timeout-recovery', 'native-request-status-rate429-db-boundary-no-content', 'actual-native-pipe-backpressure-recovery', 'actual-native-closed-reader-no-sigpipe', 'actual-native-malformed-source-alias-early-rejection', 'actual-native-malformed-source-stderr-alias-early-rejection', 'actual-two-worker-supervisor-collector', 'credential-designated-metadata-composition-two-workers']
H=None
PG=None
root=None
out=None
binary=None
proof={}
started=0
deadline=0
cleanup_limit=None
cleaning=False
cancelled=False
oldsignals={}
children=[]
child_records={}
components=[]
fds=[]
channels=[]
cleanup_errors=[]
command_logs=[]
work=None
work_identity=None
cluster=None
campaign=None
private_uncertain=False

def need(ok,label):
    if not ok:raise ValueError(label)

def active():
    need(not cancelled and not H['CANCELLED'] and time.monotonic()<deadline,'metrics work cancelled/deadline')

def require(ok,label):
    active()
    if not ok:raise AssertionError(label)

def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(65536),b''):h.update(block)
    return h.hexdigest()

def cancel(*_):
    global cancelled
    cancelled=True
    if not cleaning:H['CANCELLED']=True

def install_handlers():
    for number in (signal.SIGTERM,signal.SIGINT,signal.SIGHUP):
        oldsignals[number]=signal.getsignal(number)
        signal.signal(number,cancel)

def cleanup_end():
    global cleanup_limit
    if cleanup_limit is None:cleanup_limit=time.monotonic()+90
    return cleanup_limit

def remaining(limit):
    end=cleanup_end() if cleaning else deadline
    return max(0,min(limit,end-time.monotonic()))

def cleanup_all(*callbacks):
    primary=sys.exc_info()[1]
    for callback in callbacks:
        try:callback()
        except BaseException as error:
            cleanup_errors.append(type(error).__name__)
            if primary is None:primary=error
    if primary is not None:raise primary

def close_child(child):
    first=None
    try:
        if child.returncode is None:
            H['stop_owned'](child,grace=remaining(3))
        need(child.returncode is not None,'original metrics session unreaped')
    except BaseException as error:first=error
    for stream in (child.stdin,child.stdout,child.stderr):
        if stream is not None:
            try:stream.close()
            except BaseException as error:
                if first is None:first=error
    if id(child) in child_records:
        child_records[id(child)].update(exit=child.returncode,reaped=child.returncode is not None)
    if first is not None:raise first

def close_children():
    cleanup_all(*(lambda child=child:close_child(child) for child in children))

def remember_fd(fd):
    row={'fd':fd,'identity':None,'closed':False}
    fds.append(row)
    return row

def own_pipe():
    rd,wr=os.pipe()
    rows=[remember_fd(rd),remember_fd(wr)]
    primary=None
    for row in rows:
        try:
            info=os.fstat(row['fd'])
            row['identity']=(info.st_dev,info.st_ino,info.st_mode)
        except BaseException as error:
            if primary is None:primary=error
    if primary is not None:raise primary
    os.set_blocking(rd,False);os.set_blocking(wr,False)
    return rd,wr


def fd_row(fd):
    return next(row for row in reversed(fds) if row['fd']==fd and not row['closed'])

def close_fd(fd):
    row=fd_row(fd);s=os.fstat(fd)
    need((s.st_dev,s.st_ino,s.st_mode)==row['identity'],'original pipe identity')
    os.close(fd)
    row['closed']=True

def own_channel(api,fd):
    row=fd_row(fd)
    # The channel may close itself on EOF. Its injected close delegates the
    # original pipe authority; registry ownership is not dropped before close.
    class MetricsChannel(api['Channel']):
        def close(self):
            if self.fd is not None:
                close_fd(self.fd)
                self.fd=None
    channel=MetricsChannel(fd)
    channels.append(channel)
    return channel

def close_fds():
    cleanup_all(*(lambda row=row:close_fd(row['fd']) for row in fds if not row['closed']))

def register_component(value,close):
    row={'value':value,'close':close,'closed':False}
    components.append(row)
    return row

def close_component(row):
    row['close']()
    row['closed']=True

def close_components():
    cleanup_all(*(lambda row=row:close_component(row) for row in reversed(components) if not row['closed']))

def command_capture(argv,limit=20,env=None,cwd=None,public_role=None):
    """Original session owner, bounded independent captures; no argv export."""
    active()
    child=None;sel=None;first=None;observed=None
    buffers={0:bytearray(),1:bytearray()}
    end=min(deadline,time.monotonic()+limit)
    row={'role':public_role or 'private-command','exit':None,'reaped':False}
    proof.setdefault('command_owners',[]).append(row)
    try:
        child=subprocess.Popen([str(x) for x in argv],cwd=cwd,env=H['controlled_env']() if env is None else env,
            stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            start_new_session=True,close_fds=True)
        children.append(child);child_records[id(child)]=row
        sel=selectors.DefaultSelector()
        for index,stream in enumerate((child.stdout,child.stderr)):
            os.set_blocking(stream.fileno(),False);sel.register(stream,selectors.EVENT_READ,index)
        while sel.get_map():
            active();need(time.monotonic()<end,'metrics command deadline')
            for key,_ in sel.select(min(.05,max(0,end-time.monotonic()))):
                block=os.read(key.fileobj.fileno(),4096)
                if not block:sel.unregister(key.fileobj)
                else:
                    buffers[key.data].extend(block)
                    need(len(buffers[key.data])<=1048576,'metrics command capture bound')
        while observed is None:
            active();need(time.monotonic()<end,'metrics command terminal deadline')
            observed=H['held_status'](child)
            if observed is None:time.sleep(min(.01,max(0,end-time.monotonic())))
        row['observed_exit']=observed
    except BaseException as error:first=error
    finally:
        if sel is not None:
            try:sel.close()
            except BaseException as error:
                if first is None:first=error
        if child is not None:
            try:close_child(child)
            except BaseException as error:
                if first is None:first=error
            row.update(exit=child.returncode,reaped=child.returncode is not None)
    if first is not None:raise first
    active();need(time.monotonic()<end and child.returncode==observed,'metrics settled command')
    result=types.SimpleNamespace(returncode=observed,stdout=bytes(buffers[0]),stderr=bytes(buffers[1]))
    if public_role is not None and public_role!='owned-pg-pool' and observed==0:
        need(public_role in {'binary-version','certificate-setup','owned-PG-initdb','owned-PG-version','owned-pg-pool'},'explicit safe public role')
        for suffix,payload in [('stdout',result.stdout),('stderr',result.stderr)]:
            name=f'command-{len(command_logs)+1:03d}-{suffix}.log'
            path=out/name;path.write_bytes(payload)
            command_logs.append({'role':public_role,'name':name,'sha256':hashlib.sha256(payload).hexdigest(),'bytes':len(payload)})
    return result

def safe_run(role,argv,limit=20,env=None):
    result=command_capture(argv,limit,env,public_role=role)
    need(result.returncode==0,'safe metrics tool failed')
    return result.stdout

POOL_SELECTOR='postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar'

def pool_result(raw):
    lines=raw.decode().splitlines()
    marker='K257_OWNED_POOL_METRICS '
    prefix='test '+POOL_SELECTOR+' ... '
    markers=[];wrappers=[];plain_ok=0;summaries=[]
    for line in lines:
        if line=='':continue
        if line=='running 1 test':continue
        if line.startswith('test result:'):
            summaries.append(line);continue
        if line.startswith(prefix):
            rest=line[len(prefix):];wrappers.append(rest)
            if rest in ('ok',''):continue
            need(rest.startswith(marker),'exact pool wrapper/structured child event')
            markers.append(rest[len(marker):]);continue
        if line.startswith(marker):markers.append(line[len(marker):]);continue
        if line=='ok':plain_ok+=1;continue
        raise ValueError('unexpected pool stdout')
    need(lines.count('running 1 test')==1 and len(wrappers)==1 and len(markers)==1
        and plain_ok==(0 if wrappers==['ok'] else 1) and len(summaries)==1,'single pool owner/event/result')
    need(re.fullmatch(r'test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in [0-9]+(?:\.[0-9]+)?s',summaries[0]),'strict pool libtest summary')
    rows=json.loads(markers[0],parse_constant=lambda _:(_ for _ in ()).throw(ValueError('finite pool JSON')))
    need(type(rows) is list and 0<len(rows)<=64 and all(type(row) is list and len(row)==8
        and all(type(value) is int and 0<=value<=9007199254740991 for value in row) for row in rows),'typed finite pool snapshots')
    return rows

def export_pool(result):
    pool_result(result.stdout)
    profile='release' if binary.parent.name=='release' else 'debug'
    lines=result.stderr.decode().splitlines()
    patterns=[
        r'\s*(?:Compiling|Fresh) [A-Za-z0-9_.+-]+ v[0-9][A-Za-z0-9_.+-]*(?: \('+re.escape(str(root/'compiler'))+r'\))?',
        r'\s*Finished \x60'+('release' if profile=='release' else 'test')+r'\x60 profile \[[A-Za-z0-9 +]+\] target\(s\) in [0-9.hms ]+',
        r'\s*Running unittests src/lib\.rs \(target/'+profile+r'/deps/dil-[0-9a-f]+\)'
    ]
    need(all(line=='' or any(re.fullmatch(pattern,line) for pattern in patterns) for line in lines),'closed public pool stderr grammar')
    for suffix,payload in [('stdout',result.stdout),('stderr',result.stderr)]:
        name='owned-pg-pool-'+suffix+'.log';(out/name).write_bytes(payload)
        command_logs.append({'role':'owned-pg-pool','name':name,'sha256':hashlib.sha256(payload).hexdigest(),'bytes':len(payload)})


class MetricsPgCampaign:
    def __init__(self,directory,binaries):
        self.out=out;self.deadline=deadline;self.clusters=[];self.proof=proof
        info=directory.lstat();self.tempdirs=[(directory,(info.st_dev,info.st_ino))]
        self.pg_pins={str(binaries/name):PG['digest_file'](binaries/name) for name in ('postgres','initdb','pg_ctl','psql')}
    @property
    def stopped(self):return cancelled or H['CANCELLED']
    @property
    def cleaning(self):return cleaning
    @property
    def cleanup_deadline(self):return cleanup_end() if cleaning else self.deadline
    def need(self,value):
        need(value and time.monotonic()<self.cleanup_deadline and (cleaning or not self.stopped),'metrics PG budget')
    def command(self,label,argv,budget=20,env=None):return safe_run(label,argv,budget,env)

class OwnedChild:
    """Facade keeps wait/poll from reaping original session authority early."""
    def __init__(self,child):self.child=child
    def __getattr__(self,name):return getattr(self.child,name)
    def poll(self):
        if self.child.returncode is not None:return self.child.returncode
        return H['held_status'](self.child)
    def wait(self,timeout):
        end=time.monotonic()+min(timeout,remaining(timeout))
        status=self.poll()
        while status is None:
            need((cleaning or not cancelled and not H['CANCELLED']) and time.monotonic()<end,'metrics worker wait deadline')
            time.sleep(min(.01,max(0,end-time.monotonic())));status=self.poll()
        if self.child.returncode is None:H['stop_owned'](self.child,grace=remaining(3))
        need(self.child.returncode is not None,'original metrics worker reap')
        if id(self.child) in child_records:child_records[id(self.child)].update(observed_exit=status,exit=self.child.returncode,reaped=True)
        return self.child.returncode

class PrivateLease:
    def __init__(self,**_):
        path=Path(tempfile.mkdtemp(prefix='supervisor-',dir=work)).resolve(strict=True)
        PG['private_directory'](path.lstat())
        self.name=str(path)
    def cleanup(self):
        # The outer metrics owner removes the entire identity-bound root only
        # after every registered component, child, pipe and PG owner is closed.
        pass

class MetricsRunning:
    def __init__(self):
        self.child=None;self.selector=None;self.buffer=bytearray();self.events=[];self.closed=False
    def start(self,argv,env=None):
        active()
        original=subprocess.Popen(argv,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True,close_fds=True)
        children.append(original);self.child=OwnedChild(original)
        self.selector=selectors.DefaultSelector()
        os.set_blocking(original.stdout.fileno(),False);os.set_blocking(original.stderr.fileno(),False)
        self.selector.register(original.stdout,selectors.EVENT_READ)
        return self
    def until(self,predicate,timeout=8,after=0):
        end=time.monotonic()+min(timeout,remaining(timeout))
        while time.monotonic()<end:
            active()
            if any(predicate(event) for event in self.events[after:]):
                return next(event for event in reversed(self.events[after:]) if predicate(event))
            for key,_ in self.selector.select(min(.05,max(0,end-time.monotonic()))):
                chunk=os.read(key.fileobj.fileno(),4096)
                if not chunk:self.selector.unregister(key.fileobj);continue
                self.buffer.extend(chunk);need(len(self.buffer)<=1048576,'metrics supervisor buffer')
                while b'\n' in self.buffer:
                    line,_,rest=self.buffer.partition(b'\n');self.buffer=bytearray(rest)
                    self.events.append(json.loads(line));need(len(self.events)<=4096,'metrics supervisor events')
        raise ValueError('bounded metrics event missing')
    def finish(self,stop=True):
        if self.closed:return self.child.returncode
        first=None;err=bytearray()
        try:
            if self.child is not None:
                if stop and self.child.poll() is None:self.child.send_signal(signal.SIGTERM)
                original=self.child.child
                end=time.monotonic()+remaining(8)
                # Drain both pipes without communicate/poll consuming authority.
                with selectors.DefaultSelector() as sel:
                    for tag,stream in [('out',original.stdout),('err',original.stderr)]:
                        if stream is not None and not stream.closed:sel.register(stream,selectors.EVENT_READ,tag)
                    while sel.get_map():
                        need(time.monotonic()<end,'metrics supervisor finish deadline')
                        for key,_ in sel.select(min(.05,max(0,end-time.monotonic()))):
                            chunk=os.read(key.fileobj.fileno(),4096)
                            if not chunk:sel.unregister(key.fileobj);continue
                            target=self.buffer if key.data=='out' else err
                            target.extend(chunk);need(len(target)<=1048576,'metrics supervisor finish capture')
                self.child.wait(timeout=remaining(8))
                for line in self.buffer.splitlines():self.events.append(json.loads(line))
                self.buffer.clear();need(not err,'supervisor stderr must stay empty')
        except BaseException as error:first=error
        finally:
            if self.child is not None:
                try:close_child(self.child.child)
                except BaseException as error:
                    if first is None:first=error
            if self.selector is not None:
                try:self.selector.close()
                except BaseException as error:
                    if first is None:first=error
        if first is not None:raise first
        self.closed=True
        return self.child.returncode if self.child is not None else None

def own_fixture(W,**kwargs):
    value=object.__new__(W['Fixture'])
    running=MetricsRunning()
    def close():
        # Also works if Fixture.__init__ or Running.start partially failed.
        running.finish()
        for port in getattr(value,'ports',[]):
            need(not W['listening'](port),'owned metric supervisor still listening')
    row=register_component(value,close)
    env=dict(W['Fixture'].__init__.__globals__)
    env.update(NATIVE=binary,tempfile=types.SimpleNamespace(TemporaryDirectory=PrivateLease),
        subprocess=types.SimpleNamespace(check_output=lambda argv,timeout: safe_run('binary-version',argv,timeout)))
    init=types.FunctionType(W['Fixture'].__init__.__code__,env,argdefs=W['Fixture'].__init__.__defaults__)
    init(value,**kwargs)
    def start(env=None):
        value.running=running
        return running.start(value.args,env)
    value.start=start
    value.close=lambda:close_component(row)
    return value

def close_pg():
    previous=H['CANCELLED']
    H['CANCELLED']=False
    try:
        if cluster is not None:cluster.close()
    finally:H['CANCELLED']=previous or cancelled or H['CANCELLED']

def close_work():
    if work is None:return
    uncertain=(private_uncertain or cleanup_errors or any(c.returncode is None for c in children)
        or any(not r['closed'] for r in components) or any(not r['closed'] for r in fds)
        or cluster is not None and (not cluster.closed or any(p['child'] is not None and p['child'].returncode is None for p in cluster.probes)))
    if uncertain:
        proof['private_root_preserved_after_uncertain_owner']=True
        raise ValueError('metrics private root retained after uncertain ownership')
    s=work.lstat()
    need(work.resolve(strict=True)==work and (s.st_dev,s.st_ino,s.st_mode,s.st_uid,s.st_gid)==work_identity,'metrics private root identity')
    PG['private_directory'](s);need(time.monotonic()<cleanup_end(),'metrics private removal deadline')
    shutil.rmtree(work)
    need(not work.exists(),'metrics private root remains')

def public_inventory():
    rows=[]
    names=[r['name'] for r in command_logs]
    names+=['native-metadata.jsonl']
    if cluster is not None and cluster.closed:
        names+=['postgres-server.log'] if hasattr(cluster,'log') else []
        names +=[p['log'] for p in cluster.row['readiness'] if 'sha256' in p]
    for name in names:
        path=out/name
        if not path.exists():continue
        raw=PG['bounded_bytes'](path,8*1024**2)
        rows.append({'path':name,'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)})
    return rows

def publish(primary):
    def verdict():
        now=time.monotonic();pg=proof.get('postgres_clusters',[])
        proof.update(cancelled=cancelled or H['CANCELLED'],deadline_expired=now>=deadline,
            cleanup_deadline_expired=cleanup_limit is not None and now>=cleanup_limit,
            cleanup_errors=list(cleanup_errors),private_root_removed=work is not None and not work.exists(),
            all_original_children_reaped=all(c.returncode is not None for c in children),
            all_fds_closed=all(r['closed'] for r in fds),all_components_closed=all(r['closed'] for r in components),
            owned_pg_closed=cluster is not None and cluster.closed,command_logs=command_logs)
        good=(primary is None and proof.get('body_completed') is True and len(proof['cases'])==8
            and [c.get('name') for c in proof['cases']]==EXPECTED_CASES and all(c.get('status')=='passed' for c in proof['cases'])
            and not proof['cancelled'] and not proof['deadline_expired'] and not proof['cleanup_deadline_expired']
            and not cleanup_errors and proof['private_root_removed'] and proof['all_original_children_reaped']
            and proof['all_fds_closed'] and proof['all_components_closed'] and len(pg)==1
            and pg[0].get('normal_shutdown') is True and pg[0].get('fallback_used') is False and pg[0].get('reaped') is True)
        proof['status']='passed' if good else 'failed'
        proof.update(case_count=len(proof['cases']),failed=0 if good else 1,ignored=0)
    def write():
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    try:
        verdict();proof['raw_files']=public_inventory();verdict();write();verdict()
        if proof['status']!='passed':write()
    except BaseException as error:
        if primary is None:primary=error
        proof.update(status='failed',publication_failure=type(error).__name__)
        try:write()
        except BaseException as secondary:proof['failed_write_error']=type(secondary).__name__
    finally:
        errors=[]
        for number,handler in oldsignals.items():
            try:signal.signal(number,handler)
            except BaseException as error:
                errors.append(type(error).__name__)
                if primary is None:primary=error
        if errors:proof['handler_restore_errors']=errors
    try:verdict()
    except BaseException as error:
        if primary is None:primary=error
        proof.update(status='failed',finalization_failure=type(error).__name__)
    if primary is not None or proof['status']!='passed' or proof.get('handler_restore_errors'):
        proof['status']='failed'
        try:write()
        except BaseException as error:
            if primary is None:primary=error
        if primary is not None:raise primary
        raise ValueError('metrics acceptance failed')

class FixtureContext:
    def __init__(self,value):self.value=value
    def __enter__(self):return self.value
    def __exit__(self,kind,value,tb):
        try:self.value.close()
        except BaseException:
            if value is not None:raise value.with_traceback(tb)
            raise
        return False

def spawn_worker(argv,owner_role,**kwargs):
    active()
    child=subprocess.Popen(argv,**kwargs)
    children.append(child)
    row={'role':owner_role,'exit':None,'reaped':False}
    proof.setdefault('worker_owners',[]).append(row);child_records[id(child)]=row
    return OwnedChild(child)

def main():
    global H,PG,root,out,binary,proof,started,deadline,work,work_identity,cluster,campaign,cleaning
    p=argparse.ArgumentParser();p.add_argument('--dil',type=Path,required=True);p.add_argument('--framework-root',type=Path,required=True);p.add_argument('--postgres-bin-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    root=a.framework_root.resolve(strict=True);binary=a.dil.resolve(strict=True)
    need(binary.parent.name in ('release','debug') and binary==root/'compiler/target'/binary.parent.name/'dil','canonical native Cargo binary/profile')
    need(sha(root/'scripts/lifecycle_core.py')=='e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9' and sha(root/'scripts/tests/native-pg-owner.py')=='6e5f004a99af5685bcd9625b73df2ae574f21854f0b0bff06dc8013c18b4d9c2','unchanged shared lifecycle and foreground PG helpers')
    H=runpy.run_path(str(root/'scripts/lifecycle_core.py'));H=H['run'].__globals__
    PG=runpy.run_path(str(root/'scripts/tests/native-pg-owner.py'))
    need(all(H[name].__globals__ is H for name in ('run','held_status','stop_owned')),'actual lifecycle owner globals')
    started=time.monotonic();deadline=started+600
    out=a.output.absolute()
    need(out.parent.resolve(strict=True)==out.parent,'canonical fresh output parent')
    parent_info=out.parent.lstat()
    need(stat.S_ISDIR(parent_info.st_mode) and parent_info.st_uid==os.geteuid(),'owned canonical output parent')
    require(not out.exists(),'fresh output');out.mkdir(mode=0o700);PG['private_directory'](out.lstat())
    source=[root/'scripts/native-metrics.py',root/'scripts/native-capabilities.py',root/'scripts/worker-supervisor.py',root/'scripts/postgresql-tls-ci.py',root/'scripts/postgresql-tls11-control.py',root/'scripts/credential-baslat.py',root/'scripts/tests/worker-supervisor-test.py',Path(__file__).resolve(),root/'compiler/src/web_metrikleri.rs',root/'compiler/src/postgresql/havuz.rs',root/'compiler/src/postgresql/havuz/metrik.rs',root/'compiler/src/postgresql.rs',root/'compiler/src/main.rs',root/'dogfood/zee-cekirdek/metrik/ana.dil',root/'dogfood/zee-cekirdek/metrik/proje.dil']
    hashes=[{'path':str(x.relative_to(root)),'sha256':sha(x)} for x in source]
    proof={'schema':'zee-k257-actual-native-1','status':'running','cases':[],'scope':'native exact transport/status/rate decisions and pool boundary samples; no Prometheus endpoint/clientdelivery/fullP13 acceptance'}
    primary=None
    try:
        install_handlers();active()
        TLS=runpy.run_path(str(root/'scripts/postgresql-tls-ci.py'));API=runpy.run_path(str(root/'scripts/native-metrics.py'))
        cert_env=dict(TLS['certificates'].__globals__)
        cert_env['run']=lambda argv,**kwargs:safe_run('certificate-setup',argv,kwargs.get('timeout',20)).decode()
        certificates=types.FunctionType(TLS['certificates'].__code__,cert_env)
        proof['compiler_identity']=json.loads(safe_run('binary-version',[str(binary),'sürüm','--json'],5))
        proof['binary_sha256']=sha(binary);proof['sources']=hashes
        proof['shared_helpers']={str(path.relative_to(root)):sha(path) for path in (root/'scripts/lifecycle_core.py',root/'scripts/tests/native-pg-owner.py')}
        work=Path(tempfile.mkdtemp(prefix='zmt-',dir=Path('/tmp').resolve(strict=True))).resolve(strict=True)
        PG['private_directory'](work.lstat());info=work.lstat()
        work_identity=(info.st_dev,info.st_ino,info.st_mode,info.st_uid,info.st_gid)
        owned=work;_,_,cert,key=certificates(owned)
        (owned/'pg_hba.conf').write_text('local all all trust\nhost all all 127.0.0.1/32 trust\n')
        campaign=MetricsPgCampaign(owned,a.postgres_bin_dir.resolve(strict=True))
        cluster=PG['OwnedLocalCluster'](campaign,owned,cert,key,a.postgres_bin_dir.resolve(strict=True),H)
        campaign.clusters.append(cluster)
        cluster.start();require(cluster.version=='postgres (PostgreSQL) 16.15','owned PG16.15')
        proof['postgres_version']=cluster.version
        env=os.environ.copy();env['RUSTUP_TOOLCHAIN']='1.93.1';env['ZEE_K257_FIXTURE_HEDEFI']=f'postgresql://localhost:{cluster.port}/postgres';env['ZEE_K257_FIXTURE_URL']=f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require';env['ZEE_K257_FIXTURE_URL_TLS_CA_PEM']=(owned/'ca.crt').read_text()
        command=['cargo','test',*(['--release'] if binary.parent.name=='release' else []),'--locked','--offline','--lib','postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar','--','--exact','--ignored','--nocapture','--test-threads=1']
        result=command_capture(command,120,env,root/'compiler','owned-pg-pool')

        require(result.returncode==0 and b'1 passed; 0 failed; 0 ignored' in result.stdout,'actual concurrent4lease/5thwait/timeout/recovery')
        rows=pool_result(result.stdout);export_pool(result)
        proof['nested_pool_selector']=POOL_SELECTOR;proof['nested_pool_timeout_seconds']=120
        proof['nested_pool_profile']='release' if binary.parent.name=='release' else 'debug'
        require(any(r[:4]==[4,0,4,1] for r in rows) and rows[-1][:4]==[4,4,0,0] and rows[-1][5]==1,'actual owner state records')
        proof['cases'].append({'name':'actual-pg-pool-four-leases-wait-timeout-recovery','status':'passed','snapshots':rows})
        project=owned/'project';project.mkdir(mode=0o700)
        fixture=root/'dogfood/zee-cekirdek/metrik'
        (project/'proje.dil').write_text((fixture/'proje.dil').read_text().replace('postgresql://localhost:5432/postgres',env['ZEE_K257_FIXTURE_HEDEFI']))
        (project/'ana.dil').write_bytes((fixture/'ana.dil').read_bytes())
        check=command_capture([str(binary),'denetle',str(project)],10,env,root/'compiler',None);require(check.returncode==0,'anonymous native source denetle')
        def start(mode='consume'):
            with socket.socket() as free:free.bind(('127.0.0.1',0));port=free.getsockname()[1]
            rd,wr=own_pipe()
            if mode=='full':
                while True:
                    try:os.write(wr,b'x'*512)
                    except BlockingIOError:break
            if mode=='closed':close_fd(rd);rd=None
            child=spawn_worker([str(binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(port),'--web-oran-deposu','bellek','--web-oran-siniri','2','--web-metrik-fd',str(wr),str(project)],env=env,stdin=subprocess.DEVNULL,stdout=wr if mode=='alias' else subprocess.DEVNULL,stderr=wr if mode=='stderr-alias' else subprocess.PIPE,pass_fds=(wr,),start_new_session=True,owner_role=mode)
            close_fd(wr)

            if mode in ('alias','stderr-alias'):
                code=child.wait(timeout=10);error=child.stderr.read() if child.stderr else b''
                channel=os.read(rd,4096)
                require(code==2 and error==b'' and channel==b'','stdio alias quiet rejection')
                require(b'ZEE_K257_PRIVATE_SOURCE_CANARY' not in channel and b'zee-native-metrics-1' not in channel,'early alias rejection before private source diagnostics')
                return child,port,rd
            deadline=time.monotonic()+5
            while True:
                require(child.poll() is None and time.monotonic()<deadline,'actual worker start')
                try:
                    with socket.create_connection(('127.0.0.1',port),timeout=.1):break
                except OSError:time.sleep(.01)
            # The startup connection was EOF-only; it is a native no-response attempt.
            return child,port,rd
        def request(port,path,forward=True):
            with socket.create_connection(('127.0.0.1',port),timeout=5) as s:
                s.settimeout(5);s.sendall(b'GET '+path+b' HTTP/1.1\r\nHost: fixture.example\r\n'+(b'Forwarded: for=127.0.0.1;proto=https;host=fixture.example\r\n' if forward else b'')+b'Connection: close\r\n\r\n');reply=bytearray()
                while True:
                    b=s.recv(1024)
                    if not b:break
                    reply.extend(b);require(len(reply)<=16384,'bounded native response')
                return int(reply.split(b' ',2)[1]),bytes(reply)
        child,port,rd=start();channel=own_channel(API,rd)
        results=[request(port,b'/ok'),request(port,b'/ok'),request(port,b'/ok'),request(port,b'/missing-ZEE_K257_PRIVATE_SENTINEL'),request(port,b'/ok',False),request(port,b'/db')]
        require([r[0] for r in results]==[200,200,429,404,426,200] and results[-1][1].endswith(b'db-ok'),'actual response oracle')
        child.send_signal(signal.SIGTERM);child.wait(timeout=5)
        raw=bytearray()
        while True:
            try:part=os.read(rd,4096)
            except BlockingIOError:break
            if not part:break
            raw.extend(part);require(len(raw)<=32768,'bounded actual native metadata')
        require(b'ZEE_K257_PRIVATE_SENTINEL' not in raw and b'SELECT' not in raw and b'fixture.example' not in raw,'actual content absent from pipe')
        for line in raw.splitlines():API['decode'](line)
        (out/'native-metadata.jsonl').write_bytes(raw)
        channel.buffer.extend(raw);channel.receive();require(channel.valid,'finite native metadata')
        row,db=channel.rows[0],channel.rows[1]
        require(row[0]==row[1]==7 and row[4]==1 and row[5]==1 and row[6]==3 and row[11]==1 and row[15]==1 and row[16]==1,'native accept/EOF/prehandler429/finalstatus counters')
        require(db[0]>=1 and db[1]==db[0] and db[2]==db[3]==0 and db[4]>=2 and db[7]>=50000,'actual checkout and idle safe boundary')
        metadata=json.dumps({'request':row,'pool':db});require('ZEE_K257_PRIVATE_SENTINEL' not in metadata,'metadata secret absence')
        proof['cases'].append({'name':'native-request-status-rate429-db-boundary-no-content','status':'passed','request':row,'pool':db,'body_path_sql_absent':True})

        channel.close()
        child,port,rd=start('full');begin=time.monotonic();codes=[request(port,b'/hazir')[0] for _ in range(2)];require(codes==[200,200] and time.monotonic()-begin<2,'full pipe never delays requests')
        while True:
            try:
                if not os.read(rd,4096):break
            except BlockingIOError:break
        require(request(port,b'/ok')[0]==200,'full pipe recovery');child.send_signal(signal.SIGTERM);child.wait(timeout=5);channel=own_channel(API,rd);channel.receive();require(channel.valid and channel.dropped>=5,'actual dropped snapshots recovery');proof['cases'].append({'name':'actual-native-pipe-backpressure-recovery','status':'passed','codes':codes,'dropped':channel.dropped});channel.close()
        child,port,rd=start('closed');require(request(port,b'/hazir')[0]==200 and child.poll() is None,'closed pipe not SIGPIPE/request failure');child.send_signal(signal.SIGTERM);require(child.wait(timeout=5)==0,'closed pipe graceful exit');proof['cases'].append({'name':'actual-native-closed-reader-no-sigpipe','status':'passed'})
        original_source=(project/'ana.dil').read_bytes()
        (project/'ana.dil').write_text('ZEE_K257_PRIVATE_SOURCE_CANARY !!! malformed native source\n')
        for alias in ('alias','stderr-alias'):
            child,port,rd=start(alias);proof['cases'].append({'name':'actual-native-malformed-source-'+alias+'-early-rejection','status':'passed','private_source_canary_absent':True});close_fd(rd)
        (project/'ana.dil').write_bytes(original_source)
        W=runpy.run_path(str(root/'scripts/tests/worker-supervisor-test.py'));
        owner=own_fixture(W,native=True,app_signal=True,extra=['--runtime-metrics']);owner.args[owner.args.index('--startup-timeout')+1]='15';running=owner.start();running.until(lambda x:x.get('event')=='state' and x.get('ready') is True,timeout=15)
        for port in owner.ports:require(request(port,b'/hazir')[0]==200,'two host workers')
        metrics=running.until(lambda x:x.get('event')=='native_metrics' and x.get('kind')==0 and x['v'][0]>=4,timeout=10)
        require(len(metrics['v'])==11 and metrics['unavailable']==0 and metrics['channels']==2,'owned host finite rollup')
        proof['cases'].append({'name':'actual-two-worker-supervisor-collector','status':'passed','request_prefix':metrics['v'],'channels':2})
        running.finish();owner.close();owner=None;running=None
        # K260 now supports precisely the designated metadata capability.
        with FixtureContext(own_fixture(W,native=True,credential=True,extra=['--runtime-metrics'])) as f:
            f.args[f.args.index('--startup-timeout')+1]='15'
            r=f.start();r.until(lambda x:x.get('event')=='state' and x.get('ready') is True,timeout=15)
            for port in f.ports:require(request(port,b'/hazir')[0]==200,'credential native workers healthy')
            rollup=r.until(lambda x:x.get('event')=='native_metrics' and x.get('kind')==0 and x['v'][0]>=4,timeout=10)
            require(rollup['channels']==2 and rollup['unavailable']==0,'credential designated capability rollup')
            require(r.finish()==0,'credential metric worker groups gracefully closed')
            require(W['CANARY'] not in json.dumps(r.events),'credential plaintext absent from host metadata')
        proof['cases'].append({'name':'credential-designated-metadata-composition-two-workers','status':'passed','channels':2})
        cluster.close();require(not (cluster.data/'postmaster.pid').exists(),'owned cluster stopped');
        require(hashes==[{'path':str(x.relative_to(root)),'sha256':sha(x)} for x in source] and sha(binary)==proof['binary_sha256'],'source/binary unchanged')

        proof['body_completed']=True
    except BaseException as error:
        primary=error;proof.update(status='failed',failure_class=type(error).__name__)
    finally:
        cleaning=True;cleanup_end()
        try:cleanup_all(close_components,close_children,close_fds,close_pg,close_work)
        except BaseException as error:
            if primary is None:primary=error
        publish(primary)
    print(json.dumps({'status':proof['status'],'cases':proof['case_count'],'proof_sha256':sha(out/'proof.json')}))

if __name__=='__main__':main()
