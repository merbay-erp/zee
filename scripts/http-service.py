#!/usr/bin/env python3
"""K283: continuous watcher plus existing two-worker host, explicit Linux user service."""
import argparse,hashlib,importlib.util,json,math,os,re,signal,stat,sys,time
from pathlib import Path
HERE=Path(__file__).resolve().parent

def load(name,path):
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        st=os.fstat(fd)
        if not stat.S_ISREG(st.st_mode) or st.st_size>262144:raise ValueError('module')
        raw=bytearray()
        while len(raw)<=262144:
            block=os.read(fd,4096)
            if not block:break
            raw.extend(block)
        if len(raw)!=st.st_size or len(raw)>262144:raise ValueError('module')
    finally:os.close(fd)
    spec=importlib.util.spec_from_loader(name,loader=None);m=importlib.util.module_from_spec(spec);m.__file__=str(path);exec(compile(bytes(raw),str(path),'exec'),m.__dict__);return m
D=T=W=None
try:
    D=load('http_dispatch_support',HERE/'dispatcher-service.py');T=D.T
    W=load('http_worker_support',HERE/'worker-supervisor.py')
except Exception:pass

def need(ok):
    if not ok:raise ValueError('service refused')
class Parser(argparse.ArgumentParser):
    def error(self,message):raise ValueError('arguments')
    def exit(self,status=0,message=None):raise ValueError('arguments')

def configuration(path):
    raw=T.read_private(path);c=T.decode(raw)
    keys={'schema','repo','source_sha','dil','binary_sha256','project','project_files','probe_project','probe_files','contract','contract_sha256','application_revision','origin','worker_ports','readiness_file','health_path','health_body','state','watch_interval','probe_deadline','readiness_ttl','credential_root','credential_env','probe_credential_root','probe_credential_name','probe_ca','probe_ca_sha256'}
    need(type(c)is dict and set(c) in (keys,keys|{'allow_development'}) and c['schema']=='zee-http-service-1')
    c.setdefault('allow_development',False);need(type(c['allow_development'])is bool)
    for k,n in [('source_sha',40),('application_revision',40),('binary_sha256',64),('contract_sha256',64)]:need(type(c[k])is str and re.fullmatch('[0-9a-f]{'+str(n)+'}',c[k]) is not None)
    for k in ('repo','dil','project','probe_project','contract','readiness_file','state','probe_credential_root','probe_ca'):c[k]=T.canonical(c[k])
    need(c['repo'].is_dir() and c['project'].is_dir() and c['probe_project'].is_dir())
    T.private_directory(c['state'].parent.stat())
    for base in (c['repo'],c['project'],c['probe_project']):need(base!=c['state'] and base not in c['state'].parents and c['state'] not in base.parents)
    need(c['project'] in c['readiness_file'].parents)
    for key,base in [('project_files',c['project']),('probe_files',c['probe_project'])]:
        inventory=c[key];need(type(inventory)is dict and 1<=len(inventory)<=128 and 'proje.dil' in inventory)
        for name,h in inventory.items():
            need(type(name)is str and re.fullmatch('[A-Za-z0-9_./-]{1,256}',name) is not None and not Path(name).is_absolute() and '..' not in Path(name).parts and '.' not in name.split('/') and '//' not in name)
            need(type(h)is str and re.fullmatch('[0-9a-f]{64}',h) is not None and (base/Path(name)).resolve()==base/Path(name))
    for k,lo,hi in [('watch_interval',2,60),('probe_deadline',.1,15),('readiness_ttl',1,3600)]:need(type(c[k])in (int,float) and math.isfinite(c[k]) and lo<=c[k]<=hi)
    need(c['readiness_ttl']>=c['watch_interval']+c['probe_deadline']+2)
    need(type(c['worker_ports'])is list and len(c['worker_ports'])==2 and len(set(c['worker_ports']))==2 and all(type(x)is int and 1<=x<=65535 for x in c['worker_ports']))
    need(type(c['credential_env'])is list and len(c['credential_env'])<=16 and all(type(x)is str for x in c['credential_env']))
    need((c['credential_root'] is None)==(not c['credential_env']))
    if c['credential_root'] is not None:c['credential_root']=T.canonical(c['credential_root'])
    need(type(c['probe_credential_name'])is str and re.fullmatch('[A-Za-z0-9_][A-Za-z0-9_.-]{0,63}',c['probe_credential_name']) is not None)
    need(type(c['probe_ca_sha256'])is str and re.fullmatch('[0-9a-f]{64}',c['probe_ca_sha256']) is not None)
    c['_configuration']=T.canonical(path);c['_configuration_sha256']=hashlib.sha256(raw).hexdigest()
    # Reuse exact existing public worker profile validation, without executing children.
    worker_arguments(c)
    return c

