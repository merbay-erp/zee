#!/usr/bin/env python3
"""K292 preparation: controlled Python resolver stall, never actual native DNS evidence."""
import importlib.util,os,runpy,signal,subprocess,sys,tempfile,threading,time,unittest
from pathlib import Path
from unittest import mock
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('service_fault',HERE/'service-fault-native.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
B=runpy.run_path(str(HERE.parent/'db-hazirlik.py'));FIXTURE=HERE/'fixtures/service-fault/resolver-stall.py'
class Cases(unittest.TestCase):
    def test_native_fixture_identity_query_uses_owned_bounded_runner_and_restores_binding(self):
        c=object.__new__(M.Campaign);c.stop=False;c.end=time.monotonic()+1;c.H=type('H',(),{})();calls=[];c.H.run=lambda argv,**kw:calls.append((argv,kw))or b'{"fixed":true}'
        original=type('Legacy',(),{})()
        namespace={'subprocess':original}
        exec('class Fixture:\n def __init__(self,**kw): self.build=subprocess.check_output(["owned-dil","sürüm","--json"],timeout=5)\n',namespace)
        fixture=c.fixture_owned(namespace,native=True)
        self.assertEqual(fixture.build,b'{"fixed":true}');self.assertEqual(calls[0][1]['limit'],8192);self.assertLessEqual(calls[0][1]['timeout'],1);self.assertIs(namespace['subprocess'],original)
    def setUp(self):self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name).resolve();self.marker=self.root/'entered'
    def tearDown(self):self.temp.cleanup()
    def owned(self):
        spawned=[];original=B['subprocess'].Popen
        def popen(*a,**kw):p=original(*a,**kw);spawned.append(p);return p
        return spawned,mock.patch.object(B['subprocess'],'Popen',side_effect=popen)
    def test_actual_controlled_resolver_stall_deadline_reaps_child(self):
        children,patch=self.owned();start=time.monotonic()
        with patch,self.assertRaises(B['Reject'])as error:B['child'](Path(sys.executable),['-I','-B',str(FIXTURE),str(self.marker)],self.root,start+.3)
        self.assertEqual(error.exception.code,'timeout');self.assertEqual(self.marker.read_text(),'resolver-entered');self.assertEqual(len(children),1);self.assertIsNotNone(children[0].returncode);self.assertTrue(children[0].stdout.closed and children[0].stderr.closed)
    def test_actual_SIGTERM_during_controlled_resolver_stall_cancels_and_reaps(self):
        stop=threading.Event();previous=signal.getsignal(signal.SIGTERM);signal.signal(signal.SIGTERM,lambda n,f:stop.set())
        def trigger():
            end=time.monotonic()+1.5
            while not self.marker.exists()and time.monotonic()<end:time.sleep(.005)
            os.kill(os.getpid(),signal.SIGTERM)
        thread=threading.Thread(target=trigger);children,patch=self.owned();thread.start()
        try:
            with patch,self.assertRaises(B['Reject'])as error:B['child'](Path(sys.executable),['-I','-B',str(FIXTURE),str(self.marker)],self.root,time.monotonic()+3,stop.is_set)
            self.assertEqual(error.exception.code,'cancelled');self.assertEqual(self.marker.read_text(),'resolver-entered');self.assertIsNotNone(children[0].returncode)
        finally:thread.join(2);signal.signal(signal.SIGTERM,previous)
    def test_cancel_before_spawn_never_enters_resolver(self):
        with mock.patch.object(B['subprocess'],'Popen',side_effect=AssertionError('no child')),self.assertRaises(B['Reject'])as error:B['child'](Path(sys.executable),[],self.root,time.monotonic()+1,lambda:True)
        self.assertEqual(error.exception.code,'cancelled');self.assertFalse(self.marker.exists())
    def test_selector_setup_fault_still_reaps_actual_child(self):
        children,patch=self.owned()
        with patch,mock.patch.object(B['selectors'],'DefaultSelector',side_effect=OSError('injected')),self.assertRaises(OSError):B['child'](Path(sys.executable),['-I','-B',str(FIXTURE),str(self.marker)],self.root,time.monotonic()+1)
        self.assertEqual(len(children),1);self.assertIsNotNone(children[0].returncode);self.assertTrue(children[0].stdout.closed and children[0].stderr.closed)
    def test_native_campaign_invalid_identity_no_effect(self):
        args=type('Args',(),{'sha':'invalid','binary_sha256':'0'*64})()
        with mock.patch.object(M.subprocess,'Popen',side_effect=AssertionError('no child')),self.assertRaises(M.P.Rejected):M.Campaign(args)
    def test_cli_masked_error_contains_no_argument_value(self):
        result=subprocess.run([sys.executable,'-I','-B',str(HERE/'service-fault-native.py'),'--PRIVATE_CANARY_ARGUMENT'],capture_output=True,timeout=3)
        self.assertEqual(result.returncode,2);self.assertNotIn(b'PRIVATE_CANARY',result.stdout+result.stderr);self.assertNotIn(b'Traceback',result.stderr)
    def test_prepare_Git_cancellation_reaps_retained_child_and_restores_handlers(self):
        # Real lifecycle_core.run and a bounded inert child; clean_exact is
        # replaced only for this cancellation test, never native acceptance.
        H=M.P.load('fault_cancel_test',HERE.parent/'lifecycle_core.py');children=[];real_popen=H.subprocess.Popen
        executable=Path(sys.executable).resolve();args=type('Args',(),{'sha':'1'*40,'binary_sha256':M.P.snapshot(executable),'repo':HERE.parents[1].resolve(),'dil':executable,'out':self.root/'fresh'})()
        original_load=M.P.load;before={n:signal.getsignal(n)for n in (signal.SIGTERM,signal.SIGINT)};trigger_stop=threading.Event();fired=[];threads=[]
        def recorded(*a,**k):
            child=real_popen(*a,**k);children.append(child);return child
        def query(repo,sha):
            handler=signal.getsignal(signal.SIGTERM)
            self.assertIsInstance(getattr(handler,'__self__',None),M.Campaign)
            def trigger():
                end=time.monotonic()+2
                while not self.marker.exists() and not trigger_stop.is_set() and time.monotonic()<end:trigger_stop.wait(.005)
                if self.marker.exists() and not trigger_stop.is_set():fired.append(True);os.kill(os.getpid(),signal.SIGTERM)
            thread=threading.Thread(target=trigger);threads.append(thread);thread.start()
            H.run([executable,'-I','-B','-c','import time;from pathlib import Path;Path('+repr(str(self.marker))+').write_text("entered");time.sleep(3)'],timeout=2.5,limit=1024)
        with mock.patch.object(M.P,'load',side_effect=lambda name,path:H if name=='fault_lifecycle'else original_load(name,path)),mock.patch.object(H,'clean_exact',side_effect=query),mock.patch.object(H.subprocess,'Popen',side_effect=recorded):
            c=M.Campaign(args)
            try:
                self.assertEqual(children,[])
                try:
                    with self.assertRaises(H.Reject):c.prepare()
                finally:c.close()
                self.assertTrue(fired and c.stop and H.CANCELLED);self.assertEqual(len(children),1)
                self.assertIsNotNone(children[0].returncode);self.assertTrue(children[0].stdout.closed)
                self.assertFalse(args.out.exists());self.assertEqual(c.proof['status'],'failed')
                self.assertEqual({n:signal.getsignal(n)for n in before},before)
            finally:
                trigger_stop.set()
                for thread in threads:thread.join(3)
                for child in children:
                    if child.returncode is None:H.stop_owned(child)
                for number,handler in before.items():signal.signal(number,handler)

    def test_partial_native_fixture_constructor_removes_owned_temporary(self):
        c=object.__new__(M.Campaign);c.stop=False;c.end=time.monotonic()+1
        c.H=type('H',(),{'run':staticmethod(mock.Mock(side_effect=M.P.Rejected('controlled constructor query failure')))})()
        created=[];original=object();namespace={'subprocess':original,'created':created,'tempfile':tempfile,'root':self.root}
        exec('class Fixture:\n def __init__(self,**kw):\n  created.append(self)\n  self.temporary=tempfile.TemporaryDirectory(dir=root)\n  subprocess.check_output(["owned-dil","sürüm","--json"],timeout=5)\n',namespace)
        try:
            with self.assertRaises(M.P.Rejected):c.fixture_owned(namespace,native=True)
            self.assertEqual(len(created),1);self.assertFalse(Path(created[0].temporary.name).exists())
            self.assertIs(namespace['subprocess'],original);c.H.run.assert_called_once()
        finally:
            for fixture in created:fixture.temporary.cleanup()

    def test_SIGTERM_during_cleanup_cannot_publish_passed_receipt(self):
        c=object.__new__(M.Campaign);c.owner=None;c.foreign=None;c.stop=False;c.output_created=True;c.out=self.root;c.H=type('H',(),{'CANCELLED':False})();c.proof={'status':'cases-passed-pending-cleanup'}
        c.fixture=type('Fixture',(),{'close':staticmethod(lambda:os.kill(os.getpid(),signal.SIGTERM))})()
        previous=signal.getsignal(signal.SIGTERM);c.handlers={signal.SIGTERM:previous};signal.signal(signal.SIGTERM,c.cancel)
        try:
            c.close()
            proof=M.json.loads((self.root/'proof.json').read_text())
            self.assertTrue(c.stop and c.H.CANCELLED and proof['cancelled']);self.assertEqual(proof['status'],'failed')
            self.assertIs(signal.getsignal(signal.SIGTERM),previous)
        finally:signal.signal(signal.SIGTERM,previous)

if __name__=='__main__':unittest.main()
