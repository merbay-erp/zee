"""Named capability subprocess negatives and ordering models; not native acceptance."""
import builtins,contextlib,hashlib,importlib.util,io,json,os,socket,subprocess,sys,tempfile,unittest
from pathlib import Path
from unittest import mock
ROOT=Path(__file__).resolve().parents[2];TOOL=ROOT/'scripts/credential-baslat.py'
spec=importlib.util.spec_from_file_location('launcher',TOOL);C=importlib.util.module_from_spec(spec);spec.loader.exec_module(C)
class Capabilities(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name).resolve();self.fd=[];self.sock=socket.socket();self.sock.bind(('127.0.0.1',0));self.sock.listen();self.sock.setblocking(False);self.sockets=[self.sock];self.port=self.sock.getsockname()[1]
  self.rr,self.rw=self.pipe();self.ar,self.aw=self.pipe();self.sr,self.sw=self.pipe();os.set_blocking(self.sw,False);self.mr,self.mw=self.pipe();os.set_blocking(self.mw,False)
  self.secret=self.root/'credentials';self.secret.mkdir(mode=0o700);(self.secret/'key').write_text('K267_PRIVATE_MODEL_CANARY');(self.secret/'key').chmod(0o400)
  self.source=self.root/'private.dil';self.source.write_text('K267_PRIVATE_SOURCE_CANARY !!! malformed\n');self.binary=Path(sys.executable).resolve();self.pin=hashlib.sha256(self.binary.read_bytes()).hexdigest();self.gen='a'*32
 def pipe(self):
  pair=os.pipe();self.fd.extend(pair);return pair
 def tearDown(self):
  for sock in self.sockets:sock.close()
  for fd in self.fd:
   try:os.close(fd)
   except OSError:pass
  self.temp.cleanup()
 def args(self):
  return ['--credential-root',str(self.secret),'--env','ZEE_K267_PROBE=key','--dil',str(self.binary),'--native-listener-fd',str(self.sock.fileno()),'--native-ready-fd',str(self.rw),'--native-activate-fd',str(self.ar),'--native-startup-fd',str(self.sw),'--native-generation',self.gen,'--native-worker-port',str(self.port),'--native-binary-sha256',self.pin,'--','çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(self.port),'--web-listener-fd',str(self.sock.fileno()),'--web-ready-fd',str(self.rw),'--web-activate-fd',str(self.ar),'--web-generation',self.gen,str(self.source)]
 def change(self,args,flag,value):args[args.index(flag)+1]=str(value)
 def test_unavailable_fcntl_import_retains_quiet_unsupported_rejection(self):
  original=builtins.__import__
  def restricted(name,*args,**kwargs):
   if name=='fcntl':raise ModuleNotFoundError('modeled unavailable fcntl')
   return original(name,*args,**kwargs)
  with mock.patch.object(builtins,'__import__',side_effect=restricted):
   spec=importlib.util.spec_from_file_location('unsupported_launcher',TOOL);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
  with mock.patch.object(module,'child_environment',side_effect=AssertionError('no secret read')),contextlib.redirect_stdout(io.StringIO()) as out:self.assertEqual(module.main([]),2)
  self.assertEqual(json.loads(out.getvalue()),{'schema':'zee-credential-baslat-1','source':'launcher','exit':2})
 def test_borrowed_positive_preserves_descriptors(self):
  p=C.N.Listener(self.sock.fileno(),self.rw,self.ar,self.sw,self.gen,self.port,self.pin);self.assertEqual(C.N.borrowed(p),p);self.assertEqual(self.sock.getsockname()[1],self.port)
 def test_invalid_capability_matrix_actual_subprocess_fixed_metadata(self):
  modes=['startup_ready_alias','opposite_control_ends','stdio_alias','blocking_listener','wrong_port','wrong_ready_access','wrong_startup_access','bad_nonce','partial_host','duplicate_host','command_mismatch','postsource_binding','ambient_listener_without_host','udp_listener','wildcard_listener','not_listening']
  original=self.sock;original_port=self.port
  for mode in modes:
   if mode in ('udp_listener','wildcard_listener','not_listening'):
    self.sock=socket.socket(socket.AF_INET,socket.SOCK_DGRAM if mode=='udp_listener' else socket.SOCK_STREAM);self.sockets.append(self.sock);self.sock.bind(('0.0.0.0' if mode=='wildcard_listener' else '127.0.0.1',0));self.port=self.sock.getsockname()[1]
    if mode=='wildcard_listener':self.sock.listen()
   a=self.args();stdout=subprocess.PIPE;extra=[];self.sock.setblocking(False)
   if mode=='startup_ready_alias':self.change(a,'--native-startup-fd',self.rw)
   elif mode=='opposite_control_ends':self.change(a,'--native-activate-fd',self.rr);self.change(a,'--web-activate-fd',self.rr)
   elif mode=='stdio_alias':stdout=self.sw
   elif mode=='blocking_listener':self.sock.setblocking(True)
   elif mode=='wrong_port':self.change(a,'--native-worker-port',self.port+1)
   elif mode=='wrong_ready_access':self.change(a,'--native-ready-fd',self.rr);self.change(a,'--web-ready-fd',self.rr)
   elif mode=='wrong_startup_access':self.change(a,'--native-startup-fd',self.sr)
   elif mode=='bad_nonce':self.change(a,'--native-generation','PRIVATE_NONCE_CANARY')
   elif mode=='partial_host':i=a.index('--native-generation');del a[i:i+2]
   elif mode=='duplicate_host':a[0:0]=['--native-ready-fd',str(self.rw)]
   elif mode=='command_mismatch':self.change(a,'--web-ready-fd',self.sw)
   elif mode=='postsource_binding':i=a.index('--web-ready-fd');pair=a[i:i+2];del a[i:i+2];a+=pair
   elif mode=='ambient_listener_without_host':
    for flag in C.N.HOST_FLAGS:
     i=a.index(flag);del a[i:i+2]
   result=subprocess.run([sys.executable,'-I','-B',str(TOOL),*a],stdin=subprocess.DEVNULL,stdout=stdout,stderr=subprocess.PIPE,pass_fds=tuple(self.fd+[self.sock.fileno()]),timeout=4)
   self.assertEqual(result.returncode,2,mode);self.assertEqual(result.stderr,b'',mode)
   if stdout==subprocess.PIPE:
    value=json.loads(result.stdout);self.assertEqual(value,{'schema':'zee-credential-baslat-1','source':'launcher','exit':2,'reason':'native_listener_capability'},mode);self.assertNotIn(b'CANARY',result.stdout)
   for reader in (self.rr,self.sr):
    os.set_blocking(reader,False)
    with self.assertRaises(BlockingIOError):os.read(reader,512)
   self.assertEqual(self.sock.getsockname()[1],self.port)
   self.sock=original;self.port=original_port
  self.sock.setblocking(False)
  # Actual CLI processes: malformed/partial roles must be quiet independently
  # of successful typed extraction. Audit hooks reject secret/source reads or
  # native Popen, so silence alone cannot conceal passing the presecret boundary.
  for role in ('--native-listener-fd','--native-ready-fd','--native-activate-fd','--native-startup-fd','--native-metrics-fd'):
   for spelling in ('spaced','equals','duplicate_invalid_first','duplicate_overflow_first','duplicate_nonascii_first','duplicate_outofrange_first','duplicate_equals_first','leading_zero_equals','leading_zero_spaced'):
    for standard in ('stdout','stderr'):
     with self.subTest(role=role,spelling=spelling,standard=standard):
      read,write=self.pipe();os.set_blocking(read,False)
      number=('0'*5000 if spelling.startswith('leading_zero_') else '')+str(write)
      option=[role+'='+number] if spelling in ('equals','leading_zero_equals') else [role,number]
      if spelling.startswith('duplicate_'):
       bad={'duplicate_invalid_first':'invalid','duplicate_overflow_first':'9'*5000,'duplicate_nonascii_first':'１２','duplicate_outofrange_first':'2147483648','duplicate_equals_first':'invalid'}[spelling]
       option=([role+'='+bad] if spelling=='duplicate_equals_first' else [role,bad])+option
      command=['--credential-root',str(self.secret),'--env','ZEE_K267_PROBE=key','--dil',str(self.binary),*option,'--','çalıştır',str(self.source)]
      bootstrap='import os,runpy,sys; tool=sys.argv[1]; secret=sys.argv[2]; source=sys.argv[3]; sys.argv=[tool]+sys.argv[4:]\ndef audit(event,args):\n if event=="subprocess.Popen" or (event=="open" and isinstance(args[0],str) and (args[0]==source or args[0].startswith(secret+os.sep))):os._exit(71)\nsys.addaudithook(audit);runpy.run_path(tool,run_name="__main__")'
      result=subprocess.run([sys.executable,'-I','-B','-c',bootstrap,str(TOOL),str(self.secret),str(self.source),*command],stdin=subprocess.DEVNULL,stdout=write if standard=='stdout' else subprocess.PIPE,stderr=write if standard=='stderr' else subprocess.PIPE,pass_fds=(write,),timeout=4)
      self.assertEqual(result.returncode,2)
      self.assertEqual(result.stdout if standard=='stderr' else result.stderr,b'')
      with self.assertRaises(BlockingIOError):os.read(read,512)
 def test_invalid_role_rejects_before_any_credential_read_model(self):
  a=self.args();self.change(a,'--native-startup-fd',self.rw)
  with mock.patch.object(C,'child_environment',side_effect=AssertionError('credential read forbidden')),contextlib.redirect_stdout(io.StringIO()) as out:self.assertEqual(C.main(a),2)
  self.assertEqual(json.loads(out.getvalue())['reason'],'native_listener_capability')
 def test_native_passfds_excludes_startup_and_ambient_actual_probe(self):
  output=self.root/'probe.json';probe=self.root/'probe-dil';ambient_r,ambient_w=self.pipe()
  probe.write_text('#!'+sys.executable+'\nimport os,json\ndef exists(fd):\n try:os.fstat(fd);return True\n except OSError:return False\nv={"startup":exists('+str(self.sw)+'),"ambient":exists('+str(ambient_w)+'),"listener":exists('+str(self.sock.fileno())+')}\nopen('+repr(str(output))+',"w").write(json.dumps(v))\nprint("K267_NATIVE_STDOUT_CANARY")\n');probe.chmod(0o700);a=self.args();self.change(a,'--dil',probe);self.change(a,'--native-binary-sha256',hashlib.sha256(probe.read_bytes()).hexdigest())
  result=subprocess.run([sys.executable,'-I','-B',str(TOOL),*a],capture_output=True,pass_fds=tuple(self.fd+[self.sock.fileno()]),timeout=4);self.assertEqual(result.returncode,0,result.stdout+result.stderr);self.assertEqual(result.stderr,b'');self.assertNotIn(b'CANARY',result.stdout);self.assertEqual(json.loads(output.read_text()),{'startup':False,'ambient':False,'listener':True});os.set_blocking(self.sr,False);record=json.loads(os.read(self.sr,512));self.assertEqual(record['generation'],self.gen);self.assertEqual(record['binary_sha256'],hashlib.sha256(probe.read_bytes()).hexdigest());self.assertGreater(record['pid'],0)
if __name__=='__main__':unittest.main()
