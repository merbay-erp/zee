#!/usr/bin/env python3
"""Actual fixed native Zee SMTP/SQL effects; fixture servers never send mail."""
import argparse, shutil, contextlib, base64, email, email.policy, hashlib, importlib.util, json, os, select, selectors, signal, socket, stat, struct, subprocess, sys, tempfile, threading, time
from pathlib import Path
from types import SimpleNamespace
EFFECT_PARTIAL_STATE='FINALIZED-EFFECT-COMPLETE-LIFECYCLE-ROOT-AND-PEER-REVIEWED'
EFFECT_PARTIAL_COMPLETE=True
BASE=Path(__file__).resolve().parents[2]
FIXTURE=BASE/'compiler/tests/fixtures/native-effect-consumer'
END=time.monotonic()+180
CANCELLED=False
def need(value,reason):
 if CANCELLED or time.monotonic()>=END:raise AssertionError('campaign_cancelled_or_deadline')
 if not value:raise AssertionError(reason)
def bounded(path,maximum):
 path=Path(path);before=path.lstat();need(stat.S_ISREG(before.st_mode) and before.st_size<=maximum,'regular role bound');fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
 try:
  opened=os.fstat(fd);key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns);need(key(before)==key(opened),'role identity');data=bytearray()
  while True:
   need(True,'role deadline');block=os.read(fd,min(65536,maximum-len(data)+1))
   if not block:break
   data.extend(block);need(len(data)<=maximum,'role observed bound')
  need(len(data)==opened.st_size and key(opened)==key(os.fstat(fd))==key(path.lstat()),'role end identity');return bytes(data)
 finally:os.close(fd)
def load(name,path):
 module=type(sys)(name);module.__file__=str(path);exec(compile(bounded(path,131072),str(path),'exec'),module.__dict__);return module
H=load('effect_fixture_core',BASE/'scripts/lifecycle_core.py')
O=load('effect_fixture_owner',BASE/'scripts/worker-supervisor.py')
P=load('effect_commit_proxy',BASE/'scripts/postgresql-commit-proxy.py')
HOST=BASE/'scripts/effect-dispatcher.py'
def sha(path,maximum=131072):
 path=Path(path);before=path.lstat();need(stat.S_ISREG(before.st_mode) and before.st_size<=maximum,'hash role');fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK);h=hashlib.sha256();total=0
 try:
  opened=os.fstat(fd);key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns);need(key(before)==key(opened),'hash identity')
  while True:
   need(True,'hash deadline');block=os.read(fd,min(65536,maximum-total+1))
   if not block:break
   total+=len(block);need(total<=maximum,'hash actual bound');h.update(block)
  need(total==opened.st_size and key(opened)==key(os.fstat(fd))==key(path.lstat()),'hash end');return h.hexdigest()
 finally:os.close(fd)
# Partial test-only integration: terminal cleanup must be armed by a future main.
EFFECT_CLEANUP_END=None

def effect_begin_terminal_cleanup():
 global EFFECT_CLEANUP_END
 if EFFECT_CLEANUP_END is None:EFFECT_CLEANUP_END=time.monotonic()+90
 return EFFECT_CLEANUP_END

def effect_close_timeout():
 deadline=END if EFFECT_CLEANUP_END is None else EFFECT_CLEANUP_END
 return min(2,max(0,deadline-time.monotonic()))

def effect_cleanup_attempt(primary,action):
 try:action()
 except BaseException as error:
  if primary is None:primary=error
 return primary

def effect_close_socket(sock):
 primary=None
 try:sock.shutdown(socket.SHUT_RDWR)
 except OSError:pass
 except BaseException as error:primary=error
 try:sock.close()
 except BaseException as error:
  if primary is None:primary=error
 if primary is not None:raise primary

COMPONENTS=[]
EFFECT_CLEANUP_ERRORS=[]

def effect_component(kind,*args):
 component=kind(*args)
 COMPONENTS.append(component)
 component.start()
 return component

def effect_close_preserving(*components):
 primary=sys.exc_info()[1];body=primary
 for component in components:
  try:component.close()
  except BaseException as error:
   EFFECT_CLEANUP_ERRORS.append({'phase':'case-component-close','type':type(error).__name__})
   if primary is None:primary=error
 if primary is not None and primary is not body:raise primary

# Test-only terminal lifecycle. PG integration is a separate explicit blocker.
EFFECT_PG_INTEGRATION_COMPLETE=True
EFFECT_OLD_SIGNALS={}
EFFECT_CLEANING=False
EFFECT_OUTPUT_IDENTITY=None
EFFECT_EXPECTED_CASES=('native_mime_then_durable_SQL_ack','DATA_accepted_ACK_lost_manual_pause','native_before_DATA_failure_still_no_auto_retry','three_known_pre_effect_SQL_rejects_only_backoff','kill_after_committed_latch_no_delivery_no_replay','TERM_during_native_SMTP_pauses_and_owned_cleanup','expired_owner_token_generation_cannot_send_or_ack','prepare_COMMIT_before_manual_no_replay','prepare_COMMIT_after_manual_no_replay','ack_COMMIT_before_manual_no_replay','ack_COMMIT_after_manual_no_replay')

def effect_verify(value,reason):
 if not value:raise ValueError(reason)

def effect_cancel(*_):
 global CANCELLED
 CANCELLED=True
 H.CANCELLED=True

def effect_install_handlers(saved):
 for number in (signal.SIGTERM,signal.SIGINT,signal.SIGHUP):
  saved[number]=signal.getsignal(number)
  signal.signal(number,effect_cancel)

def effect_primary(primary,error,phase):
 EFFECT_CLEANUP_ERRORS.append({'phase':phase,'type':type(error).__name__})
 return error if primary is None else primary

