#!/usr/bin/env python3
"""Repeated readiness cancellation/terminal state preparation, not DB acceptance."""
import contextlib,io,json,runpy,sys,tempfile,threading,time,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];A=runpy.run_path(str(ROOT/'scripts/db-hazirlik.py'));G=A['main'].__globals__
class Watch(unittest.TestCase):
 def run_case(self,probe,extra=()):
  calls=[]
  class Gate:
   def __init__(self,*args):pass
   def write(self,value):calls.append(value)
   def close(self):calls.append('closed')
  argv=['--dil','/bin/true','--project','/tmp','--worker-project','/tmp','--gate','/tmp/ready.json','--expected-source','1'*40,'--application-revision','2'*40,'--contract','/tmp/contract.json','--contract-sha256','3'*64,*extra]
  out=io.StringIO()
  with patch.dict(G,Gate=Gate,canonical_directory=lambda x:Path(x),probe=probe),contextlib.redirect_stdout(out):code=A['main'](argv)
  return code,json.loads(out.getvalue()),calls
 def test_single_once_legacy_true_retained(self):
  code,value,calls=self.run_case(lambda a,g,c:g.write(True));self.assertEqual((code,value['ready'],calls),(0,True,[False,True,'closed']))
 def test_two_successful_checks_clear_gate_at_finite_completion(self):
  with patch.object(G['time'],'sleep',lambda _:None),patch.object(G['time'],'monotonic',side_effect=iter([0,3])):
   code,v,c=self.run_case(lambda a,g,c:g.write(True),('--rounds','2'))
  self.assertEqual((code,v['reason'],v['ready']),(0,'completed',False));self.assertEqual(c,[False,True,True,False,'closed'])
 def test_schema_failure_is_false_then_retried(self):
  seen=[]
  def probe(a,g,c):
   seen.append(1)
   if len(seen)==1:raise A['Reject']('schema')
   g.write(True)
  with patch.object(G['time'],'sleep',lambda _:None),patch.object(G['time'],'monotonic',side_effect=iter([0,3])):code,v,c=self.run_case(probe,('--rounds','2'))
  self.assertEqual(code,0);self.assertEqual(c,[False,False,True,False,'closed'])
 def test_input_failure_is_terminal_and_clears_gate(self):
  def probe(*a):raise A['Reject']('input')
  code,v,c=self.run_case(probe,('--rounds','2'));self.assertEqual((code,v['reason'],c),(2,'input',[False,False,False,'closed']))
 def test_cancelled_child_is_reaped(self):
  with tempfile.TemporaryDirectory() as d:
   stop=threading.Event();timer=threading.Timer(.05,stop.set);timer.start();start=time.monotonic()
   try:
    with self.assertRaises(A['Reject']) as error:A['child'](Path('/bin/sleep'),['5'],Path(d),start+3,stop.is_set)
    self.assertEqual(error.exception.code,'cancelled');self.assertLess(time.monotonic()-start,2)
   finally:timer.join()
 def test_invalid_rounds_never_open_gate(self):
  for value in ['0','1001']:
   code,v,c=self.run_case(lambda *a:self.fail(),('--rounds',value));self.assertEqual((code,c),(2,[]))
 def test_closed_pipes_terminal_wait_keeps_deadline_class(self):
  with tempfile.TemporaryDirectory() as d:
   start=time.monotonic()
   with self.assertRaises(A['Reject']) as error:A['child'](Path(sys.executable),['-I','-B','-c','import os,time;os.close(1);os.close(2);time.sleep(5)'],Path(d),start+.15)
   self.assertEqual(error.exception.code,'timeout');self.assertLess(time.monotonic()-start,2)
 def test_closed_pipes_terminal_wait_remains_cancellable(self):
  with tempfile.TemporaryDirectory() as d:
   stop=threading.Event();timer=threading.Timer(.15,stop.set);timer.start();start=time.monotonic()
   try:
    with self.assertRaises(A['Reject']) as error:A['child'](Path(sys.executable),['-I','-B','-c','import os,time;os.close(1);os.close(2);time.sleep(5)'],Path(d),start+3,stop.is_set)
    self.assertEqual(error.exception.code,'cancelled');self.assertLess(time.monotonic()-start,2)
   finally:timer.join()
 def selector_fault(self,on_close):
  spawned=[];original=A['subprocess'].Popen;factory=A['selectors'].DefaultSelector
  def popen(*a,**k):
   p=original(*a,**k);spawned.append(p);return p
  class Closing:
   def __init__(self):self.inner=factory()
   def register(self,*a,**k):return self.inner.register(*a,**k)
   def get_map(self):return self.inner.get_map()
   def select(self,*a):return self.inner.select(*a)
   def unregister(self,*a):return self.inner.unregister(*a)
   def close(self):self.inner.close();raise OSError('close fault')
  with tempfile.TemporaryDirectory() as d,patch.object(A['subprocess'],'Popen',side_effect=popen):
   fault=patch.object(A['selectors'],'DefaultSelector',Closing) if on_close else patch.object(A['selectors'],'DefaultSelector',side_effect=OSError('setup fault'))
   with fault,self.assertRaises(OSError):A['child'](Path('/bin/sleep'),['5'],Path(d),time.monotonic()+.1)
  self.assertEqual(len(spawned),1);p=spawned[0];self.assertIsNotNone(p.poll());self.assertTrue(p.stdout.closed and p.stderr.closed)
 def test_selector_setup_failure_reaps_child(self):self.selector_fault(False)
 def test_selector_close_failure_reaps_child(self):self.selector_fault(True)
if __name__=='__main__':unittest.main()
