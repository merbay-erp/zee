"""K286 actual CLI/pre-secret admission negatives; no PG or HTTP is started."""
import argparse,hashlib,importlib.util,json,os,selectors,tempfile,time
from pathlib import Path
from types import SimpleNamespace

def main():
 p=argparse.ArgumentParser(allow_abbrev=False)
 for n in ('framework-root','dil','postgres-bin-dir','output'):p.add_argument('--'+n,type=Path,required=True)
 a=p.parse_args();root=a.framework_root.resolve();out=a.output;out.mkdir(mode=0o700)
 spec=importlib.util.spec_from_file_location('bridge_owner',root/'scripts/worker-supervisor.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
 o=M.Supervisor(SimpleNamespace(project=out,stop_grace=35),None);cases=[]
 suite=root/'compiler/tests/fixtures/owned-web-kit-suite.json';fixture=root/'compiler/tests/fixtures/owned-web-kit';normal=root/'dogfood/zee-cekirdek/test-kapsami'
 def arguments(output):return [str(a.dil),'dene','--web-suite',str(suite),'--web-suite-sha256',hashlib.sha256(suite.read_bytes()).hexdigest(),'--web-fixtures',str(fixture),'--test-python',str(Path(os.sys.executable).resolve()),'--test-postgres-bin',str(a.postgres_bin_dir),'--test-output',str(output),str(normal)]
 def run(argv):
  child=o.spawn(argv,capture=True);s=selectors.DefaultSelector();s.register(child.stdout,selectors.EVENT_READ);b=bytearray();end=time.monotonic()+10
  try:
   while s.get_map():
    assert time.monotonic()<end
    for key,_ in s.select(.05):
     block=os.read(key.fd,4096)
     if not block:s.unregister(key.fileobj)
     else:b.extend(block);assert len(b)<=65536
   status=o.exit_status(child)
   while status is None:assert time.monotonic()<end;time.sleep(.01);status=o.exit_status(child)
   return status,bytes(b)
  finally:s.close();o.stop_child(child)
 proof={'schema':'zee-web-test-bridge-negatives-1','status':'running','cases':cases}
 try:
  # An incomplete operator profile must never run the unit source or create output.
  s,b=run([str(a.dil),'dene','--web-suite',str(suite),str(normal)]);assert s==2 and b==b'';cases.append('all_or_none')
  args=arguments(out/'wronghash');args[args.index('--web-suite-sha256')+1]='0'*64;s,b=run(args);assert s==2 and b==b'' and not (out/'wronghash').exists();cases.append('suite_exact_hash_before_units')
  fifo=out/'suite.fifo';os.mkfifo(fifo);args=arguments(out/'fifo-output');args[args.index('--web-suite')+1]=str(fifo);s,b=run(args);assert s==2 and b==b'' and not (out/'fifo-output').exists();cases.append('suite_fifo_no_open')
  args=arguments(out/'duplicate');args[2:2]=['--web-suite',str(suite)];s,b=run(args);assert s==2 and b==b'' and not (out/'duplicate').exists();cases.append('duplicate_role_rejected')
  bad=out/'failed.dil';bad.write_text('test "mandatory unit fails"\n    1 2 ye eşit olmalı\n');args=arguments(out/'failed-unit');args[-1]=str(bad);s,b=run(args);assert s==1;receipt=json.loads((out/'failed-unit/unit.json').read_bytes());assert receipt['passed'] is False and receipt['native_started'] is False and not (out/'failed-unit/runtime').exists() and not (out/'failed-unit/results').exists();cases.append('unit_failure_never_native')
  args=arguments(out/'unsafe-exec');args[args.index('--test-python')+1]='/bin/sh';s,b=run(args);assert s==2 and b==b'' and not (out/'unsafe-exec').exists();cases.append('operator_python_role_no_shell')
  proof['status']='passed'
 except BaseException as e:proof.update(status='failed-preserved',failure_type=type(e).__name__)
 finally:
  o.cleanup();proof['owned_groups_closed']=not o.children;(out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
 return 0 if proof['status']=='passed' and proof['owned_groups_closed'] else 2
if __name__=='__main__':raise SystemExit(main())
