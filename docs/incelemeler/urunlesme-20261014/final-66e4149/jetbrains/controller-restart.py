from pathlib import Path
import os,subprocess,time,json,signal,runpy,hashlib,sys
sys.dont_write_bytecode=True
O=Path(__file__).resolve().parent;R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');helper=R/'scripts/lifecycle_core.py'
assert hashlib.sha256(helper.read_bytes()).hexdigest()=='e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9'
H=runpy.run_path(str(helper));prep=json.loads((O/'PREPARATION.json').read_text());stop=False
for name in ['config','system','logs','tmp']:assert (O/name).is_dir() and not (O/name).is_symlink()
def cancel(*_):
 global stop
 stop=True
signal.signal(signal.SIGTERM,cancel);signal.signal(signal.SIGINT,cancel)
env={k:v for k,v in os.environ.items() if k in {'HOME','USER','LOGNAME','LANG','LC_CTYPE'}};env.update(PATH='/usr/bin:/bin',IDEA_PROPERTIES=str(O/'idea.properties'),IDEA_VM_OPTIONS=str(O/'idea.vmoptions'),TMPDIR=str(O/'tmp'))
record={'schema':'zee-owned-IDE-k280-1','actual_launcher_source':prep['actual_launcher_source'],'status':'starting','profile':str(O),'canonical_command_unchanged':True,'acceptance':False};child=None;t=time.monotonic()
def save():
 temp=O/'restart-LIVE.tmp';temp.write_text(json.dumps(record,indent=2)+'\n');os.replace(temp,O/'restart-LIVE.json')
save()
with (O/'restart-stdout.log').open('xb') as output:
 try:
  child=subprocess.Popen(['/Applications/IntelliJ IDEA.app/Contents/MacOS/idea',prep['project']],cwd=O,env=env,stdin=subprocess.DEVNULL,stdout=output,stderr=subprocess.STDOUT,start_new_session=True,close_fds=True);record.update(status='owned-process-running',issued_child_pid=child.pid);save()
  while not stop and not (O/'restart-close.request').exists():
   observed=H['held_status'](child)
   if observed is not None:record.update(status='owned-process-exited',observed_exit=observed);break
   assert time.monotonic()-t<2700,'trial deadline'
   assert (O/'restart-stdout.log').stat().st_size<=64*1024**2
   record['elapsed_seconds']=round(time.monotonic()-t,3);save();time.sleep(1)
 finally:
  if child is not None:
   code=H['stop_owned'](child,grace=15);record.update(owned_group_closed_and_reaped=True,exit=code)
  record.update(status='closed-owned-process',elapsed_seconds=round(time.monotonic()-t,3));save();(O/'restart-CLOSED.json').write_text(json.dumps(record,indent=2)+'\n')