def worker_arguments(c):
    a=['--dil',str(c['dil']),'--project',str(c['project']),'--origin',c['origin'],'--expected-source',c['source_sha'],'--application-revision',c['application_revision'],'--readiness-file',str(c['readiness_file']),'--health-path',c['health_path'],'--health-body',c['health_body'],'--readiness-ttl-seconds',str(c['readiness_ttl']),'--stop-grace','55','--runtime-metrics']
    if c.get('allow_development',False):a+=['--pilot']
    for port in c['worker_ports']:a+=['--worker-port',str(port)]
    if c['credential_root'] is not None:
        a+=['--credential-root',str(c['credential_root'])]
        for binding in c['credential_env']:a+=['--credential-env',binding]
    return W.arguments(a)

def pins(c):
    need(hashlib.sha256(T.read_private(c['_configuration'])).hexdigest()==c['_configuration_sha256'])
    T.D.clean_exact(c['repo'],c['source_sha']);need(D.snapshot(c['dil'])==c['binary_sha256'])
    result={n:D.snapshot(HERE/n,262144) for n in ('http-service.py','dispatcher-service.py','finite_timer.py','lifecycle_core.py','worker-supervisor.py','db-hazirlik.py','native-metrics.py','credential-baslat.py','native-capabilities.py')}
    total=0
    for key,base in [('project_files',c['project']),('probe_files',c['probe_project'])]:
        for name,h in c[key].items():
            file=base/name;actual=D.snapshot(file,262144);need(actual==h);total+=file.stat().st_size;need(total<=2097152);result[key+'/'+name]=h
    need(D.snapshot(c['contract'],65536)==c['contract_sha256']);need(D.snapshot(c['probe_ca'],16384)==c['probe_ca_sha256']);return result

def watcher_argv(c):
    return [sys.executable,'-I','-B',str(HERE/'db-hazirlik.py'),'--dil',str(c['dil']),'--project',str(c['probe_project']),'--worker-project',str(c['project']),'--gate',str(c['readiness_file']),'--expected-source',c['source_sha'],'--application-revision',c['application_revision'],'--contract',str(c['contract']),'--contract-sha256',c['contract_sha256'],'--deadline',str(c['probe_deadline']),'--interval',str(c['watch_interval']),'--continuous']

def watcher_environment(c):
    credential=load('http_probe_credentials',HERE/'credential-baslat.py')
    need('ZEE_READINESS_DATABASE_URL_TLS_CA_PEM' not in os.environ)
    root,bindings,_=credential.parse_arguments(['--credential-root',str(c['probe_credential_root']),'--env','ZEE_READINESS_DATABASE_URL='+c['probe_credential_name'],'--dil',str(c['dil']),'--','çalıştır',str(c['probe_project'])])
    mapped=credential.child_environment(root,bindings)
    env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','LC_ALL':'C.UTF-8','TZ':'UTC','ZEE_READINESS_DATABASE_URL':mapped['ZEE_READINESS_DATABASE_URL']}
    mapped.clear()
    raw=T.read_private(c['probe_ca']);need(hashlib.sha256(raw).hexdigest()==c['probe_ca_sha256'])
    env['ZEE_READINESS_DATABASE_URL_TLS_CA_PEM']=raw.decode('ascii')
    return env

