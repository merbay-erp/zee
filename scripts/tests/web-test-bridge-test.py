"""K286 CLI negatives; K305 cancellation/cleanup preparation. No PG or HTTP is started."""
import argparse,hashlib,importlib.util,json,os,selectors,signal,time
from pathlib import Path
from types import SimpleNamespace

CANCELLED=False

def cancel(number,frame):
 global CANCELLED
 CANCELLED=True

def check():
 if CANCELLED:raise RuntimeError('cancelled')

def main():
 global CANCELLED
 CANCELLED=False
 p=argparse.ArgumentParser(allow_abbrev=False)
 for n in ('framework-root','dil','postgres-bin-dir','output'):p.add_argument('--'+n,type=Path,required=True)
 a=p.parse_args();root=a.framework_root.resolve();out=a.output;out.mkdir(mode=0o700)
 o=None;previous={};cases=[];completed=False
 proof={'schema':'zee-web-test-bridge-negatives-1','status':'running','cases':cases,'signal_handlers_installed':False}
 suite=root/'compiler/tests/fixtures/owned-web-kit-suite.json';fixture=root/'compiler/tests/fixtures/owned-web-kit';normal=root/'dogfood/zee-cekirdek/test-kapsami'
 def arguments(output):return [str(a.dil),'dene','--web-suite',str(suite),'--web-suite-sha256',hashlib.sha256(suite.read_bytes()).hexdigest(),'--web-fixtures',str(fixture),'--test-python',str(Path(os.sys.executable).resolve()),'--test-postgres-bin',str(a.postgres_bin_dir),'--test-output',str(output),str(normal)]
 def run(argv):
  child=None;s=None;b=bytearray();end=time.monotonic()+10
  try:
   check();child=o.spawn(argv,capture=True);check()
   s=selectors.DefaultSelector();s.register(child.stdout,selectors.EVENT_READ)
   while s.get_map():
    check();assert time.monotonic()<end
    for key,_ in s.select(.05):
     check();block=os.read(key.fd,4096)
     if not block:s.unregister(key.fileobj)
     else:b.extend(block);assert len(b)<=65536
   status=o.exit_status(child)
   while status is None:check();assert time.monotonic()<end;time.sleep(.01);status=o.exit_status(child)
   check();assert time.monotonic()<end
   result=status,bytes(b)
  finally:
   try:
    if s is not None:s.close()
   finally:
    if child is not None:o.stop_child(child)
  check();return result
 try:
  # The handler only sets a flag: spawn can retain its original Popen first.
  for number in (signal.SIGTERM,signal.SIGINT):previous[number]=signal.signal(number,cancel)
  proof['signal_handlers_installed']=True
  check()
  spec=importlib.util.spec_from_file_location('bridge_owner',root/'scripts/worker-supervisor.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
  o=M.Supervisor(SimpleNamespace(project=out,stop_grace=35),None);check()
  # An incomplete operator profile must never run the unit source or create output.
  s,b=run([str(a.dil),'dene','--web-suite',str(suite),str(normal)]);assert s==2 and b==b'';cases.append('all_or_none')
  args=arguments(out/'wronghash');args[args.index('--web-suite-sha256')+1]='0'*64;s,b=run(args);assert s==2 and b==b'' and not (out/'wronghash').exists();cases.append('suite_exact_hash_before_units')
  fifo=out/'suite.fifo';os.mkfifo(fifo);args=arguments(out/'fifo-output');args[args.index('--web-suite')+1]=str(fifo);s,b=run(args);assert s==2 and b==b'' and not (out/'fifo-output').exists();cases.append('suite_fifo_no_open')
  args=arguments(out/'duplicate');args[2:2]=['--web-suite',str(suite)];s,b=run(args);assert s==2 and b==b'' and not (out/'duplicate').exists();cases.append('duplicate_role_rejected')
  bad=out/'failed.dil';bad.write_text('test "mandatory unit fails"\n    1 2 ye eşit olmalı\n');args=arguments(out/'failed-unit');args[-1]=str(bad);s,b=run(args);assert s==1;receipt=json.loads((out/'failed-unit/unit.json').read_bytes());assert receipt['passed'] is False and receipt['native_started'] is False and not (out/'failed-unit/runtime').exists() and not (out/'failed-unit/results').exists();cases.append('unit_failure_never_native')
  args=arguments(out/'unsafe-exec');args[args.index('--test-python')+1]='/bin/sh';s,b=run(args);assert s==2 and b==b'' and not (out/'unsafe-exec').exists();cases.append('operator_python_role_no_shell')
  completed=True
 except BaseException as e:proof.update(status='failed-preserved',failure_type=type(e).__name__)
 finally:
  proofpath=out/'proof.json'
  try:
   try:
    if o is not None:o.cleanup()
   except BaseException as e:proof.update(status='failed-preserved',cleanup_failure_type=type(e).__name__)
   proof['owned_groups_closed']=o is None or not o.children
   proof['cancelled']=CANCELLED
   if CANCELLED:proof.update(status='failed-preserved',failure_type='Cancelled')
   if completed and proof['status']=='running' and proof['owned_groups_closed']:proof['status']='passed'
   elif proof['status']=='running':proof['status']='failed-preserved'
   proofpath.write_text(json.dumps(proof,indent=2)+'\n')
   # Keep handlers active through publication and the final observed check.
   try:check()
   except BaseException as e:
    proof.update(status='failed-preserved',cancelled=CANCELLED,failure_type=type(e).__name__)
    proofpath.write_text(json.dumps(proof,indent=2)+'\n')
  finally:
   restore_failed=False
   for number,handler in previous.items():
    try:signal.signal(number,handler)
    except BaseException as e:
     restore_failed=True;proof.update(status='failed-preserved',handler_failure_type=type(e).__name__)
   if restore_failed or (CANCELLED and proof['status']=='passed'):
    proof.update(status='failed-preserved',cancelled=CANCELLED)
    proofpath.write_text(json.dumps(proof,indent=2)+'\n')
 return 0 if proof['status']=='passed' else 2
if __name__=='__main__':raise SystemExit(main())
