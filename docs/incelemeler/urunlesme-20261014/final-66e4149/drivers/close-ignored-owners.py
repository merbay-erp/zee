import json,hashlib
from pathlib import Path
E=Path(__file__).resolve().parents[1];S='66e4149e53451111d153f47283c219079a8d1cab';O=E/'s3-66e4149-ignored-inventory';v=json.loads((O/'INVENTORY.json').read_text());assert v['source_sha']==S and v['actual_ignored_cases']==25 and v['executed_owner_cases']==24
paths={
'metrics8':'s3-66e4149-retained-native/metrics8/proof.json',
'job-native11':'s3-66e4149-retained-native/job-native11/proof.json',
'smtp-pg-fault277':'s3-66e4149-retained-native/smtp-pg-fault11/proof.json',
'owned-web-pg4':'s3-66e4149-retained-native/owned-web-pg4/proof.json',
'pdf-backend8':'s3-66e4149-retained-native/PROOF.json',
'goc1':'s3-66e4149-retained-native/postgresql-goc-ci.json',
'pg8':'s3-66e4149-retained-native/postgresql-tls-ci.json',
'readiness-watch-pg15':'s3-66e4149-retained-native/readiness-watch-pg15/proof.json',
'mac-native/mixed-revision9':'s3-66e4149-mac-native/mixed-revision/proof.json',
'mac-native/content-starter14':'s3-66e4149-mac-native/content-starter/proof.json',
'mac-native/web-bridge4':'s3-66e4149-mac-native/web-bridge/results/proof.json',
'mac-native/module-sql9':'s3-66e4149-mac-native/module-sql/proof.json',
'mac-native/signed-pdf9':'s3-66e4149-mac-native/signed-pdf/proof.json',
'mac-native/service-fault2':'s3-66e4149-mac-native/service-fault/proof.json',
'linux/dispatcher4':'s3-66e4149-linux-native/K282-proof.json',
'linux/http7':'s3-66e4149-linux-http-retry/K283-proof.json',
'pending-release-measurement':'s3-66e4149-lsp-measurement/MEASUREMENT.json'}
rows=[]
for x in v['rows']:
 p=E/paths[x['covered_by']];q=json.loads(p.read_text())
 if x['covered_by']=='pending-release-measurement':
  assert q['source_sha']==S and q['source_begin_end_clean'] and q['p95_independently_recomputed'] and [m['open_documents'] for m in q['measurements']]==[1,41]
 elif x['covered_by'] in ['goc1','pg8']:assert q['source_sha']==S and q['source_clean'] and q['exit']==0
 else:assert q['status']=='passed'
 rows.append({**x,'covered_by':'release-LSP1/41-observation' if x['covered_by']=='pending-release-measurement' else x['covered_by'],'executed':True,'evidence_path':str(p),'evidence_sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
proof={'schema':'zee-exact-source-ignored-owner-review-1','status':'passed','source_sha':S,'inventory_sha256':hashlib.sha256((O/'INVENTORY.json').read_bytes()).hexdigest(),'actual_test_binaries':v['actual_test_binaries'],'actual_ignored_cases':25,'all_25_owners_executed':True,'rows':rows,'accounting':'Each actual ignored libtest case has one executed primary native/measurement owner; cohort internal scenarios are not added as unique Cargo tests. Two metrics owners intentionally point at one metrics8 cohort.'}
p=O/'OWNER-REVIEW.json';assert not p.exists();p.write_text(json.dumps(proof,indent=2)+'\n');print('25 actual ignored cases bound to executed source-S3 owners')
