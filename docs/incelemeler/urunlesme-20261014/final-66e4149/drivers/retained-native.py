import hashlib,json,runpy,shutil,time
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');S='66e4149e53451111d153f47283c219079a8d1cab';H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));O=E/'s3-66e4149-retained-native';assert not O.exists();O.mkdir(mode=0o700);(O/'logs').mkdir()
B=E/'s3-66e4149-macos-delivery/binaries/dil';pin='398a706f588fba06b5cadacb32af01714c7dd875a4c40a86b35137a1c7227e87';PG=Path('/private/tmp/zee-plan-evidence/pg16-runtime/install/bin');PDF=Path('/Users/mustafaerbay/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3');P='python3'
env=H['controlled_env']();env.update(RUSTUP_TOOLCHAIN='1.93.1',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='1',PYTHONDONTWRITEBYTECODE='1')
proof={'schema':'zee-retained-native-exact-source-1','source_sha':S,'binary_sha256':pin,'status':'running','steps':[],'started_ns':time.time_ns(),'boundary':'Explicit retained dedicated cohorts, not counted again in unique libtest total.'}
def save():(O/'PROOF.json').write_text(json.dumps(proof,indent=2)+'\n')
def run(name,args,cwd=R,timeout=660):
 H['clean_exact'](R,S);assert H['digest'](B)==pin
 before={str(Path(x).relative_to(R)):H['digest'](Path(x)) for x in args if str(x).startswith(str(R)+'/') and Path(x).is_file()}
 start=time.monotonic();log=O/'logs'/(name+'.log');record={'name':name,'argv':list(map(str,args)),'source_pins':before};proof['steps'].append(record)
 try:
  raw=H['run'](args,cwd=cwd,env=env,timeout=timeout,limit=8*1024**2,log_path=log)
  H['clean_exact'](R,S);assert H['digest'](B)==pin and all(H['digest'](R/n)==h for n,h in before.items())
  record.update(status='passed',exit=0,seconds=time.monotonic()-start,log_sha256=H['digest'](log));save();print(name+' passed',flush=True);return raw.decode()
 except BaseException:record.update(status='failed',seconds=time.monotonic()-start);save();raise
def command(name,script,extra=(),timeout=660):
 return run(name,[P,'-I','-B',R/'scripts'/script,*extra],timeout=timeout)
try:
 H['clean_exact'](R,S);assert H['digest'](B)==pin
 identity=json.loads(run('build-before',[B,'sürüm','--json'],timeout=20));assert identity['source_sha']==S and identity['dirty'] is False;proof['build']=identity
 command('pg8','postgresql-tls-ci.py',['--engine','local','--postgres-bin-dir',PG,'--output',O/'postgresql-tls-ci.json'])
 v=json.loads((O/'postgresql-tls-ci.json').read_text());assert v['source_sha']==S and v['source_clean'] and len(v['tests'])==8 and v['exit']==0
 command('goc1','postgresql-goc-ci.py',['--engine','local','--postgres-bin-dir',PG])
 for n in ['postgresql-goc-ci.json','postgresql-goc-ci.log']:shutil.copyfile(R/'compiler/target'/n,O/n)
 v=json.loads((O/'postgresql-goc-ci.json').read_text());assert v['source_sha']==S and v['source_clean'] and v['exit']==0
 for name,script,count in [('parola11','parola-goc-ci.py',11),('queue21','is-kuyrugu-ci.py',21)]:
  command(name,script,['--engine','local','--postgres-bin-dir',PG,'--dil',B,'--output',O/(name+'.json')])
  v=json.loads((O/(name+'.json')).read_text());assert v['status']=='passed' and len(v['cases'])==count and v['ignored']==0
 for name,script,extra in [('metrics8','native-metrics-test.py',['--framework-root',R,'--postgres-bin-dir',PG]),('job-native11','job-dispatcher-native-test.py',['--framework-root',R,'--postgres-bin-dir',PG]),('smtp-pg-fault11','effect-dispatcher-native-test.py',['--postgres-bin-dir',PG]),('readiness-watch-pg15','db-readiness-watch-native.py',['--framework-root',R,'--postgres-bin-dir',PG]),('readiness-ttl5','readiness-ttl-native.py',[])]:
  command(name,'tests/'+script,['--dil',B,'--output',O/name,*extra])
  v=json.loads((O/name/'proof.json').read_text());assert v['status']=='passed'
 suite=R/'compiler/tests/fixtures/owned-web-kit-suite.json'
 command('owned-web-pg4','owned-web-tests.py',['--fixture-root',R/'compiler/tests/fixtures/owned-web-kit','--suite',suite,'--suite-sha256',H['digest'](suite),'--dil',B,'--binary-sha256',pin,'--expected-source',S,'--postgres-bin-dir',PG,'--output',O/'owned-web-pg4'])
 v=json.loads((O/'owned-web-pg4/proof.json').read_text());assert v['status']=='passed' and len(v['cases'])==4
 raw=run('pdf-native-job',[B,'çalıştır',R/'golden/pdf-host/ana.dil'],timeout=30);assert json.loads(raw)==json.loads((R/'golden/pdf-host/job.json').read_text())
 raw=run('pdf-backend8',[PDF,'-I','-B',R/'scripts/tests/pdf-backend-acceptance.py','--framework-root',R,'--output',O/'pdf-backend8'],timeout=150);assert 'Ran 8 tests' in raw and '\nOK' in raw
 assert json.loads(run('build-after',[B,'sürüm','--json'],timeout=20))==identity
 H['clean_exact'](R,S);proof['status']='passed';proof['source_end_clean']=True;proof['binary_end_unchanged']=H['digest'](B)==pin
except BaseException as e:proof.update(status='failed',failure_type=type(e).__name__);raise
finally:proof['finished_ns']=time.time_ns();save()
