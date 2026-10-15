import json,runpy,time
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));S='66e4149e53451111d153f47283c219079a8d1cab';H['clean_exact'](R,S)
D=E/'s3-66e4149-linux-delivery-unzip-path';v=json.loads((D/'DELIVERY.json').read_text());c=json.loads((D/'CONTROLLER.json').read_text());assert v['status']==c['status']=='passed' and v['source_sha']==S
comparison=json.loads((E/'s3-66e4149-source-zip-comparison/COMPARISON.json').read_text());assert comparison['status']=='passed' and comparison['source_sha']==S and comparison['all_entry_names_and_bytes_equal'] and comparison['both_canonical_zip_manifest_and_Git_snapshot_verified'] and comparison['linux_zip_sha256']==v['source_zip_sha256']
leaf=c['guest_leaf'];assert leaf=='/home/mustafaerbay.guest/zee-k276/service-acceptance-66e4149e5345-1792057016809668000'
home=Path('/private/tmp/zk276-hih4chab/lima');i=home/'k276';d=i/'disk';assert (i.stat().st_dev,i.stat().st_ino)==(16777232,422274626) and (d.stat().st_dev,d.stat().st_ino,d.stat().st_size)==(16777232,422274644,17179869184);assert H['digest'](i/'lima.yaml')=='ac827e1be51749d62ceefb0e9cb8bdce1257a10f96e0def065ad6506e85854f9'
O=E/'s3-66e4149-linux-install-source-content';assert not O.exists();O.mkdir(mode=0o700)
env=H['controlled_env']();env['LIMA_HOME']=str(home)
def cmd(args,label,timeout=30):return H['run'](args,env=env,timeout=timeout,limit=2*1024**2,log_path=O/(label+'.log'))
def remote(code,label,timeout=30):return cmd(['/opt/homebrew/bin/limactl','shell','--workdir=/tmp','k276','--','python3','-I','-B','-c',code],label,timeout)
receipt={'source_sha':S,'status':'running','guest_leaf':leaf,'started_ns':time.time_ns()}
try:
 remote("from pathlib import Path; p=Path("+repr(leaf)+")/'install-rpc-params-content'; assert not p.exists();p.mkdir(mode=0o700)",'prepare')
 T=E/'s3-acceptance-drivers/install-rpc-params';pins=json.loads((T/'ADAPTATION.json').read_text())['files']
 for n,h in pins.items():
  assert H['digest'](T/n)==h
  cmd(['/opt/homebrew/bin/limactl','copy',T/n,'k276:'+leaf+'/install-rpc-params-content/'+n],'copy-'+n,60)
 code="""from pathlib import Path
import runpy,json
B=Path(LEAF);R=Path('/home/mustafaerbay.guest/zee-k276');H=runpy.run_path(str(B/'source/scripts/lifecycle_core.py'));E=H['controlled_env']();E.update(PATH=str(R/'tools/git-2.56.0/bin')+':'+str(R/'tools/unzip-owned/usr/bin')+':'+str(R/'ru-cargo/bin')+':/usr/bin:/bin:/usr/sbin:/sbin',RUSTUP_HOME=str(R/'ruhome'),CARGO_HOME=str(R/'cargo-home'),RUSTUP_TOOLCHAIN='1.93.1',CARGO_NET_OFFLINE='true')
D=B/'linux-delivery-unzip-path';assert H['digest'](D/'DELIVERY.json')==REPORT
for n,h in PINS.items():assert H['digest'](B/'install-rpc-params-content'/n)==h
v=json.loads((D/'DELIVERY.json').read_text());assert v['source_sha']==SOURCE and v['status']=='passed' and v['source_zip_sha256']==ZIP_HASH
for row in v['files']:assert H['digest'](D/row['path'])==row['sha256']
H['run'](['/usr/bin/python3','-I','-B',B/'install-rpc-params-content/install_acceptance.py','--repo',B/'source','--sha',SOURCE,'--delivery',D,'--delivery-sha256',REPORT,'--dil-sha256',v['binary_sha256']['dil'],'--dillsp-sha256',v['binary_sha256']['dillsp'],'--out',B/'linux-install-rpc-valid-content'],env=E,timeout=420,limit=2*1024**2,log_path=B/'linux-install-controller.log')
p=B/'linux-install-rpc-valid-content/PROOF.json';v=json.loads(p.read_text());assert v['status']=='passed' and v['acceptance'] and len(v['cases'])==8 and all(c['passed'] for c in v['cases'])
for row in v['files']:assert H['digest'](p.parent/row['path'])==row['sha256']
print(json.dumps({'status':'passed','source_sha':SOURCE,'proof_sha256':H['digest'](p),'delivery_all_raw_files_verified':True,'installation_all_raw_files_verified':True,'source_zip_equal_mac':False,'source_zip_content_equal_mac':True}))
""".replace('LEAF',repr(leaf)).replace('REPORT',repr(H['digest'](D/'DELIVERY.json'))).replace('PINS',repr(pins)).replace('SOURCE',repr(S)).replace('ZIP_HASH',repr(v['source_zip_sha256']))
 raw=remote(code,'campaign',480);result=json.loads(raw)
 for n in ['PROOF.json','lsp-smoke.json']:
  cmd(['/opt/homebrew/bin/limactl','copy','k276:'+leaf+'/linux-install-rpc-valid-content/'+n,O/n],'copy-'+n,60)
 assert H['digest'](O/'PROOF.json')==result['proof_sha256'];receipt.update(result);receipt['status']='passed';H['clean_exact'](R,S)
except BaseException as error:receipt.update(status='failed',failure_type=type(error).__name__);raise
finally:receipt['finished_ns']=time.time_ns();(O/'CONTROLLER.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('S3 Linux canonical install8 passed')
