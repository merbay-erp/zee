#!/usr/bin/env python3
"""Contract negatives without a database or compiler dependency."""
import copy
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
if __name__=='__main__':unittest.main()
