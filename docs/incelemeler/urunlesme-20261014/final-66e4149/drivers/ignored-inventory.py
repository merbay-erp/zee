import json,runpy
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));S='66e4149e53451111d153f47283c219079a8d1cab';H['clean_exact'](R,S);O=E/'s3-66e4149-ignored-inventory';assert not O.exists();O.mkdir(mode=0o700)
env=H['controlled_env']();env.update(RUSTUP_TOOLCHAIN='1.93.1',CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='1')
raw=H['run'](['cargo','test','--locked','--no-run','--message-format=json'],cwd=R/'compiler',env=env,timeout=300,limit=4*1024**2,log_path=O/'cargo-artifacts.jsonl').decode()
artifacts={}
for line in raw.splitlines():
 try:v=json.loads(line)
 except ValueError:continue
 if v.get('reason')=='compiler-artifact' and v.get('profile',{}).get('test') and v.get('executable'):
  p=Path(v['executable']).resolve(strict=True);assert p.is_relative_to(R/'compiler/target');artifacts[v['target']['kind'][0]+':'+v['target']['name']]=p
assert artifacts
old=json.loads((E/'final-S-8b491-common-inventory/ROOT-APPROVED-IGNORED-CONTRACT.json').read_text());owners={(x['target'],x['test']):x['covered_by'] for x in old['entries']};assert len(owners)==16
new={
 'test:mixed_revision_testi':'mac-native/mixed-revision9',
 'test:starter_content_testi':'mac-native/content-starter14',
 'test:web_test_koprusu_testi':'mac-native/web-bridge4',
 'test:lsp_artimli_testi':'pending-release-measurement',
 'test:modul_sql_testi':'mac-native/module-sql9',
 'test:dispatcher_service_testi':'linux/dispatcher4',
 'test:pdf_plugin_testi':'mac-native/signed-pdf9',
 'test:service_fault_testi':'mac-native/service-fault2',
 'test:http_service_testi':'linux/http7'}
rows=[];binary_rows=[]
for index,(target,p) in enumerate(sorted(artifacts.items())):
 raw=H['run']([p,'--ignored','--list','--format','terse'],cwd=R/'compiler',env=env,timeout=20,limit=65536,log_path=O/('binary-'+str(index)+'.txt')).decode()
 names=[line[:-6] for line in raw.splitlines() if line.endswith(': test')];assert len(names)==len(set(names));binary_rows.append({'target':target,'binary_sha256':H['digest'](p),'ignored_tests':names})
 for name in names:
  key=(target,name)
  if key in owners:owner=owners[key]
  else:assert target in new and len(names)==1;owner=new[target]
  rows.append({'target':target,'test':name,'covered_by':owner,'executed':owner!='pending-release-measurement'})
assert len(rows)==25 and len([x for x in rows if not x['executed']])==1
assert set(owners)<={(x['target'],x['test']) for x in rows}
H['clean_exact'](R,S)
proof={'schema':'zee-exact-source-ignored-owner-inventory-1','source_sha':S,'status':'inventory-verified-measurement-pending','actual_test_binaries':len(artifacts),'actual_ignored_cases':len(rows),'executed_owner_cases':24,'rows':rows,'binary_inventory':binary_rows,'old_contract_sha256':H['digest'](E/'final-S-8b491-common-inventory/ROOT-APPROVED-IGNORED-CONTRACT.json'),'case_accounting':'Actual libtest cases map to dedicated native cohorts; their internal assertions are not additional libtest cases.'}
(O/'INVENTORY.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps({'binaries':len(artifacts),'ignored':len(rows),'native_owners_executed':24,'pending':'release measurement only'}))
