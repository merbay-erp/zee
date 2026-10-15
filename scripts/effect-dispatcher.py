#!/usr/bin/env python3
"""Finite fixed native SMTP effect pilot; uncertain outcomes require manual review."""
import argparse, hashlib, importlib.util, json, os, re, secrets, selectors, signal, stat, subprocess, sys, tempfile, time
from pathlib import Path
from types import SimpleNamespace
try:
 import fcntl
except ImportError:
 fcntl=None
STOP=False
END=float('inf')
class Paused(Exception):pass
class Parser(argparse.ArgumentParser):
 def error(self,message):raise Paused('arguments')
def need(ok):
 if STOP or time.monotonic()>=END:raise Paused('interrupted')
 if not ok:raise Paused('fixed_profile')
def file_bytes(path,maximum):
 path=Path(path);before=path.lstat();need(stat.S_ISREG(before.st_mode) and before.st_uid==os.geteuid() and before.st_nlink==1 and before.st_size<=maximum and not before.st_mode&0o022)
 fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
 try:
  opened=os.fstat(fd);key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns);need(key(before)==key(opened));data=bytearray()
  while True:
   need(True);block=os.read(fd,min(65536,maximum+1-len(data)))
   if not block:break
   data.extend(block);need(len(data)<=maximum)
  need(len(data)==opened.st_size and key(opened)==key(os.fstat(fd))==key(path.lstat()));return bytes(data)
 finally:os.close(fd)
def digest(path,maximum=134217728):
 path=Path(path);before=path.lstat();need(stat.S_ISREG(before.st_mode) and before.st_size<=maximum)
 fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK);h=hashlib.sha256();total=0
 try:
  opened=os.fstat(fd);key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns);need(key(before)==key(opened))
  while True:
   need(True);block=os.read(fd,min(65536,maximum-total+1))
   if not block:break
   total+=len(block);need(total<=maximum);h.update(block)
  need(total==opened.st_size and key(opened)==key(os.fstat(fd))==key(path.lstat()));return h.hexdigest()
 finally:os.close(fd)
def syncdir(path):
 fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
 try:os.fsync(fd)
 finally:os.close(fd)
def write_new(path,value):
 data=(json.dumps(value,sort_keys=True)+'\n').encode();need(len(data)<=16384)
 fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
 try:
  view=memoryview(data)
  while view:n=os.write(fd,view);need(n>0);view=view[n:]
  os.fsync(fd)
 finally:os.close(fd)
 syncdir(path.parent)
def load_module(name,path):
 data=file_bytes(path,131072);module=type(sys)(name);module.__file__=str(path);exec(compile(data,str(path),'exec'),module.__dict__);return module
def run_native(argv,timeout,limit,env):
 need(os.getpid()==os.getpgrp())
 child=subprocess.Popen([str(v) for v in argv],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,close_fds=True,start_new_session=False,env=env)
 selector=None;data=bytearray();deadline=min(END,time.monotonic()+timeout)
 def terminal():return os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
 try:
  selector=selectors.DefaultSelector();selector.register(child.stdout,selectors.EVENT_READ)
  while selector.get_map() or terminal() is None:
   need(time.monotonic()<deadline)
   for key,_ in selector.select(min(.05,max(0,deadline-time.monotonic()))):
    block=os.read(key.fd,min(4096,limit-len(data)+1))
    if not block:selector.unregister(key.fileobj)
    else:data.extend(block);need(len(data)<=limit)
  status=terminal();need(status.si_code==os.CLD_EXITED and status.si_status==0)
  return bytes(data)
 finally:
  try:
   if selector is not None:selector.close()
  finally:
   try:
    if terminal() is None:
     child.terminate()
     try:child.wait(timeout=3)
     except subprocess.TimeoutExpired:child.kill();child.wait(timeout=2)
    else:child.wait(timeout=2)
   finally:child.stdout.close()

def bounded_state_directory(state):
 with os.scandir(state) as entries:
  for count,_ in enumerate(entries,1):need(count<=16)

def save_result(state,result):
 target=state/('result-'+secrets.token_hex(8)+'.json');data=(json.dumps(result,sort_keys=True)+'\n').encode();descriptor=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
 try:
  view=memoryview(data)
  while view:
   count=os.write(descriptor,view)
   if count<=0:raise OSError('write')
   view=view[count:]
  os.fsync(descriptor)
 finally:os.close(descriptor)
 syncdir(state)

