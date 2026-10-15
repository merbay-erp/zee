import importlib.util,json,os,tempfile,time,types,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1];spec=importlib.util.spec_from_file_location('ttl_host',ROOT/'worker-supervisor.py');H=importlib.util.module_from_spec(spec);spec.loader.exec_module(H)
class ReadinessTTL(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.path=Path(self.temp.name)/'ready.json';self.base=time.time_ns();self.revision='2'*40;self.write(self.base)
 def tearDown(self):self.temp.cleanup()
 def write(self,stamp,ready=True):
  self.path.write_text(json.dumps({'schema':'zee-worker-readiness-1','application_revision':self.revision,'ready':ready}));os.utime(self.path,ns=(stamp,stamp))
 def host(self,ttl=3):
  h=H.Supervisor(types.SimpleNamespace(readiness_file=self.path,application_revision=self.revision,readiness_ttl_seconds=ttl),None);h.gate_epoch_ns=self.base-1;return h
 def ready(self,h,wall,mono):
  with patch.object(H.time,'time_ns',return_value=wall),patch.object(H.time,'monotonic',return_value=mono):return h.application_ready()
 def test_existing_default_has_no_ttl(self):self.assertTrue(self.ready(self.host(None),self.base+3600*10**9,10))
 def test_expired_file_fails_without_prior_observation(self):self.assertFalse(self.ready(self.host(),self.base+4*10**9,10))
 def test_mono_deadline_does_not_rearm_on_wall_clock_rewind(self):
  h=self.host();self.assertTrue(self.ready(h,self.base+2*10**9,10));self.assertFalse(self.ready(h,self.base+10**9,12))
 def test_fresh_actual_file_rearms_but_old_receipt_cannot(self):
  h=self.host();self.assertTrue(self.ready(h,self.base+2*10**9,10));self.write(self.base+3*10**9);self.assertTrue(self.ready(h,self.base+3100000000,12));self.write(self.base);self.assertFalse(self.ready(h,self.base+2*10**9,12.5))
 def test_false_missing_future_and_previous_generation_refused(self):
  h=self.host();self.write(self.base,False);self.assertFalse(self.ready(h,self.base+10**9,10));self.write(self.base+3*10**9);self.assertFalse(self.ready(h,self.base+10**9,10));h.gate_epoch_ns=self.base+4*10**9;self.assertFalse(self.ready(h,self.base+5*10**9,10));self.path.unlink();self.assertFalse(self.ready(h,self.base+5*10**9,10))
 def test_invalid_cli_ttl_is_rejected_before_file_or_child_access(self):
  argv=['--dil','/missing-dil','--project','/missing-project','--origin','https://fixture.example','--worker-port','10001','--worker-port','10002','--expected-source','1'*40,'--application-revision','2'*40,'--readiness-file','/missing-project/ready.json','--health-path','/hazir','--health-body','ok']
  for value in ['nan','inf','-inf','0','3601']:
   with self.subTest(value=value),self.assertRaises(H.Reject) as error:H.arguments(argv+['--readiness-ttl-seconds='+value])
   self.assertEqual(error.exception.code,'budgets')
if __name__=='__main__':unittest.main()
