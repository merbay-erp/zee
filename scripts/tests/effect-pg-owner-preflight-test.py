"""UNRUN selected-AST fake resource lifecycle models; no native/PG/socket/thread actions."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import types
import unittest
from unittest import mock

STATE='FINALIZED-EFFECT-PREFLIGHT-ROOT-AND-PEER-REVIEWED'
TARGET=Path(__file__).resolve().with_name('effect-dispatcher-native-test.py')
TARGET_SHA=None  # Requires exact reviewed candidate binding before any run.

class FakeEvent:
 def __init__(self):self.stopped=False
 def set(self):self.stopped=True
 def is_set(self):return self.stopped

class FakeClock:
 def __init__(self):self.now=0
 def monotonic(self):return self.now

class FakeWorld:
 def __init__(self):self.events=[];self.sockets=[];self.threads=[];self.failures={};self.body=None;self.clock=FakeClock()
 def event(self,name):
  self.events.append(name)
  if name in self.failures:raise self.failures[name]
 def socket(self):
  self.event('socket.create');world=self;name='socket'+str(len(self.sockets))
  class Socket:
   def bind(self,value):world.event(name+'.bind')
   def getsockname(self):return ('127.0.0.1',12345)
   def listen(self,value):world.event(name+'.listen')
   def settimeout(self,value):world.event(name+'.timeout')
   def shutdown(self,value):world.event(name+'.shutdown')
   def close(self):world.event(name+'.close')
  result=Socket();self.sockets.append(result);return result
 def thread(self,**kwargs):
  self.event('thread.create');world=self;name='thread'+str(len(self.threads))
  class Thread:
   def start(self):world.event(name+'.start')
   def join(self,timeout):
    if not 0<=timeout<=2:raise AssertionError('bounded fake join')
    world.event(name+'.join')
   def is_alive(self):return False
  result=Thread();self.threads.append(result);return result
 def fault(self,*args,**kwargs):
  self.event('fault.construct');world=self
  class Fault:
   state={}
   done=types.SimpleNamespace(set=lambda:world.event('fault.done'))
   listener=None;thread=None;sockets=[];threads=[]
   def start(self):world.event('fault.start');return self
   def close(self):world.event('fault.done')
  return Fault()

class EffectLifecycleParts(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  before=TARGET.stat();raw=TARGET.read_bytes();after=TARGET.stat()
  key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
  if key(before)!=key(after) or len(raw)>131072 or hashlib.sha256(raw).hexdigest()!=TARGET_SHA:raise ValueError('exact stable reviewed public candidate required')
  tree=ast.parse(raw)
  names={'need','effect_begin_terminal_cleanup','effect_close_timeout','effect_cleanup_attempt','effect_close_socket','effect_component','effect_close_preserving','effect_verify','effect_close_commit','effect_cancel','effect_install_handlers','effect_primary','effect_publish','effect_public_wire','effect_validate_cases','effect_shared_helper_end','effect_terminal_cleanup'}
  selected=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name in names]
  if len(selected)!=len(names) or {n.name for n in selected}!=names:raise ValueError('unique exact module helpers')
  for name in ('OperationProxy','SMTP','EffectWork','Fixture','EffectSupervisor'):
   classes=[n for n in tree.body if isinstance(n,ast.ClassDef) and n.name==name]
   if len(classes)!=1:raise ValueError('unique source class')
   wanted={'cleanup','stop_child'} if name=='EffectSupervisor' else {'__init__','start','close'}
   methods=[n for n in classes[0].body if isinstance(n,ast.FunctionDef) and n.name in wanted]
   if len(methods)!=len(wanted) or {n.name for n in methods}!=wanted:raise ValueError('exact unique selected methods')
   selected.append(ast.ClassDef(name=name,bases=classes[0].bases if name=='EffectSupervisor' else [],keywords=[],body=methods,decorator_list=[]))
  if any(isinstance(n,(ast.Import,ast.ImportFrom,ast.Global)) and not isinstance(n,ast.Global) for n in ast.walk(ast.Module(body=selected,type_ignores=[]))):raise ValueError('no import in selected AST')
  cls.selected=ast.fix_missing_locations(ast.Module(body=selected,type_ignores=[]))
 def setUp(self):
  self.forbidden=[]
  def reject(*args,**kwargs):self.forbidden.append(True);raise AssertionError('real OS/network/process/thread/signal capability forbidden')
  self.guards=mock.patch.multiple('subprocess',Popen=reject,run=reject,call=reject,check_output=reject)
  self.guards.start();self.addCleanup(self.guards.stop)
  for name in ('socket.socket','socket.create_connection','threading.Thread','os.kill','os.killpg','os.fork','os.waitid','os.open','os.read','os.write','os.close','os.fstat','os.lseek','os.ftruncate','os.fsync','signal.signal'):
   patch=mock.patch(name,side_effect=reject);patch.start();self.addCleanup(patch.stop)
  self.addCleanup(lambda:self.assertEqual(self.forbidden,[]))
  self.world=FakeWorld();w=self.world
  ns={'__name__':'effect_fake_selected','__builtins__':{'__build_class__':__build_class__,'BaseException':BaseException,'OSError':OSError,'EOFError':EOFError,'TimeoutError':TimeoutError,'ValueError':ValueError,'AssertionError':AssertionError,'getattr':getattr,'list':list,'len':len,'set':set,'dict':dict,'reversed':reversed,'str':str,'all':all,'min':min,'max':max,'type':type,'isinstance':isinstance,'bool':bool,'int':int,'enumerate':enumerate,'super':super},'time':w.clock,'END':180,'CANCELLED':False,'EFFECT_CLEANUP_END':None,'COMPONENTS':[],'EFFECT_CLEANUP_ERRORS':[],'sys':types.SimpleNamespace(exc_info=lambda:(None,w.body,None)),'socket':types.SimpleNamespace(socket=w.socket,SHUT_RDWR=2),'threading':types.SimpleNamespace(Event=FakeEvent,Thread=w.thread,current_thread=lambda:None),'P':types.SimpleNamespace(CommitProxy=w.fault)}
  class Parent:
   def __init__(self,a=None,log=None):self.a=a or types.SimpleNamespace(stop_grace=1);self.children=[];self.failures={};self.events=[]
   def stop_child(self,child):
    self.events.append(child)
    if child in self.failures:raise self.failures[child]
    self.children.remove(child)
   def spawn(self,argv):child=object();self.children.append(child);return child
   def exit_status(self,child):return None
  ns['O']=types.SimpleNamespace(Supervisor=Parent,JsonLog=lambda:None);ns['SimpleNamespace']=types.SimpleNamespace
  exec(compile(self.selected,str(TARGET)+'::<selected-fake-lifecycle>','exec'),ns)
  ns['OperationProxy'].accept=lambda self:None;ns['SMTP'].work=lambda self:None;ns['json']=json;ns['EFFECT_PG_INTEGRATION_COMPLETE']=False;ns['EFFECT_OUTPUT_IDENTITY']=None
  ns['EFFECT_EXPECTED_CASES']=tuple('case'+str(i) for i in range(11));self.ns=ns
  ns['hashlib']=hashlib;ns['BASE']=Path('/fake/public-source');ns['EFFECT_PG_SOURCE_SHA']=hashlib.sha256(b'public-helper').hexdigest()
  ns['bounded']=lambda path,limit:b'public-helper';ns['EFFECT_CLEANING']=False
 def test_resource_free_constructors_have_no_socket_or_thread_start(self):
  self.ns['SMTP']();self.ns['OperationProxy'](12345,'before','prepare')
  self.assertEqual(self.world.events,['fault.construct'])
  self.assertEqual(self.world.sockets,[]);self.assertEqual(self.world.threads,[])
 def test_component_registered_before_failed_listener_bind(self):
  primary=RuntimeError('fake bind');self.world.failures['socket0.bind']=primary
  with self.assertRaises(RuntimeError) as caught:self.ns['effect_component'](self.ns['SMTP'])
  self.assertIs(caught.exception,primary);self.assertEqual(len(self.ns['COMPONENTS']),1)
  component=self.ns['COMPONENTS'][0];self.assertIs(component.listener,self.world.sockets[0]);self.world.body=primary
  self.ns['effect_close_preserving'](component)
  self.assertIn('socket0.close',self.world.events);self.assertFalse(component.cleanup_closed)
 def test_inner_fault_registered_before_partial_start_failure(self):
  primary=RuntimeError('fake inner fault start');self.world.failures['fault.start']=primary
  with self.assertRaises(RuntimeError) as caught:self.ns['effect_component'](self.ns['OperationProxy'],12345,'after','ack')
  self.assertIs(caught.exception,primary);self.assertEqual(len(self.ns['COMPONENTS']),1);self.world.body=primary
  self.ns['effect_close_preserving'](*self.ns['COMPONENTS']);self.assertIn('fault.done',self.world.events)
 def test_router_socket_error_does_not_skip_any_owned_join_or_fault(self):
  component=self.ns['effect_component'](self.ns['OperationProxy'],12345,'before','prepare')
  component.sockets.append(self.world.socket());component.threads.append(self.world.thread())
  primary=OSError('fake first close');self.world.failures['socket0.close']=primary
  with self.assertRaises(OSError) as caught:component.close()
  self.assertIs(caught.exception,primary)
  for event in ('socket1.close','thread0.join','thread1.join','fault.done'):self.assertIn(event,self.world.events)
  self.assertFalse(component.cleanup_closed);self.assertIn(component,self.ns['COMPONENTS'])
 def test_shutdown_error_still_attempts_close_and_smtp_join(self):
  component=self.ns['effect_component'](self.ns['SMTP']);component.sockets.append(self.world.socket())
  primary=RuntimeError('fake shutdown');self.world.failures['socket1.shutdown']=primary
  with self.assertRaises(RuntimeError) as caught:component.close()
  self.assertIs(caught.exception,primary);self.assertIn('socket1.close',self.world.events);self.assertIn('thread0.join',self.world.events)
 def test_case_body_primary_survives_two_close_errors(self):
  primary=RuntimeError('fake body');self.world.body=primary;events=[]
  def fail(role):
   events.append(role);raise OSError(role)
  self.ns['effect_close_preserving'](types.SimpleNamespace(close=lambda:fail('first')),types.SimpleNamespace(close=lambda:fail('second')))
  self.assertEqual(events,['first','second']);self.assertIs(self.world.body,primary);self.assertEqual(len(self.ns['EFFECT_CLEANUP_ERRORS']),2)
 def test_first_cleanup_error_identity_survives_later_cleanup_error(self):
  first=OSError('fake first');second=ValueError('fake second');events=[]
  def fail(role,error):events.append(role);raise error
  with self.assertRaises(OSError) as caught:self.ns['effect_close_preserving'](types.SimpleNamespace(close=lambda:fail('first',first)),types.SimpleNamespace(close=lambda:fail('second',second)))
  self.assertIs(caught.exception,first);self.assertEqual(events,['first','second'])
 def test_terminal_budget_arms_once_and_expired_join_is_zero(self):
  self.world.clock.now=5;self.assertEqual(self.ns['effect_begin_terminal_cleanup'](),95)
  self.world.clock.now=7;self.assertEqual(self.ns['effect_begin_terminal_cleanup'](),95)
  self.world.clock.now=95;self.assertEqual(self.ns['effect_close_timeout'](),0)
 def test_happy_close_is_idempotent_and_cancel_stays_failed(self):
  happy=self.ns['effect_component'](self.ns['SMTP']);happy.close();self.assertTrue(happy.cleanup_closed)
  before=list(self.world.events);happy.close();self.assertEqual(self.world.events,before)
  router=self.ns['effect_component'](self.ns['OperationProxy'],12345,'after','prepare');self.ns['CANCELLED']=True
  with self.assertRaises(AssertionError):router.close()
  self.assertIn('fault.done',self.world.events);self.assertFalse(router.cleanup_closed)

 def test_inner_commit_error_does_not_skip_any_original_socket_or_thread(self):
  w=self.world;fault=w.fault();fault.listener=w.socket();fault.sockets=[w.socket()];fault.thread=w.thread();fault.threads=[w.thread()]
  primary=OSError('fake first inner close');w.failures['socket0.close']=primary
  with self.assertRaises(OSError) as caught:self.ns['effect_close_commit'](fault)
  self.assertIs(caught.exception,primary)
  for event in ('fault.done','socket1.close','thread0.join','thread1.join'):self.assertIn(event,w.events)
 def test_partial_handler_mutation_saved_before_each_failed_install(self):
  for failed in (1,2,3):
   handlers={1:'old1',2:'old2',3:'old3'};saved={};events=[];primary=OSError('fake mutating install')
   def install(number,handler):
    handlers[number]=handler;events.append(number)
    if number==failed:raise primary
   self.ns['signal']=types.SimpleNamespace(SIGTERM=1,SIGINT=2,SIGHUP=3,getsignal=handlers.get,signal=install)
   with self.assertRaises(OSError) as caught:self.ns['effect_install_handlers'](saved)
   self.assertIs(caught.exception,primary);self.assertEqual(set(saved),set(range(1,failed+1)))
   self.assertEqual(saved[failed],'old'+str(failed))
 def root_fixture(self):
  class Root:
   def __init__(self):self.gone=False;self.changed=False
   def resolve(self,strict=True):return self
   def lstat(self):return types.SimpleNamespace(st_dev=1,st_ino=3 if self.changed else 2,st_mode=16832,st_uid=501,st_gid=20)
   def exists(self):return not self.gone
  root=Root();work=self.ns['EffectWork']();work.path=root;work.created_path='/fake/root';work.creation_attempted=True;work.identity=(1,2,16832,501,20)
  pg=types.SimpleNamespace(assert_root_removable=lambda:True)
  fixture=types.SimpleNamespace(closed=True,owner=types.SimpleNamespace(children=[]),foreign_owner=types.SimpleNamespace(children=[]),pg_owner=pg)
  removed=[];self.ns['shutil']=types.SimpleNamespace(rmtree=lambda path:(removed.append(path),setattr(path,'gone',True)))
  self.ns['EFFECT_CLEANUP_END']=90;self.ns['EFFECT_PG_INTEGRATION_COMPLETE']=True
  return work,root,fixture,removed
 def test_root_happy_delete_and_uncertain_original_resources_preserve(self):
  work,root,fixture,removed=self.root_fixture();work.close(fixture)
  self.assertTrue(work.removed);self.assertEqual(removed,[root])
  for bad in ('component','host','foreign','pg','identity','deadline','cleanup-error'):
   self.ns['COMPONENTS']=[];self.ns['EFFECT_CLEANUP_ERRORS']=[];self.world.clock.now=0
   work,root,fixture,removed=self.root_fixture()
   if bad=='component':self.ns['COMPONENTS']=[types.SimpleNamespace(cleanup_closed=False)]
   if bad=='host':fixture.owner.children.append(object())
   if bad=='foreign':fixture.foreign_owner.children.append(object())
   if bad=='pg':fixture.pg_owner.assert_root_removable=lambda:(_ for _ in ()).throw(ValueError('fake PG uncertainty'))
   if bad=='identity':root.changed=True
   if bad=='deadline':self.world.clock.now=90
   if bad=='cleanup-error':self.ns['EFFECT_CLEANUP_ERRORS'].append({'phase':'fake','type':'OSError'})
   with self.assertRaises(ValueError):work.close(fixture)
   self.assertEqual(removed,[]);self.assertTrue(work.preserved);self.assertFalse(work.removed)
 def good_cases(self):
  result=[]
  for index,name in enumerate(self.ns['EFFECT_EXPECTED_CASES']):
   row={'name':name,'passed':True,'smtp_connections':1,'SMTP_DATA_accepted':1,'native_MIME_attachment':False,'PG_readonly_observer':{'state':'done','attempts':1,'latches':1,'acknowledged':1}}
   if index<3:row.update(host_exit=(0,3,3)[index],native_invocations=1)
   elif index in (4,5):row.update(signal=9 if index==4 else 15,host_exit=-9 if index==4 else 3,outer_held_group_cleanup=True)
   elif index>=7:
    mode='before' if (index-7)%2==0 else 'after'
    row.update(host_exit=3,wire={'mode':mode,'commit_error':None,'commits_seen':1,'commit_seen':True,'commit_forwarded':mode=='after','commit_ready':mode=='after','commit_complete':mode=='after','completion_delivered':False,'error_delivered':False,'router_connections':1,'target_operation_connections':1})
   result.append(row)
  return result
 def publication_fixture(self,write_error=None,restore_effect=None,foreign=False):
  ns=self.ns;w=self.world;storage=bytearray();events=[];position=[0];closed=[];opened=[False]
  def statrow(inode,mode):return types.SimpleNamespace(st_dev=1,st_ino=inode,st_mode=mode,st_uid=501,st_gid=20,st_nlink=1)
  class Proof:
   def lstat(self):return statrow(8 if foreign else 7,33152)
  proof=Proof()
  class Output:
   def is_absolute(self):return True
   def resolve(self,strict=True):return self
   def lstat(self):return statrow(2,16832)
   def __truediv__(self,name):selftest.assertEqual(name,'proof.json');return proof
  selftest=self;output=Output();ns['EFFECT_OUTPUT_IDENTITY']=(1,2,16832,501,20)
  def open_(target,flags,mode):
   self.assertIs(target,proof);self.assertTrue(flags&8 and flags&4);self.assertFalse(opened[0]);opened[0]=True;return 97
  def write_(fd,data):
   self.assertEqual(fd,97);events.append('write')
   if write_error is not None:raise write_error
   start=position[0];storage[start:start+len(data)]=data;position[0]+=len(data);return len(data)
  def seek(fd,offset,kind):self.assertEqual(fd,97);position[0]=offset
  def truncate(fd,size):self.assertEqual(fd,97);del storage[size:]
  ns['os']=types.SimpleNamespace(O_WRONLY=1,O_CREAT=2,O_EXCL=4,O_NOFOLLOW=8,O_NONBLOCK=16,SEEK_SET=0,open=open_,fstat=lambda fd:statrow(7,33152),geteuid=lambda:501,lseek=seek,write=write_,ftruncate=truncate,fsync=lambda fd:None,close=lambda fd:closed.append(fd))
  ns['stat']=types.SimpleNamespace(S_ISREG=lambda mode:mode==33152)
  def restore(number,handler):
   events.append('restore'+str(number))
   if restore_effect is not None:restore_effect(number)
  ns['signal']=types.SimpleNamespace(signal=restore);ns['H']=types.SimpleNamespace(CANCELLED=False);ns['EFFECT_CLEANUP_ERRORS']=[];ns['EFFECT_CLEANUP_END']=90;ns['CANCELLED']=False;w.clock.now=0
  report={'schema':'zee-native-effect-campaign-1','status':'failed','cases':self.good_cases(),'body_completed':True,'sources_unchanged':True,'binaries_unchanged':True,'postgres_cluster':{'closed':True,'normal_signal':'SIGINT','exit':0,'observed_exit':0,'reaped':True,'fallback':False,'probe_original_owner_receipts_complete':True},'private_root_removed':True,'all_components_closed':True,'owned_groups_closed':True,'foreign_survived':True,'public_artifacts':[]}
  return report,output,storage,events,closed
 def test_body_primary_is_written_as_genuine_failed_payload(self):
  report,out,raw,events,closed=self.publication_fixture();primary=RuntimeError('fake original body')
  self.assertIs(self.ns['effect_publish'](report,out,{1:'a',2:'b',3:'c'},primary),primary)
  self.assertEqual(json.loads(raw)['status'],'failed');self.assertEqual(closed,[97])
  self.assertEqual([x for x in events if x.startswith('restore')],['restore1','restore2','restore3'])
 def test_cancel_or_deadline_during_restore_failed_only_rewrites(self):
  for mode in ('cancel','deadline'):
   def effect(number):
    if number==2:
     if mode=='cancel':self.ns['CANCELLED']=True
     else:self.world.clock.now=180
   report,out,raw,events,closed=self.publication_fixture(restore_effect=effect)
   self.assertIsNotNone(self.ns['effect_publish'](report,out,{1:'a',2:'b',3:'c'},None))
   self.assertEqual(json.loads(raw)['status'],'failed');self.assertEqual(closed,[97])
   self.assertEqual([x for x in events if x.startswith('restore')],['restore1','restore2','restore3'])
 def test_failed_write_and_restore_keep_original_body_object(self):
  primary=RuntimeError('fake body');secondary=OSError('fake failed write')
  def bad_restore(number):
   if number==1:raise ValueError('fake restore error')
  report,out,raw,events,closed=self.publication_fixture(write_error=secondary,restore_effect=bad_restore)
  self.assertIs(self.ns['effect_publish'](report,out,{1:'a',2:'b',3:'c'},primary),primary)
  self.assertEqual(report['status'],'failed');self.assertEqual(closed,[97]);self.assertEqual(raw,b'')
  self.assertEqual([x for x in events if x.startswith('restore')],['restore1','restore2','restore3'])
 def test_foreign_proof_inode_never_written(self):
  report,out,raw,events,closed=self.publication_fixture(foreign=True)
  self.assertIsNotNone(self.ns['effect_publish'](report,out,{1:'a',2:'b'},None))
  self.assertEqual(raw,b'');self.assertNotIn('write',events);self.assertEqual(closed,[97])

 def test_valid_closed_case_publication_has_only_safe_scalars(self):
  report,out,raw,events,closed=self.publication_fixture()
  self.assertIsNone(self.ns['effect_publish'](report,out,{},None));self.assertEqual(json.loads(raw)['status'],'passed');self.assertEqual(closed,[97])
 def test_malformed_case_values_never_write_public_bytes(self):
  mutations=[(0,'native_invocations',{'password':'PRIVATE'}),(0,'native_invocations',True),(0,'native_invocations',33),(0,'extra','PRIVATE'),(0,'smtp_connections',False),(0,'SMTP_DATA_accepted',33),(0,'host_exit',True),(4,'signal',True),(5,'signal',9),(0,'native_MIME_attachment','PRIVATE')]
  for index,key,value in mutations:
   with self.subTest(index=index,key=key,value=value):
    report,out,raw,events,closed=self.publication_fixture();report['cases'][index][key]=value
    self.assertIsNotNone(self.ns['effect_publish'](report,out,{},None));self.assertEqual(raw,b'');self.assertNotIn('write',events);self.assertEqual(closed,[97])
 def test_fault_wire_extra_boolean_and_wrong_count_never_write(self):
  for key,value in [('target_operation_connections',True),('router_connections',0),('commits_seen',33),('private','PRIVATE'),('mode','observe'),('commit_forwarded',True)]:
   with self.subTest(key=key,value=value):
    report,out,raw,events,closed=self.publication_fixture();report['cases'][7]['wire'][key]=value
    self.assertIsNotNone(self.ns['effect_publish'](report,out,{},None));self.assertEqual(raw,b'');self.assertNotIn('write',events)
 def test_created_root_is_registered_before_resolve_failure(self):
  primary=OSError('fake resolve');events=[]
  class Root:
   def is_dir(self):return True
   def resolve(self,strict=True):events.append('resolve');raise primary
  root=Root();self.ns['Path']=lambda path:root;self.ns['tempfile']=types.SimpleNamespace(mkdtemp=lambda **kw:'/fake/allocated')
  self.ns['shutil']=types.SimpleNamespace(rmtree=lambda path:events.append('DELETE'))
  work=self.ns['EffectWork']()
  with self.assertRaises(OSError) as caught:work.start()
  self.assertIs(caught.exception,primary);self.assertEqual(work.created_path,'/fake/allocated');self.assertIs(work.path,root);self.assertTrue(work.uncertain and work.preserved)
  with self.assertRaises(ValueError):work.close(None)
  self.assertNotIn('DELETE',events);self.assertFalse(work.removed)
 def test_original_supervisor_attempts_every_child_and_preserves_first(self):
  owner=self.ns['EffectSupervisor']();first=object();second=object();owner.children=[first,second];primary=OSError('fake first');owner.failures[first]=primary
  with self.assertRaises(OSError) as caught:owner.cleanup()
  self.assertIs(caught.exception,primary);self.assertEqual(owner.events,[first,second]);self.assertEqual(owner.children,[first]);self.assertEqual(owner.a.stop_grace,1)
 def test_fixture_resource_free_constructor_and_partial_pg_start_retained(self):
  fixture=self.ns['Fixture']('/fake/binary',Path('/fake/pg'),Path('/fake/work'),Path('/fake/out'))
  self.assertIsNone(fixture.pg_owner);self.assertEqual(self.world.events,[])
  primary=OSError('fake PG start');instances=[]
  class PG:
   def __init__(self,*args):instances.append(self)
   def run_owned(self,*args,**kwargs):return b'{}'
   def start(self):raise primary
  self.ns['effect_pg_class']=lambda:PG;self.ns['H']=object();self.ns['sha']=lambda *args:'public-hash';self.ns['sys'].executable='/fake/python'
  # This socket is only an in-memory context manager; no real socket is created.
  class Port:
   def __enter__(self):return self
   def __exit__(self,*args):return False
   def bind(self,address):pass
   def getsockname(self):return ('127.0.0.1',12345)
  self.ns['socket'].socket=Port
  with self.assertRaises(OSError) as caught:fixture.start()
  self.assertIs(caught.exception,primary);self.assertIs(fixture.pg_owner,instances[0]);self.assertEqual(len(fixture.foreign_owner.children),1);self.assertFalse(fixture.closed)
 def test_fixture_close_attempts_pg_and_foreign_after_host_error(self):
  fixture=self.ns['Fixture']('/fake/bin',None,None,None);events=[];primary=OSError('fake host');second=ValueError('fake PG')
  def fail(name,error):events.append(name);raise error
  fixture.owner.cleanup=lambda:fail('host',primary);fixture.pg_owner=types.SimpleNamespace(close=lambda:fail('pg',second))
  child=object();fixture.foreign=child;fixture.foreign_owner.children=[child];fixture.foreign_owner.cleanup=lambda:events.append('foreign')
  with self.assertRaises(OSError) as caught:fixture.close()
  self.assertIs(caught.exception,primary);self.assertEqual(events,['host','pg','foreign']);self.assertFalse(fixture.closed);self.assertTrue(fixture.foreign_survived)
 def test_terminal_cleanup_attempts_all_and_retains_body_primary(self):
  events=[];body=RuntimeError('fake body')
  def fail(name):events.append(name);raise OSError(name)
  self.ns['COMPONENTS']=[types.SimpleNamespace(close=lambda:fail('component1'),cleanup_closed=False),types.SimpleNamespace(close=lambda:fail('component2'),cleanup_closed=False)]
  fixture=types.SimpleNamespace(close=lambda:fail('fixture'),pg_owner=types.SimpleNamespace(public_state=lambda:{'closed':False}),closed=False,owner=types.SimpleNamespace(children=[object()]),foreign_owner=types.SimpleNamespace(children=[]))
  work=types.SimpleNamespace(close=lambda f:fail('work'),removed=False,preserved=True);report={}
  self.assertIs(self.ns['effect_terminal_cleanup'](body,fixture,work,report),body)
  self.assertEqual(events,['component2','component1','fixture','work']);self.assertTrue(report['private_root_preserved_after_uncertain_owner']);self.assertFalse(report['owned_groups_closed']);self.assertEqual(len(report['cleanup_errors']),4)
 def test_terminal_cancel_sticky_and_single_cleanup_deadline(self):
  events=[];self.ns['H']=types.SimpleNamespace(CANCELLED=False)
  def cancelclose():self.ns['effect_cancel'](1,None);events.append('component')
  self.ns['COMPONENTS']=[types.SimpleNamespace(close=cancelclose,cleanup_closed=True)]
  fixture=types.SimpleNamespace(close=lambda:events.append('fixture'),pg_owner=None,closed=True,owner=types.SimpleNamespace(children=[]),foreign_owner=types.SimpleNamespace(children=[]))
  work=types.SimpleNamespace(close=lambda f:events.append('work'),removed=False,preserved=True);report={}
  self.ns['effect_terminal_cleanup'](None,fixture,work,report);deadline=self.ns['EFFECT_CLEANUP_END'];self.world.clock.now=2
  self.ns['effect_terminal_cleanup'](None,fixture,work,report)
  self.assertEqual(self.ns['EFFECT_CLEANUP_END'],deadline);self.assertTrue(self.ns['CANCELLED'] and self.ns['H'].CANCELLED);self.assertEqual(events,['component','fixture','work']*2)
 def test_shared_helper_end_mismatch_fails_before_public_success(self):
  report,out,raw,events,closed=self.publication_fixture();self.ns['bounded']=lambda path,limit:b'changed-helper'
  self.assertIsNotNone(self.ns['effect_publish'](report,out,{},None));self.assertEqual(report['status'],'failed')
  if raw:self.assertEqual(json.loads(raw)['status'],'failed')
 def test_shared_helper_changed_during_restore_is_failed_only(self):
  def change(number):self.ns['bounded']=lambda path,limit:b'changed-helper'
  report,out,raw,events,closed=self.publication_fixture(restore_effect=change)
  self.assertIsNotNone(self.ns['effect_publish'](report,out,{1:'old'},None));self.assertEqual(json.loads(raw)['status'],'failed')


# Unchanged standalone PG18 class/import ASTs follow.
TARGET_SOURCE=None
import argparse

import ast

import builtins

import hashlib

from pathlib import Path

from types import SimpleNamespace

import unittest

class FakePath:
    writes = {}

    def __init__(self, value): self.value = str(value)
    def __truediv__(self, value): return FakePath(self.value + '/' + str(value))
    def __str__(self): return self.value
    def __eq__(self, other): return isinstance(other, FakePath) and self.value == other.value
    def is_absolute(self): return self.value.startswith('/')
    def resolve(self, strict=True): return self
    def exists(self): return self.value == '/private/tmp/effect-owned'
    def is_symlink(self): return False
    def lstat(self): return SimpleNamespace(st_dev=1, st_ino=100, st_mode=0o40700, st_uid=501, st_gid=20)
    def write_text(self, text): self.writes[self.value] = text

class FakePipe:
    def __init__(self, fd): self.fd = fd; self.closed = False; self.error = None; self.close_calls = 0
    def fileno(self): return self.fd
    def close(self):
        self.close_calls += 1
        if self.error is not None: raise self.error
        self.closed = True

class FakeChild:
    def __init__(self, world, terminal, payload, server=False):
        self.world = world; self.pid = 1000 + len(world.children); self.returncode = None
        self.terminal = terminal; self.server = server; self.stdout = None if server else FakePipe(self.pid + 100)
        if self.stdout is not None: world.blocks[self.stdout.fd] = [payload, b'']

class FakeSelector:
    def __init__(self, world): self.world = world; self.pipe = None; self.closed = False; self.fd = 4000 + len(world.children); self.close_calls = 0
    def fileno(self): return self.fd
    def register(self, pipe, _):
        self.pipe = pipe
        if self.world.register_error is not None: raise self.world.register_error
    def get_map(self): return {} if self.pipe is None else {1: self.pipe}
    def select(self, _): return [(SimpleNamespace(fileobj=self.pipe), 1)]
    def unregister(self, _): self.pipe = None
    def close(self):
        self.close_calls += 1
        if self.world.selector_close_error is not None: raise self.world.selector_close_error
        self.closed = True

class World:
    def __init__(self):
        self.now = 0.; self.work_end = 180.; self.cleanup_end = 180.; self.cancelled = False
        self.calls = []; self.children = []; self.blocks = {}; self.retries = []
        self.spawn_error = self.register_error = self.selector_error = self.selector_close_error = None
        self.after_spawn = None
        self.next_exit = 0; self.next_payload = b'private-output'; self.wrong_identity = False
        self.server_terminal_on_signal = 0
    def popen(self, argv, **kwargs):
        self.calls.append('Popen')
        if self.spawn_error is not None: raise self.spawn_error
        server = '-D' in argv
        terminal, payload = self.next_exit, self.next_payload
        if server: terminal, payload = None, b''
        elif '--version' in argv: terminal, payload = 0, b'postgres (PostgreSQL) 16.15\n'
        elif any('SHOW data_directory' in a for a in argv):
            terminal = self.retries.pop(0) if self.retries else 0
            payload = b'/wrong/data\n16.15\n1\n' if self.wrong_identity else b'/private/tmp/effect-owned/data\n16.15\n1\n'
        child = FakeChild(self, terminal, payload, server); self.children.append(child)
        if self.after_spawn is not None: self.after_spawn()
        return child
    def selector(self):
        self.calls.append('selector')
        if self.selector_error is not None: raise self.selector_error
        return FakeSelector(self)
    def fstat(self, fd): return SimpleNamespace(st_dev=1, st_ino=fd, st_mode=0o10600)
    def read(self, fd, bound):
        blocks = self.blocks[fd]; block = blocks.pop(0)
        if len(block) > bound: blocks.insert(0, block[bound:]); return block[:bound]
        return block
    def held(self, child):
        if child.returncode is not None: raise ValueError('lost fake original authority')
        return child.terminal
    def stop(self, child, grace):
        self.calls.append(('stop-original', child.pid, grace))
        child.returncode = child.terminal if child.terminal is not None else -15
        return child.returncode
    def kill(self, pid, number):
        self.calls.append(('SIGINT-original', pid, number))
        child = next(c for c in self.children if c.pid == pid)
        child.terminal = self.server_terminal_on_signal
    def sleep(self, value): self.now += value
    def environment(self): return {'PGPASSWORD': 'never-exported-fake', 'ZEE_OLD': 'ignored', 'PATH': '/fake'}
    def make(self):
        safe = {name: getattr(builtins, name) for name in ('__build_class__', 'staticmethod', 'Exception',
            'BaseException', 'ValueError', 'PermissionError', 'ProcessLookupError', 'bool', 'type', 'int',
            'float', 'str', 'bytes', 'bytearray', 'len', 'callable', 'min', 'max', 'all', 'any', 'dict', 'list', 'getattr',
            'isinstance', 'reversed')}
        fakeos = SimpleNamespace(geteuid=lambda: 501, fstat=self.fstat, set_blocking=lambda *_: None,
                                 read=self.read, kill=self.kill)
        # No real process, OS, socket, signal, FD or filesystem authority is supplied.
        namespace = {'__builtins__': safe, '__name__': 'effect_pg_pure_fake', 'Path': FakePath,
            'os': fakeos, 'selectors': SimpleNamespace(DefaultSelector=self.selector, EVENT_READ=1),
            'signal': SimpleNamespace(SIGINT=2), 'stat': SimpleNamespace(S_ISDIR=lambda mode: mode == 0o40700,
                S_IMODE=lambda mode: mode & 0o777),
            'subprocess': SimpleNamespace(Popen=self.popen, PIPE=-1, STDOUT=-2, DEVNULL=-3),
            'time': SimpleNamespace(monotonic=lambda: self.now, sleep=self.sleep)}
        tree = ast.parse(TARGET_SOURCE)
        chosen = [n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef)) and n.name in ('check', 'PlaintextPG')]
        assert len(chosen) == 2 and {n.name for n in chosen} == {'check', 'PlaintextPG'}
        exec(compile(ast.Module(body=chosen, type_ignores=[]), '<reviewed-effect-pg-selected>', 'exec'), namespace)
        helper = {'held_status': self.held, 'stop_owned': self.stop, 'controlled_env': self.environment,
                  'CANCELLED': False}
        instance = namespace['PlaintextPG'](helper, '/fake/pg/bin', '/private/tmp/effect-owned', 15432,
            lambda: self.work_end, lambda: self.cleanup_end, lambda: self.cancelled)
        return instance

class PlaintextPGContracts(unittest.TestCase):
    def setUp(self): self.w = World(); FakePath.writes = {}; self.pg = self.w.make()
    def test_constructor_resource_free(self): self.assertEqual(self.w.calls, []); self.assertEqual(FakePath.writes, {})
    def test_command_original_zero_receipt(self):
        self.assertEqual(self.pg.run_owned(['private'], 5, 8192), b'private-output')
        row = self.pg.public_state()['command_receipts'][0]
        self.assertEqual((row['observed_exit'], row['exit'], row['reaped'], row['fd_closed']), (0, 0, True, True))
    def test_readiness_two_then_zero_every_original_reaped(self):
        self.w.retries = [2, 0]; self.pg.start(); self.pg.close(); state = self.pg.public_state()
        self.assertEqual([state['command_receipts'][i]['exit'] for i in state['readiness_command_indexes']], [2, 0])
        self.assertTrue(state['probe_original_owner_receipts_complete']); self.assertTrue(self.pg.assert_root_removable())
    def test_readiness_non_two_rejected(self):
        self.w.retries = [3]
        with self.assertRaises(ValueError): self.pg.start()
        self.assertTrue(self.pg.commands[-1]['row']['reaped']); self.assertTrue(self.pg.failed)
    def test_wrong_show_identity_never_ready_or_removable(self):
        self.w.wrong_identity = True
        with self.assertRaises(ValueError): self.pg.start()
        self.assertFalse(self.pg.ready)
        with self.assertRaises(ValueError): self.pg.assert_root_removable()
    def test_nonzero_command_reaped_but_failed(self):
        self.w.next_exit = 4
        with self.assertRaises(ValueError): self.pg.run_owned(['private'], 5, 8192)
        self.assertTrue(self.pg.commands[0]['row']['reaped']); self.assertTrue(self.pg.failed)
    def test_spawn_failure_keeps_uncertainty_no_adoption(self):
        error = OSError('fake spawn'); self.w.spawn_error = error
        with self.assertRaises(OSError) as caught: self.pg.run_owned(['private'], 5, 8192)
        self.assertIs(caught.exception, error); self.assertTrue(self.pg.uncertain); self.assertIsNone(self.pg.commands[0]['child'])
    def test_selector_constructor_failure_attempts_original_pipe_and_child(self):
        self.w.selector_error = RuntimeError('fake selector')
        with self.assertRaises(RuntimeError): self.pg.run_owned(['private'], 5, 8192)
        retained = self.pg.commands[0]
        self.assertTrue(retained['row']['reaped']); self.assertTrue(retained['pipe_closed']); self.assertTrue(self.pg.uncertain)
    def test_selector_register_first_error_survives_close_error(self):
        primary = RuntimeError('fake register'); self.w.register_error = primary
        self.w.selector_close_error = OSError('fake selector close')
        with self.assertRaises(RuntimeError) as caught: self.pg.run_owned(['private'], 5, 8192)
        self.assertIs(caught.exception, primary); self.assertTrue(self.pg.commands[0]['row']['reaped'])
    def test_foreign_pipe_never_closed(self):
        self.pg.run_owned(['private'], 5, 8192); retained = self.pg.commands[0]
        retained['pipe_closed'] = False; retained['pipe'].closed = False; retained['pipe'].fd += 1
        count = retained['pipe'].close_calls
        with self.assertRaises(ValueError): self.pg._close_command(retained)
        self.assertEqual(retained['pipe'].close_calls, count); self.assertTrue(self.pg.uncertain)
    def test_foreign_selector_fd_never_closed(self):
        self.pg.run_owned(['private'], 5, 8192); retained = self.pg.commands[0]
        retained['selector_closed'] = False; retained['selector'].closed = False; retained['selector'].fd += 1
        count = retained['selector'].close_calls
        with self.assertRaises(ValueError): self.pg._close_command(retained)
        self.assertEqual(retained['selector'].close_calls, count); self.assertTrue(self.pg.uncertain)
    def test_pipe_close_error_retains_authority(self):
        self.pg.run_owned(['private'], 5, 8192); retained = self.pg.commands[0]
        retained['pipe_closed'] = False; retained['pipe'].closed = False; retained['pipe'].error = OSError('fake close')
        with self.assertRaises(OSError): self.pg._close_command(retained)
        self.assertIsNotNone(retained['pipe']); self.assertTrue(self.pg.uncertain)
    def test_cleanup_end_never_renews(self):
        self.w.cleanup_end = 1
        with self.assertRaises(ValueError): self.pg.run_owned(['private'], 5, 8192)
        self.assertEqual(self.w.cleanup_end, 1); self.assertTrue(self.pg.uncertain)
    def test_normal_sigint_zero_original_reaped_no_fallback(self):
        self.pg.start(); child = self.pg.child; self.pg.close()
        self.assertIs(self.pg.child, child); self.assertEqual(self.pg.observed_exit, 0)
        self.assertEqual(self.pg.exit, 0); self.assertFalse(self.pg.fallback); self.assertTrue(self.pg.closed)
    def test_nonzero_pg_after_sigint_fallback_not_success(self):
        self.pg.start(); self.w.server_terminal_on_signal = 7
        with self.assertRaises(ValueError): self.pg.close()
        self.assertTrue(self.pg.fallback); self.assertFalse(self.pg.closed)
        with self.assertRaises(ValueError): self.pg.assert_root_removable()
    def test_cancel_at_spawn_return_still_original_cleanup_no_global_reset(self):
        self.w.after_spawn = lambda: setattr(self.w, 'cancelled', True)
        with self.assertRaises(ValueError): self.pg.run_owned(['private'], 5, 8192)
        self.assertTrue(self.w.cancelled); self.assertTrue(self.pg.commands[0]['row']['reaped'])
        self.assertTrue(self.pg.commands[0]['pipe_closed'])
    def test_public_receipts_no_private_fields(self):
        self.pg.run_owned(['private-SQL-must-not-export'], 5, 8192)
        state = self.pg.public_state(); text = repr(state)
        for forbidden in ('private-SQL', 'never-exported-fake', 'private-output', '/private/', 'pid', 'argv', 'sha256'):
            self.assertNotIn(forbidden, text)
    def test_close_resource_free_before_start_is_safe(self):
        self.pg.close(); self.assertTrue(self.pg.assert_root_removable()); self.assertEqual(self.w.calls, [])


def captured_source(path,limit):
 if path.resolve(strict=True)!=path or path.is_symlink():raise ValueError('fixed canonical public source')
 before=path.stat();data=path.read_bytes();after=path.stat()
 identity=lambda s:(s.st_dev,s.st_ino,s.st_mode,s.st_uid,s.st_gid,s.st_nlink,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
 if identity(before)!=identity(after) or len(data)!=before.st_size or len(data)>limit:raise ValueError('stable bounded public source required')
 return data

def main():
 global TARGET_SHA,TARGET_SOURCE
 if STATE!='FINALIZED-EFFECT-PREFLIGHT-ROOT-AND-PEER-REVIEWED':raise ValueError('DRAFT preflight must not execute')
 native=captured_source(TARGET,131072)
 helper=Path(__file__).resolve().with_name('effect-pg-owner.py')
 pg=captured_source(helper,131072)
 if hashlib.sha256(pg).hexdigest()!='37e72f521260f5dc039840bd413efa405164d0a86ea5601e4c5abfd820e0611e':raise ValueError('exact standalone plaintext helper')
 TARGET_SHA=hashlib.sha256(native).hexdigest();TARGET_SOURCE=pg.decode()
 suite=unittest.TestSuite([unittest.defaultTestLoader.loadTestsFromTestCase(EffectLifecycleParts),unittest.defaultTestLoader.loadTestsFromTestCase(PlaintextPGContracts)])
 if suite.countTestCases()!=45:raise ValueError('exact 27 lifecycle plus18 PG methods')
 result=unittest.TextTestRunner(verbosity=2).run(suite)
 if captured_source(TARGET,131072)!=native or captured_source(helper,131072)!=pg:raise ValueError('source changed during selected models')
 return 0 if result.wasSuccessful() else 1

if __name__=='__main__':raise SystemExit(main())
