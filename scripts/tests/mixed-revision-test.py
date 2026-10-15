#!/usr/bin/env python3
"""K291 transport/preflight preparation; not native mixed-revision acceptance."""
import importlib.util,os,socket,tempfile,threading,time,unittest
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
if __name__=='__main__':unittest.main()