def effect_close_commit(fault):
 primary=None
 done=getattr(fault,'done',None)
 if done is not None:primary=effect_cleanup_attempt(primary,done.set)
 def close_sockets():
  nonlocal primary
  for value in [getattr(fault,'listener',None),*list(getattr(fault,'sockets',[]))]:
   if value is not None:primary=effect_cleanup_attempt(primary,lambda value=value:effect_close_socket(value))
 close_sockets()
 main=getattr(fault,'thread',None)
 if main is not None and main is not threading.current_thread():primary=effect_cleanup_attempt(primary,lambda:main.join(timeout=effect_close_timeout()))
 close_sockets()
 for worker in list(getattr(fault,'threads',[])):
  if worker is not None and worker is not threading.current_thread():primary=effect_cleanup_attempt(primary,lambda worker=worker:worker.join(timeout=effect_close_timeout()))
 close_sockets()
 try:
  effect_verify(main is None or not main.is_alive(),'inner COMMIT main thread remains')
  effect_verify(all(worker is None or not worker.is_alive() for worker in getattr(fault,'threads',[])),'inner COMMIT route remains')
  effect_verify(effect_close_timeout()>0,'inner COMMIT cleanup deadline')
 except BaseException as error:
  if primary is None:primary=error
 if primary is not None:raise primary

def effect_finish_commit(fault):
 # Same wait/terminal conditions as P.CommitProxy.finish, using local all-attempt close.
 completion=fault.reply_ready if fault.mode=='observe' else fault.done
 while not completion.is_set() and not fault.done.is_set():
  remaining=min(fault.deadline,END)-time.monotonic()
  if remaining<=0:break
  completion.wait(min(.05,remaining))
 effect_close_commit(fault)
 effect_verify(fault.error is None,'COMMIT fixture failure')
 effect_verify(fault.state['commit_seen'],'target COMMIT not observed')
 if fault.mode!='before':effect_verify(fault.state['commit_ready'],'COMMIT idle boundary not observed')
 if fault.mode=='after':effect_verify(fault.state['commit_complete'] and fault.state['commit_error'] is None,'COMMIT completion not proved')
 return effect_public_wire(fault.state)

def effect_public_wire(value):
 booleans=('commit_seen','commit_forwarded','commit_ready','commit_complete','completion_delivered','error_delivered')
 keys=set(booleans)|{'mode','commit_error','commits_seen'}
 effect_verify(type(value) is dict and set(value)==keys,'closed COMMIT wire metadata')
 effect_verify(value['mode'] in ('before','after','observe') and all(type(value[n]) is bool for n in booleans),'typed COMMIT wire state')
 effect_verify(type(value['commits_seen']) is int and 0<=value['commits_seen']<=32,'bounded COMMIT count')
 error=value['commit_error']
 if error is not None:
  effect_verify(type(error) is dict and set(error)=={'severity','sqlstate'} and error['severity'] in ('ERROR','FATAL','PANIC') and type(error['sqlstate']) is str and len(error['sqlstate'])==5 and error['sqlstate'].isascii() and error['sqlstate'].isalnum(),'closed SQLSTATE only, no error text')
 return dict(value)

def effect_public_host(value):
 if value is None:return None
 effect_verify(type(value) is dict and value.get('schema')=='zee-native-effect-result-1','typed host result')
 effect_verify(value.get('status') in ('done','idle','dead_letter','paused_manual','rejected','known_completion_visible_durability_unconfirmed'),'closed host status')
 result={'schema':value['schema'],'status':value['status']}
 for name in ('native_send_commands','safe_preflight_rejections'):
  count=value.get(name,0);effect_verify(type(count) is int and 0<=count<=32,'bounded host counters');result[name]=count
 if 'direct_children_reaped' in value:
  effect_verify(type(value['direct_children_reaped']) is bool,'typed host child observation');result['direct_children_reaped']=value['direct_children_reaped']
 return result

EFFECT_PG_SOURCE_SHA='37e72f521260f5dc039840bd413efa405164d0a86ea5601e4c5abfd820e0611e'

def effect_pg_class():
 path=BASE/'scripts/tests/effect-pg-owner.py'
 captured=bounded(path,131072)
 effect_verify(hashlib.sha256(captured).hexdigest()==EFFECT_PG_SOURCE_SHA,'exact reviewed plaintext PG part')
 module=type(sys)('effect_plaintext_pg_part');module.__file__=str(path)
 exec(compile(captured,str(path),'exec'),module.__dict__)
 effect_verify(module.COMPLETE_FOR_INTEGRATION_REVIEW is True,'standalone adapter integration shape')
 return module.PlaintextPG

class EffectSupervisor(O.Supervisor):
 def cleanup(self):
  primary=None
  for child in list(self.children):
   try:self.stop_child(child)
   except BaseException as error:
    if primary is None:primary=error
  if primary is not None:raise primary
 def stop_child(self,child):
  end=END if EFFECT_CLEANUP_END is None else EFFECT_CLEANUP_END
  remaining=end-time.monotonic()
  effect_verify(remaining>2,'original group cleanup wait reserve')
  previous=self.a.stop_grace
  self.a.stop_grace=min(previous,remaining-2)
  try:return super().stop_child(child)
  finally:self.a.stop_grace=previous

def effect_stop_preserving(owner,child):
 primary=sys.exc_info()[1]
 try:owner.stop_child(child)
 except BaseException as error:
  effect_primary(primary,error,'collected-original-child-close')
  if primary is None:raise

