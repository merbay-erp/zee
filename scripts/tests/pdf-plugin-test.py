#!/usr/bin/env python3
"""Stdlib-only contract negatives; no PDF dependencies, crypto keys or native CLI."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('pdf_plugin', ROOT/'scripts/pdf-plugin-host.py')
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)

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

if __name__=='__main__':unittest.main(verbosity=2)
