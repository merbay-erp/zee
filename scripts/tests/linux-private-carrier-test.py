import importlib.util,hashlib,os,stat,subprocess,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
B=Path(__file__).resolve().parents[1];spec=importlib.util.spec_from_file_location('carrier',B/'linux-private-carrier.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
class Preparation(unittest.TestCase):
 def test_control_fixed_private_identity_inert_no_scripts(self):
  b=M.control('1'*40,'arm64');self.assertIn(b'Private evaluation placeholder <noreply@invalid.invalid>',b);self.assertNotIn(b'Depends:',b);self.assertNotIn(b'http',b);self.assertIn(b'Architecture: arm64',b)
 def test_sha_arch_and_injection_rejected(self):
  for sha in ['short','A'*40,'1'*40+'\nMaintainer: fake']:
   with self.assertRaises(M.Reject):M.package_name(sha)
  with self.assertRaises(M.Reject):M.control('1'*40,'i386')
 def test_fresh_output_inside_sources_existing_symlink_refused(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td).resolve();root.chmod(0o700);repo=root/'repo';repo.mkdir();delivery=root/'delivery';delivery.mkdir();out=root/'out';self.assertEqual(M.fresh_output(out,repo,delivery),out)
   for path in [repo/'out',delivery/'out',repo]:
    with self.assertRaises(M.Reject):M.fresh_output(path,repo,delivery)
   out.symlink_to(repo)
   with self.assertRaises(M.Reject):M.fresh_output(out,repo,delivery)
 def test_writable_output_parent_refused(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td).resolve();root.chmod(0o777)
   with self.assertRaises(M.Reject):M.fresh_output(root/'out',root/'r',root/'d')
 def test_inventory_hash_mode_exact_and_foreign_content_detected(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td).resolve();M.emit(root,'usr/share/a',b'one');before=M.inventory(root);self.assertEqual(before['usr/share/a'],(hashlib.sha256(b'one').hexdigest(),0o644));(root/'usr/share/a').write_bytes(b'two');self.assertNotEqual(M.inventory(root),before)
 def test_inventory_special_link_hardlink_and_entry_budget(self):
  for role in ['symlink','fifo','hardlink','budget']:
   with self.subTest(role=role),tempfile.TemporaryDirectory() as td:
    root=Path(td).resolve();a=root/'a';a.write_bytes(b'x')
    if role=='symlink':(root/'b').symlink_to(a)
    elif role=='fifo':os.mkfifo(root/'b',0o600)
    elif role=='hardlink':os.link(a,root/'b')
    else:
     for n in range(65):(root/str(n)).write_bytes(b'x')
    with self.assertRaises(M.Reject):M.inventory(root)
 def test_emit_no_clobber(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td).resolve();M.emit(root,'a',b'first')
   with self.assertRaises(FileExistsError):M.emit(root,'a',b'second')
   self.assertEqual((root/'a').read_bytes(),b'first')
 def test_helper_regular_snapshot_hash_and_FIFO_refused(self):
  self.assertTrue(callable(M.helper('linux-package-preflight.py',M.P_SHA).delivery));self.assertTrue(callable(M.helper('lifecycle_core.py',M.H_SHA).run))
  with tempfile.TemporaryDirectory() as td:
   root=Path(td).resolve()
   with patch.object(M,'HERE',root):
    os.mkfifo(root/'f',0o600)
    with self.assertRaises(M.Reject):M.helper('f','0'*64)
    (root/'f').unlink();(root/'f').write_bytes(b'raise Exception("never execute")')
    with self.assertRaises(M.Reject):M.helper('f','0'*64)
 def test_missing_arguments_fixed_quiet_rejection(self):
  r=subprocess.run([sys.executable,'-I','-B',str(B/'linux-private-carrier.py')],capture_output=True,timeout=5);self.assertEqual(r.returncode,2);self.assertEqual(r.stdout,b'');self.assertEqual(r.stderr,b'private-carrier-rejected\n')
if __name__=='__main__':unittest.main(verbosity=2)
