import importlib.util,struct,tempfile,unittest,os,json,hashlib
from pathlib import Path
B=Path(__file__).resolve().parents[1];spec=importlib.util.spec_from_file_location('P',B/'linux-package-preflight.py');P=importlib.util.module_from_spec(spec);spec.loader.exec_module(P)
class Preparation(unittest.TestCase):
 def raw(self,machine=62):
  b=bytearray(64);b[:7]=b'\x7fELF\x02\x01\x01';struct.pack_into('<HHI',b,16,3,machine,1);struct.pack_into('<H',b,52,64);return bytes(b)
 def test_model_amd64_header_not_executable_acceptance(self):self.assertEqual(P.header(self.raw(),'amd64')['machine'],62)
 def test_model_arm64_header_not_executable_acceptance(self):self.assertEqual(P.header(self.raw(183),'arm64')['machine'],183)
 def test_wrong_arch_and_nonELF(self):
  for data in [self.raw(183),b'\xcf\xfa\xed\xfe'+b'\0'*60,b'MZ'+b'\0'*62,b'']:
   with self.assertRaises(P.Reject):P.header(data,'amd64')
 def test_invalid_class_endian_type_version(self):
  for index,value in [(4,1),(5,2),(6,0),(7,9),(16,1),(20,2),(52,63)]:
   b=bytearray(self.raw());b[index]=value
   with self.assertRaises(P.Reject):P.header(b,'amd64')
 def test_duplicates_nonfinite_and_relative_escape(self):
  for b in [b'{"x":1,"x":2}',b'{"x":NaN}']:
   with self.assertRaises(P.Reject):P.decode(b)
  for n in ['/x','../x','a/../x','a//x','a/./x','a\\x']:
   with self.assertRaises(P.Reject):P.normal(n)
 def test_regular_snapshot_hash_and_role_reject(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t).resolve()/'f';p.write_bytes(b'payload');os.chmod(p,0o600)
   self.assertEqual(P.file(p,10,True),(hashlib.sha256(b'payload').hexdigest(),b'payload'))
   q=p.with_name('link');q.symlink_to(p)
   with self.assertRaises(P.Reject):P.file(q)
   q.unlink();os.mkfifo(q,0o600)
   with self.assertRaises(P.Reject):P.file(q)
 def test_cap_reject_and_same_snapshot_elf_header(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t).resolve()/'f';p.write_bytes(self.raw());os.chmod(p,0o600)
   with self.assertRaises(P.Reject):P.file(p,63)
   h,v=P.elf(p,'amd64');self.assertEqual(h,hashlib.sha256(self.raw()).hexdigest());self.assertEqual(v['architecture'],'amd64')
 def test_clean_source_build_identity_only(self):
  d={'schema':'zee-build-v1','source_sha':'1'*40,'dirty':False,'version':'0.8.0-dev'};P.same_build(d,'1'*40)
  for k,v in [('dirty',True),('source_sha','2'*40),('schema','bad')]:
   b=dict(d);b[k]=v
   with self.assertRaises(P.Reject):P.same_build(b,'1'*40)
 def test_actual_MachO_owned_compiler_rejected_not_linux(self):
  p=B/'fixtures/observed-macos-dil-header.bin'
  self.assertEqual(P.file(p)[0],'cb883bf4f4bfb461026ccae14e003c19faa4fd9f31fcaf7554ac857c00042134')
  with self.assertRaises(P.Reject):P.elf(p,'arm64')
 def model_delivery(self,root):
  sha='1'*40;bins=root/'binaries';bins.mkdir(mode=0o700)
  values={n:self.raw()+n.encode() for n in ('dil','dillsp')};hashes={n:hashlib.sha256(b).hexdigest() for n,b in values.items()}
  files={'binaries/'+n:b for n,b in values.items()};files['binaries/SHA256SUMS']=''.join(hashes[n]+'  '+n+'\n' for n in ('dil','dillsp')).encode();sums=hashlib.sha256(files['binaries/SHA256SUMS']).hexdigest()
  files['binaries/GIT_SHA']=(sha+'\n').encode()
  prov={'_type':'https://in-toto.io/Statement/v1','predicateType':'https://slsa.dev/provenance/v1','subject':[{'name':n,'digest':{'sha256':h}} for n,h in hashes.items()],'predicate':{'buildDefinition':{'externalParameters':{'gitCommit':sha,'platform':'x86_64-unknown-linux-gnu','rustc':'rustc 1.93.1 model','buildEnvironment':{}}},'zeeTekrarUretim':{'ikiTemizKlonEsit':True},'runDetails':{'byproducts':[{'name':n,'digest':{'sha256':sums}} for n in ['SHA256SUMS','SHA256SUMS.ikinci-klon']]}}}
  files['binaries/provenance.intoto.json']=json.dumps(prov).encode();files['binaries/sbom.spdx.json']=json.dumps({'@graph':[{'type':'software_File','name':n,'verifiedUsing':[{'hashValue':h}]} for n,h in hashes.items()]}).encode()
  files['build.log']=''.join('-- '+n+' iki klonda eş: '+h+'\n' for n,h in hashes.items()).encode()
  for n,b in files.items():(root/n).write_bytes(b);os.chmod(root/n,0o600)
  r={'schema':'zee-local-owned-delivery-1','status':'passed','source_sha':sha,'signed':False,'published':False,'two_source_zip_equal':True,'bundle_head':sha,'restoration_fsck':True,'build_identity':{'schema':'zee-build-v1','source_sha':sha,'dirty':False,'version':'0.8.0-dev'},'binary_sha256':hashes,'files':[{'path':n,'sha256':hashlib.sha256(b).hexdigest()} for n,b in files.items()]}
  (root/'DELIVERY.json').write_text(json.dumps(r));os.chmod(root/'DELIVERY.json',0o600);return r,hashlib.sha256((root/'DELIVERY.json').read_bytes()).hexdigest()
 def test_top_level_nonobject_report_rejected(self):
  for raw in [b'[]',b'null',b'1',b'false']:
   with tempfile.TemporaryDirectory() as td:
    root=Path(td).resolve();(root/'DELIVERY.json').write_bytes(raw);os.chmod(root/'DELIVERY.json',0o600)
    with self.assertRaises(P.Reject):P.delivery(root,'1'*40,hashlib.sha256(raw).hexdigest(),'amd64')
 def test_model_delivery_positive_not_native_acceptance(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td).resolve();r,h=self.model_delivery(root);v=P.delivery(root,'1'*40,h,'amd64')
   self.assertFalse(v['payload_executed'] or v['packaged'] or v['installed'] or v['container_executed'])
 def test_model_delivery_source_inventory_checksum_negatives(self):
  for variant in ['source','dirty','duplicates','tampered','platform','reportpin']:
   with self.subTest(variant=variant),tempfile.TemporaryDirectory() as td:
    root=Path(td).resolve();r,h=self.model_delivery(root)
    if variant=='source':r['source_sha']='2'*40
    elif variant=='dirty':r['build_identity']['dirty']=True
    elif variant=='duplicates':r['files'].append(r['files'][0])
    elif variant=='tampered':(root/'binaries/dil').write_bytes(b'not-ELF')
    elif variant=='platform':r['binary_sha256']['dil']='0'*64
    if variant in ['source','dirty','duplicates','platform']:
     (root/'DELIVERY.json').write_text(json.dumps(r));h=hashlib.sha256((root/'DELIVERY.json').read_bytes()).hexdigest()
    if variant=='reportpin':h='0'*64
    with self.assertRaises(P.Reject):P.delivery(root,'1'*40,h,'amd64')
if __name__=='__main__':unittest.main(verbosity=2)
