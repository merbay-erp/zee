#!/usr/bin/env python3
"""K283 preparation: real owned process cleanup; HTTP/PG/user-unit acceptance separate."""
import contextlib,hashlib,importlib.util,io,json,os,runpy,signal,socket,sys,tempfile,time,unittest
from pathlib import Path
from unittest import mock
ROOT=Path(__file__).resolve().parents[1];spec=importlib.util.spec_from_file_location('http_service',ROOT/'http-service.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
A=runpy.run_path(str(ROOT/'db-hazirlik.py'));G=A['main'].__globals__

class Cases(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name).resolve();self.root.chmod(0o700)
        self.project=self.root/'worker';self.project.mkdir(mode=0o700);(self.project/'.zee').mkdir(mode=0o700)
        self.probe=self.root/'probe';self.probe.mkdir(mode=0o700)
        for p in (self.project,self.probe):
            (p/'ana.dil').write_text('"anonymous" yaz\n');(p/'proje.dil').write_text('proje "anonymous" olsun\n')
        self.contract=self.root/'contract.json';self.contract.write_text('{}')
        self.credentials=self.root/'credentials';self.credentials.mkdir(mode=0o700);(self.credentials/'db').write_text('synthetic');(self.credentials/'db').chmod(0o600)
        self.ca=self.root/'ca.pem';self.ca.write_text('public fixture CA');self.ca.chmod(0o600)
        self.repo=self.root/'repo';self.repo.mkdir();self.file=self.root/'config.json'
        ports=[]
        for _ in range(2):
            with socket.socket() as sock:sock.bind(('127.0.0.1',0));ports.append(sock.getsockname()[1])
        binary=Path(sys.executable).resolve()
        self.value={'schema':'zee-http-service-1','repo':str(self.repo),'source_sha':'1'*40,'dil':str(binary),'binary_sha256':self.sha(binary),'project':str(self.project),'project_files':{x.name:self.sha(x)for x in self.project.iterdir()if x.is_file()},'probe_project':str(self.probe),'probe_files':{x.name:self.sha(x)for x in self.probe.iterdir()},'contract':str(self.contract),'contract_sha256':self.sha(self.contract),'application_revision':'2'*40,'origin':'https://fixture.example','worker_ports':ports,'readiness_file':str(self.project/'.zee/ready.json'),'health_path':'/health','health_body':'ok','state':str(self.root/'state'),'watch_interval':2,'probe_deadline':1,'readiness_ttl':6,'credential_root':None,'credential_env':[],'probe_credential_root':str(self.credentials),'probe_credential_name':'db','probe_ca':str(self.ca),'probe_ca_sha256':self.sha(self.ca)}
        self.write();M.T.D.CANCELLED=False
    def tearDown(self):M.T.D.CANCELLED=False;self.temp.cleanup()
    def sha(self,p):return hashlib.sha256(p.read_bytes()).hexdigest()
    def write(self):self.file.write_text(json.dumps(self.value));self.file.chmod(0o600)
    def config(self):return M.configuration(self.file)
    def test_default_check_has_no_child_or_state(self):
        with mock.patch.object(M,'pins'),mock.patch.object(M,'execute',side_effect=AssertionError('must not execute')):self.assertEqual(M.main(['--config',str(self.file)]),0)
        self.assertFalse((self.root/'state').exists())
    def test_strict_paths_source_inventory_and_ttl_budget(self):
        for field,value in [('readiness_ttl',2),('worker_ports',[10,10]),('project_files',{'../foreign':'0'*64}),('credential_env',['PRIVATE=secret'])]:
            old=self.value[field];self.value[field]=value;self.write()
            with self.assertRaises((ValueError,M.W.Reject)):self.config()
            self.value[field]=old
    def test_source_or_contract_drift_refuses_before_spawn(self):
        c=self.config();self.contract.write_text('changed')
        with mock.patch.object(M.T.D,'clean_exact'),self.assertRaises(ValueError):M.pins(c)
    def test_render_is_simple_not_notify_and_has_reload_and_cgroup_budget(self):
        M.render(self.file,self.root/'units');unit=(self.root/'units/zee-http.service').read_text()
        for text in ('Type=simple','ExecReload=/bin/kill -HUP $MAINPID','KillMode=mixed','TimeoutStopSec=210','NoNewPrivileges=yes'):self.assertIn(text,unit)
        self.assertNotIn('WantedBy',unit)
    def test_actual_watcher_death_cleans_all_owned_workers_foreign_survives(self):
        c=self.config();children=[];original=M.W.Supervisor.spawn
        # K-319: yavaş CI'da test 30 sn'yi aşınca yabancı süreç kendiliğinden bitip 'öldürüldü' sanılıyordu; finally temizler.
        foreign=M.T.D.subprocess.Popen([sys.executable,'-c','import time;time.sleep(600)'],start_new_session=True)
        def spawn(owner,*args,**kw):child=original(owner,*args,**kw);children.append(child);return child
        def run(owner):
            workers=[owner.spawn([sys.executable,'-c','import time;time.sleep(30)'])for _ in range(2)]
            owner.live(workers);os.kill(owner.watcher.pid,signal.SIGTERM)
            deadline=time.monotonic()+2
            while owner.exit_status(owner.watcher)is None:
                self.assertLess(time.monotonic(),deadline);time.sleep(.01)
            owner.live(workers)
        try:
            with mock.patch.object(M,'pins',return_value={}),mock.patch.object(M,'watcher_argv',return_value=[sys.executable,'-c','import time;time.sleep(30)']),mock.patch.object(M.W.Supervisor,'spawn',spawn),mock.patch.object(M.W.Supervisor,'run',run),self.assertRaises(M.W.Reject):M.execute(c)
            self.assertEqual(len(children),3);self.assertTrue(all(p.returncode is not None for p in children));self.assertIsNone(M.T.D.held_status(foreign))
        finally:M.T.D.stop_owned(foreign,grace=.2)
    def test_signal_during_setup_does_not_launch_worker(self):
        def cancelled(c):os.kill(os.getpid(),signal.SIGTERM)
        with mock.patch.object(M,'pins',side_effect=cancelled),mock.patch.object(M,'execute',side_effect=AssertionError('no child')),self.assertRaises(ValueError):M.main(['--config',str(self.file),'--run'])
    def test_cleanup_failure_cannot_publish_success(self):
        c=self.config();previous=signal.getsignal(signal.SIGTERM)
        with mock.patch.object(M,'pins',return_value={}),mock.patch.object(M.Service,'run'),mock.patch.object(M.Service,'cleanup',side_effect=OSError('injected')),self.assertRaises(OSError):M.execute(c)
        self.assertEqual(signal.getsignal(signal.SIGTERM),previous)
        self.assertFalse((self.root/'state/last.json').exists())
    def test_probe_environment_no_ambient_secret_forwarding(self):
        c=self.config()
        with mock.patch.dict(os.environ,{'K283_AMBIENT_PRIVATE_CANARY':'must-not-forward'}):
            env=M.watcher_environment(c)
        self.assertNotIn('K283_AMBIENT_PRIVATE_CANARY',env)
        self.assertEqual(set(env),{'PATH','LANG','LC_ALL','TZ','ZEE_READINESS_DATABASE_URL','ZEE_READINESS_DATABASE_URL_TLS_CA_PEM'})
        with mock.patch.dict(os.environ,{'ZEE_READINESS_DATABASE_URL':'forbidden'}),self.assertRaises(Exception):M.watcher_environment(c)
    def test_query_budget_restored_after_identity_failure(self):
        c=self.config()
        with M.T.locked(c['state'])as anchor:
            service=M.Service(c,anchor,M.W.JsonLog())
            def query(owner,*args):
                self.assertEqual(owner.a.stop_grace,1);raise ValueError('modeled failed identity')
            with mock.patch.object(M.W.Supervisor,'query_json',query),self.assertRaises(ValueError):service.query_json('--json')
            self.assertEqual(service.a.stop_grace,55)
    def test_native_driver_unsupported_preflight_has_no_output_or_commands(self):
        support=runpy.run_path(str(ROOT/'tests/service-native-support.py'));args=type('Args',(),{'sha':'1'*40,'binary_sha256':'2'*64})()
        with mock.patch.object(support['sys'],'platform','unsupported'),mock.patch.object(support['D'].T.D,'run',side_effect=AssertionError('no command')),self.assertRaises(ValueError):support['Campaign'](args,'test')
    def test_native_driver_replaced_link_is_not_deleted(self):
        support=runpy.run_path(str(ROOT/'tests/service-native-support.py'));campaign=support['Campaign'].__new__(support['Campaign']);campaign.stopped=False;campaign.deadline=time.monotonic()+5
        target=self.root/'unit';target.write_text('owned');foreign=self.root/'foreign';foreign.write_text('foreign');link=self.root/'link';link.symlink_to(target);st=link.lstat()
        campaign.units={'zee-http.service':True};campaign.links=[{'path':link,'target':target,'identity':(st.st_dev,st.st_ino),'sha256':self.sha(target)}]
        link.unlink();link.symlink_to(foreign)
        with mock.patch.object(campaign,'stop'),self.assertRaises(ValueError):campaign.unlink()
        self.assertTrue(link.is_symlink());self.assertEqual(foreign.read_text(),'foreign')
    def test_cgroup_scan_caps_all_entries_before_eager_allocation(self):
        support=runpy.run_path(str(ROOT/'tests/service-native-support.py'));campaign=support['Campaign'].__new__(support['Campaign']);campaign.stopped=False;campaign.deadline=time.monotonic()+5
        base=self.root/'cgroup';group=base/'user.slice/owned';group.mkdir(parents=True)
        for n in range(257):(group/str(n)).write_text('')
        realpath=Path
        def path(value):return base if value=='/sys/fs/cgroup' else realpath(value)
        with mock.patch.object(campaign,'system',return_value=b'/user.slice/owned'),mock.patch.dict(campaign.empty.__globals__,Path=path),self.assertRaises(ValueError):campaign.empty('zee-http.service')
    def test_native_driver_cleanup_failure_requires_nonzero_campaign(self):
        support=runpy.run_path(str(ROOT/'tests/service-native-support.py'));campaign=support['Campaign'].__new__(support['Campaign']);campaign.proof={'status':'failed'}
        with mock.patch.object(campaign,'close'),self.assertRaises(ValueError):campaign.__exit__(None,None,None)
    def test_explicit_development_optin_never_allows_dirty_build(self):
        for flag in (False,True):
            self.value['allow_development']=flag;self.write();c=self.config();self.assertEqual(M.worker_arguments(c).pilot,flag)
            with M.T.locked(c['state'])as anchor:
                service=M.Service(c,anchor,M.W.JsonLog())
                with mock.patch.object(M.W.Supervisor,'query_json',return_value={'dirty':True,'source_sha':c['source_sha']}),self.assertRaises(ValueError):service.query_json('--json')
                with mock.patch.object(M.W.Supervisor,'query_json',return_value={'dirty':False,'source_sha':c['source_sha']}):self.assertFalse(service.query_json('--json')['dirty'])
        del self.value['allow_development'];self.write();self.assertFalse(self.config()['allow_development'])
    def test_quiet_invalid_cli_subprocess(self):
        r=M.T.D.subprocess.run([sys.executable,'-I','-B',str(ROOT/'http-service.py'),'--PRIVATE_CANARY'],capture_output=True,timeout=5)
        self.assertEqual(r.returncode,2);self.assertEqual(r.stdout+r.stderr,b'')
    def watch(self,probe,extra=()):
        calls=[]
        class Gate:
            def __init__(self,*a):pass
            def write(self,value):calls.append(value)
            def close(self):calls.append('closed')
        argv=['--dil','/bin/true','--project','/tmp','--worker-project','/tmp','--gate','/tmp/ready.json','--expected-source','1'*40,'--application-revision','2'*40,'--contract','/tmp/contract.json','--contract-sha256','3'*64,'--continuous',*extra]
        out=io.StringIO()
        with mock.patch.dict(G,Gate=Gate,canonical_directory=lambda p:Path(p),probe=probe),contextlib.redirect_stdout(out):code=A['main'](argv)
        return code,json.loads(out.getvalue()),calls
    def test_continuous_actual_term_final_false_restores_handler(self):
        old=signal.getsignal(signal.SIGTERM)
        def probe(a,g,c):g.write(True);os.kill(os.getpid(),signal.SIGTERM)
        code,result,calls=self.watch(probe)
        self.assertEqual(code,2);self.assertEqual(result['reason'],'cancelled');self.assertEqual(calls,[False,True,False,'closed']);self.assertEqual(signal.getsignal(signal.SIGTERM),old)
    def test_continuous_rejects_combined_finite_round_count(self):
        code,result,calls=self.watch(lambda *a:self.fail(),('--rounds','2'));self.assertEqual(code,2);self.assertEqual(calls,[])
    def test_continuous_migration_failure_retries_then_term(self):
        counter=[]
        def probe(a,g,c):
            counter.append(1)
            if len(counter)==1:raise A['Reject']('migration')
            g.write(True);os.kill(os.getpid(),signal.SIGTERM)
        with mock.patch.object(G['time'],'monotonic',side_effect=range(0,100,3)),mock.patch.object(G['time'],'sleep'):
            code,result,calls=self.watch(probe)
        self.assertEqual(code,2);self.assertEqual(calls,[False,False,True,False,'closed'])

if __name__=='__main__':unittest.main()
