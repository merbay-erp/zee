#!/usr/bin/env python3
"""Actual fixed native Zee SMTP/SQL effects; fixture servers never send mail."""
import argparse, contextlib, base64, email, email.policy, hashlib, importlib.util, json, os, select, selectors, signal, socket, stat, struct, subprocess, sys, tempfile, threading, time
from pathlib import Path
from types import SimpleNamespace
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
class OperationProxy:
 def __init__(self,backend,mode,operation):
  self.backend=backend;self.target='effect_'+operation;self.fault=P.CommitProxy('127.0.0.1',backend,mode,application=self.target,deadline_seconds=25).start();self.stop=threading.Event();self.deadline=time.monotonic()+25;self.sockets=[];self.threads=[];self.connections=0;self.target_connections=0;self.error=None
  self.listener=socket.socket();self.listener.bind(('127.0.0.1',0));self.port=self.listener.getsockname()[1];self.listener.listen(8);self.listener.settimeout(.05);self.thread=threading.Thread(target=self.accept,daemon=True);self.thread.start()
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
  finally:P.kapat(*([client,backend] if backend is not None else [client]))
 def finish(self):
  value=self.fault.finish();need(self.error is None,'router failure');return {**value,'router_connections':self.connections,'target_operation_connections':self.target_connections}
 def close(self):
  self.stop.set();P.kapat(self.listener,*self.sockets);self.thread.join(timeout=2)
  for thread in self.threads:thread.join(timeout=2)
  self.fault.close();need(not self.thread.is_alive() and all(not t.is_alive() for t in self.threads),'owned router threads closed')
class SMTP:
 def __init__(self,mode='ok'):
  self.mode=mode;self.connections=0;self.deliveries=0;self.mime_ok=False;self.accepted=threading.Event();self.stop=threading.Event();self.error=None
  self.listener=socket.socket();self.listener.bind(('127.0.0.1',0));self.port=self.listener.getsockname()[1];self.listener.listen(4);self.listener.settimeout(.05)
  self.thread=threading.Thread(target=self.work,daemon=True);self.thread.start()
 def work(self):
  try:
   while not self.stop.is_set():
    try:conn,_=self.listener.accept()
    except socket.timeout:continue
    except OSError:break
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
  except BaseException as error:self.error=type(error).__name__
 def close(self):
  self.stop.set();self.listener.close();self.thread.join(timeout=2);need(not self.thread.is_alive(),'SMTP thread cleanup');need(self.error is None,'SMTP fixture failure')
