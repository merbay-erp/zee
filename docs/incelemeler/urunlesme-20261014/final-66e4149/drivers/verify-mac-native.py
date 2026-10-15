import hashlib,json,subprocess
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');B=E/'s3-66e4149-mac-native';D=E/'s3-66e4149-macos-delivery';S='66e4149e53451111d153f47283c219079a8d1cab'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def clean():
 assert subprocess.check_output(['git','-C',str(R),'rev-parse','HEAD']).decode().strip()==S
 assert not subprocess.check_output(['git','-C',str(R),'status','--porcelain','--untracked-files=all'])
clean();d=json.loads((D/'DELIVERY.json').read_text());assert d['status']=='passed' and d['source_sha']==S and d['two_source_zip_equal'] and d['restoration_fsck']
checked=[]
for f in d['files']:
 assert sha(D/f['path'])==f['sha256'];checked.append(str((D/f['path']).relative_to(E)))
root=json.loads((B/'PROOF.json').read_text());assert root['status']=='passed' and root['source_sha']==S and root['build']['dirty'] is False and root['delivery_sha256']==sha(D/'DELIVERY.json')
binary=d['binary_sha256']['dil'];assert root['binary_sha256']==binary
assert all(step['exit']==0 for step in root['steps'])
paths={n:B/n/'proof.json' for n in ['composed-tr-en','signed-pdf','module-sql','content-starter','source-coverage','bridge-negatives','mixed-revision','service-fault','native-metric-stdio']}
paths['web-bridge']=B/'web-bridge/results/proof.json'
summaries={}
for name,p in paths.items():
 v=json.loads(p.read_text());assert sha(p)==root[name]['proof_sha256']
 if 'build' in v:assert v['build']['source_sha']==S and v['build']['dirty'] is False
 if 'binary_sha256' in v:assert v['binary_sha256']==binary
 assert all(c['passed'] for c in v['cases']) if name!='bridge-negatives' else v['status']=='passed' and len(v['cases'])==6
 for key in ['owned_groups_closed','owned_handles_reaped','source_pins_unchanged','owned_cleanup','foreign_survived','source_pins_end_equal','binary_end_equal','source_end_unchanged','binary_end_unchanged','temporary_private_root_removed','private_root_removed','synthetic_private_keys_removed','owned_native_generations_reaped','foreign_owned_sentinel_reaped','command_groups_empty']:
  if key in v:assert v[key] is True,(name,key)
 if 'cleanup_errors' in v:assert v['cleanup_errors']==[]
 for key in ['sources','source_pins_start']:
  if key in v:
   for n,h in v[key].items():assert sha(R/n)==h,(name,n)
 for key in ['bindings','source_files']:
  if key in v:
   for item in v[key]:assert sha(R/item['path'])==item['sha256'],(name,item['path'])
 for key in ['executables','postgres_binary_pins','postgres_pins']:
  if key in v:
   for n,h in v[key].items():assert sha(Path(n) if Path(n).is_absolute() else Path('/private/tmp/zee-plan-evidence/pg16-runtime/install/bin')/n)==h,(name,n)
 summaries[name]={'proof_sha256':sha(p),'cases':len(v['cases']),'producer_status':v.get('status',v.get('result'))}
 checked.append(str(p.relative_to(E)))
v=json.loads(paths['source-coverage'].read_text());assert v['candidate_only'] is True and v['status']=='passed-short-candidate'
for name,hit in [('dene',4),('native',6)]:
 report=json.loads((B/'source-coverage'/(name+'.json')).read_text());assert report['complete'] and report['execution_success'];assert len(report['edges'])==report['product']['payda']==14
 assert sum(e['vurus']>0 for e in report['edges'])==hit==report['product']['vurulan'];assert len(report['product']['eksik'])==14-hit
 if name=='native':assert all(e['vurus']==0 for e in report['edges'] if e['sahip']=='henuzSinanmayan')
v=json.loads(paths['web-bridge'].read_text());assert sum(c['requests'] for c in v['cases'])==19
clusters=[c['cluster_system_identifier'] for c in v['cases'] if c['cluster_system_identifier']];assert len(clusters)==len(set(clusters))==2
assert all(c['worker_groups_empty'] and c['pg_groups_empty'] for c in v['cleanup'])
for c in v['cases']:assert sha(Path(c['source_coverage']['path']))==c['source_coverage']['sha256']
v=json.loads(paths['native-metric-stdio'].read_text());assert len(v['cases'])==26 and sha(R/'regression/cli/native-metric-stdio-alias.dil')==v['fixture_source_sha256'];assert all(c['control_bytes']==0 for c in v['cases'] if 'control_bytes' in c)
for dirname in ['s3-66e4149-dependency-probe','s3-66e4149-budget-probe']:
 p=E/dirname/'EXACT-SOURCE.json';v=json.loads(p.read_text());assert v['source_sha']==S
 checked.append(str(p.relative_to(E)))
clean()
out={'schema':'zee-root-independent-native-review-1','status':'passed','source_sha':S,'canonical_delivery_files_checked':len(d['files']),'proofs':summaries,'checked_files':checked,'scope':'Second-wave exact-S3 native cohorts and canonical macOS delivery. Producer source-coverage candidate flag preserved; clean source and canonical binary independently bound. No physical-host, live-provider or product adoption claim.'}
p=E/'s3-acceptance-drivers/MAC-NATIVE-REVIEW.json';assert not p.exists();p.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
