#!/usr/bin/env python3
"""Bounded starter front protocol, immutable media and uncertainty contracts."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import socket
import struct
import tempfile
import time
import unittest
from unittest.mock import patch
import zlib
R=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('starter',R/'scripts/starter-content-host.py');M=importlib.util.module_from_spec(s);s.loader.exec_module(M)
def png():
    def chunk(name,value):return struct.pack('!I',len(value))+name+value+struct.pack('!I',zlib.crc32(name+value)&0xffffffff)
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('!IIBBBBB',1,1,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\0\xff\0\0\xff'))+chunk(b'IEND',b'')
class Contracts(unittest.TestCase):
    def test_exact_route_and_encoded_traversal_rejection(self):
        self.assertEqual(M.target('GET','/icerik?id=news')[1],{'id':'news'})
        self.assertEqual(M.target('GET','/media/'+'a'*64+'.png')[2],'a'*64)
        for target in ['/media/%2e%2e/x','/media/../x','//admin','http://foreign/admin','/admin#x','/admin%2f','/icerik?id=a&id=b','/icerik?id=%FF','/icerik?id=%GG','/admin?extra=1','/media/'+'A'*64+'.png']:
            with self.assertRaises((ValueError,UnicodeError)):M.target('GET',target)
        with self.assertRaises(M.Rejected):M.target('POST','/media/'+'a'*64+'.png')
    def test_form_types_boundaries_and_plaintext_remain_data(self):
        import urllib.parse
        values={'id':'news','title':'<script>Ö</script>','body':"'); DROP TABLE x;--",'_csrf':'a'*64}
        raw=urllib.parse.urlencode(values).encode();self.assertEqual(M.form('/ekle',raw),raw)
        for v in [dict(values,id='../x'),dict(values,extra='x'),dict(values,title='x'*513),dict(values,body='x'*8193),dict(values,_csrf='bad')]:
            with self.assertRaises(M.Rejected):M.form('/ekle',urllib.parse.urlencode(v).encode())
        with self.assertRaises(M.Rejected):M.form('/ekle',raw+b'&id=other')
    def parse(self,wire,deadline=None):
        a,b=socket.socketpair()
        try:b.sendall(wire);b.shutdown(socket.SHUT_WR);return M.request(a,'127.0.0.1:9443',deadline or time.monotonic()+1)
        finally:a.close();b.close()
    def test_host_origin_headers_and_single_length_fail_closed(self):
        good=b'GET / HTTP/1.1\r\nHost: 127.0.0.1:9443\r\n\r\n';self.assertEqual(self.parse(good)[0],'GET')
        for wire in [good.replace(b'9443',b'9444'),good.replace(b'\r\n\r\n',b'\r\nHost: 127.0.0.1:9443\r\n\r\n'),good.replace(b'\r\n\r\n',b'\r\nTransfer-Encoding: chunked\r\n\r\n'),good.replace(b'\r\n\r\n',b'\r\nForwarded: for=foreign\r\n\r\n'),good+b'EXTRA',b'POST /medya-yukle?id=news HTTP/1.1\r\nHost: 127.0.0.1:9443\r\nOrigin: http://127.0.0.1:9443\r\nContent-Type: application/octet-stream\r\nContent-Length: 1\r\nX-Zee-CSRF: '+b'a'*64+b'\r\n\r\nx']:
            with self.assertRaises(M.Rejected):self.parse(wire)
    def test_absolute_deadline_short_body_and_capture_bounds(self):
        wire=b'POST /medya-yukle?id=news HTTP/1.1\r\nHost: 127.0.0.1:9443\r\nOrigin: https://127.0.0.1:9443\r\nContent-Type: application/octet-stream\r\nContent-Length: 2\r\nX-Zee-CSRF: '+b'a'*64+b'\r\n\r\nx'
        with self.assertRaises(M.Rejected):self.parse(wire)
        with self.assertRaises(M.Rejected):self.parse(wire.replace(b'Content-Length: 2',b'Content-Length: 1048577'))
        a,b=socket.socketpair();start=time.monotonic()
        try:
            b.sendall(b'GET / HTTP/1.1\r\n')
            with self.assertRaises((M.Rejected,OSError)):M.request(a,'127.0.0.1:9443',start+.05)
            self.assertLess(time.monotonic()-start,.5)
        finally:a.close();b.close()
    def test_png_content_hash_type_dimensions_and_symlink(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td).resolve();root.chmod(0o700);(root/'media').mkdir(mode=0o700);raw=png();h=hashlib.sha256(raw).hexdigest();p=root/'media'/(h+'.png');p.write_bytes(raw);p.chmod(0o600)
            self.assertEqual(M.png_bytes(root,h),raw)
            p.write_bytes(raw+b'x')
            with self.assertRaises(M.Rejected):M.png_bytes(root,h)
            p.unlink();p.symlink_to(root/'foreign')
            with self.assertRaises(OSError):M.png_bytes(root,h)
    def test_unknown_pending_retained_and_no_replaced_foreign_unlink(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td).resolve();root.chmod(0o700);issued=M.begin_effect(root,'/ekle',b'PRIVATE_BODY')
            value=json.loads(issued);self.assertNotIn(b'PRIVATE_BODY',issued);self.assertEqual(value['schema'],'zee-content-starter-pending-1')
            with self.assertRaises(M.Rejected):M.begin_effect(root,'/ekle',b'next')
            p=root/'starter-pending.json';p.write_bytes(b'foreign')
            with self.assertRaises(M.Rejected):M.complete_effect(root,issued)
            self.assertEqual(p.read_bytes(),b'foreign')
    def test_visible_clear_fsync_failure_is_not_fake_paused(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td).resolve();root.chmod(0o700);issued=M.begin_effect(root,'/sil',b'body')
            with patch.object(M,'syncdir',side_effect=OSError('controlled')):
                with self.assertRaises(M.Visible):M.complete_effect(root,issued)
            self.assertFalse((root/'starter-pending.json').exists())
    def test_config_duplicate_and_fifo_rejected_before_blocking(self):
        with self.assertRaises(M.Rejected):M.decode(b'{"a":1,"a":2}')
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/'fifo';os.mkfifo(p);start=time.monotonic()
            with self.assertRaises(M.Rejected):M.bounded(p,1024)
            self.assertLess(time.monotonic()-start,.5)
            cert=Path(td)/'cert';key=Path(td)/'key';cert.write_bytes(b'-----BEGIN CERTIFICATE-----\nowned');key.write_bytes(b'-----BEGIN ENCRYPTED PRIVATE KEY-----\nPRIVATE_CANARY');cert.chmod(0o600);key.chmod(0o600)
            with self.assertRaises(M.Rejected):M.tls_material(cert,key)
if __name__=='__main__':unittest.main()