class Fixture:
 def __init__(self,binary,pg,work,output):
  self.binary=binary;self.pg=pg;self.work=work;self.output=output;self.index=0;self.closed=False
  self.owner=O.Supervisor(SimpleNamespace(project=work,dil=binary,stop_grace=1),O.JsonLog())
  self.foreign_owner=O.Supervisor(SimpleNamespace(project=work,dil=binary,stop_grace=.1),O.JsonLog())
  self.foreign=None
  self.build=json.loads(H.run([binary,'sürüm','--json'],timeout=5,limit=8192));self.binaryhash=sha(binary,134217728)
 def start(self):
  self.foreign=self.foreign_owner.spawn([sys.executable,'-I','-B','-c','import time;time.sleep(120)'])
  self.data=self.work/'data';H.run([self.pg/'initdb','--no-locale','--encoding=UTF8','--auth=trust','--username=postgres','--pgdata',self.data],timeout=20,limit=131072)
  with socket.socket() as sock:sock.bind(('127.0.0.1',0));self.port=sock.getsockname()[1]
  (self.data/'postgresql.conf').write_text(f"listen_addresses='127.0.0.1'\nport={self.port}\nunix_socket_directories='{self.work}'\nmax_connections=20\nlog_min_messages=fatal\n")
  self.postgres=self.owner.spawn([self.pg/'postgres','-D',self.data]);deadline=time.monotonic()+10
  while True:
   try:self.sql('SELECT 1');break
   except H.Reject:need(time.monotonic()<deadline,'PG actual start');time.sleep(.05)
  need(H.run([self.pg/'postgres','--version'],timeout=2,limit=512).strip()==b'postgres (PostgreSQL) 16.15','actual PG16.15')
  self.sql(bounded(FIXTURE/'gocler/0001_effect.sql',131072).decode())
  self.sql("CREATE ROLE effect_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE; CREATE ROLE effect_observer LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE; ALTER ROLE effect_observer SET default_transaction_read_only=on; GRANT SELECT,INSERT,UPDATE ON effect_jobs,effect_latches TO effect_app; GRANT USAGE,SELECT ON SEQUENCE effect_jobs_id_seq TO effect_app; GRANT SELECT ON effect_jobs,effect_latches TO effect_observer;")
 def sql(self,text,observer=False):
  role='effect_observer' if observer else 'postgres'
  return H.run([self.pg/'psql','-X','-q','-t','-A','-v','ON_ERROR_STOP=1','-h','127.0.0.1','-p',str(self.port),'-U',role,'-d','postgres','-c',text],timeout=5,limit=8192).decode().strip()
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
  finally:self.owner.stop_child(child)
 def native(self,project,values,op,lease,owner='effect-a'):
  job,token,generation=lease
  return H.run([self.binary,'çalıştır',project,op,job,owner,token,generation],timeout=25,limit=256,env={**O.safe_environment(),'ZEE_EFFECT_DB_URL':values['db'],'ZEE_EFFECT_SMTP_URL':values['mail']}).decode().strip()
 def close(self):
  if self.closed:return
  failures=[]
  try:self.owner.cleanup();need(not self.owner.children,'owned all PG/host groups closed')
  except BaseException as error:failures.append(error)
  try:
   if self.foreign in self.foreign_owner.children:need(self.foreign_owner.exit_status(self.foreign) is None,'foreign sentinel survived')
  except BaseException as error:failures.append(error)
  finally:
   try:self.foreign_owner.cleanup()
   except BaseException as error:failures.append(error)
  if failures:raise failures[0]
  self.closed=True