class EffectWork:
 def __init__(self):self.path=None;self.created_path=None;self.creation_attempted=False;self.uncertain=False;self.identity=None;self.removed=False;self.preserved=False
 def start(self):
  effect_verify(self.path is None and self.created_path is None and not self.creation_attempted,'fresh private root')
  self.creation_attempted=True
  try:
   # Register the returned name before Path construction, resolve, stat or validation.
   self.created_path=tempfile.mkdtemp(prefix='zee-effect-campaign-',dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp')
   self.path=Path(self.created_path)
   effect_verify(self.path.resolve(strict=True)==self.path,'canonical allocated root')
   value=self.path.lstat()
   effect_verify(stat.S_ISDIR(value.st_mode) and value.st_uid==os.geteuid() and stat.S_IMODE(value.st_mode)==0o700,'private root0700')
   self.identity=(value.st_dev,value.st_ino,value.st_mode,value.st_uid,value.st_gid)
  except BaseException:
   self.uncertain=True;self.preserved=True
   raise
 def close(self,fixture):
  if self.path is None and not self.creation_attempted and self.created_path is None:self.removed=True;return
  try:
   effect_verify(self.path is not None and self.created_path is not None and self.creation_attempted and not self.uncertain and self.identity is not None,'allocated root authority uncertain')
   effect_verify(not EFFECT_CLEANUP_ERRORS and all(component.cleanup_closed for component in COMPONENTS),'all retained components closed')
   if fixture is not None:
    effect_verify(fixture.closed and not fixture.owner.children and not fixture.foreign_owner.children,'all original fixture groups closed')
    pg=getattr(fixture,'pg_owner',None)
    effect_verify(EFFECT_PG_INTEGRATION_COMPLETE is True and pg is not None,'reviewed plaintext PG/probe integration required')
    pg.assert_root_removable()
   value=self.path.lstat()
   effect_verify(self.path.resolve(strict=True)==self.path and (value.st_dev,value.st_ino,value.st_mode,value.st_uid,value.st_gid)==self.identity,'same private root inode')
   effect_verify(time.monotonic()<effect_begin_terminal_cleanup(),'private removal cleanup deadline')
   shutil.rmtree(self.path)
   effect_verify(not self.path.exists(),'private root remains')
   self.removed=True
  except BaseException:
   self.preserved=True
   raise

def effect_terminal_cleanup(primary,fixture,work,report):
 global EFFECT_CLEANING
 effect_begin_terminal_cleanup();EFFECT_CLEANING=True
 try:
  for component in reversed(COMPONENTS):
   try:component.close()
   except BaseException as error:primary=effect_primary(primary,error,'terminal-component-close')
  if fixture is not None:
   try:fixture.close()
   except BaseException as error:primary=effect_primary(primary,error,'terminal-fixture-close')
  try:work.close(fixture)
  except BaseException as error:primary=effect_primary(primary,error,'terminal-private-root-close')
 finally:
  EFFECT_CLEANING=False
  pg=getattr(fixture,'pg_owner',None) if fixture is not None else None
  if pg is not None:report['postgres_cluster']=pg.public_state()
  report.update(private_root_removed=work.removed,private_root_preserved_after_uncertain_owner=work.preserved,
   all_components_closed=all(component.cleanup_closed for component in COMPONENTS),
   owned_groups_closed=fixture is not None and fixture.closed and not fixture.owner.children and not fixture.foreign_owner.children,
   foreign_survived=fixture is not None and getattr(fixture,'foreign_survived',False),cleanup_errors=list(EFFECT_CLEANUP_ERRORS))
 return primary

def effect_validate_cases(rows):
 effect_verify(type(rows) is list and len(rows)<=len(EFFECT_EXPECTED_CASES),'bounded ordered public case prefix')
 base={'name','passed','smtp_connections','SMTP_DATA_accepted','native_MIME_attachment','PG_readonly_observer'}
 for index,row in enumerate(rows):
  extra={'host_exit','native_invocations'} if index<3 else {'signal','host_exit','outer_held_group_cleanup'} if index in (4,5) else {'wire','host_exit'} if index>=7 else set()
  effect_verify(type(row) is dict and set(row)==base|extra and row['name']==EFFECT_EXPECTED_CASES[index] and row['passed'] is True,'exact public case shape/order')
  for key in ('smtp_connections','SMTP_DATA_accepted'):
   effect_verify(type(row[key]) is int and 0<=row[key]<=32,'typed bounded SMTP counts')
  effect_verify(type(row['native_MIME_attachment']) is bool,'typed MIME observation')
  observed=row['PG_readonly_observer']
  effect_verify(type(observed) is dict and set(observed)=={'state','attempts','latches','acknowledged'} and observed['state'] in ('ready','leased','effect_started','done','retry_wait','dead_letter'),'closed public SQL state')
  effect_verify(all(type(observed[key]) is int and 0<=observed[key]<=32 for key in ('attempts','latches','acknowledged')),'typed bounded SQL counters')
  if 'native_invocations' in extra:effect_verify(type(row['native_invocations']) is int and 0<=row['native_invocations']<=32,'typed bounded native invocation count')
  if 'host_exit' in extra:
   expected=(0,3,3)[index] if index<3 else -9 if index==4 else 3
   effect_verify(type(row['host_exit']) is int and row['host_exit']==expected,'typed original host outcome')
  if 'signal' in extra:
   effect_verify(type(row['signal']) is int and row['signal']==(9 if index==4 else 15) and row['outer_held_group_cleanup'] is True,'typed exact signal observation')
  if 'wire' in extra:
   wire=row['wire'];wire_extra={'router_connections','target_operation_connections'}
   effect_verify(type(wire) is dict and set(wire)=={'mode','commit_error','commits_seen','commit_seen','commit_forwarded','commit_ready','commit_complete','completion_delivered','error_delivered'}|wire_extra,'closed public fault wire shape')
   safe=effect_public_wire({name:value for name,value in wire.items() if name not in wire_extra})
   for name in wire_extra:effect_verify(type(wire[name]) is int and 0<=wire[name]<=32,'typed bounded fault connection counts')
   effect_verify(wire['target_operation_connections']==1 and wire['router_connections']>=1,'one targeted fault connection')
   mode='before' if (index-7)%2==0 else 'after'
   effect_verify(safe['mode']==mode and safe['commits_seen']==1 and safe['commit_seen'] is True and safe['commit_forwarded'] is (mode=='after'),'same exact COMMIT wire observation')
   if mode=='after':effect_verify(safe['commit_complete'] is True and safe['commit_ready'] is True and safe['commit_error'] is None,'same after-COMMIT completion')

def effect_shared_helper_end():
 captured=bounded(BASE/'scripts/tests/effect-pg-owner.py',131072)
 effect_verify(hashlib.sha256(captured).hexdigest()==EFFECT_PG_SOURCE_SHA,'shared plaintext owner end hash')

def effect_publish(report,output,saved,primary):
 holder={'fd':None,'identity':None}
 def verdict():
  now=time.monotonic()
  report.update(cancelled=CANCELLED or H.CANCELLED,deadline_expired=now>=END,
   cleanup_deadline_expired=EFFECT_CLEANUP_END is not None and now>=EFFECT_CLEANUP_END,cleanup_errors=list(EFFECT_CLEANUP_ERRORS))
  good=(primary is None and report.get('body_completed') is True and report.get('sources_unchanged') is True and report.get('binaries_unchanged') is True
   and [row.get('name') for row in report['cases']]==list(EFFECT_EXPECTED_CASES) and all(row.get('passed') is True for row in report['cases'])
   and not report['cancelled'] and not report['deadline_expired'] and not report['cleanup_deadline_expired'] and not EFFECT_CLEANUP_ERRORS
   and report.get('postgres_cluster',{}).get('closed') is True and report['postgres_cluster'].get('normal_signal')=='SIGINT' and report['postgres_cluster'].get('exit')==report['postgres_cluster'].get('observed_exit')==0 and report['postgres_cluster'].get('reaped') is True and report['postgres_cluster'].get('fallback') is False and report['postgres_cluster'].get('probe_original_owner_receipts_complete') is True
   and report.get('private_root_removed') is True and report.get('all_components_closed') is True and report.get('owned_groups_closed') is True and report.get('foreign_survived') is True)
  report.update(status='passed' if good else 'failed',case_count=len(report['cases']))
 def write():
  if output is None:return
  value=output.lstat()
  effect_verify(output.is_absolute() and output.resolve(strict=True)==output and (value.st_dev,value.st_ino,value.st_mode,value.st_uid,value.st_gid)==EFFECT_OUTPUT_IDENTITY,'same owned public output inode')
  target=output/'proof.json'
  if holder['fd'] is None:
   holder['fd']=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_NONBLOCK,0o600)
   value=os.fstat(holder['fd'])
   effect_verify(stat.S_ISREG(value.st_mode) and value.st_uid==os.geteuid() and value.st_nlink==1,'original proof FD')
   holder['identity']=(value.st_dev,value.st_ino,value.st_mode,value.st_uid,value.st_gid,value.st_nlink)
  expected=holder['identity'];named=target.lstat();opened=os.fstat(holder['fd'])
  identity=lambda value:(value.st_dev,value.st_ino,value.st_mode,value.st_uid,value.st_gid,value.st_nlink)
  effect_verify(identity(named)==identity(opened)==expected,'same original proof inode')
  effect_verify(report.get('public_artifacts')==[],'no discovered or private raw artifacts')
  effect_validate_cases(report['cases'])
  payload=(json.dumps(report,indent=2)+'\n').encode();effect_verify(len(payload)<=1048576,'bounded closed public proof')
  os.lseek(holder['fd'],0,os.SEEK_SET);offset=0
  while offset<len(payload):
   count=os.write(holder['fd'],payload[offset:]);effect_verify(count>0,'original proof write progress');offset+=count
  os.ftruncate(holder['fd'],len(payload));os.fsync(holder['fd'])
  effect_verify(identity(target.lstat())==identity(os.fstat(holder['fd']))==expected,'original proof inode after write')
 try:
  if primary is None and report.get('body_completed') is True:effect_shared_helper_end()
  verdict();write();verdict()
  if report['status']!='passed':write()
 except BaseException as error:
  if primary is None:primary=error
  report.update(status='failed',publication_failure=type(error).__name__)
  try:write()
  except BaseException as secondary:report['failed_write_error']=type(secondary).__name__
 finally:
  for number,handler in saved.items():
   try:signal.signal(number,handler)
   except BaseException as error:primary=effect_primary(primary,error,'handler-restore')
 try:
  if primary is None and report.get('body_completed') is True:effect_shared_helper_end()
  verdict()
 except BaseException as error:
  if primary is None:primary=error
  report.update(status='failed',finalization_failure=type(error).__name__)
 # Only failed writes are allowed after restoring handlers; preserve the first exception.
 if primary is not None or report['status']!='passed':
  report['status']='failed'
  try:write()
  except BaseException as error:
   if primary is None:primary=error
  if primary is None:primary=ValueError('effect acceptance failed')
 if holder['fd'] is not None:
  try:os.close(holder['fd'])
  except BaseException as error:
   if primary is None:primary=error
   report.update(status='failed',proof_close_failure=type(error).__name__)
  finally:holder['fd']=None
 try:
  if primary is None and report['status']=='passed':effect_shared_helper_end()
  if CANCELLED or H.CANCELLED or time.monotonic()>=END or EFFECT_CLEANUP_END is not None and time.monotonic()>=EFFECT_CLEANUP_END:
   if primary is None:primary=ValueError('effect final cancel/deadline')
   report['status']='failed'
 except BaseException as error:
  if primary is None:primary=error
  report['status']='failed'
 return primary

