#!/usr/bin/env python3
"""Preparation only: real preflight/helper subprocesses, not native SMTP acceptance."""
import argparse,contextlib,hashlib,importlib.util,json,os,stat,shutil,sys,tempfile,unittest
from pathlib import Path
BASE=Path(__file__).resolve().parents[2]
FIXTURE=BASE/'compiler/tests/fixtures/native-effect-consumer'
spec=importlib.util.spec_from_file_location('prep_core',BASE/'scripts/lifecycle_core.py');H=importlib.util.module_from_spec(spec);spec.loader.exec_module(H)
parser=argparse.ArgumentParser();parser.add_argument('--dil',type=Path,required=True);ARGS,REST=parser.parse_known_args();BINARY=ARGS.dil.resolve(strict=True)
HOST=BASE/'scripts/effect-dispatcher.py'
def sha(p):
 before=p.lstat()
 if not stat.S_ISREG(before.st_mode) or before.st_size>134217728:raise ValueError('binary role')
 fd=os.open(p,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK);value=hashlib.sha256();count=0
 try:
  while True:
   data=os.read(fd,65536)
   if not data:break
   count+=len(data)
   if count>134217728:raise ValueError('binary role')
   value.update(data)
  if count!=before.st_size:raise ValueError('binary changed')
 finally:os.close(fd)
 return value.hexdigest()
