import importlib.util,json,os,signal,socket,subprocess,sys,tempfile,time,unittest
from pathlib import Path
from unittest.mock import patch
BASE=Path(__file__).resolve().parents[2]
BINARY=None
FIXTURES=BASE/'compiler/tests/fixtures/yonetilen-moduller'
spec=importlib.util.spec_from_file_location('manager',BASE/'scripts/modul-yoneticisi.py'); M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
class ManagerTests(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory(prefix='k273-owned-');self.work=Path(self.temp.name).resolve();os.chmod(self.work,0o700)
  self.root=self.work/'project';self.m=M.Manager(self.root,BINARY)
  with socket.socket() as s:s.bind(('127.0.0.1',0));self.port=s.getsockname()[1]
  self.m.init(self.port)
 def tearDown(self):self.temp.cleanup()
 def install(self,name):self.m.change(src=FIXTURES/name)
 def projection(self):return {str(p.relative_to(self.root)):p.read_bytes() for p in self.root.rglob('*') if p.is_file()}
 def request(self,path):
  with socket.create_connection(('127.0.0.1',self.port),timeout=1) as s:
   s.sendall(('GET '+path+' HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n').encode());out=b''
   while True:
    b=s.recv(8192)
    if not b:break
    out+=b;self.assertLess(len(out),65536)
   return out
 def launch(self):
  p=subprocess.Popen([sys.executable,'-I','-B',str(BASE/'scripts/modul-yoneticisi.py'),'--root',str(self.root),'--binary',str(BINARY),'launch'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,start_new_session=True)
  try:
   for _ in range(150):
    try:self.request('/');return p
    except OSError:time.sleep(.02)
   self.fail('owned native did not start')
  except BaseException:
   try:os.killpg(p.pid,signal.SIGKILL)
   except (ProcessLookupError,PermissionError):pass
   p.wait(timeout=3);p.stderr.close();raise
 def stop(self,p):
  try:p.terminate();p.wait(timeout=7)
  finally:
   if p.poll() is None:os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=2)
   p.stderr.close()
 def test_actual_two_modules_remove_core_menu_content_sql_archive(self):
  (self.root/'.zee/appdata').mkdir(parents=True);(self.root/'.zee/appdata/sentinel').write_bytes(b'owned-state')
  core=(self.root/'ana.dil').read_bytes();ino=self.root.stat().st_ino
  self.install('haber');self.install('katalog');p=self.launch()
  try:
   self.assertIn('Anonim çekirdek'.encode(),self.request('/'));self.assertIn(b'Anonim Haber',self.request('/haber'));self.assertIn(b'Anonim Katalog',self.request('/katalog'))
   self.assertIn(b'Anonim Haber|Anonim Katalog',self.request('/moduller'));self.assertIn(b'haber_sayfasi|katalog_sayfasi',self.request('/icerik-turleri'))
   with self.assertRaises((M.Reject,BlockingIOError)):self.m.change(remove='haber')
  finally:self.stop(p)
  self.assertEqual(list((self.m.control/'active').iterdir()),[])
  self.m.change(remove='haber');p=self.launch()
  try:
   self.assertIn(b'Anonim Katalog',self.request('/katalog'));self.assertNotIn(b'Anonim Haber',self.request('/moduller'));self.assertIn('Anonim çekirdek'.encode(),self.request('/'))
  finally:self.stop(p)
  self.assertEqual((self.root/'ana.dil').read_bytes(),core);self.assertEqual(self.root.stat().st_ino,ino)
  self.assertEqual((self.root/'.zee/appdata/sentinel').read_bytes(),b'owned-state')
  self.assertEqual((self.root/'archive/haber/001100.sql').read_bytes(),(FIXTURES/'haber/sql/001100.sql').read_bytes())
  self.assertFalse((self.root/'migrations').exists())
  with self.assertRaises(M.Reject):self.install('haber')
 def test_collision_before_any_write(self):
  self.install('haber');before=self.projection()
  import shutil
  src=self.work/'candidate';shutil.copytree(FIXTURES/'katalog',src)
  m=json.loads((src/'zee-modul.json').read_text());m['menu']['slug']='haber';m['menu']['path']='/haber';(src/'zee-modul.json').write_text(json.dumps(m))
  with self.assertRaises(M.Reject):self.m.change(src=src)
  self.assertEqual(before,self.projection())
 def test_foreign_receipt_and_edited_source_reject(self):
  r=self.m.receipt();r['root']=[0,0];M.write(self.m.control/'receipt.json',M.encoded(r));before=self.projection()
  with self.assertRaises(M.Reject):self.install('haber')
  self.assertEqual(before,self.projection())
 def test_edited_core_reject_no_overwrite(self):
  (self.root/'ana.dil').write_bytes(b'foreign');before=self.projection()
  with self.assertRaises(M.Reject):self.install('haber')
  self.assertEqual(before,self.projection())
 def test_journal_fault_blocks_launch_explicit_finish(self):
  original=M.write
  def fault(p,b):
   if Path(p)==self.root/'modul_haber.dil':raise OSError('injected')
   return original(p,b)
  with patch.object(M,'write',side_effect=fault):
   with self.assertRaises(OSError):self.install('haber')
  self.assertTrue((self.m.control/'journal.json').exists())
  with self.assertRaises(M.Reject):self.m.ready()
  self.m.reconcile();self.assertFalse((self.m.control/'journal.json').exists());self.assertIn('haber',self.m.ready()['modules'])
 def test_postclear_fsync_explicit_visible_outcome(self):
  original=M.syncdir
  def fault(p):
   if Path(p)==self.m.control and not (self.m.control/'journal.json').exists() and 'haber' in self.m.receipt(check=False)['modules']:raise OSError('injected')
   return original(p)
  with patch.object(M,'syncdir',side_effect=fault):
   with self.assertRaises(M.Published):self.install('haber')
  self.assertFalse((self.m.control/'journal.json').exists());self.assertIn('haber',self.m.ready()['modules'])
 def test_active_crash_blocks_updates_no_pid_adoption(self):
  p=self.launch()
  try:
   p.kill();os.waitid(os.P_PID,p.pid,os.WEXITED|os.WNOWAIT)
   self.assertTrue(list((self.m.control/'active').iterdir()))
   with self.assertRaises(M.Reject):self.install('haber')
  finally:
   try:os.killpg(p.pid,signal.SIGKILL)
   except ProcessLookupError:pass
   p.wait(timeout=2);p.stderr.close()
  with self.assertRaises(M.Reject):self.install('haber')
 def test_prespawn_failure_marker_retained(self):
  with patch.object(M.subprocess,'Popen',side_effect=OSError('injected')):
   with self.assertRaises(OSError):self.m.launch()
  self.assertTrue(list((self.m.control/'active').iterdir()))
  with self.assertRaises(M.Reject):self.install('haber')
 def test_duplicate_json_and_symlink_snapshot(self):
  with self.assertRaises(M.Reject):M.decode(b'{"x":1,"x":2}')
  with self.assertRaises(M.Reject):M.decode(b'{"x":NaN}')
  link=self.work/'link';link.symlink_to(self.root/'ana.dil')
  with self.assertRaises(OSError):M.snapshot(link)
 def test_snapshot_actual_mutation_and_swap_rejected(self):
  p=self.work/'snapshot';p.write_bytes(b'original');real=M.os.read;done=[False]
  def mutate(fd,n):
   b=real(fd,n)
   if b and not done[0]:done[0]=True;p.write_bytes(b'changed!')
   return b
  with patch.object(M.os,'read',side_effect=mutate):
   with self.assertRaises(M.Reject):M.snapshot(p)
  p.write_bytes(b'original');done[0]=False
  def swap(fd,n):
   b=real(fd,n)
   if b and not done[0]:
    done[0]=True;q=self.work/'replacement';q.write_bytes(b'original');os.replace(q,p)
   return b
  with patch.object(M.os,'read',side_effect=swap):
   with self.assertRaises(M.Reject):M.snapshot(p)
 def test_active_reconcile_exact_review_and_durable_clear(self):
  with patch.object(M.subprocess,'Popen',side_effect=OSError('injected')):
   with self.assertRaises(OSError):self.m.launch()
  active=self.m.control/'active';paths=list(active.iterdir());review=self.work/'review.json'
  r={'schema':'zee-module-offline-review-1','root':list(self.m.identity),'markers':{p.name:M.sha(p.read_bytes()) for p in paths},'ownership':'owned-service-group-cleanup-confirmed'}
  bad=dict(r);bad['markers']={};review.write_bytes(M.encoded(bad))
  with self.assertRaises(M.Reject):self.m.reconcile_active(review)
  self.assertEqual(paths,list(active.iterdir()))
  review.write_bytes(M.encoded(r));self.m.reconcile_active(review)
  self.assertEqual([],list(active.iterdir()));self.install('haber')
 def test_two_shared_locks_and_exclusive_refusal(self):
  other=M.Manager(self.root,BINARY)
  with self.m.locked(shared=True),other.locked(shared=True):
   with self.assertRaises(BlockingIOError):self.install('haber')
 def test_malformed_journal_rejects_before_touch(self):
  before=self.projection();M.write(self.m.control/'journal.json',M.encoded({'schema':'zee-module-journal-1','root':list(self.m.identity),'operations':[{'path':'foreign','old':None,'new':'eA=='}],'receipt':self.m.receipt()}))
  with self.assertRaises(M.Reject):self.m.reconcile()
  self.assertFalse((self.root/'foreign').exists());self.assertEqual((self.root/'ana.dil').read_bytes(),before['ana.dil'])
 def prepared_journal(self):
  original=self.m.finish
  with patch.object(self.m,'finish',side_effect=M.Reject('prepared')):
   with self.assertRaises(M.Reject):self.install('haber')
  return M.decode(M.snapshot(self.m.control/'journal.json'))
 def assert_bad_journal_unchanged(self,j):
  M.write(self.m.control/'journal.json',M.encoded(j));before=self.projection()
  with self.assertRaises((M.Reject,ValueError)):self.m.reconcile()
  self.assertEqual(before,self.projection())
 def test_journal_nested_receipt_reject_before_any_write(self):
  j=self.prepared_journal();j['receipt']['modules']=[];self.assert_bad_journal_unchanged(j)
 def test_journal_new_hash_inventory_consistency_before_write(self):
  import base64
  j=self.prepared_journal();j['operations'][0]['new']=base64.b64encode(b'wrong bytes').decode();self.assert_bad_journal_unchanged(j)
 def test_journal_missing_operation_reject_before_write(self):
  j=self.prepared_journal();j['operations'].pop();self.assert_bad_journal_unchanged(j)
 def test_journal_later_foreign_state_reject_before_first_write(self):
  j=self.prepared_journal();op=j['operations'][-1];p=self.root/op['path'];M.write(p,b'foreign-owned-fixture');self.assert_bad_journal_unchanged(j)
 def test_active_directory_symlink_no_external_write_or_unlink(self):
  outside=self.work/'outside';outside.mkdir(mode=0o700);sentinel=outside/'sentinel';sentinel.write_bytes(b'foreign')
  active=self.m.control/'active';active.rmdir();active.symlink_to(outside,target_is_directory=True)
  with self.assertRaises((M.Reject,OSError)):self.m.launch()
  with self.assertRaises((M.Reject,OSError)):self.install('haber')
  with self.assertRaises((M.Reject,OSError)):self.m.reconcile_active(self.work/'missing')
  self.assertEqual({p.name:p.read_bytes() for p in outside.iterdir()},{'sentinel':b'foreign'})
 def test_archive_directory_symlink_no_external_write(self):
  outside=self.work/'outside';outside.mkdir(mode=0o700);(outside/'sentinel').write_bytes(b'foreign');(self.root/'archive').symlink_to(outside,target_is_directory=True)
  before=self.projection()
  with self.assertRaises((M.Reject,OSError)):self.install('haber')
  self.assertEqual({'sentinel':b'foreign'},{p.name:p.read_bytes() for p in outside.iterdir()});self.assertEqual(before,self.projection())
 def test_helper_fifo_actual_quiet_bounded_owned_group(self):
  import shutil
  launch=self.work/'launch';launch.mkdir(mode=0o700);shutil.copy2(BASE/'scripts/modul-yoneticisi.py',launch/'module_manager.py');os.mkfifo(launch/'lifecycle_core.py',0o600)
  code="import subprocess,sys; r=subprocess.run(sys.argv[1:],capture_output=True,timeout=1); assert r.returncode==2 and r.stdout==b'' and r.stderr==b'module-operation-rejected\\n'; print('quiet2')"
  out=M.H['run']([sys.executable,'-I','-B','-c',code,sys.executable,'-I','-B',str(launch/'module_manager.py'),'--root',str(self.work/'private-canary'),'--binary',str(BINARY),'init','8090'],timeout=3,limit=4096)
  self.assertEqual(out,b'quiet2\n')
 def test_binary_fifo_and_observed_growth_bound(self):
  fifo=self.work/'binaryfifo';os.mkfifo(fifo,0o600);fake=object.__new__(M.Manager);fake.binary=fifo
  started=time.monotonic()
  with self.assertRaises(M.Reject):fake.large_binary()
  self.assertLess(time.monotonic()-started,1)
  p=self.work/'tiny';p.write_bytes(b'x');fake.binary=p;reads=[0]
  def endless(fd,n):reads[0]+=1;return b'x'*n
  with patch.object(M.os,'read',side_effect=endless):
   with self.assertRaises(M.Reject):fake.large_binary()
  self.assertEqual(reads[0],2049)
 def test_quiet_cli_invalid_args_no_canary(self):
  code="import subprocess,sys; r=subprocess.run(sys.argv[1:],capture_output=True,timeout=1); assert r.returncode==2 and r.stdout==b'' and r.stderr==b'module-operation-rejected\\n'; print('quiet2')"
  out=M.H['run']([sys.executable,'-I','-B','-c',code,sys.executable,'-I','-B',str(BASE/'scripts/modul-yoneticisi.py'),'--private-canary-unknown'],timeout=3,limit=4096)
  self.assertEqual(out,b'quiet2\n')
 def test_journal_strict_nested_receipt_metadata_matrix(self):
  import copy
  j=self.prepared_journal()
  for field,value in [('migration_range',[True,1199]),('menu',{'slug':'haber','label':'x','path':'/foreign'}),('files',{})]:
   bad=copy.deepcopy(j);bad['receipt']['modules']['haber'][field]=value;self.assert_bad_journal_unchanged(bad)
  bad=copy.deepcopy(j);bad['receipt']['retired']['haber']=bad['receipt']['modules']['haber'];self.assert_bad_journal_unchanged(bad)
  bad=copy.deepcopy(j);bad['receipt']['files']['extra.dil']='0'*64;self.assert_bad_journal_unchanged(bad)
if __name__=='__main__':
 import argparse,hashlib
 parser=argparse.ArgumentParser(allow_abbrev=False)
 parser.add_argument('--binary',type=Path,required=True)
 parser.add_argument('--expected-source',required=True)
 parser.add_argument('--proof',type=Path,required=True)
 args,tests=parser.parse_known_args()
 BINARY=args.binary.resolve(strict=True)
 proof={'schema':'zee-managed-module-tests-1','status':'failed','expected_source':args.expected_source}
 try:
  assert len(args.expected_source)==40 and all(c in '0123456789abcdef' for c in args.expected_source)
  assert not args.proof.exists() and args.proof.parent.is_dir()
  probe=object.__new__(M.Manager);probe.binary=BINARY
  binary_hash=probe.large_binary()
  runtime=[BASE/'scripts/modul-yoneticisi.py',BASE/'scripts/lifecycle_core.py',Path(__file__).resolve()]
  fixture_files=sorted(p for p in FIXTURES.rglob('*') if p.is_file())
  source_paths=runtime+fixture_files
  assert len(fixture_files)==10
  source_pins={str(p.relative_to(BASE)):M.sha(M.snapshot(p)) for p in source_paths}
  build=json.loads(M.H['run']([str(BINARY),'sürüm','--json'],timeout=15,limit=8192))
  assert build.get('schema')=='zee-build-v1' and build.get('source_sha')==args.expected_source
  proof.update(binary_sha256=binary_hash,build=build,sources=source_pins)
  result=unittest.main(argv=[sys.argv[0]]+tests,verbosity=2,exit=False).result
  assert result.testsRun==23 and not result.skipped and result.wasSuccessful()
  assert probe.large_binary()==binary_hash
  assert all(M.sha(M.snapshot(BASE/n))==h for n,h in source_pins.items())
  assert json.loads(M.H['run']([str(BINARY),'sürüm','--json'],timeout=15,limit=8192))==build
  proof.update(status='passed',tests=23,source_pins_unchanged=True,owned_test_scopes_completed=True)
 except BaseException as error:
  proof.update(status='failed',failure_type=type(error).__name__)
 finally:
  args.proof.write_text(json.dumps(proof,indent=2)+'\n')
 raise SystemExit(0 if proof['status']=='passed' else 2)