class OperationProxy:
 def __init__(self,backend,mode,operation):
  self.backend=backend;self.target='effect_'+operation;self.stop=threading.Event();self.sockets=[];self.threads=[];self.connections=0;self.target_connections=0;self.error=None;self.listener=None;self.thread=None;self.cleanup_closed=False
  # The inner constructor is resource free; retain it BEFORE its explicit start.
  self.fault=P.CommitProxy('127.0.0.1',backend,mode,application=self.target,deadline_seconds=25)
 def start(self):
  self.deadline=time.monotonic()+25
  self.fault.start()
  self.listener=socket.socket()
  self.listener.bind(('127.0.0.1',0));self.port=self.listener.getsockname()[1];self.listener.listen(8);self.listener.settimeout(.05)
  self.thread=threading.Thread(target=self.accept,daemon=True);self.thread.start()
  return self
 @property
 def state(self):return self.fault.state
 def accept(self):
  try:
   while not self.stop.is_set() and time.monotonic()<self.deadline:
    try:client,_=self.listener.accept()
    except socket.timeout:continue
    except OSError:break
    self.connections+=1;need(self.connections<=32,'fixed operation proxy connection bound');self.sockets.append(client);thread=threading.Thread(target=self.route,args=(client,),daemon=True);self.threads.append(thread);thread.start()
  except BaseException as error:self.error=type(error).__name__
 def route(self,client):
  backend=None
  try:
   header=P.tam_oku(client,4,self.deadline);length=struct.unpack('!I',header)[0];need(8<=length<=65536,'owned startup bound');startup=P.tam_oku(client,length-4,self.deadline);need(startup[:4]==struct.pack('!I',196608) and startup.endswith(b'\0\0'),'fixed plaintext fixture startup');parts=startup[4:].split(b'\0')[:-2];need(len(parts)%2==0,'startup fields');settings=dict(zip(parts[::2],parts[1::2]));target=settings.get(b'application_name')==self.target.encode()
   if target:self.target_connections+=1;need(self.target_connections==1,'one exact fault operation connection')
   backend=socket.create_connection(('127.0.0.1',self.fault.port if target else self.backend),timeout=max(.001,self.deadline-time.monotonic()));self.sockets.append(backend);backend.sendall(header+startup);total=0
   while not self.stop.is_set():
    left=self.deadline-time.monotonic();need(left>0,'router deadline');ready,_,_=select.select([client,backend],[],[],min(.05,left))
    for source in ready:
     data=source.recv(65536)
     if not data:return
     total+=len(data);need(total<=2097152,'router fixed transcript bound');dest=backend if source is client else client;dest.settimeout(max(.001,self.deadline-time.monotonic()));dest.sendall(data)
  except (OSError,EOFError):pass
  except BaseException as error:self.error=type(error).__name__
  finally:
   for connection in [client,*([backend] if backend is not None else [])]:
    try:effect_close_socket(connection)
    except BaseException as error:
     self.error=type(error).__name__
     EFFECT_CLEANUP_ERRORS.append({'phase':'route-original-socket-close','type':type(error).__name__})
 def finish(self):
  value=effect_finish_commit(self.fault);need(self.error is None,'router failure');return {**value,'router_connections':self.connections,'target_operation_connections':self.target_connections}
 def close(self):
  if self.cleanup_closed:return
  primary=None
  stop=getattr(self,'stop',None)
  if stop is not None:primary=effect_cleanup_attempt(primary,stop.set)
  def sockets():
   nonlocal primary
   values=[getattr(self,'listener',None),*list(getattr(self,'sockets',[]))]
   for value in values:
    if value is not None:primary=effect_cleanup_attempt(primary,lambda value=value:effect_close_socket(value))
  sockets()
  thread=getattr(self,'thread',None)
  if thread is not None:primary=effect_cleanup_attempt(primary,lambda:thread.join(timeout=effect_close_timeout()))
  # The accept owner may have registered a route/backend during the first sweep.
  sockets()
  for route in list(getattr(self,'threads',[])):
   primary=effect_cleanup_attempt(primary,lambda route=route:route.join(timeout=effect_close_timeout()))
  fault=getattr(self,'fault',None)
  if fault is not None:primary=effect_cleanup_attempt(primary,lambda:effect_close_commit(fault))
  sockets()
  def checked():
   if self.thread is None:raise ValueError('router start incomplete')
   need(not self.thread.is_alive() and all(not t.is_alive() for t in self.threads),'owned router threads closed')
  primary=effect_cleanup_attempt(primary,checked)
  if effect_close_timeout()<=0 and primary is None:primary=TimeoutError('router cleanup absolute deadline')
  if primary is not None:raise primary
  self.cleanup_closed=True
