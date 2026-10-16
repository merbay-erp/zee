#!/usr/bin/env python3
"""Stdlib contracts and bounded lifecycle children; no PDF dependencies, crypto keys or native CLI."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import signal
import sys
import threading
import time
import unittest
from unittest import mock
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('pdf_plugin', ROOT/'scripts/pdf-plugin-host.py')
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)

spec=importlib.util.spec_from_file_location('pdf_native',ROOT/'scripts/tests/pdf-plugin-native.py')
N=importlib.util.module_from_spec(spec);spec.loader.exec_module(N)

class Contract(unittest.TestCase):
    def test_request_schema_duplicate_size_and_nonfinite_fail_closed(self):
        for raw in [b'{}', b'[]', b'{"schema":"x","schema":"y"}', b'x'*16385,
                    b'{"schema":"zee-pdf-plugin-request-1","job":NaN}',
                    json.dumps({'schema':'zee-pdf-plugin-request-1','job':'x'*8193}).encode()]:
            with self.subTest(size=len(raw)), self.assertRaises(Exception):M.request(raw)
        job=b'{"schema":"zee-pdf-job-1"}'
        self.assertEqual(M.request(json.dumps({'schema':'zee-pdf-plugin-request-1','job':job.decode()}).encode()),job)

    def test_receipt_unknown_status_extra_path_and_missing_output_reject(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)
            for value in [{'schema':'zee-owned-pdf-receipt-1','status':'unknown','files':[]},
                          {'schema':'zee-owned-pdf-receipt-1','status':'produced','files':[]},
                          {'schema':'zee-owned-pdf-receipt-1','status':'produced','files':[{'name':'../escape','bytes':1,'sha256':'a'*64}]*3}]:
                with self.assertRaises(Exception):M.receipt(json.dumps(value).encode(),p)
            value={'schema':'zee-owned-pdf-receipt-1','status':'produced','files':[]}
            for n in ['generated.pdf','selected.pdf','catalog.pdf']:
                (p/n).write_bytes(b'%PDF-owned')
                value['files'].append({'name':n,'bytes':10,'sha256':M.digest(b'%PDF-owned')})
            self.assertEqual(M.receipt(json.dumps(value).encode(),p),value)
            (p/'catalog.pdf').write_bytes(b'changed')
            with self.assertRaises(Exception):M.receipt(json.dumps(value).encode(),p)

    def test_fifo_symlink_oversize_rejected_before_open(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t);os.mkfifo(p/'fifo');(p/'big').write_bytes(b'xx');(p/'link').symlink_to(p/'big')
            with mock.patch.object(M.os,'open',side_effect=AssertionError('pre-open rejection')):
                for n in ['fifo','big','link']:
                    with self.assertRaises(M.Rejected):M.bounded(p/n,1)

    def test_streamed_inventory_stops_on_first_excess_or_duplicate(self):
        class Items:
            def __init__(self):self.count=0
            def __enter__(self):return self
            def __exit__(self,*a):pass
            def __iter__(self):return self
            def __next__(self):self.count+=1;return type('Item',(),{'name':'a'})()
        items=Items()
        with mock.patch.object(M.os,'scandir',return_value=items):
            with self.assertRaises(M.Rejected):M.exact_names(Path('/unused'),{'a','b'})
        self.assertEqual(items.count,2)

    def test_wrong_trust_purpose_never_loads_code(self):
        # Purpose validation precedes helper execution and package code loading.
        with tempfile.TemporaryDirectory() as t:
            p=Path(t);(p/'assets').mkdir()
            for name in M.FILES:(p/name).write_bytes(b'owned')
            (p/'plugin.json').write_text('{}');(p/'plugin.json.zee-imza').write_text('{}')
            trust={'schema':'zee-compiler-trust-1','purpose':'compiler','publisher':'sha256:'+'a'*64,
                   'manifest_sha256':'a'*64,'python_sha256':'b'*64}
            with mock.patch.object(M,'Owned',side_effect=AssertionError('no owner/native before trust')):
                with self.assertRaises(M.Rejected):M.package(p,json.dumps(trust).encode())

    def test_missing_waitid_refuses_before_helper_or_process(self):
        with mock.patch.object(M.os, 'waitid', None), mock.patch.object(M, 'bounded', side_effect=AssertionError('no helper load')):
            with self.assertRaises(M.Rejected):M.Owned(Path('/unused'), Path('/unused'))

    def test_request_and_receipt_frames_no_second_or_partial_line(self):
        class Child:pass
        for data in [b'{}\n{}\n',b'x'*16386]:
            r,w=os.pipe();child=Child();child.stdout=os.fdopen(r,'rb',buffering=0)
            # Bound write size fits pipe; oversize tested through repeated selector reads.
            try:
                if len(data)>4096:
                    with mock.patch.object(M.os,'read',return_value=data):
                        os.write(w,b'x');owner=object.__new__(M.Owned);owner.deadline=M.time.monotonic()+1
                        with self.assertRaises(M.Rejected):owner.read(child,16385,line=True)
                else:
                    os.write(w,data);owner=object.__new__(M.Owned);owner.deadline=M.time.monotonic()+1
                    with self.assertRaises(M.Rejected):owner.read(child,16385,line=True)
            finally:os.close(w);child.stdout.close()

    def test_native_partial_keygen_SIGTERM_reaps_and_removes_both_keys(self):
        # No Ed25519 or native CLI: the fixed child creates inert bytes, then
        # stalls only at the second key's pre-stat window in the real harness.
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();marker=root/'partial';done=root/'done'
            cli=root/'fake-dil'
            cli.write_text('#!'+sys.executable+'\n'+
                'import json,os,signal,sys,time\nfrom pathlib import Path\n'+
                'if sys.argv[1]=="sürüm": print(json.dumps({"source_sha":"1"*40,"dirty":False}));sys.exit(0)\n'+
                'assert sys.argv[1:3]==["anahtar","üret"]\nkey=Path(sys.argv[3])\n'+
                'fd=os.open(key,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600);os.write(fd,b"inert test bytes");os.close(fd)\n'+
                'if key.name=="synthetic.key": sys.exit(0)\n'+
                'stop=False\ndef cancel(n,f):\n global stop\n stop=True\n'+
                'signal.signal(signal.SIGTERM,cancel)\n'+
                'Path('+repr(str(marker))+').write_text(str(key.parent))\n'+
                'end=time.monotonic()+5\n'+
                'while not stop and time.monotonic()<end: time.sleep(.01)\n'+
                'Path('+repr(str(done))+').write_text("closed")\n')
            cli.chmod(0o700);children=[];owners=[];trigger_stop=threading.Event();fired=[]
            original_owned=M.Owned;original_temporary=tempfile.TemporaryDirectory
            def tracked_owner(*args):
                owner=original_owned(*args);owners.append(owner);spawn=owner.owner.spawn;command=owner.command
                def recorded(*a,**k):
                    child=spawn(*a,**k);children.append(child);return child
                def timed_command(argv,*a,**k):
                    if list(argv[1:3])==['anahtar','üret'] and Path(argv[-1]).name=='other.key':thread.start()
                    return command(argv,*a,**k)
                owner.owner.spawn=recorded;owner.command=timed_command;return owner
            def private_directory(*args,**kwargs):return original_temporary(*args,dir=root,**kwargs)
            def trigger():
                end=time.monotonic()+5
                while not marker.exists() and not trigger_stop.is_set() and time.monotonic()<end:trigger_stop.wait(.005)
                if marker.exists() and not trigger_stop.is_set():fired.append(True);os.kill(os.getpid(),signal.SIGTERM)
            previous={n:signal.getsignal(n) for n in (signal.SIGTERM,signal.SIGINT)}
            before_cancel=M.CANCELLED;thread=threading.Thread(target=trigger)
            try:
                with mock.patch.object(N,'load_host',return_value=M),mock.patch.object(M,'Owned',side_effect=tracked_owner),mock.patch.object(N.tempfile,'TemporaryDirectory',side_effect=private_directory):
                    with self.assertRaises(Exception):N.main(['--framework-root',str(ROOT),'--dil',str(cli),'--verifier',str(Path(sys.executable).resolve()),'--python',str(Path(sys.executable).resolve()),'--output',str(root/'out')])
                self.assertTrue(fired);self.assertEqual(done.read_text(),'closed')
                private=Path(marker.read_text());self.assertEqual(private.parent,root);self.assertFalse(private.exists())
                proof=json.loads((root/'out/proof.json').read_text())
                self.assertEqual(proof['status'],'failed');self.assertTrue(proof['cancelled'])
                self.assertTrue(proof['owned_groups_closed'] and proof['synthetic_private_keys_removed'])
                self.assertEqual(proof['cases'],[]);self.assertEqual(len(owners),1);self.assertEqual(len(children),4)
                self.assertFalse(owners[0].owner.children)
                self.assertTrue(all(child.returncode is not None and child.stdout.closed for child in children))
                self.assertEqual({n:signal.getsignal(n) for n in previous},previous)
            finally:
                trigger_stop.set()
                if thread.ident is not None:thread.join(6)
                try:
                    for owner in owners:owner.close()
                finally:
                    M.CANCELLED=before_cancel
                    for number,handler in previous.items():signal.signal(number,handler)

    def test_interrupted_host_grace_allows_nested_original_owner_to_reap(self):
        # A short synthetic two-level owner exercises actual group shutdown.
        # The inner child needs >1s cooperative cleanup; no PDF/runtime/native
        # acceptance case or crypto key is executed by this stdlib test.
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();marker=root/'host-ready';receipt=root/'closed.json';inner_ready=root/'inner-ready'
            inner=root/'inner.py';host=root/'host.py'
            inner.write_text('import signal,time,sys\nfrom pathlib import Path\n'+
                'def finish(n,f):\n signal.signal(signal.SIGTERM,signal.SIG_IGN)\n time.sleep(1.25)\n sys.exit(0)\n'+
                'signal.signal(signal.SIGTERM,finish)\nPath('+repr(str(inner_ready))+').write_text("ready")\ntime.sleep(8)\n')
            host.write_text('import importlib.util,json,signal,subprocess,sys,time\nfrom pathlib import Path\n'+
                'spec=importlib.util.spec_from_file_location("held",'+repr(str(ROOT/'scripts/lifecycle_core.py'))+');H=importlib.util.module_from_spec(spec);spec.loader.exec_module(H)\n'+
                'stop=False\ndef cancel(n,f):\n global stop\n stop=True\n'+
                'signal.signal(signal.SIGTERM,cancel)\n'+
                'child=subprocess.Popen([sys.executable,"-I","-B",'+repr(str(inner))+'],stdout=subprocess.PIPE,start_new_session=True)\n'+
                'try:\n end=time.monotonic()+5\n'+
                ' while not Path('+repr(str(inner_ready))+').exists() and not stop and time.monotonic()<end: time.sleep(.005)\n'+
                ' Path('+repr(str(marker))+').write_text("ready")\n'+
                ' while not stop and time.monotonic()<end: time.sleep(.005)\n'+
                'finally:\n H.stop_owned(child,grace=2)\n child.stdout.close()\n'+
                ' Path('+repr(str(receipt))+').write_text(json.dumps({"inner_exit":child.returncode,"inner_pipe_closed":child.stdout.closed}))\n')
            owner=M.Owned(ROOT,root);children=[];spawn=owner.owner.spawn
            def recorded(*a,**k):
                child=spawn(*a,**k);children.append(child);return child
            owner.owner.spawn=recorded;previous=signal.getsignal(signal.SIGTERM);before_cancel=M.CANCELLED
            trigger_stop=threading.Event();fired=[]
            def cancel(n,f):M.CANCELLED=True
            def trigger():
                end=time.monotonic()+5
                while not marker.exists() and not trigger_stop.is_set() and time.monotonic()<end:trigger_stop.wait(.005)
                if marker.exists() and not trigger_stop.is_set():fired.append(True);os.kill(os.getpid(),signal.SIGTERM)
            signal.signal(signal.SIGTERM,cancel);thread=threading.Thread(target=trigger)
            try:
                self.assertEqual(N.host_command(owner,[sys.executable,'-I','-B','-c','print("ordinary")']),b'ordinary\n')
                self.assertEqual(owner.owner.a.stop_grace,1)
                thread.start()
                with self.assertRaises(M.Rejected):N.host_command(owner,[sys.executable,'-I','-B',host])
                self.assertTrue(fired);self.assertEqual(owner.owner.a.stop_grace,10)
                owner.close()
                self.assertEqual(json.loads(receipt.read_text()),{'inner_exit':0,'inner_pipe_closed':True})
                self.assertEqual(len(children),2);self.assertTrue(all(p.returncode==0 and p.stdout.closed for p in children));self.assertFalse(owner.owner.children)
            finally:
                trigger_stop.set()
                if thread.ident is not None:thread.join(6)
                # Keep the negative/assertion path owned as well: the assertion
                # above still rejects a regressed 1s grace before this cleanup.
                owner.owner.a.stop_grace=max(10,owner.owner.a.stop_grace)
                try:owner.close()
                finally:M.CANCELLED=before_cancel;signal.signal(signal.SIGTERM,previous)

if __name__=='__main__':unittest.main(verbosity=2)
