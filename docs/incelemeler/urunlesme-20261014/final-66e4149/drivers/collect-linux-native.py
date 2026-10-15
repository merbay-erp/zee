import hashlib,json,runpy,zipfile
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));S='66e4149e53451111d153f47283c219079a8d1cab';O=E/'s3-66e4149-linux-raw-review';assert not O.exists();O.mkdir(mode=0o700)
home=Path('/private/tmp/zk276-hih4chab/lima');i=home/'k276';d=i/'disk';assert (i.stat().st_dev,i.stat().st_ino)==(16777232,422274626) and (d.stat().st_dev,d.stat().st_ino,d.stat().st_size)==(16777232,422274644,17179869184);assert H['digest'](i/'lima.yaml')=='ac827e1be51749d62ceefb0e9cb8bdce1257a10f96e0def065ad6506e85854f9'
env=H['controlled_env']();env['LIMA_HOME']=str(home)
n=json.loads((E/'s3-66e4149-linux-native/CONTROLLER.json').read_text());leaf=n['guest_leaf']
assert leaf=='/home/mustafaerbay.guest/zee-k276/service-acceptance-66e4149e5345-1792057016809668000'
code="""import hashlib,json,os,stat,zipfile
from pathlib import Path
B=Path(LEAF);R=B/'source';S=SHA
assert R.resolve()==R
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
archive=B/'native-logs-reviewed.zip';assert not archive.exists()
entries=[];summary=[]
for directory,name,count in [('k282-native','K282',4),('k283-native-retry-manager-clean','K283',7)]:
 p=B/directory/'proof.json';v=json.loads(p.read_text());assert v['source_sha']==S and v['status']=='passed' and len(v['cases'])==count and all(c['passed'] for c in v['cases']);assert v['owned_units_released'] and v['temporary_cleanup_complete'] and v['cleanup_errors']==[]
 for n,h in v['source_pins_start'].items():assert digest(R/n)==h
 for n,h in v['postgres_binary_pins_start'].items():assert digest(Path(n))==h
 assert digest(R/'compiler/target/debug/dil')==v['binary_sha256']
 entries.append(p)
 for item in v['commands']:
  p=B/directory/item['log'];st=p.lstat();assert stat.S_ISREG(st.st_mode) and st.st_size==item['bytes'] and digest(p)==item['sha256'];entries.append(p)
 summary.append({'role':name,'commands':len(v['commands']),'cases':count,'proof_sha256':digest(B/directory/'proof.json')})
latch=B/'k282-native/unknown-state/inflight.json';assert digest(latch)=='9390eaecfc326878a771e4c18441303e130163e5ed1718353c95cb01103440c4';entries.append(latch)
assert sum(p.stat().st_size for p in entries)<16*1024**2
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED) as z:
 for p in entries:z.write(p,p.relative_to(B).as_posix())
print(json.dumps({'status':'passed','source_sha':S,'roles':summary,'files':[{'path':p.relative_to(B).as_posix(),'sha256':digest(p),'bytes':p.stat().st_size} for p in entries],'archive_sha256':digest(archive),'latch_preserved':True}))
""".replace('LEAF',repr(leaf)).replace('SHA',repr(S))
raw=H['run'](['/opt/homebrew/bin/limactl','shell','--workdir=/tmp','k276','--','python3','-I','-B','-c',code],env=env,timeout=60,limit=1024*1024,log_path=O/'guest-review.log')
v=json.loads(raw);assert v['status']=='passed' and v['source_sha']==S
H['run'](['/opt/homebrew/bin/limactl','copy','k276:'+leaf+'/native-logs-reviewed.zip',O/'native-logs-reviewed.zip'],env=env,timeout=60,limit=1024*1024,log_path=O/'copy.log')
assert H['digest'](O/'native-logs-reviewed.zip')==v['archive_sha256']
with zipfile.ZipFile(O/'native-logs-reviewed.zip') as z:
 assert set(z.namelist())=={f['path'] for f in v['files']}
 for item in v['files']:
  p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts
  data=z.read(item['path']);assert len(data)==item['bytes'] and hashlib.sha256(data).hexdigest()==item['sha256']
  out=O/'files'/p;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
H['clean_exact'](R,S)
(O/'LINUX-RAW-REVIEW.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps({'status':'passed','files':len(v['files']),'roles':v['roles']}))
