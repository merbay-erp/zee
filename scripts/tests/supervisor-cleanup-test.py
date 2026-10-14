#!/usr/bin/env python3
"""K263 actual held-leader lifecycle and controlled permission transitions."""
import errno,hashlib,importlib.util,json,os,signal,subprocess,sys,tempfile,time,unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];TOOL=ROOT/'scripts/worker-supervisor.py';spec=importlib.util.spec_from_file_location('cleanup_owner',TOOL);S=importlib.util.module_from_spec(spec);spec.loader.exec_module(S)
class Log:
    dropped=0
    def emit(self,*args,**kw):pass
class Cleanup(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='zee-k263-');self.root=Path(self.temp.name).resolve();self.owner=S.Supervisor(SimpleNamespace(project=self.root,stop_grace=.08),Log());self.foreign_owner=S.Supervisor(SimpleNamespace(project=self.root,stop_grace=.08),Log());self.foreign=self.foreign_owner.spawn([sys.executable,'-c','import time;time.sleep(30)']);self.created=[]
    def tearDown(self):
        for owner in (self.owner,self.foreign_owner):
            for child in list(owner.children):owner.stop_child(child)
        self.temp.cleanup()
    def child(self,source='import time;time.sleep(30)',capture=False):
        child=self.owner.spawn([sys.executable,'-c',source],capture=capture);self.created.append(child);return child
    def foreign_alive(self):self.assertIsNone(self.foreign_owner.exit_status(self.foreign));os.killpg(self.foreign.pid,0)
    def test_transient_eperm_pending_then_terminal_retains_leader_and_reaps_once(self):
        child=self.child('');real_kill=os.killpg;real_status=self.owner.exit_status;calls=[];probes=[0];status_calls=[0]
        def kill(pid,num):
            calls.append((pid,num))
            if pid==child.pid and num==0 and probes[0]<3:probes[0]+=1;raise PermissionError(errno.EPERM,'fixture')
            return real_kill(pid,num)
        def status(c):
            status_calls[0]+=1
            if c is child and probes[0] in (1,2):return None
            return real_status(c)
        with patch.object(S.os,'killpg',side_effect=kill),patch.object(self.owner,'exit_status',side_effect=status):self.owner.stop_child(child)
        self.assertIsNotNone(child.returncode);self.assertNotIn(child,self.owner.children);self.assertGreaterEqual(probes[0],3);self.assertGreater(status_calls[0],3);self.assertTrue(all(pid==child.pid for pid,_ in calls));self.foreign_alive()
    def test_persistent_eperm_pending_is_one_deadline_unreaped_failclosed(self):
        child=self.child();real=os.killpg;begin=time.monotonic();calls=[]
        def kill(pid,num):
            calls.append((pid,num))
            if pid==child.pid:raise PermissionError(errno.EPERM,'fixture')
            return real(pid,num)
        with patch.object(S.os,'killpg',side_effect=kill):
            with self.assertRaises(S.Reject) as error:self.owner.stop_child(child)
        self.assertEqual(error.exception.code,'ownership');elapsed=time.monotonic()-begin;self.assertGreaterEqual(elapsed,.075);self.assertLess(elapsed,.5);self.assertIsNone(child.returncode);self.assertIn(child,self.owner.children);self.assertIsNone(self.owner.exit_status(child));self.assertTrue(all(num==0 for _,num in calls));self.foreign_alive()
    def test_term_permission_transient_reobserves_same_owned_group(self):
        child=self.child();real=os.killpg;denied=[False]
        def kill(pid,num):
            if pid==child.pid and num==signal.SIGTERM and not denied[0]:denied[0]=True;raise PermissionError(errno.EPERM,'fixture')
            return real(pid,num)
        with patch.object(S.os,'killpg',side_effect=kill):self.owner.stop_child(child)
        self.assertTrue(denied[0]);self.assertIsNotNone(child.returncode);self.foreign_alive()
    def test_term_permission_persistent_never_kills_or_reaps_on_ambiguous_deadline(self):
        child=self.child();real=os.killpg;signals=[]
        def kill(pid,num):
            if pid==child.pid and num!=0:signals.append(num);raise PermissionError(errno.EPERM,'fixture')
            return real(pid,num)
        with patch.object(S.os,'killpg',side_effect=kill):
            with self.assertRaises(S.Reject):self.owner.stop_child(child)
        self.assertTrue(signals and all(n==signal.SIGTERM for n in signals));self.assertIsNone(child.returncode);self.assertIn(child,self.owner.children);self.foreign_alive()
    def test_kill_permission_after_grace_is_failclosed_without_reap_or_new_budget(self):
        child=self.child('import signal,time\nsignal.signal(signal.SIGTERM,signal.SIG_IGN)\nprint("ready",flush=True)\nwhile True:time.sleep(1)\n',capture=True)
        import selectors
        selector=selectors.DefaultSelector();selector.register(child.stdout,selectors.EVENT_READ);self.assertTrue(selector.select(3));self.assertEqual(os.read(child.stdout.fileno(),64),b'ready\n');selector.close()
        real=os.killpg;begin=time.monotonic();denied=[]
        def kill(pid,num):
            if pid==child.pid and num==signal.SIGKILL:denied.append(num);raise PermissionError(errno.EPERM,'fixture')
            return real(pid,num)
        with patch.object(S.os,'killpg',side_effect=kill):
            with self.assertRaises(S.Reject):self.owner.stop_child(child)
        self.assertEqual(denied,[signal.SIGKILL]);self.assertLess(time.monotonic()-begin,.5);self.assertIsNone(child.returncode);self.assertIn(child,self.owner.children);self.foreign_alive()
    def test_terminal_leader_live_descendant_group_is_killed_before_reap(self):
        marker=self.root/'descendant.ready';child=self.child('import os,time\nfrom pathlib import Path\np=os.fork()\nif p==0:\n Path('+repr(str(marker))+').write_text("ready")\n while True:time.sleep(1)\nos._exit(0)\n');end=time.monotonic()+3
        while (not marker.exists() or self.owner.exit_status(child) is None) and time.monotonic()<end:time.sleep(.005)
        self.assertTrue(marker.exists());self.assertEqual(self.owner.exit_status(child),0);os.killpg(child.pid,0);self.owner.stop_child(child);self.assertEqual(child.returncode,0);self.assertNotIn(child,self.owner.children)
        try:os.killpg(child.pid,0)
        except (ProcessLookupError,PermissionError):pass
        else:raise AssertionError('owned descendant group alive')
        self.foreign_alive()
    def test_returncode_set_and_foreign_reap_echild_fail_before_any_signal(self):
        child=self.child('');child.wait(timeout=3)
        with patch.object(S.os,'killpg') as kill:
            with self.assertRaises(S.Reject):self.owner.stop_child(child)
            kill.assert_not_called()
        self.owner.children.remove(child)
        child=self.child('');os.waitpid(child.pid,0)
        with patch.object(S.os,'killpg') as kill:
            with self.assertRaises(ChildProcessError):self.owner.stop_child(child)
            kill.assert_not_called()
        self.owner.children.remove(child);child.returncode=0;self.foreign_alive()
    def test_actual_many_fast_owned_queries_and_foreign_group_safe(self):
        for _ in range(50):
            child=self.child('');end=time.monotonic()+3
            while self.owner.exit_status(child) is None and time.monotonic()<end:pass
            self.owner.stop_child(child);self.assertEqual(child.returncode,0)
        self.foreign_alive()
if __name__=='__main__':unittest.main()
