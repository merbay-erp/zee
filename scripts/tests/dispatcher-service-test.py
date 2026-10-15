#!/usr/bin/env python3
"""K282 host/owned-child tests; no native SQL/provider acceptance implied."""
import contextlib,hashlib,importlib.util,json,os,signal,stat,sys,tempfile,threading,time,unittest
from pathlib import Path
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('dispatcher_service',ROOT/'dispatcher-service.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)

spec=importlib.util.spec_from_file_location('native_service_support',ROOT/'tests/service-native-support.py');N=importlib.util.module_from_spec(spec);spec.loader.exec_module(N)

class Cases(unittest.TestCase):
    def campaign_model(self):
        c=object.__new__(N.Campaign);c.repo=self.repo;c.out=self.root/'logs';c.out.mkdir(mode=0o700);c.sources={};c.resources=[];c.commands=[];c.command_sequence=0;c.stopped=False;c.deadline=time.monotonic()+2;return c
    def test_native_fixture_allowlist_excludes_unlisted_SQL_and_FIFO(self):
        c=self.campaign_model();base=self.repo/'fixture';base.mkdir();(base/'ana.dil').write_bytes(b'fixed');os.mkfifo(base/'extra.sql');(base/'extra.dil').write_bytes(b'untrusted')
        c.sources['fixture/ana.dil']=self.sha(base/'ana.dil');target=self.root/'copied';c.copy_fixture('fixture',target,('ana.dil',))
        self.assertEqual([p.name for p in target.iterdir()],['ana.dil']);self.assertEqual((target/'ana.dil').read_bytes(),b'fixed')
        (base/'ana.dil').unlink();(base/'ana.dil').symlink_to(base/'extra.dil')
        with self.assertRaises(ValueError):c.fixture('fixture/ana.dil')
    def test_native_command_logs_preserve_each_observation_and_remaining_budget(self):
        c=self.campaign_model();budgets=[]
        def run(argv,**kw):budgets.append(kw['timeout']);kw['log_path'].write_bytes(b'observed');return b'observed'
        with mock.patch.object(N.D.T.D,'run',side_effect=run):
            self.assertEqual(c.command('SQL-observer',['fixed'],30),b'observed');c.command('SQL-observer',['fixed'],30)
        self.assertEqual(len(c.commands),2);self.assertNotEqual(c.commands[0]['log'],c.commands[1]['log']);self.assertTrue(all(0<x<=2 for x in budgets));self.assertTrue(all(x['status']=='passed' and x['bytes']==8 for x in c.commands))
    def test_native_interactive_blocker_owned_leader_cleanup(self):
        c=self.campaign_model();code='import sys,time;[sys.stdin.readline() for _ in range(3)];print("BARRIER_READY",flush=True);time.sleep(10)'
        with N.OwnedSqlBlocker(c,[sys.executable,'-u','-c',code],'SELECT fixed')as blocker:self.assertIsNone(N.D.T.D.held_status(blocker.child))
        self.assertIsNotNone(blocker.child.returncode);self.assertTrue(blocker.child.stdout.closed and blocker.child.stdin.closed)
    def test_native_interactive_blocker_cancellation_still_reaps(self):
        c=self.campaign_model();blocker=N.OwnedSqlBlocker(c,[sys.executable,'-u','-c','import time;time.sleep(10)'],'SELECT fixed');timer=threading.Timer(.05,lambda:setattr(c,'stopped',True));timer.start()
        try:
            with self.assertRaises(ValueError):blocker.__enter__()
            self.assertIsNotNone(blocker.child.returncode);self.assertTrue(blocker.child.stdout.closed)
        finally:timer.join(1)
    def test_native_failed_owned_cleanup_retains_private_cluster_directory(self):
        c=self.campaign_model();directory=self.root/'cluster-retained';directory.mkdir(mode=0o700);(directory/'server.log').write_bytes(b'owned diagnostics');st=directory.stat();c.tempdirs=[(directory,(st.st_dev,st.st_ino))]
        class Failed:
            def close(self):raise OSError('controlled cleanup fault')
        c.previous={};c.units={};c.links=[];c.clusters=[Failed()];c.resources=[];c.finished=False;c.proof={}
        c.close();self.assertTrue((directory/'server.log').exists());self.assertEqual(c.proof['status'],'failed');self.assertFalse(c.proof['temporary_cleanup_complete'])
    def test_native_blocker_cleanup_failure_remains_registered_for_campaign_retry(self):
        c=self.campaign_model();blocker=N.OwnedSqlBlocker(c,[sys.executable,'-u','-c','import sys,time;[sys.stdin.readline()for _ in range(3)];print("BARRIER_READY",flush=True);time.sleep(10)'],'SELECT fixed');blocker.__enter__();original=N.D.T.D.stop_owned
        try:
            with mock.patch.object(N.D.T.D,'stop_owned',side_effect=OSError('controlled stop fault')):
                with self.assertRaises(OSError):blocker.__exit__(None,None,None)
            self.assertIn(blocker,c.resources);self.assertFalse(blocker.closed);self.assertIsNone(blocker.child.returncode);self.assertFalse(blocker.child.stdout.closed)
            c.previous={};c.units={};c.links=[];c.clusters=[];c.finished=False;c.proof={};c.tempdirs=[];c.close()
            self.assertTrue(blocker.closed);self.assertIsNotNone(blocker.child.returncode);self.assertTrue(blocker.child.stdout.closed);self.assertEqual(c.proof['cleanup_errors'],[])
            blocker.close()
        finally:
            if not blocker.closed:original(blocker.child,grace=.2);blocker.child.stdin.close();blocker.child.stdout.close()
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name).resolve();self.root.chmod(0o700)
        self.cwd=self.root/'app';self.cwd.mkdir(mode=0o700);self.repo=self.root/'repo';self.repo.mkdir();self.state=self.root/'state'
        self.data=self.cwd/'input.json';self.data.write_text('{"job":"anonymous"}\n');self.data.chmod(0o600)
        self.exe=self.cwd/'adapter';self.program('print(\'{"schema":"zee-app-dispatch-result-1","outcome":"completed","completed":1}\')')
        self.path=self.root/'config.json';self.document={'schema':'zee-dispatch-service-1','kind':'application-adapter','repo':str(self.repo),'source_sha':'a'*40,'interval_seconds':1,'state':str(self.state),'executable':str(self.exe),'executable_sha256':self.sha(self.exe),'cwd':str(self.cwd),'data':str(self.data),'data_sha256':self.sha(self.data),'argv':[],'timeout_seconds':2,'contract':'zee-app-dispatch-result-1'}
        self.write();M.T.D.CANCELLED=False;M.STOP=False
    def tearDown(self):M.T.D.CANCELLED=False;self.temp.cleanup()
    def sha(self,p):return hashlib.sha256(p.read_bytes()).hexdigest()
    def program(self,body):self.exe.write_text('#!'+str(Path(sys.executable).resolve())+'\n'+body+'\n');self.exe.chmod(0o700)
    def write(self):self.path.write_text(json.dumps(self.document));self.path.chmod(0o600)
    def config(self):return M.configuration(self.path)
    @contextlib.contextmanager
    def actual(self,c):
        # Mac preparation models Linux proc resolution explicitly. Native Linux tests must use real proc.
        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch.object(M.T.D,'clean_exact'))
            if sys.platform!='linux':
                stack.enter_context(mock.patch.object(M.T,'execution_paths',side_effect=lambda a:(a.path,a.path/'tmp')))
                stack.enter_context(mock.patch.object(M,'adapter_cwd',return_value=self.cwd))
            yield
    def test_actual_adapter_success_records_no_input_or_stdout(self):
        c=self.config()
        with self.actual(c):self.assertEqual(M.adapter_tick(c),0)
        self.assertFalse((self.state/'inflight.json').exists());self.assertEqual(json.loads((self.state/'last.json').read_text())['completed'],1)
        self.assertNotIn(b'anonymous',(self.state/'last.json').read_bytes())
    def test_child_consumes_verified_private_snapshot_and_postchange_pauses(self):
        c=self.config();original=M.T.D.run;seen=[]
        def run(argv,**kw):
            self.assertNotEqual(Path(argv[-1]),self.data)
            self.assertEqual(Path(argv[-1]).read_bytes(),self.data.read_bytes())
            self.data.write_text('changed after snapshot')
            value=original(argv,**kw);self.assertEqual(json.loads(value)['completed'],1);seen.append(True);return value
        with self.actual(c),mock.patch.object(M.T.D,'run',side_effect=run),self.assertRaises(M.T.Paused):M.adapter_tick(c)
        self.assertEqual(seen,[True]);self.assertTrue((self.state/'inflight.json').exists())
    def test_actual_nonzero_unknown_retains_latch_and_never_replays(self):
        self.program('raise SystemExit(3)');self.document['executable_sha256']=self.sha(self.exe);self.write();c=self.config()
        with self.actual(c):
            with self.assertRaises(M.T.Paused):M.adapter_tick(c)
            with mock.patch.object(M.T.D,'run',side_effect=AssertionError('replay forbidden')):
                with self.assertRaises(M.T.Paused):M.adapter_tick(c)
        self.assertTrue((self.state/'inflight.json').exists())
    def test_actual_output_overflow_retains_pause_no_raw_log(self):
        self.program('print("PRIVATE_CANARY"*5000)');self.document['executable_sha256']=self.sha(self.exe);self.write();c=self.config()
        with self.actual(c),self.assertRaises(M.T.Paused):M.adapter_tick(c)
        for p in self.state.iterdir():
            if p.is_file():self.assertNotIn(b'PRIVATE_CANARY',p.read_bytes())
    def test_actual_timeout_reaps_and_retains_pause(self):
        self.program('import time;time.sleep(20)');self.document.update(executable_sha256=self.sha(self.exe),timeout_seconds=1);self.write();c=self.config();children=[];original=M.T.D.subprocess.Popen
        def spawn(*a,**kw):p=original(*a,**kw);children.append(p);return p
        with self.actual(c),mock.patch.object(M.T.D.subprocess,'Popen',side_effect=spawn),self.assertRaises(M.T.Paused):M.adapter_tick(c)
        self.assertEqual(len(children),1);self.assertIsNotNone(children[0].returncode)
    def test_actual_term_during_child_reaps_and_pauses(self):
        self.program('import time;time.sleep(20)');self.document['executable_sha256']=self.sha(self.exe);self.write();c=self.config()
        previous=signal.getsignal(signal.SIGTERM);timer=threading.Timer(.15,lambda:os.kill(os.getpid(),signal.SIGTERM))
        signal.signal(signal.SIGTERM,lambda n,f:setattr(M.T.D,'CANCELLED',True));timer.start()
        try:
            with self.actual(c),self.assertRaises(M.T.Paused):M.adapter_tick(c)
            self.assertTrue((self.state/'inflight.json').exists())
        finally:timer.join();signal.signal(signal.SIGTERM,previous)
    def test_executable_or_data_pin_drift_refuses_before_child(self):
        c=self.config();self.data.write_text('changed')
        with self.actual(c),mock.patch.object(M.T.D,'run',side_effect=AssertionError('no child')),self.assertRaises(ValueError):M.adapter_tick(c)
        self.assertFalse(self.state.exists())
    def test_invalid_ack_after_zero_exit_remains_unknown(self):
        self.program('print(\'{"schema":"zee-app-dispatch-result-1","outcome":"idle","completed":true}\')');self.document['executable_sha256']=self.sha(self.exe);self.write();c=self.config()
        with self.actual(c),self.assertRaises(M.T.Paused):M.adapter_tick(c)
        self.assertTrue((self.state/'inflight.json').exists())
    def test_lifetime_exclusion_between_ticks(self):
        with M.service_lock(self.state):
            with self.assertRaises(M.T.Busy):
                with M.service_lock(self.state):pass
        with M.service_lock(self.state):pass
    def test_pause_is_durable_before_popen_failure(self):
        c=self.config()
        def fail(*a,**k):self.assertTrue((self.state/'inflight.json').is_file());raise OSError('synthetic')
        with self.actual(c),mock.patch.object(M.T.D.subprocess,'Popen',side_effect=fail),self.assertRaises(M.T.Paused):M.adapter_tick(c)
    def test_post_clear_fsync_distinct_visible_outcome(self):
        c=self.config();original=M.T.os.fsync
        def fsync(fd):
            if stat.S_ISDIR(os.fstat(fd).st_mode) and (self.state/'last.json').exists() and not (self.state/'inflight.json').exists():raise OSError('injected')
            return original(fd)
        with self.actual(c),mock.patch.object(M.T.os,'fsync',side_effect=fsync),self.assertRaises(M.T.ClearVisible):M.adapter_tick(c)
        self.assertFalse((self.state/'inflight.json').exists())
    def test_symlink_fifo_and_private_mode_reject(self):
        target=self.root/'fifo';os.mkfifo(target)
        with self.assertRaises(ValueError):M.snapshot(target)
        self.path.chmod(0o644)
        with self.assertRaises(ValueError):self.config()
        self.path.unlink();self.path.symlink_to(self.data)
        with self.assertRaises(ValueError):self.config()
    def test_strict_schema_no_implicit_execution_or_ambient_argv(self):
        self.document['shell']='PRIVATE_CANARY';self.write()
        with self.assertRaises(ValueError):self.config()
        with mock.patch.object(M,'configuration',return_value={}),mock.patch.object(M,'pins'),mock.patch.object(M,'run',side_effect=AssertionError('check only')):self.assertEqual(M.main(['--config',str(self.path)]),0)
    def test_calendar_render_fixed_names_no_persistent_backfill(self):
        M.render(self.path,self.root/'units','*-*-* *:*:00');text=(self.root/'units/zee-dispatch.timer').read_text();self.assertIn('Persistent=false',text);self.assertIn('OnCalendar=*-*-* *:*:00',text)
        with self.assertRaises(ValueError):M.render(self.path,self.root/'evil','daily\nExecStart=/bad')
    def test_stopped_wait_never_starts_an_extra_tick(self):
        c=self.config()
        def once(c):M.STOP=True;return 0
        with mock.patch.object(M,'pins'),mock.patch.object(M,'tick',side_effect=once) as t:self.assertEqual(M.run(c),0)
        self.assertEqual(t.call_count,1)

if __name__=='__main__':unittest.main()