# Subclass only existing lifecycle/health owner; no alternate process discovery.
if W is not None:
    class Service(W.Supervisor):
        def __init__(self,c,anchor,log):
            super().__init__(worker_arguments(c),log);self.c=c;self.anchor=anchor;self.watcher=None
            paths=[c['_configuration'],c['dil'],c['contract'],c['probe_ca'],*[HERE/n for n in ('http-service.py','dispatcher-service.py','finite_timer.py','lifecycle_core.py','worker-supervisor.py','db-hazirlik.py','native-metrics.py','credential-baslat.py','native-capabilities.py')],*[c['project']/n for n in c['project_files']],*[c['probe_project']/n for n in c['probe_files']]]
            self.file_keys={p:T.identity(p.lstat()) for p in paths}
        def query_json(self,option):
            # Identity-only query groups have no admitted HTTP request to drain.
            # Keep the native worker55s contract; query descendants get a separate1s cleanup budget.
            previous=self.a.stop_grace
            try:
                self.a.stop_grace=1
                record=super().query_json(option)
                if option=='--json':need(type(record)is dict and record.get('dirty')is False and record.get('source_sha')==self.c['source_sha'])
                return record
            finally:self.a.stop_grace=previous
        def live(self,workers):
            self.anchor.check();need(all(T.identity(p.lstat())==key for p,key in self.file_keys.items()));super().live(workers)
            if self.watcher is not None:W.require(self.exit_status(self.watcher)is None,'watcher_exit')
        def run(self):
            environment=watcher_environment(self.c)
            try:self.watcher=self.spawn(watcher_argv(self.c),environment=environment)
            finally:environment.clear()
            return super().run()


def execute(c):
    initial=pins(c);service=None;previous={};clean=False
    with T.locked(c['state']) as anchor:
        try:
            log=W.JsonLog();service=Service(c,anchor,log)
            for number in (signal.SIGTERM,signal.SIGINT,signal.SIGHUP):previous[number]=signal.signal(number,service.signal)
            service.run();anchor.check();need(pins(c)==initial);clean=True
        finally:
            # Flag-only handlers remain installed through every cleanup attempt.
            try:
                if service is not None:service.cleanup()
            finally:
                for number,handler in previous.items():signal.signal(number,handler)
        if clean:T.store(anchor.fd,'last.json',{'schema':'zee-http-service-stop-1','source_sha':c['source_sha'],'owned_cleanup':True,'source_pins_unchanged':True})
    return 0

def render(config,out):
    paths=[Path(sys.executable).resolve(),Path(__file__).resolve(),T.canonical(config)]
    need(all(re.fullmatch('/[A-Za-z0-9_./-]+',str(p))for p in paths));python,runner,config=paths
    # Type=simple says process started, NOT application/DB readiness.
    unit='[Unit]\nDescription=Zee owned continuous HTTP workers and DB watch\n[Service]\nType=simple\nUMask=0077\nExecStart='+str(python)+' -I -B '+str(runner)+' --config '+str(config)+' --run\nExecReload=/bin/kill -HUP $MAINPID\nRestart=on-failure\nRestartPreventExitStatus=2 75\nRestartSec=5\nKillMode=mixed\nTimeoutStopSec=210\nSendSIGKILL=yes\nNoNewPrivileges=yes\nStandardInput=null\nStandardOutput=null\nStandardError=null\n'
    out=T.canonical(out);T.private_directory(out.parent.stat());need(not out.exists());out.mkdir(mode=0o700);(out/'zee-http.service').write_text(unit)

def main(argv=None):
    need(D is not None and T is not None and T.D is not None and W is not None and os.name=='posix' and os.geteuid()!=0)
    p=Parser(add_help=False,allow_abbrev=False);p.add_argument('--config',type=Path,required=True);g=p.add_mutually_exclusive_group();g.add_argument('--run',action='store_true');g.add_argument('--render',type=Path);a=p.parse_args(argv)
    previous={};T.D.CANCELLED=False
    def cancelled(number,frame):
        if number!=signal.SIGHUP:T.D.CANCELLED=True
    try:
        for number in (signal.SIGTERM,signal.SIGINT,signal.SIGHUP):previous[number]=signal.signal(number,cancelled)
        c=configuration(a.config);pins(c);need(not T.D.CANCELLED)
        if a.render:render(a.config,a.render);return 0
        if not a.run:return 0
        need(sys.platform=='linux')
        try:return execute(c)
        except T.Busy:return 75
        except (Exception,KeyboardInterrupt):return 1
    finally:
        for number,handler in previous.items():signal.signal(number,handler)
if __name__=='__main__':
    try:raise SystemExit(main())
    except (Exception,KeyboardInterrupt):raise SystemExit(2)