def main():
 global END,CANCELLED
 END=time.monotonic()+180;CANCELLED=False
 def cancel(number,frame):
  global CANCELLED
  CANCELLED=True
  H.CANCELLED=True
 for number in (signal.SIGTERM,signal.SIGINT):signal.signal(number,cancel)
 class MaskedParser(argparse.ArgumentParser):
  def error(self,message):raise ValueError('arguments')
 p=MaskedParser(allow_abbrev=False);p.add_argument('--dil',type=Path,required=True);p.add_argument('--postgres-bin-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();need(not a.output.exists() and not a.output.is_symlink() and a.output.parent.resolve().stat().st_uid==os.geteuid() and os.geteuid()!=0,'fresh owned proof output')
 binaries={str(x.resolve()):sha(x,134217728) for x in [a.dil,*[a.postgres_bin_dir/n for n in ('initdb','postgres','psql')]]};sources={};source_total=0
 for name in ('compiler/tests/fixtures/native-effect-consumer/ana.dil', 'compiler/tests/fixtures/native-effect-consumer/catalog.pdf', 'compiler/tests/fixtures/native-effect-consumer/gocler/0001_effect.sql', 'compiler/tests/fixtures/native-effect-consumer/proje.dil', 'docs/native-effect-consumer.md', 'scripts/credential-baslat.py', 'scripts/effect-dispatcher.py', 'scripts/lifecycle_core.py', 'scripts/native-capabilities.py', 'scripts/native-metrics.py', 'scripts/postgresql-commit-proxy.py', 'scripts/tests/effect-dispatcher-native-test.py', 'scripts/tests/effect-dispatcher-test.py', 'scripts/worker-supervisor.py'):
  file=BASE/name;source_total+=file.lstat().st_size;need(source_total<=2097152,'fixed source inventory bytes');sources[name]=sha(file)
 a.output.mkdir(mode=0o700)
 report={'schema':'zee-native-effect-campaign-1','status':'failed','cases':[],'source_pins':sources,'executables':binaries,'profile':'owned-loopback-plaintext-only; no production/provider acceptance','exactly_once_claim':False};fixture=None
 def passed(name,server,obs,extra=None):
  need(fixture.foreign_owner.exit_status(fixture.foreign) is None,'foreign survived case');row={'name':name,'passed':True,'smtp_connections':server.connections,'SMTP_DATA_accepted':server.deliveries,'native_MIME_attachment':server.mime_ok,'PG_readonly_observer':obs};row.update(extra or {});report['cases'].append(row);print(name+' passed',flush=True)
 try:
  with contextlib.ExitStack() as stack:
   tmp=stack.enter_context(tempfile.TemporaryDirectory(prefix='zee-effect-campaign-',dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp'))
   fixture=Fixture(a.dil.resolve(strict=True),a.postgres_bin_dir.resolve(strict=True),Path(tmp).resolve(),a.output);stack.callback(fixture.close);fixture.start();report['build']=fixture.build
   for mode,name,expected in [('ok','native_mime_then_durable_SQL_ack','done'),('drop-data-ack','DATA_accepted_ACK_lost_manual_pause','effect_started'),('greet-reject','native_before_DATA_failure_still_no_auto_retry','effect_started')]:
    fixture.reset();server=SMTP(mode)
    try:
     child,state,_,_=fixture.launch(server);rc,result=fixture.collect(child);obs=fixture.observed();need(obs['state']==expected and obs['attempts']==1 and obs['latches']==1,'latch before external effect')
     need(rc==(0 if mode=='ok' else 3),'typed host outcome');need(server.connections==1 and server.deliveries==(0 if mode=='greet-reject' else 1),'actual SMTP counts')
     if mode!='ok':
      child,_,_,_=fixture.launch(server,state=state);rc2,_=fixture.collect(child);need(rc2==3 and server.connections==1,'same local pending guard no replay')
      child,_,_,_=fixture.launch(server);rc3,r3=fixture.collect(child);need(rc3==0 and r3['status']=='idle' and server.connections==1,'distinct local owner blocked by SQL latch')
     passed(name,server,obs,{'host_exit':rc,'native_invocations':result['native_send_commands']})
    finally:server.close()
   fixture.reset(False);server=SMTP()
   try:
    started=time.monotonic();child,state,_,_=fixture.launch(server);rc,result=fixture.collect(child);obs=fixture.observed();need(rc==0 and obs=={'state':'dead_letter','attempts':3,'latches':0,'acknowledged':0},'known pre-effect SQL three rejects');need(result['safe_preflight_rejections']==3 and result['native_send_commands']==0 and server.connections==0 and time.monotonic()-started>=3,'safe bounded backoff zero SMTP invocation');passed('three_known_pre_effect_SQL_rejects_only_backoff',server,obs)
   finally:server.close()
   for sig,name in [(signal.SIGKILL,'kill_after_committed_latch_no_delivery_no_replay'),(signal.SIGTERM,'TERM_during_native_SMTP_pauses_and_owned_cleanup')]:
    fixture.reset();server=SMTP('hold')
    try:
     child,state,_,_=fixture.launch(server);need(server.accepted.wait(5),'native SMTP reached owned greeting barrier');before=fixture.observed();need(before['state']=='effect_started' and before['latches']==1,'independent durable latch before SMTP bytes');os.kill(child.pid,sig);rc,result=fixture.collect(child);need(rc==(-9 if sig==signal.SIGKILL else 3),'actual signal outcome');need(server.deliveries==0,'zero DATA at controlled barrier')
     child,_,_,_=fixture.launch(server,state=state);rc2,_=fixture.collect(child);need(rc2==3 and server.connections==1,'signal leaves pending guard');passed(name,server,fixture.observed(),{'signal':int(sig),'host_exit':rc,'outer_held_group_cleanup':True})
    finally:server.close()
   fixture.reset();server=SMTP()
   try:
    project=fixture.project(fixture.port,server.port);values={'db':f'host=127.0.0.1 port={fixture.port} user=effect_app dbname=postgres sslmode=disable','mail':f'smtp://fixture:K277_SYNTHETIC_PASSWORD@127.0.0.1:{server.port}?from=sender@example.invalid'}
    claim=fixture.native(project,values,'claim',('0','old-token','0'));jid,g=claim.split(':');old=(jid,'old-token',g);fixture.sql("UPDATE effect_jobs SET expires=clock_timestamp()-interval '1 second'")
    fresh=fixture.native(project,values,'claim',('0','new-token','0'),owner='effect-b');jid2,g2=fresh.split(':');new=(jid2,'new-token',g2);need(int(g2)==int(g)+1,'fresh generation')
    need(fixture.native(project,values,'prepare',old)=='preflight' and fixture.native(project,values,'safe-fail',old)=='denied' and fixture.native(project,values,'send',old)=='denied' and fixture.native(project,values,'ack',old)=='denied' and server.connections==0,'old lease cannot start/send/fail/ack')
    need(fixture.native(project,values,'prepare',new,'effect-b')=='started' and fixture.native(project,values,'send',new,'effect-b')=='sent' and fixture.native(project,values,'ack',new,'effect-b')=='done','fresh owner actual native effect');need(server.deliveries==1,'one actual current owner delivery');passed('expired_owner_token_generation_cannot_send_or_ack',server,fixture.observed())
   finally:server.close()
   for operation,skips in [('prepare',1),('ack',3)]:
    for mode in ('before','after'):
     fixture.reset();server=SMTP();proxy=OperationProxy(fixture.port,mode,operation)
     try:
      child,state,_,_=fixture.launch(server,dbport=proxy.port,app='effect_fault');rc,result=fixture.collect(child);report.setdefault('fault_diagnostics',[]).append({'operation':operation,'mode':mode,'host_exit':rc,'host_result':result,'proxy_state_before_finish':dict(proxy.state)});wire=proxy.finish();obs=fixture.observed();need(rc==3 and (state/'pending.json').exists(),'unknown COMMIT manual pending pause');need(wire['commits_seen']==1 and wire['commit_forwarded']==(mode=='after'),'actual COMMIT wire fault')
      if operation=='prepare':need(server.connections==0 and obs['latches']==(1 if mode=='after' else 0),'unknown start never invokes native SMTP')
      else:need(server.deliveries==1 and obs['acknowledged']==(1 if mode=='after' else 0),'delivery distinct from unknown durable ack')
      child,_,_,_=fixture.launch(server,state=state);rc2,_=fixture.collect(child);need(rc2==3 and server.connections==(0 if operation=='prepare' else 1),'unknown outcome no automatic replay')
      passed(operation+'_COMMIT_'+mode+'_manual_no_replay',server,obs,{'wire':wire,'host_exit':rc})
     finally:
      try:proxy.close()
      finally:server.close()
   need(all(sha(Path(n),134217728)==v for n,v in binaries.items()) and all(sha(BASE/n)==v for n,v in sources.items()),'whole campaign source/binary end pins');fixture.close();fixture=None;report.update(status='passed',case_count=len(report['cases']),sources_unchanged=True,binaries_unchanged=True,owned_groups_closed=True,foreign_survived=True)
 except BaseException as error:report['failure_type']=type(error).__name__;raise
 finally:
  try:
   if fixture is not None:fixture.close()
  except BaseException as error:report.update(status='failed',cleanup_failure_type=type(error).__name__,owned_groups_closed=False)
  finally:(a.output/'proof.json').write_text(json.dumps(report,indent=2)+'\n')
if __name__=='__main__':
 try:main()
 except BaseException as error:
  print(json.dumps({'schema':'zee-native-effect-campaign-1','status':'failed','failure_type':type(error).__name__}),file=sys.stderr);raise SystemExit(2)
