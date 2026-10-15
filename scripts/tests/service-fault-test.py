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
if __name__=='__main__':unittest.main()
