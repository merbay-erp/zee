"""Standard-library-only models. No PDF parser/backend availability claim."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
ROOT = Path(__file__).resolve().parents[2]
P = ROOT/'scripts/pdf-katalog-host.py'
spec = importlib.util.spec_from_file_location('pdf_job',P)
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)
JOB = {'schema':'zee-pdf-job-1','locale':'tr','title':'Anonim','product':'Örnek','description':'Görsel','selection':'cover-back','placement':'after-cover'}
class Models(unittest.TestCase):
 def test_valid_flat_text_contract(self):
  self.assertEqual(B.validate(json.dumps(JOB).encode()),JOB)
 def test_schema_types_fields_controls_and_duplicates(self):
  rows=[]
  for key,value in [('schema','bad'),('locale','de'),('title',True),('title','x'*81),('description','x\nPRIVATE'),('description','\ud800'),('selection','../PRIVATE')]:
   j=copy.deepcopy(JOB);j[key]=value;rows.append(json.dumps(j).encode())
  for key in ['url','html','shell','font','image']:
   j=copy.deepcopy(JOB);j[key]='PRIVATE';rows.append(json.dumps(j).encode())
  rows.extend([b'{',b'[]',b'{"schema":"x","schema":"x"}',b'x'*8193])
  for row in rows:
   with self.subTest(length=len(row)):
    with self.assertRaises((B.Rejected,ValueError,UnicodeError)):B.validate(row)
 def test_buffer_cap_rejects_before_extra_write(self):
  stream=B.BoundedBuffer();stream.write(b'x'*B.MAX_PDF)
  with self.assertRaises(B.Rejected):stream.write(b'x')
  self.assertEqual(stream.tell(),B.MAX_PDF)
 def test_file_roles_reject_before_open(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t);os.mkfifo(root/'fifo');(root/'big').write_bytes(b'x'*8193);(root/'link').symlink_to(root/'big')
   with patch.object(B.os,'open',side_effect=AssertionError('must not open')):
    for name in ['fifo','big','link']:
     with self.subTest(name=name):
      with self.assertRaises(B.Rejected):B.bounded_file(root/name,8192)
 def test_same_regular_bytes(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'data';p.write_bytes(b'fixed');self.assertEqual(B.bounded_file(p,5),b'fixed')
 def test_exact_version_contract_without_installed_pdf_packages(self):
  with patch.object(B.importlib.metadata,'version',side_effect=lambda n:B.EXPECTED_LIBRARIES[n]):self.assertEqual(B.backend_contract(),B.EXPECTED_LIBRARIES)
  with patch.object(B.importlib.metadata,'version',return_value='0.0.0'):
   with self.assertRaises(B.Rejected):B.backend_contract()
  with patch.object(B.importlib.metadata,'version',side_effect=B.importlib.metadata.PackageNotFoundError):
   with self.assertRaises(B.importlib.metadata.PackageNotFoundError):B.backend_contract()
 def test_missing_backend_has_no_parser_import_or_fallback(self):
  with patch.object(B.importlib.metadata,'version',return_value='wrong'):
   with self.assertRaises(B.Rejected):B.produce(JOB,Path('/unused-private'))
 def test_actual_cli_invalid_argument_is_fixed_and_quiet(self):
  p=subprocess.run([sys.executable,'-I','-B',str(P),'--PRIVATE_PDF_CANARY'],capture_output=True,timeout=3)
  self.assertEqual(p.returncode,2);self.assertEqual(p.stdout,b'');self.assertEqual(p.stderr,b'{"type":"Rejected","profile":"owned-pdf-prototype"}\n')
if __name__=='__main__':unittest.main()
