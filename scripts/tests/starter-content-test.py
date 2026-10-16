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

# K308 inert lifecycle selection; never imports the native campaign.
import ast
import contextlib
import sys
import types
TARGET=R/'scripts/tests/starter-content-native.py'

class StarterLifecycleContracts(unittest.TestCase):
    def setUp(self):
        self.guards=contextlib.ExitStack()
        def forbidden(*a,**k):raise AssertionError('real process/socket/signal forbidden')
        for name in ['subprocess.Popen','socket.socket','socket.create_connection','os.kill','os.killpg','os.fork','signal.signal']:
            self.guards.enter_context(patch(name,forbidden))
    def tearDown(self):self.guards.close()
    def environment(self):
        tree=ast.parse(TARGET.read_bytes())
        names={'cancel','cleanup_end','close_pg','stop_host','publish_proof','PrimaryExitStack','command'}
        nested_names={'close_pg','stop_host'}
        nodes=[n for n in tree.body if isinstance(n,(ast.FunctionDef,ast.ClassDef)) and n.name in names-nested_names]
        mains=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='main']
        self.assertEqual(len(mains),1)
        nodes += [n for n in ast.walk(mains[0]) if isinstance(n,ast.FunctionDef) and n.name in nested_names]
        self.assertEqual(len(nodes),len(names))
        self.assertEqual({n.name for n in nodes},names)
        e={'H':{'CANCELLED':False},'campaign_cancelled':False,'cleaning':False,'campaign_started':0,'campaign_deadline':600,'cleanup_limit':None,
           'cleanup':['owned-front-and-native-closed'],'proof':{'status':'passed','front_terminal':{'reaped':True}},'raw':[],'json':json,'sys':sys,'Path':Path,
           'contextlib':contextlib,'now':1,'exists':False,'writes':{},'on_close':lambda:None,'on_write':lambda _:None,'failwrites':{},'restores':[],'oldsignals':{},'command_counter':0}
        e['time']=types.SimpleNamespace(monotonic=lambda:e['now'])
        class File:
            def __init__(self,name):self.label=name;self.name=name.split('/')[-1]
            def __truediv__(self,name):return File(self.label+'/'+name)
            def is_file(self):return True
            def exists(self):
                if e.get('exists_error'):raise OSError('fake exists')
                return e['exists']
            def write_text(self,text):
                e['on_write'](self.label)
                failures=e['failwrites'].get(self.label,[])
                if failures:raise failures.pop(0)
                e['writes'][self.label]=text
        class Signal:
            SIGTERM=15;SIGINT=2
            def __init__(self):self.original={15:lambda *_:None,2:lambda *_:None};self.handlers=dict(self.original);self.n=0
            def signal(self,n,h):
                if h is self.original[n]:
                    e['restores'].append(n)
                    if e.get('on_restore'):e['on_restore']()
                    if e.get('restore_late'):e['now']=600
                    if n==e.get('restore_error'):raise OSError('fake restore')
                else:
                    self.n+=1
                    if self.n==e.get('install_error'):raise OSError('fake install')
                prev=self.handlers[n];self.handlers[n]=h;return prev
        e.update(signal=Signal(),work=File('work'),a=types.SimpleNamespace(output=File('output')),cluster=types.SimpleNamespace(close=lambda:e['on_close']()),M=types.SimpleNamespace(sha=lambda b:'public-empty-digest',bounded=lambda p,n:b'PUBLIC_ONLY'))
        exec(compile(ast.Module(body=nodes,type_ignores=[]),str(TARGET),'exec'),e)
        main=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='main');protected=next(n for n in main.body if isinstance(n,ast.Try));install=protected.body[0];self.assertIsInstance(install,ast.For)
        e['install']=lambda:exec(compile(ast.Module(body=[install],type_ignores=[]),str(TARGET),'exec'),e)
        return e
    def installed(self):e=self.environment();e['install']();return e
    def failed(self,e,kind=ValueError):
        with self.assertRaises(kind):e['publish_proof']()
        self.assertEqual(e['proof']['status'],'failed')
        if 'output/proof.json' in e['writes']:self.assertEqual(json.loads(e['writes']['output/proof.json'])['status'],'failed')
    def host(self,e,exit=0,stderr=b''):
        child=types.SimpleNamespace(returncode=None,stderr=types.SimpleNamespace(read=lambda _:stderr,close=lambda:None))
        calls=[]
        def stop(c,grace):self.assertIs(c,child);calls.append(c);c.returncode=exit;return exit
        e['host']=child;e['H']['stop_owned']=stop;return child,calls
    def test_starter_uncancelled_cleanup_and_publication(self):
        e=self.installed();e['close_pg']();e['publish_proof']();self.assertEqual(e['proof']['status'],'passed');self.assertEqual(e['restores'],[15,2]);self.assertFalse(e['cleaning'])
    def test_starter_TERM_INT_during_cleanup_stays_sticky(self):
        for sig in [15,2]:
            with self.subTest(signal=sig):
                e=self.installed();e['on_close']=lambda:e['signal'].handlers[sig](sig,None);e['close_pg']();self.assertTrue(e['campaign_cancelled']);self.assertTrue(e['H']['CANCELLED']);self.failed(e)
    def test_starter_prior_repeat_cancel_never_resets(self):
        e=self.installed();e['cancel']();e['close_pg']();e['close_pg']();self.assertTrue(e['H']['CANCELLED']);self.failed(e)
    def test_starter_close_failure_restores_mask_and_cleaning(self):
        e=self.installed();e['H']['CANCELLED']=True
        def close():raise OSError('fake close')
        e['on_close']=close
        with self.assertRaises(OSError):e['close_pg']()
        self.assertFalse(e['cleaning']);self.assertTrue(e['H']['CANCELLED']);self.assertEqual(e['cleanup'][-1],'owned-pg-cleanup-failed');self.failed(e)
    def test_starter_partial_handler_install_restores_subset(self):
        e=self.environment();e['install_error']=2
        with self.assertRaises(OSError):e['install']()
        e['proof']['status']='failed';self.failed(e);self.assertEqual(e['restores'],[15])
    def test_starter_restore_error_attempts_all_handlers(self):
        e=self.installed();e['close_pg']();e['restore_error']=15;self.failed(e,OSError);self.assertEqual(e['restores'],[15,2])
    def test_starter_deadline_and_late_publish_or_restore_reject(self):
        for phase in ['before','write','restore']:
            with self.subTest(phase=phase):
                e=self.installed();e['close_pg']()
                if phase=='before':e['now']=600
                if phase=='write':e['on_write']=lambda _:e.__setitem__('now',600)
                if phase=='restore':e['restore_late']=True
                self.failed(e)
    def test_starter_body_primary_survives_all_stack_cleanup_errors(self):
        e=self.environment();primary=LookupError('fake first body');called=[]
        def bad(n):called.append(n);raise OSError('fake cleanup')
        try:
            with e['PrimaryExitStack']() as stack:
                stack.callback(bad,1);stack.callback(bad,2);raise primary
        except BaseException as error:self.assertIs(error,primary)
        self.assertEqual(called,[2,1])
    def test_starter_first_publication_error_survives_failed_rewrite(self):
        e=self.installed();e['close_pg']();primary=OSError('fake first write');e['failwrites']['output/raw-metadata.json']=[primary];e['failwrites']['output/proof.json']=[RuntimeError('fake rewrite')]
        with self.assertRaises(OSError) as got:e['publish_proof']()
        self.assertIs(got.exception,primary);self.assertEqual(e['restores'],[15,2]);self.assertEqual(e['proof']['status'],'failed')
    def test_starter_normal_host_same_original_is_reaped(self):
        e=self.installed();e['cleanup']=[];c,calls=self.host(e);e['stop_host']();self.assertEqual(calls,[c]);self.assertIsNone(e['host']);self.assertTrue(e['proof']['front_terminal']['reaped']);self.assertEqual(e['cleanup'],['owned-front-and-native-closed'])
    def test_starter_host_badexit_or_stderr_retains_object(self):
        for code,stderr in [(1,b''),(0,b'fakepublicerror'),(None,b'')]:
            with self.subTest(exit=code):
                e=self.installed();c,calls=self.host(e,code,stderr)
                if code is None:
                    def bad_stop(original,grace):
                        self.assertIs(original,c);calls.append(original);raise OSError('fake original host close error')
                    e['H']['stop_owned']=bad_stop
                with self.assertRaises(OSError if code is None else AssertionError):e['stop_host']()
                self.assertIs(e['host'],c);self.assertEqual(calls,[c]);self.assertFalse(e['cleaning'])
    def test_starter_host_cleanup_cancel_and_global90_one_shot(self):
        e=self.installed();c,calls=self.host(e);old=e['H']['stop_owned']
        def stop(*a,**k):e['cancel']();return old(*a,**k)
        e['H']['stop_owned']=stop;e['stop_host']();self.assertTrue(e['H']['CANCELLED']);first=e['cleanup_end']();e['now']+=20;self.assertEqual(e['cleanup_end'](),first);self.assertEqual(first,91)
    def test_starter_sensitive_command_never_logs_hashes_or_bytes(self):
        e=self.environment();calls=[];secret=b'PRIVATE_PHC_PASSWORD_COOKIE_CANARY'
        e['H']['run']=lambda argv,**kw:(calls.append(kw) or secret)
        self.assertEqual(e['command'](['fake-owned-hash'],sensitive_output=True),secret);self.assertEqual(e['raw'],[]);self.assertNotIn('log_path',calls[0]);self.assertEqual(e['writes'],{})
        e=self.environment();e['H']['run']=lambda argv,**kw:b'PUBLIC_ONLY'
        e['command'](['fake-public-tool'])
        self.assertEqual(e['proof']['command_logs'],[{'name':'command-001.log','sha256':'public-empty-digest','bytes':11}])
        e=self.environment();primary=LookupError('fake original command')
        def broken(*a,**k):raise primary
        def export(*a,**k):raise OSError('fake public log export')
        e['H']['run']=broken;e['M'].bounded=export
        with self.assertRaises(LookupError) as got:e['command'](['fake-public-tool'])
        self.assertIs(got.exception,primary);self.assertEqual(e['proof']['command_log_export_errors'],['OSError'])
    def test_starter_private_remains_or_observation_error_cannot_pass(self):
        for error in [False,True]:
            with self.subTest(error=error):
                e=self.installed();e['close_pg']();e['exists']=True;e['exists_error']=error;self.failed(e,OSError if error else ValueError)
    def work_environment(self):
        e=self.environment();e['host']=None
        e['cluster']=types.SimpleNamespace(closed=True,child=types.SimpleNamespace(returncode=0),probes=[])
        class Work:
            removed=False;changed=False;mode=0o40700
            def lstat(self):return types.SimpleNamespace(st_dev=1,st_ino=3 if self.changed else 2,st_mode=self.mode,st_uid=1,st_gid=1)
            def resolve(self,strict=True):return self
            def exists(self):return not self.removed
        work=Work();e['work']=work;e['work_identity']=(1,2,work.mode,1,1)
        def remove(path):self.assertIs(path,work);work.removed=True
        def private(info):
            if info.st_mode!=0o40700:raise ValueError('fake private directory mode')
        e.update(shutil=types.SimpleNamespace(rmtree=remove),PG=types.SimpleNamespace(private_directory=private))
        node=next(n for n in ast.walk(ast.parse(TARGET.read_bytes())) if isinstance(n,ast.FunctionDef) and n.name=='close_work')
        exec(compile(ast.Module(body=[node],type_ignores=[]),str(TARGET),'exec'),e)
        return e
    def test_starter_actual_close_work_happy_owned_delete(self):
        e=self.work_environment();e['close_work']();self.assertFalse(e['work'].exists())
    def test_starter_actual_close_work_uncertain_identity_deadline_preserve(self):
        for reason in ['host','pgclosed','postmaster','probe','inode','mode','deadline']:
            with self.subTest(reason=reason):
                e=self.work_environment()
                if reason=='host':e['host']=types.SimpleNamespace(returncode=None)
                if reason=='pgclosed':e['cluster'].closed=False
                if reason=='postmaster':e['cluster'].child.returncode=None
                if reason=='probe':e['cluster'].probes=[{'child':types.SimpleNamespace(returncode=None)}]
                if reason=='inode':e['work'].changed=True
                if reason=='mode':e['work'].mode=0o40755
                if reason=='deadline':e['cleanup_limit']=0
                with self.assertRaises(ValueError):e['close_work']()
                self.assertTrue(e['work'].exists())
    def test_starter_cancellation_during_publication_or_restore_cannot_pass(self):
        for phase in ['write','restore']:
            with self.subTest(phase=phase):
                e=self.installed();e['close_pg']()
                if phase=='write':e['on_write']=lambda _:e['cancel']()
                else:e['on_restore']=lambda:e['cancel']()
                self.failed(e);self.assertTrue(e['campaign_cancelled']);self.assertEqual(e['restores'],[15,2])
    def test_starter_active_body_exception_survives_failed_publication(self):
        e=self.installed();e['close_pg']();primary=LookupError('fake original body')
        try:raise primary
        except LookupError:
            e['proof'].update(status='failed',failure_type='LookupError')
            e['failwrites']['output/proof.json']=[OSError('fake first failed-save'),RuntimeError('fake second failed-save')]
            with self.assertRaises(LookupError) as got:e['publish_proof']()
            self.assertIs(got.exception,primary)
        self.assertEqual(e['restores'],[15,2]);self.assertEqual(e['proof']['status'],'failed')

class StarterForegroundPreparation(unittest.TestCase):
    def test_foreground_starter_postgres_preflight(self):
        import subprocess
        result=subprocess.run([sys.executable,'-I','-B',str(R/'scripts/tests/starter-pg-owner-preflight-test.py')],
                              cwd=R,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,check=False)
        self.assertEqual(result.returncode,0,result.stderr.decode('utf8','replace'))
        self.assertEqual(result.stdout,b'')
        stderr=result.stderr.decode('utf8','strict')
        self.assertIn('\nRan 23 tests in ',stderr)
        self.assertTrue(stderr.rstrip().endswith('\nOK'),stderr)

if __name__=='__main__':unittest.main()
