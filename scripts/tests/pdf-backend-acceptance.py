"""Explicit owned backend acceptance; requires exact installed PDF packages. Not normal CI model."""
import argparse,copy,hashlib,importlib.util,io,json,os,subprocess,sys,tempfile,time,unittest
from pathlib import Path
class Args(argparse.ArgumentParser):
 def error(self,message):raise ValueError('arguments')
try:
 parser=Args(allow_abbrev=False)
 parser.add_argument('--framework-root',type=Path,required=True)
 parser.add_argument('--output',type=Path,required=True)
 args=parser.parse_args()
 ROOT=args.framework_root.resolve(strict=True);S=args.output
 P=ROOT/'scripts/pdf-katalog-host.py';F=ROOT/'golden/pdf-host'
 spec=importlib.util.spec_from_file_location('backend',P);B=importlib.util.module_from_spec(spec);spec.loader.exec_module(B)
 B.backend_contract()
 from pypdf import PdfReader
 JOB=json.loads((F/'job.json').read_text())
 B.validate(json.dumps(JOB).encode())
 S.mkdir(mode=0o700)
 artifact=S/'output/pdf/tr';artifact.mkdir(parents=True)
 for name,data in B.produce(JOB,F).items():(artifact/name).write_bytes(data)
except Exception:
 print('{"type":"UnsupportedOrRejected","profile":"owned-pdf-backend-acceptance"}',file=sys.stderr)
 sys.exit(2)
class Tests(unittest.TestCase):
 def test_real_pdf_text_font_image_and_original_page_operators(self):
  base=PdfReader(F/'assets/base.pdf');catalog=PdfReader(S/'output/pdf/tr/catalog.pdf');selected=PdfReader(S/'output/pdf/tr/selected.pdf');generated=PdfReader(S/'output/pdf/tr/generated.pdf')
  self.assertEqual((len(base.pages),len(catalog.pages),len(selected.pages),len(generated.pages)),(3,3,2,1))
  for original,new in [(0,0),(2,2)]:self.assertEqual(base.pages[original].get_contents().get_data(),catalog.pages[new].get_contents().get_data())
  self.assertEqual([p.extract_text() for p in selected.pages],[base.pages[i].extract_text() for i in [0,2]])
  text='\n'.join(p.extract_text() for p in catalog.pages)
  for value in ['Anonim ürün kataloğu','Örnek ürün - ŞĞİıÇÖÜ','OWNED BASE COVER','OWNED BASE BACK COVER']:self.assertIn(value,text)
  self.assertNotIn('OWNED BASE REMOVED PAGE',text)
  self.assertEqual(len(generated.pages[0].images),1)
  fonts=generated.pages[0]['/Resources']['/Font'].get_object();self.assertTrue(any('/FontFile2' in x.get_object().get('/FontDescriptor',{}) for x in fonts.values()))
 def test_literal_html_url_never_becomes_pdf_action(self):
  job=copy.deepcopy(JOB);job['description']='<script>PRIVATE_PDF_CANARY</script> file:///PRIVATE_PDF_CANARY https://fixture.invalid/'
  data=B.produce(B.validate(json.dumps(job).encode()),F)['generated.pdf'];r=PdfReader(io.BytesIO(data));text=r.pages[0].extract_text();self.assertIn('PRIVATE_PDF_CANARY',text);self.assertNotIn('/Annots',r.pages[0]);self.assertNotIn('/OpenAction',r.trailer['/Root']);self.assertNotIn('/Names',r.trailer['/Root'])
 def test_invalid_job_fields_and_types(self):
  bad=[]
  for key,value in [('schema','other'),('locale','de'),('selection','../x'),('placement','file:///PRIVATE'),('title',''),('title','x'*81),('title',True),('description','x\nPRIVATE'),('description','\u0000'),('description','\ud800')]:
   j=copy.deepcopy(JOB);j[key]=value;bad.append(json.dumps(j).encode())
  for key in ['source','html','font','image','shell','url']:
   j=copy.deepcopy(JOB);j[key]='PRIVATE';bad.append(json.dumps(j).encode())
  bad += [b'{',b'[]',b'{"schema":"x","schema":"y"}',b'x'*8193]
  for data in bad:
   with self.subTest(data_hash=hashlib.sha256(data).hexdigest()):
    with self.assertRaises((B.Rejected,ValueError,UnicodeError)):B.validate(data)
 def test_bound_buffer_rejects_before_write(self):
  out=B.BoundedBuffer();out.write(b'x'*B.MAX_PDF)
  with self.assertRaises(B.Rejected):out.write(b'x')
  self.assertEqual(out.tell(),B.MAX_PDF)
 def test_regular_file_fifo_symlink_oversize_before_open(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t);os.mkfifo(p/'fifo');(p/'big').write_bytes(b'x'*8193);(p/'link').symlink_to(p/'big')
   old=B.os.open
   def fail(*args,**kwargs):raise AssertionError('must not open invalid role')
   B.os.open=fail
   try:
    for name in ['fifo','link','big']:
     with self.subTest(name=name):
      with self.assertRaises(B.Rejected):B.bounded_file(p/name,8192)
   finally:B.os.open=old
 def test_tampered_fixed_asset_rejects(self):
  import shutil
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'fixture';shutil.copytree(F,root);(root/'assets/base.pdf').write_bytes(b'hostile')
   with self.assertRaises(B.Rejected):B.produce(JOB,root)
 def test_actual_cli_quiet_negatives_and_existing_output(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t);job=root/'PRIVATE_PDF_CANARY';job.write_bytes(b'{invalid PRIVATE_PDF_CANARY');fifo=root/'fifo';os.mkfifo(fifo);big=root/'big';big.write_bytes(b'x'*8193);output=root/'never'
   for bad in [job,fifo,big]:
    p=subprocess.run([sys.executable,'-I','-B',str(P),'--fixture-root',str(F),'--job',str(bad),'--output',str(output)],capture_output=True,timeout=3)
    self.assertEqual(p.returncode,2);self.assertEqual(p.stdout,b'');self.assertEqual(p.stderr,b'{"type":"Rejected","profile":"owned-pdf-prototype"}\n');self.assertFalse(output.exists())
   job.write_text(json.dumps(JOB));output.mkdir();(output/'foreign').write_bytes(b'foreign')
   p=subprocess.run([sys.executable,'-I','-B',str(P),'--fixture-root',str(F),'--job',str(job),'--output',str(output)],capture_output=True,timeout=3);self.assertEqual(p.returncode,2);self.assertEqual((output/'foreign').read_bytes(),b'foreign')
 def test_all_pages_supported_fixed_selection(self):
  job=copy.deepcopy(JOB);job['selection']='all';r=B.produce(job,F);self.assertEqual(len(PdfReader(io.BytesIO(r['selected.pdf'])).pages),3);self.assertEqual(len(PdfReader(io.BytesIO(r['catalog.pdf'])).pages),4)
if __name__=='__main__':unittest.main(argv=[sys.argv[0]])
