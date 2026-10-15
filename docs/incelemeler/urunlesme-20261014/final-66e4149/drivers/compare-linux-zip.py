import hashlib,importlib.util,json,runpy,zipfile
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');S='66e4149e53451111d153f47283c219079a8d1cab';H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));H['clean_exact'](R,S)
D=E/'s3-66e4149-linux-delivery-unzip-path';v=json.loads((D/'DELIVERY.json').read_text());c=json.loads((D/'CONTROLLER.json').read_text());assert v['status']==c['status']=='passed'
O=E/'s3-66e4149-source-zip-comparison';assert not O.exists();O.mkdir(mode=0o700);env=H['controlled_env']();env['LIMA_HOME']='/private/tmp/zk276-hih4chab/lima'
i=Path(env['LIMA_HOME'])/'k276';assert (i.stat().st_dev,i.stat().st_ino)==(16777232,422274626);assert H['digest'](i/'lima.yaml')=='ac827e1be51749d62ceefb0e9cb8bdce1257a10f96e0def065ad6506e85854f9'
name='zee-kaynak-'+S+'.zip';remote='k276:'+c['guest_leaf']+'/linux-delivery-unzip-path/'+name
H['run'](['/opt/homebrew/bin/limactl','copy',remote,O/'linux-source.zip'],env=env,timeout=90,limit=1024**2,log_path=O/'copy.log');assert H['digest'](O/'linux-source.zip')==v['source_zip_sha256']
mac=E/'s3-66e4149-macos-delivery'/name
spec=importlib.util.spec_from_file_location('zip_verifier',R/'scripts/kaynak-ci-esleme.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
expected=M.archive_hashes(R,S);M.zip_hashes(mac.read_bytes(),S,expected);M.zip_hashes((O/'linux-source.zip').read_bytes(),S,expected)
def inventory(p):
 with zipfile.ZipFile(p) as z:
  return {i.filename:{'sha256':hashlib.sha256(z.read(i)).hexdigest(),'crc':i.CRC,'bytes':i.file_size,'compressed_bytes':i.compress_size,'compression':i.compress_type,'date':i.date_time,'create_system':i.create_system,'external_attr':i.external_attr,'flag':i.flag_bits} for i in z.infolist() if not i.is_dir()}
a,b=inventory(mac),inventory(O/'linux-source.zip');assert set(a)==set(b) and all(a[n]['sha256']==b[n]['sha256'] for n in a)
diff={n:{k:[a[n][k],b[n][k]] for k in a[n] if a[n][k]!=b[n][k]} for n in a if a[n]!=b[n]}
proof={'schema':'zee-cross-platform-source-content-review-1','status':'passed','source_sha':S,'mac_zip_sha256':H['digest'](mac),'linux_zip_sha256':v['source_zip_sha256'],'zip_byte_equal':H['digest'](mac)==v['source_zip_sha256'],'all_entry_names_and_bytes_equal':True,'both_canonical_zip_manifest_and_Git_snapshot_verified':True,'files':len(a),'entry_metadata_differences':diff,'boundary':'Per-platform two-clone ZIP byte equality passed; cross-platform byte equality was an extra external assertion and is not claimed. Exact source content equality independently verified.'}
H['clean_exact'](R,S);(O/'COMPARISON.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps({k:proof[k] for k in ['status','zip_byte_equal','all_entry_names_and_bytes_equal','files']}))
