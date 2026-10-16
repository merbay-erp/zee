#!/usr/bin/env python3
"""Contract negatives without a database or compiler dependency."""
import ast
import copy
import sys
import types
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('module_sql', ROOT / 'scripts/modul-sql-yoneticisi.py')
M = importlib.util.module_from_spec(spec); spec.loader.exec_module(M)
SPEC = {'schema':'zee-sql-module-1','id':'news','label':'News','fields':[{'name':'title','type':'text'},{'name':'amount','type':'integer'},{'name':'active','type':'boolean'}]}
class Contracts(unittest.TestCase):
    def test_generic_types_and_parameterized_sql_preserve_hostile_data(self):
        data={'id':"x'); DROP TABLE victim;--",'title':'<script>Ö</script>','amount':-3,'active':False}
        params=M.values(SPEC,data,'create')
        self.assertEqual(params,[data['id'],data['title'],'-3','false'])
        sql=M.action_sql(SPEC,'create')
        self.assertNotIn(data['id'],sql);self.assertNotIn(data['title'],sql)
        self.assertIn('$2::text,$3::bigint,$4::boolean',sql)
        self.assertIn(json.dumps(data['id']),M.crud_source(SPEC,'create',data).decode())
        reserved=copy.deepcopy(SPEC);reserved['fields'][0]['name']='select'
        self.assertIn('"select"',M.action_sql(reserved,'create'))
        self.assertIn('"select" text',M.migration(reserved,'up',True).decode())
    def test_identifier_collision_reserved_and_injection_rejected(self):
        for name in ['A','public.x','x;DROP','id','x"','x'*25]:
            d=copy.deepcopy(SPEC);d['id']=name
            with self.assertRaises(M.Rejected):M.definition(d)
        d=copy.deepcopy(SPEC);d['fields'].append(d['fields'][0])
        with self.assertRaises(M.Rejected):M.definition(d)
    def test_exact_schema_field_and_type_bounds(self):
        for change in [{'schema':'other'},{'extra':1},{'fields':[]},{'fields':[{'name':'x','type':'float'}]}]:
            d=copy.deepcopy(SPEC);d.update(change)
            with self.assertRaises(M.Rejected):M.definition(d)
        good={'id':'x','title':'v','amount':1,'active':True}
        for field,value in [('amount',True),('amount',2**63),('active',1),('title','x'*4097),('title','\0')]:
            bad=dict(good);bad[field]=value
            with self.assertRaises(M.Rejected):M.values(SPEC,bad,'create')
        with self.assertRaises(M.Rejected):M.values(SPEC,dict(good,extra='x'),'create')
    def test_profile_loopback_explicit_fixed_environment(self):
        p={'schema':'zee-sql-module-profile-1','target':'postgresql://localhost:5555/owned','environment':'ZEE_MODULE_DATABASE_URL'}
        self.assertEqual(M.profile(p),p)
        for key,value in [('target','postgresql://example.org:5555/db'),('target','postgresql://localhost:22/db'),('environment','PATH')]:
            with self.assertRaises(M.Rejected):M.profile(dict(p,**{key:value}))
    def test_migration_down_is_real_ddl_and_preserves_rows(self):
        up=M.migration(SPEC,'up',True).decode();down=M.migration(SPEC,'down').decode();again=M.migration(SPEC,'up').decode()
        self.assertIn('CREATE TABLE public.zee_mod_news(',up)
        self.assertIn('ALTER TABLE public.zee_mod_news RENAME TO zee_mod_news_archive',down)
        self.assertIn('ALTER TABLE public.zee_mod_news_archive RENAME TO zee_mod_news',again)
        for raw in [up,down,again]:self.assertNotIn('DROP ',raw);self.assertNotIn('DELETE ',raw)
        self.assertIn(M.digest(M.encode(SPEC)),down)
    def test_pending_closes_admission_before_receipt_or_subprocess(self):
        with tempfile.TemporaryDirectory() as td:
            m=object.__new__(M.Manager);m.control=Path(td);m.in_transition=False
            for name in ['sql-pending.json','sql-write-pending.json']:
                (m.control/name).write_bytes(b'{}')
                with patch.object(M.Parent,'ready',side_effect=AssertionError('must not reach receipt/native')):
                    with self.assertRaises(M.Rejected):m.ready()
                (m.control/name).unlink()
    def test_state_exact_sequence_and_schema_not_just_matching_keys(self):
        m=object.__new__(M.Manager);m.identity=(2,3);m.db_profile={'schema':'zee-sql-module-profile-1','target':'postgresql://localhost:5555/owned','environment':'ZEE_MODULE_DATABASE_URL'}
        s={'schema':'zee-sql-module-state-1','root':[2,3],'profile':M.digest(M.encode(m.db_profile)),'sequence':0,'modules':{},'migrations':{}}
        self.assertEqual(m.validate_state(s),s)
        for change in [{'sequence':True},{'root':[0,0]},{'extra':1},{'sequence':1,'migrations':{'000002_module.sql':'a'*64}},{'modules':{'news':{'definition':SPEC,'state':'unknown'}}}]:
            with self.assertRaises(M.Rejected):m.validate_state(dict(s,**change))
    def test_missing_helper_cli_is_fixed_quiet_before_execution(self):
        import subprocess,sys,shutil
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/'PRIVATE_MODULE_PATH.py';shutil.copyfile(ROOT/'scripts/modul-sql-yoneticisi.py',p)
            result=subprocess.run([sys.executable,'-I','-B',str(p),'--root','PRIVATE_CANARY','--binary','PRIVATE_CANARY','list','news'],capture_output=True,timeout=5)
            self.assertEqual(result.returncode,2);self.assertEqual(result.stdout,b'');self.assertEqual(result.stderr,b'sql-module-operation-rejected\n')
    def test_missing_actual_metadata_keeps_pending_and_blocks_publication(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td);m=object.__new__(M.Manager);m.control=root;m.sqlroot=root/'sql';m.sqlroot.mkdir();(m.sqlroot/'gocler').mkdir()
            m.identity=(2,3);m.binary_hash='a'*64;m.db_profile={'schema':'zee-sql-module-profile-1','target':'postgresql://localhost:5555/owned','environment':'ZEE_MODULE_DATABASE_URL'}
            old={'schema':'zee-sql-module-state-1','root':[2,3],'profile':M.digest(M.encode(m.db_profile)),'sequence':0,'modules':{},'migrations':{}}
            raw=M.migration(SPEC,'up',True);name='000001_module.sql';(m.sqlroot/'gocler'/name).write_bytes(raw)
            new=copy.deepcopy(old);new.update(sequence=1,modules={'news':{'definition':SPEC,'state':'up'}},migrations={name:M.digest(raw)})
            pending={'schema':'zee-sql-module-pending-1','root':[2,3],'binary':'a'*64,'old_state':M.digest(M.encode(old)),'state':new,'name':'news','migration':name,'projection':{}}
            (root/'sql-state.json').write_bytes(M.encode(old));(root/'sql-pending.json').write_bytes(M.encode(pending))
            with patch.object(m,'native',return_value=b'[]\n') as native,patch.object(m,'publish',side_effect=AssertionError('must not publish')):
                with self.assertRaises(M.Rejected):m.finish_sql(pending)
                self.assertEqual(native.call_count,1)
            self.assertEqual((root/'sql-state.json').read_bytes(),M.encode(old));self.assertTrue((root/'sql-pending.json').exists())
    def test_old_module_helper_bytes_unchanged_and_new_loader_verified(self):
        self.assertIsNotNone(M.BASE)
        self.assertEqual(M.digest((ROOT/'scripts/modul-yoneticisi.py').read_bytes()),'84752ef56d5c1d347f2e211ebe721def648aa5e496df1bd8bfec2375933237b6')