class SMTP:
 def __init__(self,mode='ok'):
  self.mode=mode;self.connections=0;self.deliveries=0;self.mime_ok=False;self.accepted=threading.Event();self.stop=threading.Event();self.error=None;self.listener=None;self.thread=None;self.sockets=[];self.cleanup_closed=False
 def start(self):
  self.listener=socket.socket()
  self.listener.bind(('127.0.0.1',0));self.port=self.listener.getsockname()[1];self.listener.listen(4);self.listener.settimeout(.05)
  self.thread=threading.Thread(target=self.work,daemon=True);self.thread.start()
  return self
 def work(self):
  try:
   while not self.stop.is_set():
    try:conn,_=self.listener.accept()
    except socket.timeout:continue
    except OSError:break
    self.sockets.append(conn)
    self.connections+=1;self.accepted.set();deadline=time.monotonic()+10
    with conn:
     def send(value):conn.settimeout(max(.001,deadline-time.monotonic()));conn.sendall(value)
     buf=bytearray()
     def line():
      while b'\n' not in buf:
       need(time.monotonic()<deadline,'SMTP fixture deadline');conn.settimeout(min(.1,max(.001,deadline-time.monotonic())))
       try:block=conn.recv(4096)
       except socket.timeout:
        if self.stop.is_set():raise EOFError()
        continue
       if not block:raise EOFError()
       buf.extend(block);need(len(buf)<=131072,'SMTP input bound')
      at=buf.index(10)+1;value=bytes(buf[:at]);del buf[:at];return value
     try:
      if self.mode=='hold':
       while not self.stop.is_set() and time.monotonic()<deadline:time.sleep(.01)
       continue
      if self.mode=='greet-reject':send(b'550 synthetic refusal\r\n');continue
      send(b'220 owned.invalid\r\n')
      while True:
       command=line()
       if command.startswith(b'EHLO'):send(b'250 owned.invalid\r\n')
       elif command.startswith(b'AUTH PLAIN '):
        need(base64.b64decode(command.split()[2])==b'\0fixture\0K277_SYNTHETIC_PASSWORD','synthetic AUTH binding');send(b'235 ok\r\n')
       elif command.startswith(b'MAIL FROM:'):send(b'250 ok\r\n')
       elif command.startswith(b'RCPT TO:'):need(command==b'RCPT TO:<sink@example.invalid>\r\n','fixed recipient');send(b'250 ok\r\n')
       elif command==b'DATA\r\n':
        send(b'354 data\r\n');body=bytearray()
        while True:
         part=line()
         if part==b'.\r\n':break
         body.extend(part[1:] if part.startswith(b'..') else part);need(len(body)<=131072,'MIME bound')
        message=email.message_from_bytes(bytes(body),policy=email.policy.default)
        pdfs=[part.get_payload(decode=True) for part in message.walk() if part.get_filename()=='catalog.pdf']
        need(pdfs==[bounded(FIXTURE/'catalog.pdf',131072)],'native MIME attachment bytes');self.mime_ok=True;self.deliveries+=1
        if self.mode=='drop-data-ack':break
        send(b'250 accepted\r\n')
       elif command==b'QUIT\r\n':send(b'221 bye\r\n');break
       else:raise AssertionError('unexpected fixed native SMTP command')
     except (EOFError,ConnectionError):pass
  except BaseException as error:
   if not (self.stop.is_set() and isinstance(error,(OSError,EOFError))):self.error=type(error).__name__
 def close(self):
  if self.cleanup_closed:return
  primary=None
  stop=getattr(self,'stop',None)
  if stop is not None:primary=effect_cleanup_attempt(primary,stop.set)
  listener=getattr(self,'listener',None)
  if listener is not None:primary=effect_cleanup_attempt(primary,lambda:effect_close_socket(listener))
  for active in list(self.sockets):primary=effect_cleanup_attempt(primary,lambda active=active:effect_close_socket(active))
  thread=getattr(self,'thread',None)
  if thread is not None:primary=effect_cleanup_attempt(primary,lambda:thread.join(timeout=effect_close_timeout()))
  for active in list(self.sockets):primary=effect_cleanup_attempt(primary,lambda active=active:effect_close_socket(active))
  def checked():
   if self.thread is None:raise ValueError('SMTP start incomplete')
   need(not self.thread.is_alive(),'SMTP thread cleanup');need(self.error is None,'SMTP fixture failure')
  primary=effect_cleanup_attempt(primary,checked)
  if effect_close_timeout()<=0 and primary is None:primary=TimeoutError('SMTP cleanup absolute deadline')
  if primary is not None:raise primary
  self.cleanup_closed=True
