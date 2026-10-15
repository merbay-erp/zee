import argparse,copy,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--findings',type=Path,required=True);a=p.parse_args()
E=Path(__file__).resolve().parents[1];D=E/'s3-evidence-publication-stage/docs/incelemeler/urunlesme-20261014/final-66e4149';R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');S='66e4149e53451111d153f47283c219079a8d1cab';M=D/'RAW-REPRESENTATION-MAPPING.json';m=json.loads(M.read_text());f='scripts/credential-baslat.py';blob=subprocess.check_output(['git','-C',str(R),'show',S+':'+f]);h=hashlib.sha256(blob).hexdigest();assert h==hashlib.sha256((R/f).read_bytes()).hexdigest()
findings=json.loads(a.findings.read_text());group={}
for x in findings:
 p=Path(x['File']);assert p.is_relative_to(D) and x['RuleID']=='generic-api-key' and x['Match'].startswith('credential-baslat.py');group.setdefault(p,[]).append(x)
count=0
for p,rows in group.items():
 raw=p.read_bytes();v=json.loads(raw);metadata=copy.deepcopy(v);bindings=[]
 def walk(node,path):
  if isinstance(node,dict):
   for key,value in list(node.items()):
    if key in (f,'credential-baslat.py') and value==h:
     bindings.append({'original_field':path+[key],'sha256':h});node[key]={'sha256_binding':len(bindings)-1}
    else:walk(value,path+[key])
  elif isinstance(node,list):
   for i,value in enumerate(node):walk(value,path+[i])
 walk(metadata,[]);assert len(bindings)==len(rows);count+=len(bindings)
 record=next(r for r in m['rows'] if r['representation']==p.relative_to(D).as_posix());assert record['kind']=='raw-byte-copy' and hashlib.sha256(raw).hexdigest()==record['original_sha256'] and Path(record['original_path']).read_bytes()==raw
 envelope={'schema':'zee-reversible-metadata-representation-1','original_path':record['original_path'],'original_sha256':record['original_sha256'],'metadata':metadata,'bindings':bindings};restore=copy.deepcopy(metadata)
 for i,b in enumerate(bindings):
  parent=restore
  for key in b['original_field'][:-1]:parent=parent[key]
  key=b['original_field'][-1];assert parent[key]=={'sha256_binding':i};parent[key]=b['sha256']
 assert restore==v
 q=p.with_suffix('.derived.json');assert not q.exists();q.write_text(json.dumps(envelope,indent=2)+'\n');p.unlink();record.update(representation=q.relative_to(D).as_posix(),representation_sha256=hashlib.sha256(q.read_bytes()).hexdigest(),kind='reversible-parsed-JSON-envelope',reason='Default Gitleaks false positive equals exact clean S3 Git blob SHA256; inverse parsed equality asserted')
M.write_text(json.dumps(m,indent=2)+'\n');print(json.dumps({'files':len(group),'source_hash_fields':count,'inverse_equality':True,'raw_unchanged':True}))
