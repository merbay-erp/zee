#!/usr/bin/env python3
"""K292: actual one-native-worker loss, sibling cleanup and stale gate refusal."""
import argparse,hashlib,importlib.util,json,os,runpy,signal,socket,subprocess,sys,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('fault_mixed_support',HERE/'mixed-revision-native.py');P=importlib.util.module_from_spec(spec);spec.loader.exec_module(P)

class Campaign:
    def __init__(self,args):
        self.end=time.monotonic()+120;self.a=args;self.owner=None;self.foreign=None;self.fixture=None;self.handlers={};self.stop=False;self.events=[];self.cases=[];self.output_created=False
        P.need(os.name=='posix' and os.geteuid()!=0 and P.re.fullmatch('[0-9a-f]{40}',args.sha)and P.re.fullmatch('[0-9a-f]{64}',args.binary_sha256))
        self.repo=Path(args.repo);self.binary=Path(args.dil);self.out=Path(args.out)
        for p in (self.repo,self.binary,self.out):P.need(p.is_absolute()and p.resolve()==p)
        P.need(not self.out.exists()and self.out.parent.stat().st_uid==os.geteuid()and self.out.parent.stat().st_mode&0o077==0 and self.repo not in self.out.parents and self.out not in self.repo.parents)
        self.H=P.load('fault_lifecycle',self.repo/'scripts/lifecycle_core.py')
        self.proof={'schema':'zee-service-fault-native-1','status':'running','source_sha':args.sha,'binary_sha256':args.binary_sha256,'cases':self.cases,'native_DNS_cancellation_claim':False,'boundary':'own Popen/native worker and readiness boundaries only; not systemd or provider/global fault matrix'}
    def prepare(self):
        # Constructor validation has no child processes. All effectful setup
        # is covered by main's finally, with handlers before the first H.run.
        for n in (signal.SIGTERM,signal.SIGINT):self.handlers[n]=signal.signal(n,self.cancel)
        P.need(not self.stop and time.monotonic()<self.end)
        self.H.clean_exact(self.repo,self.a.sha);P.need(P.snapshot(self.binary)==self.a.binary_sha256)
        self.W=P.load('fault_owner',self.repo/'scripts/worker-supervisor.py')
        P.need(not self.stop and time.monotonic()<self.end)
        self.out.mkdir(mode=0o700);self.output_created=True
        self.sources={str(p.relative_to(self.repo)):P.snapshot(p,262144)for p in [Path(__file__).resolve(),HERE/'mixed-revision-native.py',self.repo/'scripts/worker-supervisor.py',self.repo/'scripts/lifecycle_core.py',self.repo/'scripts/tests/worker-supervisor-test.py']}
        self.proof['source_pins_start']=self.sources;self.proof['generated_fixture_sha256']={}
    def cancel(self,n,f):
        self.stop=True;self.H.CANCELLED=True
        if self.owner is not None:self.owner.signal(n,f)
    def fixture_owned(self,module,**kwargs):
        # Override only this borrowed fixture's subprocess binding, never the global module.
        namespace=module['Fixture'].__init__.__globals__;original=namespace['subprocess'];campaign=self
        class Queries:
            def check_output(self,argv,**options):
                P.need(not campaign.stop and time.monotonic()<campaign.end and options=={'timeout':5})
                return campaign.H.run(argv,timeout=min(5,campaign.end-time.monotonic()),limit=8192)
            def __getattr__(self,name):return getattr(original,name)
        namespace['subprocess']=Queries()
        fixture=None
        try:
            fixture=module['Fixture'].__new__(module['Fixture'])
            module['Fixture'].__init__(fixture,**kwargs)
            return fixture
        except BaseException:
            # The borrowed constructor performs only its owned identity query
            # before returning; no worker has been started at this point.
            if fixture is not None and hasattr(fixture,'temporary'):fixture.temporary.cleanup()
            raise
        finally:namespace['subprocess']=original
    def main(self):
        P.need(not self.stop and time.monotonic()<self.end);identity=json.loads(self.H.run([self.binary,'sürüm','--json'],timeout=min(10,self.end-time.monotonic()),limit=8192));P.need(identity['source_sha']==self.a.sha and identity['dirty']is False);self.proof['build']=identity
        fixture_module=runpy.run_path(str(self.repo/'scripts/tests/worker-supervisor-test.py'));fixture_module['Fixture'].__init__.__globals__['NATIVE']=self.binary
        P.need(not self.stop and time.monotonic()<self.end)
        self.foreign=subprocess.Popen([sys.executable,'-c','import time;time.sleep(120)'],start_new_session=True)
        P.need(not self.stop and time.monotonic()<self.end)
        campaign=self
        class Log(self.W.JsonLog):
            def emit(self,event,**fields):
                P.need(not campaign.stop and time.monotonic()<campaign.end)
                if event=='state':
                    P.need(len(campaign.events)<64);campaign.events.append({k:v for k,v in {'event':event,**fields}.items()if k in ('event','state','ready','application_ready','healthy_workers','workers')})
        Base=self.W.Supervisor
        class Fault(Base):
            def __init__(self,a,log):super().__init__(a,log);self.fired=False;self.workers=[]
            def live(self,workers):
                super().live(workers)
                if not self.fired and len(workers)==2 and self.application_ready()and all(self.probe(port)for port in self.a.worker_port):
                    self.workers=list(workers);self.fired=True
                    victim=workers[0];P.need(victim in self.children and self.exit_status(victim)is None)
                    os.kill(victim.pid,signal.SIGKILL)
        self.fixture=self.fixture_owned(fixture_module,native=True,ready=True)
        for n in ('program.dil','proje.dil'):
            (self.out/('loss-'+n)).write_bytes((self.fixture.project/n).read_bytes());self.proof['generated_fixture_sha256']['loss-'+n]=P.snapshot(self.fixture.project/n)
        args=self.W.arguments(self.fixture.args[4:]);self.owner=Fault(args,Log());reason=None
        try:
            P.need(not self.stop and time.monotonic()<self.end)
            self.owner.run()
        except self.W.Reject as e:reason=e.code
        finally:self.owner.cleanup()
        P.need(reason=='worker_exit'and self.owner.fired and len(self.owner.workers)==2 and all(p.returncode is not None for p in self.owner.workers)and not self.owner.children)
        P.need(self.owner.workers[0].returncode==-signal.SIGKILL and self.H.held_status(self.foreign)is None)
        P.need(all(not fixture_module['listening'](port)for port in self.fixture.ports));self.cases.append({'name':'actual-one-native-worker-SIGKILL-sibling-cleanup-foreign-survival','passed':True,'native_workers':2,'victim_exit':-signal.SIGKILL,'all_owned_reaped':True,'foreign_alive':True,'no_PID_discovery':True})
        P.need(all(P.snapshot(self.fixture.project/n)==self.proof['generated_fixture_sha256']['loss-'+n]for n in ('program.dil','proje.dil')))
        self.fixture.close();self.fixture=None;self.owner=None
        # Fresh liveness-only children must not borrow a true gate written before their owner epoch.
        self.fixture=self.fixture_owned(fixture_module,native=True,ready=False);fixture_module['gate'](self.fixture.gate,True);old=self.fixture.gate.read_bytes();old_ns=self.fixture.gate.stat().st_mtime_ns
        for n in ('program.dil','proje.dil'):
            (self.out/('stale-'+n)).write_bytes((self.fixture.project/n).read_bytes());self.proof['generated_fixture_sha256']['stale-'+n]=P.snapshot(self.fixture.project/n)
        class Stale(Base):
            def __init__(self,a,log):super().__init__(a,log);self.healthy_workers_seen=0
            def live(self,workers):
                super().live(workers)
                if len(workers)==2 and all(self.probe(port)for port in self.a.worker_port):self.healthy_workers_seen=2
        self.events.clear();self.owner=Stale(self.W.arguments(self.fixture.args[4:]),Log());reason=None
        try:
            P.need(not self.stop and time.monotonic()<self.end)
            self.owner.run()
        except self.W.Reject as e:reason=e.code
        finally:self.owner.cleanup()
        P.need(reason=='startup'and not any(e.get('ready')is True for e in self.events)and self.owner.healthy_workers_seen==2)
        P.need(self.fixture.gate.read_bytes()==old and self.fixture.gate.stat().st_mtime_ns==old_ns and not self.owner.children and self.H.held_status(self.foreign)is None)
        self.cases.append({'name':'actual-two-native-liveness200-stale-true-gate-refused','passed':True,'healthy_workers':2,'healthy_observation':'build-validated-direct-probe','ready_true_events':0,'old_gate_unchanged':True,'owned_reaped':True})
        P.need(all(P.snapshot(self.fixture.project/n)==self.proof['generated_fixture_sha256']['stale-'+n]for n in ('program.dil','proje.dil')))
        self.fixture.close();self.fixture=None;self.owner=None
        self.H.clean_exact(self.repo,self.a.sha);P.need(P.snapshot(self.binary)==self.a.binary_sha256 and all(P.snapshot(self.repo/n,262144)==v for n,v in self.sources.items()));self.proof.update(status='cases-passed-pending-cleanup',source_pins_end_equal=True,binary_end_equal=True)
    def close(self):
        # Handlers only set flags, including during cleanup. Do not clear a
        # cancellation or allow it to become a passed campaign receipt.
        errors=[]
        try:
            if self.owner is not None:
                try:self.owner.cleanup()
                except BaseException as e:errors.append(type(e).__name__)
            if self.fixture is not None:
                try:self.fixture.close()
                except BaseException as e:errors.append(type(e).__name__)
            if self.foreign is not None:
                try:self.H.stop_owned(self.foreign,grace=.2)
                except BaseException as e:errors.append(type(e).__name__)
            self.proof.update(status='passed'if self.proof['status']=='cases-passed-pending-cleanup'and not self.stop and not errors else 'failed',cancelled=self.stop,cleanup_errors=errors,foreign_owned_sentinel_reaped=self.foreign is None or self.foreign.returncode is not None)
            if self.output_created:(self.out/'proof.json').write_text(json.dumps(self.proof,indent=2)+'\n')
        finally:
            for n,h in self.handlers.items():signal.signal(n,h)

def main():
    p=P.Parser()
    for name in ('repo','dil','out'):p.add_argument('--'+name,type=Path,required=True)
    for name in ('sha','binary-sha256'):p.add_argument('--'+name,required=True)
    c=Campaign(p.parse_args())
    try:
        c.prepare()
        c.main()
    finally:c.close()
    P.need(c.proof['status']=='passed')
if __name__=='__main__':
    try:main()
    except (Exception,KeyboardInterrupt):sys.stderr.write('owned-service-fault-campaign-failed\n');raise SystemExit(2)
