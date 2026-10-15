"""External Unix owned-listener pilot, fixed application revision only."""
import hashlib,importlib.util,json,os,secrets,selectors,socket,subprocess,time
from pathlib import Path
from types import SimpleNamespace
SCRIPTS=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('owner',SCRIPTS/'worker-supervisor.py');OWNER=importlib.util.module_from_spec(spec);spec.loader.exec_module(OWNER)
class Host:
 def __init__(self,binary,project,build,revision,expected_binary_sha):
  self.binary,self.project,self.build,self.revision=binary,project,build,revision
  self.expected_binary_sha=expected_binary_sha
  OWNER.require(hashlib.sha256(binary.read_bytes()).hexdigest()==expected_binary_sha,'binary_pin')
  self.application_hashes={name:hashlib.sha256(OWNER.regular_bytes(project/name)[0]).hexdigest() for name in ['ana.dil','proje.dil']}
  self.owner=OWNER.Supervisor(SimpleNamespace(project=project,dil=binary,stop_grace=.5),OWNER.JsonLog())
  self.listener=socket.socket(socket.AF_INET,socket.SOCK_STREAM,socket.IPPROTO_TCP);self.listener.bind(('127.0.0.1',0));self.listener.listen(128);self.listener.setblocking(False);self.port=self.listener.getsockname()[1]
  self.generation_root=project/'receipts';self.generation_root.mkdir(mode=0o700)
 def spawn(self,extra=(),descriptor=None,metrics=False,metric_first=False,closed_metric_reader=False):
  OWNER.require(hashlib.sha256(self.binary.read_bytes()).hexdigest()==self.expected_binary_sha,'binary_pin')
  OWNER.require(all(hashlib.sha256(OWNER.regular_bytes(self.project/name)[0]).hexdigest()==expected for name,expected in self.application_hashes.items()),'application_pin')
  nonce=secrets.token_hex(16);directory=self.generation_root/nonce;directory.mkdir(mode=0o700);receipt=directory/'ready.json';ready_r,ready_w=os.pipe();activate_r,activate_w=os.pipe();start=time.time_ns()
  received=self.listener.fileno() if descriptor is None else descriptor
  metric_read=metric_write=None
  if metrics:
   metric_read,metric_write=os.pipe();os.set_blocking(metric_write,False)
   if closed_metric_reader:os.close(metric_read);metric_read=None
   else:os.set_blocking(metric_read,False)
  argv=[str(self.binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(self.port),'--web-listener-fd',str(received),'--web-ready-fd',str(ready_w),'--web-activate-fd',str(activate_r),'--web-generation',nonce,*extra,str(self.project),str(receipt.relative_to(self.project))]
  inherited=[received,ready_w,activate_r]
  if metrics:
   location=2 if metric_first else len(argv)-2
   argv[location:location]=['--web-metrik-fd',str(metric_write)];inherited.append(metric_write)
  try:
   child=self.owner.spawn(argv,capture=True,passed_fds=tuple(inherited))
  except BaseException:
   for fd in [ready_r,activate_w,metric_read]:
    if fd is not None:os.close(fd)
   raise
  finally:
   for fd in [ready_w,activate_r,metric_write]:
    if fd is not None:os.close(fd)
  return {'child':child,'nonce':nonce,'receipt':receipt,'start_ns':start,'ready':ready_r,'activate':activate_w,'closing':False,'closed':False,'metric':metric_read}
 def ready(self,g):
  data=bytearray();deadline=time.monotonic()+5
  with selectors.DefaultSelector() as selector:
   selector.register(g['ready'],selectors.EVENT_READ)
   while True:
    OWNER.require(time.monotonic()<deadline,'ready_timeout')
    if not selector.select(.1):continue
    chunk=os.read(g['ready'],4097);data.extend(chunk);OWNER.require(len(data)<=4096,'ready_bound')
    if not chunk:break
  os.close(g['ready']);g['ready']=None
  if not data:
   self.await_exit(g,2);self.stop(g);raise AssertionError('native control EOF before ready')
  value=OWNER.decode_json(data)
  OWNER.require(set(value)=={'schema','pid','generation','port','build'} and value['schema']=='zee-native-listener-ready-1' and value['pid']==g['child'].pid and value['generation']==g['nonce'] and value['port']==self.port and value['build']==self.build,'ready_identity')
  self.receipt(g);return value
 def receipt(self,g):
  stat=g['receipt'].lstat();OWNER.require(stat.st_mtime_ns>=g['start_ns'],'receipt_fresh')
  value=OWNER.decode_json(OWNER.regular_bytes(g['receipt'])[0])
  OWNER.require(set(value)=={'schema','application_revision','ready'} and value=={'schema':'zee-worker-readiness-1','application_revision':self.revision,'ready':True},'receipt_revision')
 def activate(self,g,message=None):
  self.receipt(g);os.write(g['activate'],message if message is not None else ('ACTIVATE '+g['nonce']+'\n').encode());os.close(g['activate']);g['activate']=None
 def await_exit(self,g,timeout):
  deadline=time.monotonic()+timeout
  while self.owner.exit_status(g['child']) is None:
   OWNER.require(time.monotonic()<deadline,'owned_exit_timeout');time.sleep(.02)
  return self.owner.exit_status(g['child'])
 def stop(self,g):
  if g['closed']:return
  OWNER.require(not g['closing'],'closing_ownership');g['closing']=True
  for key in ['ready','activate','metric']:
   if g[key] is not None:os.close(g[key]);g[key]=None
  self.owner.stop_child(g['child'])
  if g['child'].stderr:g['child'].stderr.close()
  g['closed']=True
 def cleanup(self):
  self.owner.cleanup();self.listener.close()
