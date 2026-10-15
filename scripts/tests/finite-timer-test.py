import ast,contextlib,hashlib,importlib.util,json,os,signal,subprocess,sys,tempfile,threading,time,unittest
from pathlib import Path
from unittest import mock
HERE=Path(__file__).resolve().parents[1]
s=importlib.util.spec_from_file_location('finite',HERE/'finite_timer.py');M=importlib.util.module_from_spec(s);s.loader.exec_module(M)
class Preparation(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name).resolve();self.state=self.root/'state';self.config=self.root/'config.json';self.c={'schema':'zee-sql-timer-config-1','source_sha':'a'*40,'binary_sha256':'b'*64,'project_sha256':'c'*64,'pg_port':54321,'credential_name':'ci-url','owner':'owned-timer'}
  for name in ('repo','project','credential_root'):
   p=self.root/name;p.mkdir(mode=0o700);self.c[name]=p
  for name in ('ca','dil'):
   p=self.root/name;p.write_bytes(b'synthetic path fixture, never native');self.c[name]=p
  self.save();M.D.CANCELLED=False
  self.real_paths=M.execution_paths;self.paths=mock.patch.object(M,'execution_paths',side_effect=lambda a:(a.path,a.path/'tmp'));self.paths.start();self.platform=mock.patch.object(M,'runtime_supported',return_value=True);self.platform.start()
 def tearDown(self):self.paths.stop();self.platform.stop();M.D.CANCELLED=False;self.temp.cleanup()
 def save(self):self.config.write_text(json.dumps({k:str(v) if isinstance(v,Path) else v for k,v in self.c.items()}));self.config.chmod(0o600)
 def pinned(self):return mock.patch.object(M,'source_pin',return_value={'fixture':'pinned'})
 def python(self,code):return mock.patch.object(M,'argv',return_value=[sys.executable,'-I','-B','-c',code])
 def test_syntax_reviewed_owner_and_fixed_cli(self):
  ast.parse((HERE/'finite_timer.py').read_text());self.assertEqual(M.D.digest(HERE/'lifecycle_core.py'),M.HELPER);a=M.argv(self.c);self.assertEqual(a[-2:],['--ticks','1']);self.assertEqual(a.count('--credential-env'),1);self.assertIn('ZEE_JOB_CI_URL=ci-url',a);self.assertNotIn('SMTP',str(a))
 def test_private_config_strict_keys_duplicates_and_mode(self):
  self.assertEqual(M.config(self.config)['pg_port'],54321)
  self.c['secret']='NEVER_ALLOWED';self.save()
  with self.assertRaises(ValueError):M.config(self.config)
  with self.assertRaises(ValueError):M.decode(b'{"a":1,"a":2}')
  self.config.chmod(0o644)
  with self.assertRaises(ValueError):M.config(self.config)
 def test_config_pin_binds_same_single_snapshot_as_parsed_values(self):
  real=M.read_private
  with mock.patch.object(M,'read_private',wraps=real) as read:c=M.config(self.config)
  self.assertEqual(read.call_count,1);self.assertEqual(c['_config_sha256'],hashlib.sha256(self.config.read_bytes()).hexdigest())
 def test_config_changed_after_snapshot_rejects_before_source_queries(self):
  c=M.config(self.config);self.config.write_text('{}')
  with mock.patch.object(M.D,'clean_exact',side_effect=AssertionError('no source query')),self.assertRaises(ValueError):M.source_pin(c)
 def test_state_symlink_scope_mode_and_root_reject(self):
  self.state.symlink_to(self.c['project'],target_is_directory=True)
  with self.assertRaises(ValueError):M.state_path(self.state,self.c)
  self.state.unlink();self.state.mkdir(mode=0o755)
  with self.assertRaises(ValueError):M.state_path(self.state,self.c)
  with self.assertRaises(ValueError):M.state_path(self.c['credential_root']/'child',self.c)
  with mock.patch.object(M.os,'geteuid',return_value=0),self.assertRaises(ValueError):M.state_path(self.root/'fresh',self.c)
 def test_check_only_never_creates_state_or_child(self):
  with self.pinned(),mock.patch.object(M,'execute',side_effect=AssertionError('must not run')):self.assertEqual(M.main(['--config',str(self.config),'--state',str(self.state)]),0)
  self.assertFalse(self.state.exists())
 def test_latch_durable_before_spawn_and_failure_blocks_next_launch(self):
  def before(*a,**k):
   self.assertEqual(json.loads((self.state/'inflight.json').read_text())['source_sha'],self.c['source_sha']);raise OSError('modeled before Popen')
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=before) as child:
   with self.assertRaises(M.Paused):M.execute(self.c,self.state)
   with self.assertRaises(M.Paused):M.execute(self.c,self.state)
   self.assertEqual(child.call_count,1)
 def test_actual_short_python_success_no_output_canary_persisted(self):
  with self.pinned(),self.python('print("K268_PRIVATE_OUTPUT_CANARY")'):self.assertEqual(M.execute(self.c,self.state),0)
  self.assertFalse((self.state/'inflight.json').exists());self.assertEqual(json.loads((self.state/'last.json').read_text())['outcome'],'completed')
  for p in self.state.iterdir():
   if p.is_file():self.assertNotIn(b'CANARY',p.read_bytes())
 def test_actual_short_python_exit3_pauses_not_native_dispatcher(self):
  with self.pinned(),self.python('print("K268_PRIVATE_OUTPUT_CANARY");raise SystemExit(3)'),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertTrue((self.state/'inflight.json').exists());self.assertFalse((self.state/'last.json').exists())
  for p in self.state.iterdir():
   if p.is_file():self.assertNotIn(b'CANARY',p.read_bytes())
 def test_postsource_pin_failure_retains_latch(self):
  with mock.patch.object(M,'source_pin',side_effect=[{'fixture':'before'},{'fixture':'changed'}]),self.python('pass'),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertTrue((self.state/'inflight.json').exists())
 def test_lock_conflict_no_second_launch(self):
  with M.locked(self.state),self.pinned(),mock.patch.object(M.D,'run',side_effect=AssertionError('no child')),self.assertRaises(M.Busy):M.execute(self.c,self.state)
  self.assertFalse((self.state/'inflight.json').exists())
 def test_latch_symlink_and_foreign_lock_mode_refuse(self):
  self.state.mkdir(mode=0o700);outside=self.root/'outside';outside.write_bytes(b'foreign');(self.state/'inflight.json').symlink_to(outside)
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=AssertionError('no child')),self.assertRaises((OSError,ValueError)):M.execute(self.c,self.state)
  self.assertEqual(outside.read_bytes(),b'foreign');(self.state/'inflight.json').unlink();(self.state/'lock').chmod(0o644)
  with self.assertRaises(ValueError):
   with M.locked(self.state):pass
 def test_cancel_before_popen_leaves_pause(self):
  M.D.CANCELLED=True
  with self.pinned(),self.python('raise AssertionError("must not spawn")'),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertTrue((self.state/'inflight.json').exists())
 def test_actual_term_after_spawn_reaps_owned_child_foreign_survives(self):
  ready=self.root/'child-ready';foreign=subprocess.Popen([sys.executable,'-I','-B','-c','import time;time.sleep(10)'],start_new_session=True);seen=[]
  def interrupt():
   end=time.monotonic()+3
   while not ready.exists() and time.monotonic()<end:time.sleep(.01)
   if ready.exists():seen.append(int(ready.read_text()));os.kill(os.getpid(),signal.SIGTERM)
  thread=threading.Thread(target=interrupt);thread.start()
  try:
   code='import os,time;from pathlib import Path;Path('+repr(str(ready))+').write_text(str(os.getpid()));time.sleep(10)'
   with self.pinned(),self.python(code),self.assertRaises(M.Paused):M.main(['--config',str(self.config),'--state',str(self.state),'--run'])
   thread.join(timeout=4);self.assertEqual(len(seen),1)
   with self.assertRaises(ProcessLookupError):os.kill(seen[0],0)
   os.kill(foreign.pid,0);self.assertTrue((self.state/'inflight.json').exists())
  finally:
   if foreign.poll() is None:foreign.terminate()
   foreign.wait(timeout=3);thread.join(timeout=4)
 def test_selector_setup_failure_reaps_actual_owned_child(self):
  children=[];real=M.D.subprocess.Popen
  def owned(*a,**kw):c=real(*a,**kw);children.append(c);return c
  with self.pinned(),self.python('import time;time.sleep(10)'),mock.patch.object(M.D.subprocess,'Popen',side_effect=owned),mock.patch.object(M.D.selectors,'DefaultSelector',side_effect=OSError('modeled selector')),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertEqual(len(children),1);self.assertIsNotNone(children[0].returncode);self.assertTrue((self.state/'inflight.json').exists())
 def review(self):
  latch=(self.state/'inflight.json').read_bytes();return {'schema':'zee-sql-timer-reconcile-1','source_sha':self.c['source_sha'],'binary_sha256':self.c['binary_sha256'],'inflight_sha256':hashlib.sha256(latch).hexdigest(),'service_inactive_reviewed':True,'owned_cgroup_empty_reviewed':True,'sql_outcome_reviewed':'rollback-verified'}
 def receipt(self,value):p=self.root/'review.json';p.write_text(json.dumps(value));p.chmod(0o600);return p,M.D.digest(p)
 def test_reconcile_exact_receipt_only_removes_owned_latch(self):
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=OSError()),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  p,pin=self.receipt(self.review());self.assertEqual(M.reconcile(self.c,self.state,p,pin),0);self.assertFalse((self.state/'inflight.json').exists());self.assertTrue((self.state/'last-reconciliation.json').exists())
 def test_reconcile_stale_hash_false_review_bad_schema_refuse(self):
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=OSError()),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  for field,value in [('service_inactive_reviewed',False),('owned_cgroup_empty_reviewed',False),('inflight_sha256','0'*64),('sql_outcome_reviewed','smtp-delivered')]:
   review=self.review();review[field]=value;p,pin=self.receipt(review)
   with self.subTest(field=field),self.assertRaises(ValueError):M.reconcile(self.c,self.state,p,pin)
   self.assertTrue((self.state/'inflight.json').exists())
 def test_invalid_cli_masks_arguments_and_orphan_receipt_pin(self):
  import io
  for args in (['--private-canary','K268_SECRET_ARG'],['--config',str(self.config),'--state',str(self.state),'--reconciliation-sha256','0'*64]):
   with contextlib.redirect_stderr(io.StringIO()) as err,self.assertRaises(ValueError):M.main(args)
   self.assertEqual(err.getvalue(),'')
  self.assertFalse(self.state.exists())
 def test_private_parent_group_write_rejected(self):
  self.root.chmod(0o770)
  try:
   with self.assertRaises(ValueError):M.state_path(self.state,self.c)
   with self.assertRaises(ValueError):
    with M.locked(self.state):pass
  finally:self.root.chmod(0o700)
 def test_snapshot_bounded_loop_retries_short_reads(self):
  value=self.root/'snapshot';value.write_bytes(b'x'*9000);value.chmod(0o600);real=M.os.read
  with mock.patch.object(M.os,'read',side_effect=lambda fd,n:real(fd,min(97,n))) as reads:self.assertEqual(M.read_private(value),b'x'*9000)
  self.assertGreater(reads.call_count,90)
 def test_snapshot_inplace_mutation_and_path_swap_rejected(self):
  for kind in ('mutate','swap'):
   p=self.root/('snapshot-'+kind);p.write_bytes(b'original');p.chmod(0o600);real=M.os.read;changed=[]
   def race(fd,n):
    raw=real(fd,n)
    if not changed:
     changed.append(True)
     if kind=='mutate':p.write_bytes(b'changed-longer')
     else:
      p.rename(self.root/'snapshot-held');p.write_bytes(b'original');p.chmod(0o600)
    return raw
   with self.subTest(kind=kind),mock.patch.object(M.os,'read',side_effect=race),self.assertRaises(ValueError):M.read_private(p)
 def test_snapshot_swap_before_open_and_parent_namespace_swap_rejected(self):
  p=self.root/'before-open';p.write_bytes(b'original');p.chmod(0o600);real=M.os.open;changed=[]
  def swap(name,flags,*a,**kw):
   if name==p.name and not changed:
    changed.append(True);p.rename(self.root/'before-open-held');p.write_bytes(b'original');p.chmod(0o600)
   return real(name,flags,*a,**kw)
  with mock.patch.object(M.os,'open',side_effect=swap),self.assertRaises(ValueError):M.read_private(p)
  parent=self.root/'parent';parent.mkdir(mode=0o700);p=parent/'value';p.write_bytes(b'original');p.chmod(0o600);read=M.os.read;changed=[]
  def swap_parent(fd,n):
   raw=read(fd,n)
   if not changed:
    changed.append(True);parent.rename(self.root/'held-parent');parent.mkdir(mode=0o700);q=parent/'value';q.write_bytes(b'original');q.chmod(0o600)
   return raw
  with mock.patch.object(M.os,'read',side_effect=swap_parent),self.assertRaises(ValueError):M.read_private(p)
 def test_latch_snapshot_path_swap_rejected(self):
  with M.locked(self.state) as anchor:
   M.store(anchor.fd,'inflight.json',{'a':1},True);real=M.os.read;changed=[]
   def swap(fd,n):
    raw=real(fd,n)
    if not changed:
     changed.append(True);p=self.state/'inflight.json';p.rename(self.state/'held.json');p.write_text('{"a":1}');p.chmod(0o600)
    return raw
   with mock.patch.object(M.os,'read',side_effect=swap),self.assertRaises(ValueError):M.bytes_at(anchor.fd,'inflight.json')
 def test_state_and_tmp_directory_swap_rejected_with_held_descriptors(self):
  with M.locked(self.state) as anchor:
   anchor.temporary();(self.state/'tmp').rename(self.state/'held-tmp');(self.state/'tmp').mkdir(mode=0o700)
   with self.assertRaises(ValueError):anchor.check()
  other=self.root/'other-state'
  with M.locked(other) as anchor:
   other.rename(self.root/'held-state');other.mkdir(mode=0o700)
   with self.assertRaises(ValueError):anchor.check()
 def test_proc_paths_derive_only_own_pid_and_held_fds_model_no_fallback(self):
  with M.locked(self.state) as anchor:
   anchor.temporary();real=M.os.stat
   def info(path,*a,**kw):
    if str(path)==f'/proc/{os.getpid()}/fd/{anchor.fd}':return os.fstat(anchor.fd)
    if str(path)==f'/proc/{os.getpid()}/fd/{anchor.tmp}':return os.fstat(anchor.tmp)
    return real(path,*a,**kw)
   with mock.patch.object(M.os,'stat',side_effect=info):
    cwd,tmp=self.real_paths(anchor)
   self.assertEqual(str(cwd),f'/proc/{os.getpid()}/fd/{anchor.fd}');self.assertEqual(str(tmp),f'/proc/{os.getpid()}/fd/{anchor.tmp}')
   with mock.patch.object(M,'runtime_supported',return_value=False),self.assertRaises(ValueError):self.real_paths(anchor)
 def test_directory_swap_during_spawn_model_cannot_clear_pause_or_write_redirected_result(self):
  held=self.root/'held-state'
  def swap(*a,**kw):self.state.rename(held);self.state.mkdir(mode=0o700);return b''
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=swap),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertTrue((held/'inflight.json').exists());self.assertFalse((self.state/'last.json').exists())
 def test_missing_fcntl_and_bad_helper_quiet_subprocess_no_traceback(self):
  code="""import builtins,runpy,sys
old=builtins.__import__
def hook(name,*a,**kw):
 if name=='fcntl':raise ModuleNotFoundError()
 return old(name,*a,**kw)
builtins.__import__=hook
sys.argv=[sys.argv[1],'--config','/unread','--state','/unread-state']
runpy.run_path(sys.argv[0],run_name='__main__')
"""
  result=subprocess.run([sys.executable,'-I','-B','-c',code,str(HERE/'finite_timer.py')],capture_output=True,timeout=3);self.assertEqual(result.returncode,2,result.stderr);self.assertEqual(result.stdout+result.stderr,b'')
  copy=self.root/'bad-helper';copy.mkdir(mode=0o700);(copy/'finite_timer.py').write_bytes((HERE/'finite_timer.py').read_bytes());(copy/'lifecycle_core.py').write_bytes(b'raise RuntimeError("must never execute")')
  result=subprocess.run([sys.executable,'-I','-B',str(copy/'finite_timer.py'),'--config','/unread','--state','/unread-state'],capture_output=True,timeout=3);self.assertEqual(result.returncode,2);self.assertEqual(result.stdout+result.stderr,b'')
 def test_narrow_core_exact_lineage_without_artifact_entrypoints(self):
  lineage=json.loads((HERE.parent/'compiler/tests/fixtures/sonlu-dispatcher-lineage-v1.json').read_text());core=(HERE/'lifecycle_core.py').read_text();self.assertEqual(hashlib.sha256(core.encode()).hexdigest(),lineage['core_sha256']);self.assertEqual(lineage['source_sha256'],'509106d70e5e35c1142594c98df022b01e689966415bdd94a8f67feecb8587d8')
  after={n.name:ast.get_source_segment(core,n) for n in ast.parse(core).body if isinstance(n,(ast.FunctionDef,ast.ClassDef))}
  self.assertEqual(set(after),set(lineage['functions_exact_source_sha256']))
  for name in after:self.assertEqual(hashlib.sha256(after[name].encode()).hexdigest(),lineage['functions_exact_source_sha256'][name])
  self.assertEqual(M.D.MAX_CAPTURE,8*1024*1024);self.assertFalse(M.D.CANCELLED);self.assertNotIn('tomllib',core);self.assertNotIn('def deliver(',core);self.assertNotIn('source_zip',core)
 def test_reconcile_latch_changed_during_receipt_write_is_not_removed(self):
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=OSError()),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  p,pin=self.receipt(self.review());store=M.store
  def changed(fd,name,value,*a,**kw):
   store(fd,name,value,*a,**kw)
   if name=='last-reconciliation.json':(self.state/'inflight.json').write_text('{"changed":true}')
  with mock.patch.object(M,'store',side_effect=changed),self.assertRaises(ValueError):M.reconcile(self.c,self.state,p,pin)
  self.assertEqual((self.state/'inflight.json').read_text(),'{"changed":true}')
 def test_completed_clear_visible_fsync_failure_is_exit4_domain_and_next_tick_permitted(self):
  fsync=M.os.fsync;calls=[]
  def fail(fd):
   if __import__('stat').S_ISDIR(os.fstat(fd).st_mode) and (self.state/'last.json').exists() and not (self.state/'inflight.json').exists():calls.append(True);raise OSError('PRIVATE_FSYNC_CANARY')
   return fsync(fd)
  with self.pinned(),self.python('print("known completed short model")'),mock.patch.object(M.os,'fsync',side_effect=fail),self.assertRaises(M.ClearVisible) as error:M.execute(self.c,self.state)
  self.assertEqual(error.exception.outcome,'completion-visible-durability-unconfirmed');self.assertEqual(len(calls),1);self.assertFalse((self.state/'inflight.json').exists());self.assertEqual(json.loads((self.state/'last.json').read_text())['outcome'],'completed')
  with self.pinned(),self.python('pass'):self.assertEqual(M.execute(self.c,self.state),0)
 def test_failed_unlink_remains_paused_and_no_next_admission(self):
  unlink=M.os.unlink
  def fail(name,*a,**kw):
   if name=='inflight.json':raise OSError('modeled unlink refusal')
   return unlink(name,*a,**kw)
  with self.pinned(),self.python('pass'),mock.patch.object(M.os,'unlink',side_effect=fail),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertTrue((self.state/'inflight.json').exists())
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=AssertionError('no next child')),self.assertRaises(M.Paused):M.execute(self.c,self.state)
 def test_reconcile_postclear_fsync_failure_reports_review_visible_not_paused(self):
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=OSError()),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  p,pin=self.receipt(self.review());fsync=M.os.fsync
  def fail(fd):
   if __import__('stat').S_ISDIR(os.fstat(fd).st_mode) and not (self.state/'inflight.json').exists():raise OSError('modeled directory fsync')
   return fsync(fd)
  with mock.patch.object(M.os,'fsync',side_effect=fail),self.assertRaises(M.ClearVisible) as error:M.reconcile(self.c,self.state,p,pin)
  self.assertEqual(error.exception.outcome,'reconciliation-visible-durability-unconfirmed');self.assertFalse((self.state/'inflight.json').exists());self.assertTrue((self.state/'last-reconciliation.json').exists())
 def test_strict_latch_schema_types_duplicates_nonfinite_preserved_before_reconcile(self):
  self.state.mkdir(mode=0o700);valid={'schema':'zee-sql-timer-inflight-1','source_sha':self.c['source_sha'],'binary_sha256':self.c['binary_sha256'],'created_ns':1};bad=[]
  for key,value in [('schema','wrong'),('created_ns',True),('created_ns',0),('created_ns',-1),('created_ns',2**63),('created_ns',1.5),('created_ns','1')]:x=dict(valid);x[key]=value;bad.append(json.dumps(x).encode())
  bad += [json.dumps({k:v for k,v in valid.items() if k!='created_ns'}).encode(),json.dumps(dict(valid,extra=1)).encode(),json.dumps(valid).replace('"created_ns": 1','"created_ns": NaN').encode(),json.dumps(valid).replace('"created_ns": 1','"created_ns": Infinity').encode(),json.dumps(valid).replace('"created_ns": 1','"created_ns": 1, "created_ns": 2').encode()]
  for raw in bad:
   latch=self.state/'inflight.json';latch.write_bytes(raw);latch.chmod(0o600);p,pin=self.receipt(self.review())
   with self.subTest(raw=raw),self.assertRaises(ValueError):M.reconcile(self.c,self.state,p,pin)
   self.assertEqual(latch.read_bytes(),raw);self.assertFalse((self.state/'last-reconciliation.json').exists())
  self.assertEqual(M.validate_latch(json.dumps(valid),self.c),valid)
 def test_postclear_metadata_is_fixed_and_contains_no_exception_or_paths(self):
  import io
  for operation,outcome in [('completion','completion-visible-durability-unconfirmed'),('reconciliation','reconciliation-visible-durability-unconfirmed')]:
   with contextlib.redirect_stderr(io.StringIO()) as out:M.report_clear(M.ClearVisible(operation))
   self.assertEqual(json.loads(out.getvalue()),{'schema':'zee-sql-timer-clear-publication-1','outcome':outcome});self.assertNotIn(str(self.root),out.getvalue())
 def test_actual_cli_postclear_handler_exit4_fixed_receipt_model(self):
  code="""import runpy,sys
script=sys.argv[1];operation=sys.argv[2]
def trace(frame,event,arg):
 if event=='call' and frame.f_code.co_name=='main' and frame.f_code.co_filename==script:raise frame.f_globals['ClearVisible'](operation)
 return trace
sys.argv=[script];sys.settrace(trace);runpy.run_path(script,run_name='__main__')
"""
  for operation in ('completion','reconciliation'):
   r=subprocess.run([sys.executable,'-I','-B','-c',code,str(HERE/'finite_timer.py'),operation],capture_output=True,timeout=3)
   self.assertEqual(r.returncode,4,r.stderr);self.assertEqual(r.stdout,b'');self.assertEqual(json.loads(r.stderr),{'schema':'zee-sql-timer-clear-publication-1','outcome':operation+'-visible-durability-unconfirmed'})
 def test_sql_unknown_exit3_never_reaches_postclear_fault_or_next_admission(self):
  fsync=M.os.fsync;clears=[]
  def fail(fd):
   if __import__('stat').S_ISDIR(os.fstat(fd).st_mode) and (self.state/'last.json').exists() and not (self.state/'inflight.json').exists():clears.append(True);raise OSError('must not reach')
   return fsync(fd)
  with self.pinned(),self.python('raise SystemExit(3)'),mock.patch.object(M.os,'fsync',side_effect=fail),self.assertRaises(M.Paused):M.execute(self.c,self.state)
  self.assertEqual(clears,[]);self.assertTrue((self.state/'inflight.json').exists())
  with self.pinned(),mock.patch.object(M.D,'run',side_effect=AssertionError('no next native')),self.assertRaises(M.Paused):M.execute(self.c,self.state)
 def test_unit_recipe_finite_no_restart_no_catchup_no_root_install(self):
  service,timer=M.unit_text(HERE/'finite_timer.py',self.config,self.state)
  for fixed in ('Type=oneshot','Restart=no','KillMode=mixed','TimeoutStartSec=180','TimeoutStopSec=10','NoNewPrivileges=yes','StandardError=null'):self.assertIn(fixed,service)
  for fixed in ('OnUnitInactiveSec=60s','Persistent=false','Unit=zee-sql-intent.service'):self.assertIn(fixed,timer)
  for invalid in ('unsafe space','unsafe%name','unsafe$name'):
   with self.assertRaises(ValueError):M.unit_text(self.root/invalid,self.config,self.state)
if __name__=='__main__':unittest.main()