class Fixture:
 def __init__(self,binary,pg,work,output):
  self.binary=binary;self.pg=pg;self.work=work;self.output=output;self.index=0;self.closed=False
  self.owner=EffectSupervisor(SimpleNamespace(project=work,dil=binary,stop_grace=1),O.JsonLog())
  self.foreign_owner=EffectSupervisor(SimpleNamespace(project=work,dil=binary,stop_grace=.1),O.JsonLog())
  self.foreign=None
  self.build=None;self.binaryhash=None;self.pg_owner=None;self.foreign_survived=False
 def start(self):
  with socket.socket() as sock:sock.bind(('127.0.0.1',0));self.port=sock.getsockname()[1]
  self.pg_owner=effect_pg_class()(H,self.pg,self.work,self.port,lambda:END,lambda:END if EFFECT_CLEANUP_END is None else EFFECT_CLEANUP_END,lambda:CANCELLED)
  self.build=json.loads(self.pg_owner.run_owned([self.binary,'sürüm','--json'],timeout=5,limit=8192));self.binaryhash=sha(self.binary,134217728)
  self.foreign=self.foreign_owner.spawn([sys.executable,'-I','-B','-c','import time;time.sleep(120)'])
  self.pg_owner.start();self.data=self.pg_owner.data;self.postgres=self.pg_owner.child
  deadline=min(END,time.monotonic()+10)
  while True:
   try:self.sql('SELECT 1');break
   except H.Reject:need(time.monotonic()<deadline,'PG actual start');time.sleep(.05)
  need(H.run([self.pg/'postgres','--version'],timeout=2,limit=512).strip()==b'postgres (PostgreSQL) 16.15','actual PG16.15')
  self.sql(bounded(FIXTURE/'gocler/0001_effect.sql',131072).decode())
  self.sql("CREATE ROLE effect_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE; CREATE ROLE effect_observer LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE; ALTER ROLE effect_observer SET default_transaction_read_only=on; GRANT SELECT,INSERT,UPDATE ON effect_jobs,effect_latches TO effect_app; GRANT USAGE,SELECT ON SEQUENCE effect_jobs_id_seq TO effect_app; GRANT SELECT ON effect_jobs,effect_latches TO effect_observer;")
 def sql(self,text,observer=False):
  role='effect_observer' if observer else 'postgres'
  return self.pg_owner.run_owned([self.pg/'psql','-X','-q','-t','-A','-v','ON_ERROR_STOP=1','-h','127.0.0.1','-p',str(self.port),'-U',role,'-d','postgres','-c',text],timeout=5,limit=8192,role='observer-sql' if observer else 'fixture-sql').decode().strip()
 def observed(self):
  text=self.sql("SELECT state||':'||attempts::text||':'||(SELECT count(*) FROM effect_latches)::text||':'||(SELECT count(*) FROM effect_latches WHERE status='done')::text FROM effect_jobs",True)
  need(self.sql('SHOW transaction_read_only',True)=='on','independent readonly observer');state,attempts,latches,done=text.split(':');return {'state':state,'attempts':int(attempts),'latches':int(latches),'acknowledged':int(done)}
 def reset(self,preflight=True):self.sql("TRUNCATE effect_latches,effect_jobs RESTART IDENTITY; INSERT INTO effect_jobs(business_key,revision,payload,preflight_ok) VALUES('synthetic-catalog',1,'fixed-catalog-v1',"+str(preflight).lower()+")")
 def project(self,dbport,mailport):
  self.index+=1;directory=self.work/('project'+str(self.index));directory.mkdir(mode=0o700)
  for name in ('ana.dil','catalog.pdf','gocler/0001_effect.sql','proje.dil'):
   source=FIXTURE/name;target=directory/name;data=bounded(source,131072)
   target.parent.mkdir(mode=0o700,parents=True,exist_ok=True)
   if name=='proje.dil':data=data.replace(b':18093',(':'+str(dbport)).encode()).replace(b':18094',(':'+str(mailport)).encode())
   target.write_bytes(data);target.chmod(0o600)
  return directory
 def launch(self,server,state=None,dbport=None,app=''):
  dbport=dbport or self.port;project=self.project(dbport,server.port);state=state or self.work/('state'+str(self.index))
  if not state.exists():state.mkdir(mode=0o700)
  credential=self.work/('credentials'+str(self.index));credential.mkdir(mode=0o700)
  values={'db':f'host=127.0.0.1 port={dbport} user=effect_app dbname=postgres sslmode=disable','mail':f'smtp://fixture:K277_SYNTHETIC_PASSWORD@127.0.0.1:{server.port}?from=sender@example.invalid'}
  for name,value in values.items():(credential/name).write_text(value);(credential/name).chmod(0o400)
  names=('ana.dil','proje.dil','gocler/0001_effect.sql','catalog.pdf');ph=hashlib.sha256('\n'.join(n+':'+sha(project/n) for n in names).encode()).hexdigest()
  argv=[sys.executable,'-I','-B',HOST,'--dil',self.binary,'--project',project,'--credential-root',credential,'--state',state,'--expected-source',self.build['source_sha'],'--binary-sha256',self.binaryhash,'--project-sha256',ph,'--owner','effect-a','--env','ZEE_EFFECT_DB_URL=db','--env','ZEE_EFFECT_SMTP_URL=mail','--candidate']
  child=self.owner.spawn([str(v) for v in argv],capture=True);return child,state,project,values
 def collect(self,child,timeout=35):
  deadline=time.monotonic()+timeout;data=bytearray();os.set_blocking(child.stdout.fileno(),False)
  try:
   while True:
    need(time.monotonic()<deadline,'owned host deadline')
    try:block=os.read(child.stdout.fileno(),4096)
    except BlockingIOError:block=None
    if block:data.extend(block);need(len(data)<=16384,'host capture budget')
    status=self.owner.exit_status(child)
    if status is not None and block==b'':break
    time.sleep(.01)
   need(b'K277_SYNTHETIC_PASSWORD' not in data and b'postgresql://' not in data and b'smtp://' not in data,'no secret URL output')
   return status,json.loads(data) if data else None
  finally:effect_stop_preserving(self.owner,child)
 def native(self,project,values,op,lease,owner='effect-a'):
  job,token,generation=lease
  return self.pg_owner.run_owned([self.binary,'çalıştır',project,op,job,owner,token,generation],timeout=25,limit=256,env={**O.safe_environment(),'ZEE_EFFECT_DB_URL':values['db'],'ZEE_EFFECT_SMTP_URL':values['mail']},role='native-operation').decode().strip()
 def close(self):
  if self.closed:return
  failures=[]
  try:self.owner.cleanup();need(not self.owner.children,'owned all PG/host groups closed')
  except BaseException as error:failures.append(error)
  try:
   if getattr(self,'pg_owner',None) is not None:self.pg_owner.close()
  except BaseException as error:failures.append(error)
  try:
   if self.foreign in self.foreign_owner.children:need(self.foreign_owner.exit_status(self.foreign) is None,'foreign sentinel survived');self.foreign_survived=True
  except BaseException as error:failures.append(error)
  finally:
   try:self.foreign_owner.cleanup()
   except BaseException as error:failures.append(error)
  if failures:raise failures[0]
  self.closed=True
