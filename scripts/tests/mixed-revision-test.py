#!/usr/bin/env python3
"""K291 transport/preflight preparation; not native mixed-revision acceptance."""
import importlib.util,os,signal,socket,sys,tempfile,threading,time,unittest
from pathlib import Path
from unittest import mock
HERE=Path(__file__).resolve().parent;spec=importlib.util.spec_from_file_location('mixed_revision',HERE/'mixed-revision-native.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
class Cases(unittest.TestCase):
    def test_legacy_private_identity_adapter_bounded_and_global_binding_restored(self):
        with tempfile.TemporaryDirectory()as td:
            root=Path(td).resolve();path=root/'legacy.py';path.write_text('import subprocess,sys\nassert subprocess.check_output([sys.argv[4],"sürüm","--json"])==b"identity"\n')
            run=object.__new__(M.Run);run.repo=root;run.binary=root/'owned-dil';run.end=time.monotonic()+30;run.stop=False;calls=[];run.H=type('H',(),{})();run.H.run=lambda argv,**kw:calls.append((argv,kw))or b'identity'
            original=M.subprocess.check_output;argv=M.sys.argv
            run.legacy_deadline(path,root/'out.json');self.assertEqual(calls[0][1]['limit'],8192);self.assertLessEqual(calls[0][1]['timeout'],5);self.assertIs(M.subprocess.check_output,original);self.assertIs(M.sys.argv,argv)
            path.write_text('import subprocess\nsubprocess.check_output(["foreign"])\n')
            with self.assertRaises(M.Rejected):run.legacy_deadline(path,root/'bad.json')
            self.assertIs(M.subprocess.check_output,original);self.assertIs(M.sys.argv,argv);self.assertEqual(len(calls),1)
            path.write_text('import time\ntime.sleep(100)\n');timer=threading.Timer(.02,lambda:setattr(run,'stop',True));start=time.monotonic();timer.start()
            try:
                with self.assertRaises(M.Rejected):run.legacy_deadline(path,root/'cancel.json')
                self.assertLess(time.monotonic()-start,.3);self.assertIs(M.sys.argv,argv)
            finally:timer.join(1)

    def server(self,response):
        sock=socket.socket();sock.bind(('127.0.0.1',0));sock.listen();port=sock.getsockname()[1]
        def serve():
            try:
                with sock.accept()[0]as child:child.recv(4096);child.sendall(response)
            finally:sock.close()
        thread=threading.Thread(target=serve);thread.start();return port,thread
    def test_actual_complete_response(self):
        port,t=self.server(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok')
        try:self.assertEqual(M.http(port)[:2],(200,b'ok'))
        finally:t.join(2);self.assertFalse(t.is_alive())
    def test_actual_partial_response_is_failure_not_success_status(self):
        port,t=self.server(b'HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nok')
        try:
            with self.assertRaises(M.ClientFailure)as error:M.http(port)
            self.assertEqual(error.exception.kind,'partial_or_framing')
        finally:t.join(2)
    def test_actual_refused_has_fixed_type(self):
        with socket.socket()as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
        with self.assertRaises(M.ClientFailure)as error:M.http(port)
        self.assertEqual(error.exception.kind,'refused')
    def test_connect_deadline_prevents_expired_send(self):
        fake=mock.MagicMock();fake.__enter__.return_value=fake
        with mock.patch.object(M.time,'monotonic',side_effect=[0,.1,6]),mock.patch.object(M.socket,'create_connection',return_value=fake),self.assertRaises(M.ClientFailure):M.http(1234)
        fake.sendall.assert_not_called()
        fake.reset_mock()
        with mock.patch.object(M.time,'monotonic',side_effect=[0,.1,.3]),mock.patch.object(M.socket,'create_connection',return_value=fake)as connect,self.assertRaises(M.ClientFailure):M.http(1234,deadline=.2)
        self.assertAlmostEqual(connect.call_args.kwargs['timeout'],.1);fake.sendall.assert_not_called()

    def test_actual_response_overflow_is_bounded_failure(self):
        port,t=self.server(b'HTTP/1.1 200 OK\r\nContent-Length: 40000\r\nConnection: close\r\n\r\n'+b'x'*40000)
        try:
            with self.assertRaises(M.ClientFailure)as error:M.http(port)
            self.assertEqual(error.exception.kind,'output_bound')
        finally:t.join(2)
    def test_snapshot_fifo_refused_without_read(self):
        with tempfile.TemporaryDirectory()as d:
            p=Path(d)/'fifo';os.mkfifo(p)
            with self.assertRaises(M.Rejected):M.snapshot(p)
    def test_invalid_identity_preflight_has_no_command(self):
        args=type('Args',(),{'sha':'not-a-source','binary_sha256':'0'*64})()
        with mock.patch.object(M.subprocess,'Popen',side_effect=AssertionError('no spawn')),self.assertRaises(M.Rejected):M.Run(args)
    def native_args(self,root):
        executable=Path(sys.executable).resolve()
        return type('Args',(),{'sha':'1'*40,'binary_sha256':M.snapshot(executable),'repo':HERE.parents[1].resolve(),'dil':executable,'out':root/'fresh'})()

    def test_constructor_is_child_free_and_canonical_prepare_preserves_strict_paths(self):
        with tempfile.TemporaryDirectory()as temporary:
            root=Path(temporary).resolve();args=self.native_args(root);H=M.load('revision_setup_test',HERE.parent/'lifecycle_core.py')
            before={n:signal.getsignal(n)for n in (signal.SIGTERM,signal.SIGINT)}
            with mock.patch.object(M,'load',return_value=H),mock.patch.object(H,'clean_exact',return_value=None)as clean,mock.patch.object(H.subprocess,'Popen',side_effect=AssertionError('no child')):
                c=M.Run(args)
                try:
                    clean.assert_not_called();self.assertFalse(args.out.exists());self.assertFalse(c.output_created)
                    c.prepare();clean.assert_called_once_with(args.repo,args.sha)
                    self.assertTrue(c.output_created);self.assertEqual(len(c.source_pins),6);self.assertEqual(c.proof['status'],'running')
                finally:c.close()
                self.assertEqual(c.proof['status'],'failed');self.assertEqual({n:signal.getsignal(n)for n in before},before)
                args.out=root/'alias-out';args.repo=args.repo/'compiler'/'..'
                with self.assertRaises(M.Rejected):M.Run(args)
                self.assertEqual(clean.call_count,1);self.assertFalse(args.out.exists())

    def test_prepare_Git_cancellation_reaps_retained_child_and_restores_handlers(self):
        # Only the identity query is replaced; lifecycle_core.run retains the
        # actual inert Popen and reaps it. This is not native worker acceptance.
        H=M.load('revision_cancel_test',HERE.parent/'lifecycle_core.py');children=[];real_popen=H.subprocess.Popen
        with tempfile.TemporaryDirectory()as temporary:
            root=Path(temporary).resolve();args=self.native_args(root);marker=root/'entered'
            before={n:signal.getsignal(n)for n in (signal.SIGTERM,signal.SIGINT)};trigger_stop=threading.Event();fired=[];threads=[]
            def recorded(*a,**k):
                child=real_popen(*a,**k);children.append(child);return child
            def query(repo,sha):
                self.assertIsInstance(getattr(signal.getsignal(signal.SIGTERM),'__self__',None),M.Run)
                def trigger():
                    end=time.monotonic()+2
                    while not marker.exists()and not trigger_stop.is_set()and time.monotonic()<end:trigger_stop.wait(.005)
                    if marker.exists()and not trigger_stop.is_set():fired.append(True);os.kill(os.getpid(),signal.SIGTERM)
                thread=threading.Thread(target=trigger);threads.append(thread);thread.start()
                H.run([args.dil,'-I','-B','-c','import time;from pathlib import Path;Path('+repr(str(marker))+').write_text("entered");time.sleep(3)'],timeout=2.5,limit=1024)
            with mock.patch.object(M,'load',return_value=H),mock.patch.object(H,'clean_exact',side_effect=query),mock.patch.object(H.subprocess,'Popen',side_effect=recorded):
                c=M.Run(args)
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

    def test_partial_prepare_main_finally_publishes_failed_receipt_and_restores_handlers(self):
        with tempfile.TemporaryDirectory()as temporary:
            root=Path(temporary).resolve();args=self.native_args(root);c=M.Run(args);original=M.snapshot
            before={n:signal.getsignal(n)for n in (signal.SIGTERM,signal.SIGINT)}
            def fail_source(path,*a):
                if Path(path)==HERE/'mixed-revision-native.py':raise M.Rejected('injected source snapshot failure')
                return original(path,*a)
            with mock.patch.object(M.Parser,'parse_args',return_value=args),mock.patch.object(M,'Run',return_value=c),mock.patch.object(c.H,'clean_exact',return_value=None),mock.patch.object(M,'snapshot',side_effect=fail_source),mock.patch.object(M.subprocess,'Popen',side_effect=AssertionError('no child')):
                with self.assertRaises(M.Rejected):M.main()
            proof=M.json.loads((args.out/'proof.json').read_text())
            self.assertTrue(c.output_created);self.assertEqual(proof['status'],'failed');self.assertFalse(proof['cancelled'])
            self.assertEqual(proof['cleanup_errors'],[]);self.assertNotIn('source_pins_start',proof)
            self.assertEqual({n:signal.getsignal(n)for n in before},before)

    def test_partial_host_constructor_listener_is_retained_and_closed(self):
        for phase in ('constructor_failure','constructor_cancel'):
            with self.subTest(phase=phase),tempfile.TemporaryDirectory()as temporary:
                root=Path(temporary).resolve();args=self.native_args(root);c=M.Run(args)
                module=M.load('revision_partial_host_test',args.repo/'scripts/native-listener-host.py');original_mkdir=Path.mkdir;original_init=module.Host.__init__
                before={n:signal.getsignal(n)for n in (signal.SIGTERM,signal.SIGINT)}
                def fail_receipts(path,*a,**kw):
                    if phase=='constructor_failure'and path==c.project/'receipts':raise OSError('injected receipt directory failure')
                    return original_mkdir(path,*a,**kw)
                def construct(host,*a,**kw):
                    original_init(host,*a,**kw)
                    if phase=='constructor_cancel':c.cancel(signal.SIGTERM,None)
                try:
                    with mock.patch.object(c.H,'clean_exact',return_value=None),mock.patch.object(M.subprocess,'Popen',side_effect=AssertionError('no child'))as popen:
                        c.prepare();c.project=args.out/'project';c.project.mkdir(mode=0o700)
                        for name in ('ana.dil','proje.dil'):(c.project/name).write_text('controlled fixture\n')
                        error=OSError if phase=='constructor_failure'else M.Rejected
                        with mock.patch.object(Path,'mkdir',new=fail_receipts),mock.patch.object(module.Host,'__init__',new=construct),self.assertRaises(error):c.host_owned(module,{},'1'*40)
                        self.assertIsNotNone(c.host);self.assertEqual(c.host_initialized,phase=='constructor_cancel');self.assertGreaterEqual(c.host.listener.fileno(),0)
                        self.assertEqual(c.host.owner.children,[]);self.assertIsNone(c.foreign);popen.assert_not_called()
                finally:c.close()
                self.assertEqual(c.host.listener.fileno(),-1);self.assertEqual(c.host.owner.children,[])
                proof=M.json.loads((args.out/'proof.json').read_text());self.assertEqual(proof['status'],'failed');self.assertEqual(proof['cleanup_errors'],[])
                self.assertEqual(proof['cancelled'],phase=='constructor_cancel')
                self.assertEqual({n:signal.getsignal(n)for n in before},before)

    def test_cleanup_preserves_cancellation_and_only_uncancelled_completion_can_pass(self):
        # Synthetic pending receipts exercise cleanup, not campaign oracles.
        for phase in ('before','during','publication','none'):
            with self.subTest(phase=phase),tempfile.TemporaryDirectory()as temporary:
                root=Path(temporary).resolve();c=M.Run(self.native_args(root));c.out.mkdir(mode=0o700);c.output_created=True;c.proof['status']='cases-passed-pending-cleanup'
                before={n:signal.getsignal(n)for n in (signal.SIGTERM,signal.SIGINT)}
                for number,handler in before.items():c.oldhandlers[number]=handler;signal.signal(number,c.cancel)
                c.host_initialized=True;c.host=type('Host',(),{'cleanup':staticmethod(lambda:os.kill(os.getpid(),signal.SIGTERM)if phase=='during'else None)})()
                original_write=Path.write_text;writes=[]
                def publication(path,*a,**kw):
                    writes.append(path)
                    if phase=='publication'and len(writes)==1:os.kill(os.getpid(),signal.SIGTERM)
                    return original_write(path,*a,**kw)
                try:
                    if phase=='before':c.cancel(signal.SIGTERM,None)
                    with mock.patch.object(Path,'write_text',new=publication):c.close()
                    proof=M.json.loads((c.out/'proof.json').read_text());cancelled=phase!='none'
                    self.assertEqual((c.stop,c.H.CANCELLED,proof['cancelled']),(cancelled,)*3)
                    self.assertEqual(proof['status'],'failed'if cancelled else 'passed')
                    self.assertEqual({n:signal.getsignal(n)for n in before},before)
                finally:
                    for number,handler in before.items():signal.signal(number,handler)

if __name__=='__main__':unittest.main()
