import json,hashlib
from pathlib import Path
E=Path(__file__).resolve().parents[1];B=E/'s3-66e4149-retained-native';S='66e4149e53451111d153f47283c219079a8d1cab';v=json.loads((B/'PROOF.json').read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
expected={'metrics8':8,'job-native11':11,'smtp-pg-fault11':11,'readiness-watch-pg15':15,'readiness-ttl5':5,'owned-web-pg4':4};reviews={}
for name,count in expected.items():
 p=B/name/'proof.json';q=json.loads(p.read_text());assert q['status']=='passed' and len(q['cases'])==count
 if name=='readiness-ttl5':assert all(isinstance(c,str) and c for c in q['cases']) and len(set(q['cases']))==5
 else:assert all(c.get('passed') is True or c.get('status')=='passed' for c in q['cases'])
 for key in ['owned_workers_closed','owned_pg_closed','owned_cluster_closed','owned_cleanup','owned_groups_closed','command_groups_empty']:
  if key in q:assert q[key] is True
 for key in ['build','compiler_identity','build_identity']:
  if key in q:assert q[key]['source_sha']==S and q[key]['dirty'] is False
 reviews[name]={'proof_sha256':sha(p),'cases':count,'status':'passed'}
for step in v['steps']:assert step['status']=='passed' and sha(B/'logs'/(step['name']+'.log'))==step['log_sha256']
for name in ['postgresql-tls-ci','postgresql-goc-ci']:
 q=json.loads((B/(name+'.json')).read_text());assert q['source_sha']==S and q['source_clean'] and q['exit']==0
pg=json.loads((E/'pg16-runtime/build-proof.json').read_text())
for n,h in pg['binaries_sha256'].items():assert sha(Path(pg['prefix'])/'bin'/n)==h
out={'schema':'zee-retained-native-independent-review-1','status':'passed','source_sha':S,'proof_sha256':sha(B/'PROOF.json'),'steps':len(v['steps']),'native_cohorts':reviews,'postgres_build_proof_sha256':sha(E/'pg16-runtime/build-proof.json'),'postgres_binaries_end_match_original_build':True,'raw_command_hashes_verified':True,'actual_original_controller_closed':json.loads((E/'s3-retained-native-controller.json').read_text())['owned_group_closed_and_reaped'],'case_accounting':'Internal native assertions and Cargo ignored owners are separate; not a unique test total.'}
p=E/'s3-acceptance-drivers/RETAINED-NATIVE-REVIEW.json';assert not p.exists();p.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
