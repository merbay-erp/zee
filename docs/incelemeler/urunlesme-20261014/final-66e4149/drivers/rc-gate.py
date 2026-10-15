import json,re,runpy,time
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');S='66e4149e53451111d153f47283c219079a8d1cab';H=runpy.run_path(str(R/'scripts/lifecycle_core.py'))
controller=json.loads((E/'s3-rc-controller.json').read_text());assert controller['status']=='passed' and controller['owned_group_closed_and_reaped']
campaign=E/'s3-66e4149-rc';p=json.loads((campaign/'campaign-proof.json').read_text());assert p['result']=='passed-local-technical-campaign' and p['source_sha']==S
H['clean_exact'](R,S);O=E/'s3-66e4149-rc-gate';assert not O.exists();O.mkdir(mode=0o700);bundle=E/'s3-66e4149-rc-bundle';assert not bundle.exists()
env=H['controlled_env']();env.update(RUSTUP_TOOLCHAIN='1.93.1',CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true',PYTHONDONTWRITEBYTECODE='1')
proof={'schema':'zee-exact-source-post-RC-gate-1','source_sha':S,'status':'running','started_ns':time.time_ns(),'source_begin_clean':True,'commands':[]}
def run(name,argv,timeout):
 start=time.monotonic();record={'name':name,'argv':list(map(str,argv))};proof['commands'].append(record)
 try:
  raw=H['run'](argv,cwd=R,env=env,timeout=timeout,limit=16*1024**2,log_path=O/(name+'.txt'));record.update(exit=0,status='passed',seconds=time.monotonic()-start,log_sha256=H['digest'](O/(name+'.txt')));return raw.decode()
 finally:(O/'GATE-RECEIPT.json').write_text(json.dumps(proof,indent=2)+'\n')
try:
 raw=run('export',['python3','-I','-B',R/'scripts/rc-kaniti.py','export','--framework-root',R,'--campaign',campaign,'--output',bundle],300);export=json.loads(raw);manifest=export['manifest_sha256'];assert H['digest'](bundle/'manifest.json')==manifest
 raw=run('check',['python3','-I','-B',R/'scripts/rc-kaniti.py','check','--framework-root',R,'--bundle',bundle,'--sha256',manifest],300);assert json.loads(raw)['status']=='validated-integrity-and-policy'
 raw=run('security-RC',['bash',R/'scripts/guvenlik-kapisi.sh','--surum-adayi','--rc-kanit',bundle,'--rc-kanit-sha256',manifest],2400)
 body=raw.split('-- Clippy ve tam test paketi',1)[1].split('-- Yayımlanmış kanıt korukları',1)[0];totals=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',body)];assert totals and all(t[1]==0 for t in totals)
 H['clean_exact'](R,S)
 proof.update(status='passed',source_end_clean=True,source_end_sha=S,bundle_manifest_sha256=manifest,bundle_files=len(json.loads((bundle/'manifest.json').read_text())['files']),full_cargo_groups=len(totals),full_cargo_passed=sum(t[0] for t in totals),full_cargo_failed=sum(t[1] for t in totals),full_cargo_ignored=sum(t[2] for t in totals),export_check_source_exact=True)
except BaseException as error:proof.update(status='failed',failure_type=type(error).__name__);raise
finally:proof['finished_ns']=time.time_ns();(O/'GATE-RECEIPT.json').write_text(json.dumps(proof,indent=2)+'\n')
print(json.dumps({k:proof[k] for k in ['status','source_sha','bundle_manifest_sha256','full_cargo_groups','full_cargo_passed','full_cargo_failed','full_cargo_ignored']}))