TARGET=ROOT/'scripts/tests/modul-sql-native.py'

class NativeCleanupContracts(unittest.TestCase):
    def environment(self):
        tree=ast.parse(TARGET.read_bytes())
        functions=[n for n in ast.walk(tree) if isinstance(n,ast.FunctionDef) and n.name in {'cancel','close_cluster','publish_proof'}]
        self.assertEqual({n.name for n in functions},{'cancel','close_cluster','publish_proof'})
        env={'H':{'CANCELLED':False},'campaign_cancelled':False,'cleaning':False,'campaign_started':0,'campaign_deadline':600,'cleanup':[],
             'proof':{'status':'passed','cases':[{'passed':True} for _ in range(9)]},'raw':[],'json':json,'sys':sys,'now':1,'work_exists':False,
             'log_exists':False,'writes':{},'on_close':lambda:None,'on_write':lambda name:None,'write_failures':{},'restore_calls':[]}
        env['time']=types.SimpleNamespace(monotonic=lambda:env['now'])
        class File:
            def __init__(self,name):self.name=name
            def __truediv__(self,name):return File(self.name+'/'+name)
            def exists(self):
                env['exists_calls']=env.get('exists_calls',0)+1
                if env['exists_calls']==env.get('exists_error_at'):
                    raise OSError('controlled final directory observation')
                return env['work_exists'] if self.name=='work' else False
            def is_file(self):return env['log_exists']
            def write_text(self,text):
                env['on_write'](self.name)
                failures=env['write_failures'].get(self.name,[])
                if failures:raise failures.pop(0)
                env['writes'][self.name]=text
            def write_bytes(self,data):env['writes'][self.name]=data
        class Signal:
            SIGTERM=15;SIGINT=2
            def __init__(self):
                self.original={15:lambda *_:None,2:lambda *_:None};self.handlers=dict(self.original)
                self.install_count=0;self.fail_install=None;self.fail_restore=None
            def signal(self,n,handler):
                if handler is self.original[n]:
                    env['restore_calls'].append(n)
                    if n==self.fail_restore:raise OSError('controlled handler restoration')
                else:
                    self.install_count+=1
                    if self.install_count==self.fail_install:raise OSError('controlled partial installation')
                previous=self.handlers[n];self.handlers[n]=handler;return previous
        signal=Signal();env['signal']=signal;env['oldsignals']={};env['a']=types.SimpleNamespace(output=File('output'))
        env['work']=File('work');env['certdir']=File('certdir');env['M']=types.SimpleNamespace(BASE={'snapshot':lambda _:b'public-log'})
        env['cluster']=types.SimpleNamespace(close=lambda:env['on_close']())
        exec(compile(ast.Module(body=functions,type_ignores=[]),str(TARGET),'exec'),env)
        mains=[n for n in tree.body if isinstance(n,ast.Try) and n.body and isinstance(n.body[0],ast.For)]
        self.assertEqual(len(mains),1)
        env['install']=lambda:exec(compile(ast.Module(body=[mains[0].body[0]],type_ignores=[]),str(TARGET),'exec'),env)
        return env
    def installed(self):
        e=self.environment();e['install']();return e
    def assert_failed(self,e,error=ValueError):
        with self.assertRaises(error):e['publish_proof']()
        self.assertEqual(e['proof']['status'],'failed')
        if 'output/proof.json' in e['writes']:self.assertEqual(json.loads(e['writes']['output/proof.json'])['status'],'failed')
    def test_native_cleanup_uncancelled_success_after_temp_removal(self):
        e=self.installed();e['close_cluster']();e['publish_proof']()
        self.assertFalse(e['H']['CANCELLED']);self.assertFalse(e['cleaning'])
        self.assertEqual(e['proof']['status'],'passed');self.assertEqual(e['restore_calls'],[15,2])
        self.assertEqual(e['signal'].handlers,e['signal'].original)
    def test_native_cleanup_term_and_int_remain_sticky(self):
        for number in (15,2):
            with self.subTest(signal=number):
                e=self.installed();observed=[]
                def close():
                    e['signal'].handlers[number](number,None);observed.append(e['H']['CANCELLED'])
                e['on_close']=close;e['close_cluster']()
                self.assertEqual(observed,[False]);self.assertTrue(e['campaign_cancelled']);self.assertTrue(e['H']['CANCELLED'])
                self.assertEqual(e['cleanup'],['owned-pg-closed']);self.assert_failed(e)
    def test_native_cleanup_prior_and_repeated_cancellation_never_reset(self):
        e=self.installed();e['cancel'](15,None)
        def close():
            self.assertFalse(e['H']['CANCELLED'])
            e['cancel'](15,None);e['cancel'](2,None)
        e['on_close']=close;e['close_cluster']();self.assertTrue(e['H']['CANCELLED']);self.assert_failed(e)
    def test_native_cleanup_close_error_does_not_record_success(self):
        e=self.installed()
        def close():raise OSError('controlled close failure')
        e['on_close']=close
        with self.assertRaises(OSError):e['close_cluster']()
        self.assertEqual(e['cleanup'],['owned-pg-cleanup-failed']);self.assertFalse(e['cleaning']);self.assert_failed(e)
    def test_native_cleanup_log_snapshot_failure_restores_state(self):
        e=self.installed();e['H']['CANCELLED']=True;e['log_exists']=True
        def snapshot(_):raise OSError('controlled log snapshot')
        e['M'].BASE['snapshot']=snapshot
        with self.assertRaises(OSError):e['close_cluster']()
        self.assertFalse(e['cleaning']);self.assertTrue(e['H']['CANCELLED']);e['proof']['status']='failed';self.assert_failed(e)
    def test_native_cleanup_partial_install_restores_installed_subset(self):
        e=self.environment();e['signal'].fail_install=2
        with self.assertRaises(OSError):e['install']()
        self.assertEqual(set(e['oldsignals']),{15});e['proof']['status']='failed';self.assert_failed(e)
        self.assertEqual(e['restore_calls'],[15]);self.assertEqual(e['signal'].handlers,e['signal'].original)
    def test_native_cleanup_one_restore_failure_attempts_other_handler(self):
        e=self.installed();e['close_cluster']();e['signal'].fail_restore=15;self.assert_failed(e,OSError)
        self.assertEqual(e['restore_calls'],[15,2]);self.assertEqual(e['proof']['handler_restore_errors'],['OSError'])
    def test_native_cleanup_deadline_expiry_blocks_pass(self):
        e=self.installed();e['on_close']=lambda:e.update(now=600);e['close_cluster']();self.assert_failed(e)
        self.assertTrue(e['proof']['deadline_expired']);self.assertEqual(e['cleanup'],['owned-pg-closed'])
    def test_native_cleanup_cancel_during_raw_or_proof_publication(self):
        for name in ('output/raw.json','output/proof.json'):
            with self.subTest(file=name):
                e=self.installed();e['close_cluster']();fired=[]
                def write(current):
                    if current==name and not fired:
                        fired.append(True);e['signal'].handlers[15](15,None)
                e['on_write']=write;self.assert_failed(e);self.assertTrue(e['proof']['cancelled'])
    def test_native_cleanup_publication_and_rewrite_errors_keep_first(self):
        for name in ('output/raw.json','output/proof.json'):
            with self.subTest(file=name):
                e=self.installed();e['close_cluster']();first=OSError('first publication error')
                e['write_failures'][name]=[first]
                e['write_failures'].setdefault('output/proof.json',[]).extend([RuntimeError('later failed rewrite')])
                with self.assertRaises(OSError) as observed:e['publish_proof']()
                self.assertIs(observed.exception,first);self.assertEqual(e['restore_calls'],[15,2]);self.assertEqual(e['proof']['status'],'failed')
    def test_native_cleanup_body_failure_keeps_original_exception(self):
        e=self.installed();e['close_cluster']();e['proof']['status']='failed';original=LookupError('original body')
        try:raise original
        except LookupError:
            with self.assertRaises(LookupError) as observed:e['publish_proof']()
        self.assertIs(observed.exception,original);self.assertEqual(e['restore_calls'],[15,2])
    def test_native_cleanup_final_directory_check_error_fails_publication(self):
        e=self.installed();e['close_cluster']();e['exists_error_at']=4
        self.assert_failed(e,OSError)
        self.assertEqual(e['proof']['finalization_failure'],'OSError')
    def test_native_cleanup_remaining_private_directory_blocks_pass(self):
        e=self.installed();e['close_cluster']();e['work_exists']=True;self.assert_failed(e)
        self.assertFalse(e['proof']['temporary_private_root_removed'])

if __name__=='__main__':unittest.main()