class ContractTests(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory(prefix='k277-prep-',dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp');self.root=Path(self.temp.name).resolve();os.chmod(self.root,0o700)
  self.project=self.root/'project';shutil.copytree(FIXTURE,self.project);self.state=self.root/'state';self.state.mkdir(mode=0o700);self.cred=self.root/'credential';self.cred.mkdir(mode=0o700)
  for name,value in [('db','host=127.0.0.1 port=18093 user=effect_app dbname=postgres sslmode=disable'),('mail','smtp://fixture:K277_SYNTHETIC_PASSWORD@127.0.0.1:18094?from=sender@example.invalid')]:p=self.cred/name;p.write_text(value);p.chmod(0o400)
 def tearDown(self):self.temp.cleanup()
 def argv(self,binary=None,projecthash=None):
  names=('ana.dil','proje.dil','gocler/0001_effect.sql','catalog.pdf');ph=projecthash or hashlib.sha256('\n'.join(n+':'+sha(self.project/n) for n in names).encode()).hexdigest()
  return ['--dil',str(binary or BINARY),'--project',str(self.project),'--credential-root',str(self.cred),'--state',str(self.state),'--expected-source','0'*40,'--binary-sha256',sha(BINARY),'--project-sha256',ph,'--owner','prep','--env','ZEE_EFFECT_DB_URL=db','--env','ZEE_EFFECT_SMTP_URL=mail','--candidate']
 def cli(self,args,expected=2,ambient=False):
  code='''import contextlib,io,json,os,runpy,sys
out,err=io.StringIO(),io.StringIO()
def audit(event,args):
 if event=='subprocess.Popen':raise SystemExit(71)
sys.addaudithook(audit)
if sys.argv[1]=='ambient':os.environ['ZEE_EFFECT_DB_URL']='CANARY_PRIVATE_ARG'
path=sys.argv[2];sys.argv=[path]+sys.argv[3:]
with contextlib.redirect_stdout(out),contextlib.redirect_stderr(err):
 try:runpy.run_path(path,run_name='__main__')
 except SystemExit as error:status=error.code
assert status==EXPECTED,(status,out.getvalue(),err.getvalue())
text=out.getvalue()+err.getvalue();assert 'CANARY_PRIVATE_ARG' not in text and 'K277_SYNTHETIC_PASSWORD' not in text
record=json.loads(out.getvalue() if out.getvalue() else err.getvalue());assert record['status'] in ('rejected','paused_manual')
print('ok')
'''.replace('EXPECTED',str(expected))
  self.assertEqual(H.run([sys.executable,'-I','-B','-c',code,'ambient' if ambient else 'ordinary',HOST,*args],timeout=6,limit=8192),b'ok\n')
 def test_fixed_source_and_origin_rejections_no_native_spawn(self):
  (self.project/'ana.dil').write_text('"different handler" yaz\n');self.cli(self.argv())
  shutil.copy2(FIXTURE/'ana.dil',self.project/'ana.dil');p=self.project/'proje.dil';p.write_text(p.read_text().replace('smtp://127.0.0.1:18094','smtp://provider.example:587'));self.cli(self.argv())
 def test_actual_regular_role_rejections_before_native_spawn(self):
  fifo=self.root/'fifo';os.mkfifo(fifo,0o600);self.cli(self.argv(binary=fifo))
  large=self.root/'large'
  with large.open('wb') as f:f.truncate(134217729)
  self.cli(self.argv(binary=large))
  (self.project/'ana.dil').unlink();os.mkfifo(self.project/'ana.dil',0o600);self.cli(self.argv(projecthash='0'*64))
 def test_actual_masked_unknown_cli(self):self.cli(['--unknown-CANARY_PRIVATE_ARG'])
 def test_actual_ambient_and_credential_FIFO_rejected(self):
  self.cli(self.argv(),ambient=True);(self.cred/'db').unlink();os.mkfifo(self.cred/'db',0o400);self.cli(self.argv())
 def test_actual_pending_marker_blocks_all_native_spawns(self):
  for index in range(17):(self.state/('entry'+str(index))).touch(mode=0o600)
  self.cli(self.argv())
  for path in self.state.iterdir():path.unlink()
  (self.state/'pending.json').write_text('{}');self.cli(self.argv(),expected=3)
  self.helper('''count=[0];closed=[False]
class Entries:
 def __enter__(self):return self
 def __exit__(self,*args):closed[0]=True
 def __iter__(self):
  for index in range(1000):count[0]+=1;yield None
real=m.os.scandir;m.os.scandir=lambda path:Entries()
try:
 try:m.bounded_state_directory(Path('/unused'));raise AssertionError('accepted')
 except m.Paused:pass
finally:m.os.scandir=real
assert count[0]==17 and closed[0]
print('ok')
''')
  self.helper('''import contextlib,hashlib,io,json,shutil,tempfile
with tempfile.TemporaryDirectory(prefix='k277-visible-model-') as td:
 root=Path(td).resolve();state=root/'state';state.mkdir(mode=0o700);project=root/'project';shutil.copytree(Path(m.__file__).parents[1]/'compiler/tests/fixtures/native-effect-consumer',project);cred=root/'cred';cred.mkdir(mode=0o700)
 for name,value in [('db','host=127.0.0.1 port=18093 user=effect_app dbname=postgres sslmode=disable'),('mail','smtp://fixture:K277_SYNTHETIC_PASSWORD@127.0.0.1:18094?from=sender@example.invalid')]:
  path=cred/name;path.write_text(value);path.chmod(0o400)
 binary=Path(BINARY_LITERAL);names=('ana.dil','proje.dil','gocler/0001_effect.sql','catalog.pdf');digest=lambda path:hashlib.sha256(path.read_bytes()).hexdigest();ph=hashlib.sha256('\\n'.join(n+':'+digest(project/n) for n in names).encode()).hexdigest()
 sys.argv=[str(m.__file__),'--dil',str(binary),'--project',str(project),'--credential-root',str(cred),'--state',str(state),'--expected-source','a'*40,'--binary-sha256',digest(binary),'--project-sha256',ph,'--owner','model','--env','ZEE_EFFECT_DB_URL=db','--env','ZEE_EFFECT_SMTP_URL=mail','--candidate']
 build={'schema':'zee-build-v1','source_sha':'a'*40,'dirty':True};calls=[];second=[False];fired=[False]
 def native(args,**kwargs):
  if args[1]=='sürüm':return json.dumps(build).encode()
  op=args[3];calls.append(op);return ({'claim':'bos' if second[0] else '1:1','prepare':'started','send':'sent','ack':'done'}[op]+'\\n').encode()
 m.run_native=native;real=m.syncdir
 def fault(path):
  if not(state/'pending.json').exists():fired[0]=True;raise OSError('controlled permanent final fsync')
  real(path)
 m.syncdir=fault
 with contextlib.redirect_stdout(io.StringIO()) as out:first=m.main()
 record=json.loads(out.getvalue());assert first==4 and record['status']=='known_completion_visible_durability_unconfirmed' and record['completed_status']=='done' and not record['admission_guard_visible'] and record['durability_review_required'] and record['result_publication_unconfirmed'] and not(state/'pending.json').exists()
 second[0]=True;m.syncdir=real
 with contextlib.redirect_stdout(io.StringIO()) as out:again=m.main()
 assert again==0 and json.loads(out.getvalue())['status']=='idle' and calls==['claim','prepare','send','ack','claim']
print('ok')
'''.replace('BINARY_LITERAL',repr(str(BINARY))))
 def helper(self,code):
  prelude="import importlib.util,os,subprocess,sys,time;from pathlib import Path;spec=importlib.util.spec_from_file_location('subject',sys.argv[1]);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)\n"
  self.assertEqual(H.run([sys.executable,'-I','-B','-c',prelude+code,HOST],timeout=8,limit=4096),b'ok\n')
 def test_actual_bounded_native_capture_and_timeout(self):
  self.helper('''real=m.subprocess.Popen;children=[]
def spawn(*a,**k):
 p=real(*a,**k);children.append(p);return p
m.subprocess.Popen=spawn
try:
 for code,timeout in [("print('x'*257)",1),("import time;time.sleep(20)",.08)]:
  try:m.run_native([sys.executable,'-I','-c',code],timeout,256,{'PATH':'/usr/bin:/bin'});raise AssertionError('accepted')
  except m.Paused:pass
  p=children[-1];assert p.returncode is not None and p.stdout.closed
finally:
 for p in children:
  if p.returncode is None:p.kill();p.wait(timeout=2)
print('ok')
''')
 def test_actual_default_child_fds_and_environment(self):
  self.helper('''r,w=os.pipe();os.set_inheritable(r,True)
try:
 code="import os; ok=False\\ntry:os.fstat("+str(r)+")\\nexcept OSError:ok=True\\nassert ok and 'AMBIENT_CANARY' not in os.environ;print('ok')"
 os.environ['AMBIENT_CANARY']='private'
 assert m.run_native([sys.executable,'-I','-c',code],2,256,{'PATH':'/usr/bin:/bin'})==b'ok\\n'
finally:os.close(r);os.close(w)
print('ok')
''')
 def test_fixture_cleanup_failure_still_cleans_retained_groups(self):
  code='''import importlib.util,os,shutil,sys,tempfile
from pathlib import Path
from types import SimpleNamespace
s=Path(sys.argv[1]);spec=importlib.util.spec_from_file_location('fixture',s);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
with tempfile.TemporaryDirectory(prefix='k277-owned-cleanup-') as td:
 root=Path(td).resolve();original_fixture=m.FIXTURE;fixed=root/'fixed';fixed.mkdir();names=('ana.dil','catalog.pdf','gocler/0001_effect.sql','proje.dil')
 for name in names:
  target=fixed/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(m.bounded(original_fixture/name,131072))
 (fixed/'gocler/9999_UNLISTED.sql').write_text('UNLISTED SQL MUST NEVER EXECUTE');(fixed/'extra.dil').write_text('UNLISTED SOURCE');os.mkfifo(fixed/'unexpected_fifo')
 m.FIXTURE=fixed
 try:
  project_owner=object.__new__(m.Fixture);project_owner.index=0;project_owner.work=root
  project=project_owner.project(12093,12094)
  assert sorted(str(p.relative_to(project)) for p in project.rglob('*') if p.is_file())==sorted(names)
  for name in names:
   data=m.bounded(original_fixture/name,131072)
   if name=='proje.dil':data=data.replace(b':18093',b':12093').replace(b':18094',b':12094')
   assert (project/name).read_bytes()==data
  assert not(project/'extra.dil').exists() and not(project/'gocler/9999_UNLISTED.sql').exists() and not(project/'unexpected_fifo').exists()
 finally:m.FIXTURE=original_fixture
 f=object.__new__(m.Fixture);f.closed=False
 f.owner=m.O.Supervisor(SimpleNamespace(project=Path(td),dil=Path(sys.executable),stop_grace=.1),m.O.JsonLog())
 f.foreign_owner=m.O.Supervisor(SimpleNamespace(project=Path(td),dil=Path(sys.executable),stop_grace=.1),m.O.JsonLog())
 f.owner.spawn([sys.executable,'-I','-c','import time;time.sleep(20)']);f.foreign=f.foreign_owner.spawn([sys.executable,'-I','-c','import time;time.sleep(20)'])
 original=f.owner.cleanup
 def fail_after_cleanup():original();raise OSError('controlled')
 f.owner.cleanup=fail_after_cleanup
 try:
  try:f.close();raise AssertionError('accepted')
  except OSError:pass
  assert not f.owner.children and not f.foreign_owner.children and not f.closed
 finally:
  f.owner.cleanup=original
  try:f.owner.cleanup()
  finally:f.foreign_owner.cleanup()
print('ok')
'''
  self.assertEqual(H.run([sys.executable,'-I','-B','-c',code,BASE/'scripts/tests/effect-dispatcher-native-test.py'],timeout=6,limit=4096),b'ok\n')
if __name__=='__main__':unittest.main(argv=[sys.argv[0]]+REST,verbosity=2)