def main():
 global END,CANCELLED,EFFECT_CLEANUP_END,COMPONENTS,EFFECT_CLEANUP_ERRORS,EFFECT_OUTPUT_IDENTITY
 if EFFECT_PARTIAL_STATE!='FINALIZED-EFFECT-COMPLETE-LIFECYCLE-ROOT-AND-PEER-REVIEWED' or EFFECT_PARTIAL_COMPLETE is not True or EFFECT_PG_INTEGRATION_COMPLETE is not True:raise ValueError('DRAFT/incomplete effect integration must not execute')
 END=time.monotonic()+180;CANCELLED=False;EFFECT_CLEANUP_END=None;COMPONENTS=[];EFFECT_CLEANUP_ERRORS=[]
 saved={};output=None;fixture=None;work=EffectWork();primary=None
 report={'schema':'zee-native-effect-campaign-1','status':'failed','cases':[],'profile':'owned-loopback-plaintext-only; no production/provider acceptance','exactly_once_claim':False,'body_completed':False,'sources_unchanged':False,'binaries_unchanged':False,'public_artifacts':[]}
 def passed(name,server,obs,extra=None):
  need(fixture.foreign_owner.exit_status(fixture.foreign) is None,'foreign survived case');row={'name':name,'passed':True,'smtp_connections':server.connections,'SMTP_DATA_accepted':server.deliveries,'native_MIME_attachment':server.mime_ok,'PG_readonly_observer':obs};row.update(extra or {});report['cases'].append(row);print(name+' passed',flush=True)
 try:
  effect_install_handlers(saved)
  class MaskedParser(argparse.ArgumentParser):
   def error(self,message):raise ValueError('arguments')
  p=MaskedParser(allow_abbrev=False);p.add_argument('--dil',type=Path,required=True);p.add_argument('--postgres-bin-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();need(not a.output.exists() and not a.output.is_symlink() and a.output.parent.resolve().stat().st_uid==os.geteuid() and os.geteuid()!=0,'fresh owned proof output')
  binaries={str(x.resolve()):sha(x,134217728) for x in [a.dil,*[a.postgres_bin_dir/n for n in ('initdb','postgres','psql')]]};sources={};source_total=0
  for name in ('compiler/tests/fixtures/native-effect-consumer/ana.dil', 'compiler/tests/fixtures/native-effect-consumer/catalog.pdf', 'compiler/tests/fixtures/native-effect-consumer/gocler/0001_effect.sql', 'compiler/tests/fixtures/native-effect-consumer/proje.dil', 'docs/native-effect-consumer.md', 'scripts/credential-baslat.py', 'scripts/effect-dispatcher.py', 'scripts/lifecycle_core.py', 'scripts/native-capabilities.py', 'scripts/native-metrics.py', 'scripts/postgresql-commit-proxy.py', 'scripts/tests/effect-dispatcher-native-test.py', 'scripts/tests/effect-dispatcher-test.py', 'scripts/worker-supervisor.py'):
   file=BASE/name;source_total+=file.lstat().st_size;need(source_total<=2097152,'fixed source inventory bytes');sources[name]=sha(file)
  effect_verify(a.output.is_absolute() and a.output.parent.resolve(strict=True)==a.output.parent,'canonical owned public output parent')
  a.output.mkdir(mode=0o700);output=a.output
  value=output.lstat();EFFECT_OUTPUT_IDENTITY=(value.st_dev,value.st_ino,value.st_mode,value.st_uid,value.st_gid)
  report.update(source_pins=sources,executables=binaries,shared_helpers={'scripts/tests/effect-pg-owner.py':EFFECT_PG_SOURCE_SHA})
  work.start()
  fixture=Fixture(a.dil.resolve(strict=True),a.postgres_bin_dir.resolve(strict=True),work.path,a.output)
  fixture.start();report['build']=fixture.build
  for mode,name,expected in [('ok','native_mime_then_durable_SQL_ack','done'),('drop-data-ack','DATA_accepted_ACK_lost_manual_pause','effect_started'),('greet-reject','native_before_DATA_failure_still_no_auto_retry','effect_started')]:
   fixture.reset();server=effect_component(SMTP,mode)
   try:
    child,state,_,_=fixture.launch(server);rc,result=fixture.collect(child);obs=fixture.observed();need(obs['state']==expected and obs['attempts']==1 and obs['latches']==1,'latch before external effect')
    need(rc==(0 if mode=='ok' else 3),'typed host outcome');need(server.connections==1 and server.deliveries==(0 if mode=='greet-reject' else 1),'actual SMTP counts')
    if mode!='ok':
     child,_,_,_=fixture.launch(server,state=state);rc2,_=fixture.collect(child);need(rc2==3 and server.connections==1,'same local pending guard no replay')
     child,_,_,_=fixture.launch(server);rc3,r3=fixture.collect(child);need(rc3==0 and r3['status']=='idle' and server.connections==1,'distinct local owner blocked by SQL latch')
    passed(name,server,obs,{'host_exit':rc,'native_invocations':result['native_send_commands']})
   finally:effect_close_preserving(server)
  fixture.reset(False);server=effect_component(SMTP)
  try:
   started=time.monotonic();child,state,_,_=fixture.launch(server);rc,result=fixture.collect(child);obs=fixture.observed();need(rc==0 and obs=={'state':'dead_letter','attempts':3,'latches':0,'acknowledged':0},'known pre-effect SQL three rejects');need(result['safe_preflight_rejections']==3 and result['native_send_commands']==0 and server.connections==0 and time.monotonic()-started>=3,'safe bounded backoff zero SMTP invocation');passed('three_known_pre_effect_SQL_rejects_only_backoff',server,obs)
  finally:effect_close_preserving(server)
  for sig,name in [(signal.SIGKILL,'kill_after_committed_latch_no_delivery_no_replay'),(signal.SIGTERM,'TERM_during_native_SMTP_pauses_and_owned_cleanup')]:
   fixture.reset();server=effect_component(SMTP,'hold')
   try:
    child,state,_,_=fixture.launch(server);need(server.accepted.wait(5),'native SMTP reached owned greeting barrier');before=fixture.observed();need(before['state']=='effect_started' and before['latches']==1,'independent durable latch before SMTP bytes');os.kill(child.pid,sig);rc,result=fixture.collect(child);need(rc==(-9 if sig==signal.SIGKILL else 3),'actual signal outcome');need(server.deliveries==0,'zero DATA at controlled barrier')
    child,_,_,_=fixture.launch(server,state=state);rc2,_=fixture.collect(child);need(rc2==3 and server.connections==1,'signal leaves pending guard');passed(name,server,fixture.observed(),{'signal':int(sig),'host_exit':rc,'outer_held_group_cleanup':True})
   finally:effect_close_preserving(server)
  fixture.reset();server=effect_component(SMTP)
  try:
   project=fixture.project(fixture.port,server.port);values={'db':f'host=127.0.0.1 port={fixture.port} user=effect_app dbname=postgres sslmode=disable','mail':f'smtp://fixture:K277_SYNTHETIC_PASSWORD@127.0.0.1:{server.port}?from=sender@example.invalid'}
   claim=fixture.native(project,values,'claim',('0','old-token','0'));jid,g=claim.split(':');old=(jid,'old-token',g);fixture.sql("UPDATE effect_jobs SET expires=clock_timestamp()-interval '1 second'")
   fresh=fixture.native(project,values,'claim',('0','new-token','0'),owner='effect-b');jid2,g2=fresh.split(':');new=(jid2,'new-token',g2);need(int(g2)==int(g)+1,'fresh generation')
   need(fixture.native(project,values,'prepare',old)=='preflight' and fixture.native(project,values,'safe-fail',old)=='denied' and fixture.native(project,values,'send',old)=='denied' and fixture.native(project,values,'ack',old)=='denied' and server.connections==0,'old lease cannot start/send/fail/ack')
   need(fixture.native(project,values,'prepare',new,'effect-b')=='started' and fixture.native(project,values,'send',new,'effect-b')=='sent' and fixture.native(project,values,'ack',new,'effect-b')=='done','fresh owner actual native effect');need(server.deliveries==1,'one actual current owner delivery');passed('expired_owner_token_generation_cannot_send_or_ack',server,fixture.observed())
  finally:effect_close_preserving(server)
  for operation,skips in [('prepare',1),('ack',3)]:
   for mode in ('before','after'):
    fixture.reset();server=effect_component(SMTP);proxy=effect_component(OperationProxy,fixture.port,mode,operation)
    try:
     child,state,_,_=fixture.launch(server,dbport=proxy.port,app='effect_fault');rc,result=fixture.collect(child);report.setdefault('fault_diagnostics',[]).append({'operation':operation,'mode':mode,'host_exit':rc,'host_result':effect_public_host(result),'proxy_state_before_finish':effect_public_wire(dict(proxy.state))});wire=proxy.finish();obs=fixture.observed();need(rc==3 and (state/'pending.json').exists(),'unknown COMMIT manual pending pause');need(wire['commits_seen']==1 and wire['commit_forwarded']==(mode=='after'),'actual COMMIT wire fault')
     if operation=='prepare':need(server.connections==0 and obs['latches']==(1 if mode=='after' else 0),'unknown start never invokes native SMTP')
     else:need(server.deliveries==1 and obs['acknowledged']==(1 if mode=='after' else 0),'delivery distinct from unknown durable ack')
     child,_,_,_=fixture.launch(server,state=state);rc2,_=fixture.collect(child);need(rc2==3 and server.connections==(0 if operation=='prepare' else 1),'unknown outcome no automatic replay')
     passed(operation+'_COMMIT_'+mode+'_manual_no_replay',server,obs,{'wire':wire,'host_exit':rc})
    finally:effect_close_preserving(proxy,server)
  report['body_completed']=True
 except BaseException as error:
  primary=error;report.update(status='failed',failure_type=type(error).__name__)
 finally:
  try:primary=effect_terminal_cleanup(primary,fixture,work,report)
  except BaseException as error:primary=effect_primary(primary,error,'terminal-driver')
  if report['body_completed']:
   try:
    need(all(sha(Path(n),134217728)==v for n,v in binaries.items()) and all(sha(BASE/n)==v for n,v in sources.items()),'whole campaign source/binary end pins')
    report.update(sources_unchanged=True,binaries_unchanged=True)
   except BaseException as error:primary=effect_primary(primary,error,'post-cleanup-source-binary-pins')
  primary=effect_publish(report,output,saved,primary)
 if primary is not None:raise primary
if __name__=='__main__':
 try:main()
 except BaseException as error:
  print(json.dumps({'schema':'zee-native-effect-campaign-1','status':'failed','failure_type':type(error).__name__}),file=sys.stderr);raise SystemExit(2)