def main():
 global END,STOP
 END=time.monotonic()+90;STOP=False
 def cancel(number,frame):
  global STOP
  STOP=True
 for number in (signal.SIGTERM,signal.SIGINT):signal.signal(number,cancel)
 p=Parser(allow_abbrev=False)
 for name in ('dil','project','credential-root','state'):p.add_argument('--'+name,type=Path,required=True)
 for name in ('expected-source','binary-sha256','project-sha256','owner'):p.add_argument('--'+name,required=True)
 p.add_argument('--env',action='append',required=True);p.add_argument('--candidate',action='store_true')
 a=p.parse_args();need(fcntl is not None and os.name=='posix' and os.geteuid()!=0 and hasattr(os,'WNOWAIT') and os.getpid()==os.getpgrp())
 need(re.fullmatch('[0-9a-f]{40}',a.expected_source) and re.fullmatch('[0-9a-f]{64}',a.binary_sha256) and re.fullmatch('[A-Za-z0-9_-]{1,48}',a.owner))
 binary=a.dil.resolve(strict=True);need(digest(binary)==a.binary_sha256)
 base=Path(__file__).resolve().parent
 dependencies=['effect-dispatcher.py','credential-baslat.py','native-capabilities.py']
 sourcepins={name:digest(base/name,131072) for name in dependencies}
 credential=load_module('effect_credential',base/'credential-baslat.py')
 names=('ana.dil','proje.dil','gocler/0001_effect.sql','catalog.pdf');files={n:file_bytes(a.project/n,131072) for n in names}
 projecthash=hashlib.sha256('\n'.join(n+':'+hashlib.sha256(files[n]).hexdigest() for n in names).encode()).hexdigest();need(projecthash==a.project_sha256)
 # Only the existing reviewed fixed source/template is admitted, never a job-selected handler.
 for n,expected in FIXED.items():need(hashlib.sha256(files[n]).hexdigest()==expected)
 manifest=files['proje.dil'].decode();need(re.fullmatch(MANIFEST_PATTERN,manifest))
 dbport,mailport=re.search('postgresql://127.0.0.1:([0-9]+)/postgres',manifest)[1],re.search('smtp://127.0.0.1:([0-9]+)',manifest)[1]
 need(all(1<=int(v)<=65535 for v in (dbport,mailport)))
 ca=['--credential-root',str(a.credential_root),'--dil',str(binary)]
 for binding in a.env:ca+=['--env',binding]
 ca+=['--','çalıştır',str(a.project)]
 root,bindings,_=credential.parse_arguments(ca);need({n for n,_ in bindings}=={'ZEE_EFFECT_DB_URL','ZEE_EFFECT_SMTP_URL'})
 mapped=credential.child_environment(root,bindings)
 # Explicit local fixture-only target. URLs never come from queued payload.
 need(re.fullmatch(r'host=127\.0\.0\.1 port='+dbport+r' user=effect_app dbname=postgres sslmode=disable',mapped['ZEE_EFFECT_DB_URL']))
 need(mapped['ZEE_EFFECT_SMTP_URL']==f'smtp://fixture:K277_SYNTHETIC_PASSWORD@127.0.0.1:{mailport}?from=sender@example.invalid')
 env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','LC_ALL':'C.UTF-8','TZ':'UTC',**{n:mapped[n] for n,_ in bindings}};mapped.clear()
 state=a.state;need(state.resolve()==state and state.is_dir() and state.stat().st_uid==os.geteuid() and state.stat().st_mode&0o077==0)
 bounded_state_directory(state)
 lock=state/'lock';fd=os.open(lock,os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW,0o600);st=os.fstat(fd);need(stat.S_ISREG(st.st_mode) and st.st_nlink==1 and st.st_uid==os.geteuid() and st.st_mode&0o077==0)
 result={'schema':'zee-native-effect-result-1','status':'paused_manual','binary_sha256':a.binary_sha256,'sources':sourcepins,'project_sha256':projecthash,'native_send_commands':0,'safe_preflight_rejections':0,'direct_children_reaped':False}
 visible_completion=False;known_outcome=None;guard_established=False;pending_blocked=False
 try:
  fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB);pending_blocked=(state/'pending.json').exists() or (state/'pending.json').is_symlink();need(not pending_blocked)
  build=json.loads(run_native([binary,'sürüm','--json'],timeout=5,limit=8192,env=env));need(build.get('schema')=='zee-build-v1' and build.get('source_sha')==a.expected_source and (a.candidate or build.get('dirty') is False));result['build']=build
  write_new(state/'pending.json',{'schema':'zee-native-effect-pending-1','binary_sha256':a.binary_sha256,'project_sha256':projecthash,'source':a.expected_source,'operator_reconcile_required':True});guard_established=True
  with tempfile.TemporaryDirectory(prefix='zee-effect-',dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp') as tmp:
   project=Path(tmp).resolve()/'project';project.mkdir(mode=0o700)
   for n,v in files.items():target=project/n;target.parent.mkdir(mode=0o700,exist_ok=True);target.write_bytes(v);target.chmod(0o400)
   def query(op,lease=('0','unused','0')):
    need(op in ('claim','prepare','safe-fail','send','ack') and digest(binary)==a.binary_sha256)
    job,token,generation=lease;need(re.fullmatch('[0-9]{1,20}',job) and re.fullmatch('[A-Za-z0-9_-]{1,64}',token) and re.fullmatch('[0-9]{1,20}',generation))
    result['stage']=op
    result['direct_children_reaped']=False
    operation_env={**env,'ZEE_EFFECT_DB_URL':env['ZEE_EFFECT_DB_URL']+' application_name=effect_'+op.replace('-','')}
    raw=run_native([binary,'çalıştır',project,op,job,a.owner,token,generation],timeout=25 if op=='send' else 5,limit=256,env=operation_env)
    result['direct_children_reaped']=True
    need(digest(binary)==a.binary_sha256 and re.fullmatch(rb'(?:[0-9]{1,20}(?::[0-9]{1,20})?|bos|preflight|started|done|denied|sent|failed)\n',raw));return raw.decode().strip()
   for index in range(3):
    token='effect-'+secrets.token_hex(16);claim=query('claim',('0',token,'0'))
    if claim=='bos':result['status']='idle';break
    need(re.fullmatch('[0-9]+:[0-9]+',claim));job,generation=claim.split(':');lease=(job,token,generation)
    prepared=query('prepare',lease)
    if prepared=='preflight':
     attempt=query('safe-fail',lease);need(attempt in ('1','2','3'));result['safe_preflight_rejections']+=1
     if attempt=='3':result['status']='dead_letter';break
     deadline=time.monotonic()+2**(int(attempt)-1)
     while time.monotonic()<deadline:need(True);time.sleep(min(.02,max(0,deadline-time.monotonic())))
     continue
    need(prepared=='started')
    result['native_send_commands']+=1
    need(query('send',lease)=='sent')
    need(query('ack',lease)=='done');result['status']='done';break
   need(result['status'] in ('done','idle','dead_letter'))
   need(all(digest(base/n,131072)==v for n,v in sourcepins.items()) and digest(binary)==a.binary_sha256)
   need(json.loads(run_native([binary,'sürüm','--json'],timeout=5,limit=8192,env=env))==build)
   # Known completed outcome only: uncertainty/crash never clears this local admission guard.
   known_outcome=result['status']
   (state/'pending.json').unlink();visible_completion=True
   syncdir(state)
  result['direct_children_reaped']=True
 except BaseException as error:
  result.update(status='known_completion_visible_durability_unconfirmed' if visible_completion else ('paused_manual' if guard_established or pending_blocked or (state/'pending.json').exists() else 'rejected'),failure_type=type(error).__name__)
  if visible_completion:result.update(completed_status=known_outcome,admission_guard_visible=False,durability_review_required=True)
 finally:
  env.clear();os.close(fd)
  try:save_result(state,result)
  except OSError as error:
   result.update(result_publication_unconfirmed=True,publication_failure_type=type(error).__name__)
   if visible_completion:result.update(status='known_completion_visible_durability_unconfirmed',completed_status=known_outcome,admission_guard_visible=False,durability_review_required=True)
 print(json.dumps(result,sort_keys=True))
 return 0 if result['status'] in ('done','idle','dead_letter') else (4 if result['status']=='known_completion_visible_durability_unconfirmed' else (3 if result['status']=='paused_manual' else 2))
# Filled by author from fixed source; project declaration admits only the two explicit loopback ports.
FIXED={'ana.dil': '179d177e10b3c5cae09baf9ce814b51e5887b7cbc6302e4b03f4b0c9ddc4c7c4', 'gocler/0001_effect.sql': 'e62cb51dd88c3c37ea9d34aa545c97cd507f2063f35be6ac7c91ad92e4d7ec8d', 'catalog.pdf': '6c48cfc8780ccf55784d4d25dd9a3cbe947c31f4dd781a23c62f8a5325f56288'}
MANIFEST_PATTERN='proje\\ "owned\\-fixed\\-effect"\\ olsun\\\nsürüm\\ "0\\.1\\.0"\\ olsun\\\nmorfoloji\\ "zee\\-tr\\-1"\\ olsun\\\ngiriş\\ "ana\\.dil"\\ olsun\\\nyetkinlikler\\ "veritabanı",\\ "ağ",\\ "yerel\\-ağ",\\ "dosya\\-okuma"\\ listesi\\ olsun\\\nağ_hedefleri\\ "smtp://127\\.0\\.0\\.1:[0-9]{1,5}"\\ listesi\\ olsun\\\ne_posta_bağlantı_değişkeni\\ "ZEE_EFFECT_SMTP_URL"\\ olsun\\\nveritabanı_hedefi\\ "postgresql://127\\.0\\.0\\.1:[0-9]{1,5}/postgres"\\ olsun\\\nveritabanı_bağlantı_değişkeni\\ "ZEE_EFFECT_DB_URL"\\ olsun\\\nveritabanı_göçleri\\ "gocler"\\ olsun\\\n'
if __name__=='__main__':
 try:raise SystemExit(main())
 except (Exception,KeyboardInterrupt):
  print(json.dumps({'schema':'zee-native-effect-result-1','status':'rejected'}),file=sys.stderr);raise SystemExit(2)
