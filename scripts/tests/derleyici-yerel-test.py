#!/usr/bin/env python3
"""Actual owned-store boundaries plus ephemeral signed protocol/native processes."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / 'derleyici-yerel.py'
M = runpy.run_path(str(SCRIPT))
F = runpy.run_path(str(HERE / 'derleyici-secici-test.py'))


def sha(data):
    return hashlib.sha256(data).hexdigest()


class LocalToolsTests(unittest.TestCase):
    def setUp(self):
        self.f = F['Fixture']()
        self.pin, self.bundle, self.record = self.f.package('A')
        self.project = self.f.project('project', self.pin)
        self.destination = self.f.root / 'destination'
        self.destination.mkdir(mode=0o700)

    def tearDown(self):
        self.f.temp.cleanup()

    def params(self):
        return (self.project, self.destination, self.f.catalog, self.f.publisher, self.f.tool, self.f.tool_sha)

    def do_import(self, pilot=False):
        return M['import_bundle'](self.project, self.bundle, self.destination, self.f.catalog,
                                  self.f.publisher, self.f.tool, self.f.tool_sha, pilot)

    def launcher(self, *args, cwd=None, **kwargs):
        return subprocess.run([str(self.project / '.zee-araclar/zee'), *args],
                              cwd=cwd or self.f.root, capture_output=True, timeout=15, **kwargs)

    def target(self):
        return self.destination / self.pin['source_sha'] / self.pin['platform']

    def test_import_exact_bytes_idempotent_and_different_payload_not_overwritten(self):
        self.assertTrue(self.do_import()['published'])
        self.assertEqual({p.name:p.read_bytes() for p in self.bundle.iterdir()},
                         {p.name:p.read_bytes() for p in self.target().iterdir()})
        self.assertTrue(self.do_import()['existing_exact'])
        (self.target()/'dil').write_bytes(b'foreign payload')
        with self.assertRaises(M['Reject']): self.do_import()
        self.assertEqual((self.target()/'dil').read_bytes(), b'foreign payload')

    def test_corrupt_bundle_no_candidate_no_target_pin_unchanged(self):
        before=(self.project/'derleyici.kilit.json').read_bytes()
        (self.bundle/'dil').write_bytes(b'not authenticated')
        with self.assertRaises(M['Reject']): self.do_import()
        self.assertFalse(self.target().exists())
        self.assertFalse((self.f.root/'A-executed').exists())
        self.assertEqual((self.project/'derleyici.kilit.json').read_bytes(),before)

    def test_source_swap_after_validation_publishes_only_private_snapshot(self):
        original=(self.bundle/'dil').read_bytes()
        publish=M['publish']
        def swap(*args):
            (self.bundle/'dil').write_bytes(b'changed after validation')
            return publish(*args)
        with mock.patch.dict(M['import_bundle'].__globals__,publish=swap): self.do_import()
        self.assertEqual((self.target()/'dil').read_bytes(),original)

    def test_symlink_and_world_writable_store_rejected(self):
        alias=self.f.root/'alias';alias.symlink_to(self.bundle,target_is_directory=True)
        with self.assertRaises(M['Reject']):
            M['import_bundle'](self.project,alias,*self.params()[1:])
        self.destination.chmod(0o777)
        with self.assertRaises(M['Reject']): self.do_import()
        self.destination.chmod(0o700)
        self.assertFalse(self.target().exists())

    def test_replaced_acquired_lock_not_unlinked_and_no_publication(self):
        original=M['write_files']
        lock=self.destination/'.target.lock'
        def replace(*args):
            original(*args);lock.unlink();lock.write_bytes(b'foreign replacement')
        with mock.patch.dict(M['publish'].__globals__,write_files=replace):
            with self.assertRaises(M['Reject']): M['publish'](self.destination,'target',{'a':b'a'},set(),False)
        self.assertEqual(lock.read_bytes(),b'foreign replacement')
        self.assertFalse((self.destination/'target').exists())
        self.assertFalse(list(self.destination.glob('.target.staging-*')))

    def test_post_rename_fsync_failure_preserves_published_target_and_reports_state(self):
        original=M['fsync_directory']
        def fail_parent(path):
            if path==self.destination: raise OSError('PRIVATE_PATH_CANARY')
            original(path)
        with mock.patch.dict(M['publish'].__globals__,fsync_directory=fail_parent):
            with self.assertRaises(M['PublicationError']) as context:
                M['publish'](self.destination,'target',{'a':b'a'},set(),False)
        self.assertTrue(context.exception.published)
        self.assertEqual((self.destination/'target/a').read_bytes(),b'a')
        self.assertFalse((self.destination/'.target.lock').exists())

    def test_prepublication_write_failure_cleans_only_owned_stage(self):
        other=self.destination/'foreign';other.write_bytes(b'preserve')
        with mock.patch.dict(M['publish'].__globals__,write_files=mock.Mock(side_effect=OSError('PRIVATE'))):
            with self.assertRaises(M['PublicationError']) as context:
                M['publish'](self.destination,'target',{'a':b'a'},set(),False)
        self.assertFalse(context.exception.published)
        self.assertEqual(list(self.destination.iterdir()),[other])

    def test_concurrent_process_publication_is_whole_or_busy(self):
        code='''import json,runpy,sys
m=runpy.run_path(sys.argv[1])
try:
 print(json.dumps(m['publish'](__import__('pathlib').Path(sys.argv[2]),'target',{'a':b'x'*65536},set(),True)))
except FileExistsError: print('busy')
'''
        jobs=[subprocess.Popen([sys.executable,'-I','-c',code,str(SCRIPT),str(self.destination)],stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(3)]
        results=[]
        try:
            for child in jobs:
                out,err=child.communicate(timeout=10);self.assertEqual(child.returncode,0,err);results.append(out)
        finally:
            for child in jobs:
                if child.returncode is None: child.kill();child.communicate(timeout=5)
        self.assertTrue(any(b'"published": true' in r for r in results))
        self.assertEqual((self.destination/'target/a').read_bytes(),b'x'*65536)
        self.assertFalse(list(self.destination.glob('.target.staging-*')))

    def test_launcher_current_pin_project_scope_and_foreign_install_preservation(self):
        self.do_import();M['install_launcher'](*self.params())
        before=(self.project/'derleyici.kilit.json').read_bytes()
        result=self.launcher('çalıştır','denetle')
        self.assertEqual(result.returncode,0,result.stderr);self.assertIn(b'A:project:',result.stdout)
        with self.assertRaises(M['Reject']): M['install_launcher'](*self.params())
        self.assertEqual((self.project/'derleyici.kilit.json').read_bytes(),before)
        other=self.f.root/'other';other.mkdir()
        result=self.launcher('çalıştır','denetle',str(other));self.assertEqual(result.returncode,2)
        self.assertFalse((other/'.zee-araclar').exists())
        # Change current pin, not launcher policy; no hidden old compiler fallback.
        pin,bundle,_=self.f.package('B');self.bundle=bundle
        (self.project/'derleyici.kilit.json').write_text(json.dumps(pin))
        self.do_import()
        result=self.launcher('çalıştır','denetle');self.assertEqual(result.returncode,0,result.stderr)
        self.assertIn(b'B:project:',result.stdout)

    def test_launcher_selector_tamper_and_invalid_cli_mask_private_canary(self):
        self.do_import();M['install_launcher'](*self.params())
        (self.project/'.zee-araclar/selector.py').write_text('raise Exception("PRIVATE_CANARY")')
        result=self.launcher('seç');self.assertEqual(result.returncode,2)
        self.assertNotIn(b'PRIVATE_CANARY',result.stderr);self.assertNotIn(str(self.project).encode(),result.stderr)
        result=subprocess.run([sys.executable,'-I',str(SCRIPT),'import','--PRIVATE_CANARY'],capture_output=True,timeout=10)
        self.assertEqual(result.returncode,2);self.assertNotIn(b'PRIVATE_CANARY',result.stderr)
        self.assertNotIn(b'usage',result.stderr)
        private=self.f.root/'PRIVATE_SECRET_PATH'
        argv=[sys.executable,'-I',str(SCRIPT),'import','--proje',str(self.project),'--depo',str(self.destination),
              '--kaynak',str(self.bundle),'--guven',str(private),'--guven-anahtari',self.f.publisher,
              '--dogrulayici',str(self.f.tool),'--dogrulayici-sha256',self.f.tool_sha]
        result=subprocess.run(argv,capture_output=True,timeout=15)
        self.assertEqual(result.returncode,2);self.assertNotIn(b'PRIVATE_SECRET_PATH',result.stderr)
        self.assertNotIn(str(self.f.root).encode(),result.stderr)
        # Existing selector OSError must be caught at the installed launcher boundary too.
        shutil.copyfile(SCRIPT.with_name('derleyici-secici.py'),self.project/'.zee-araclar/selector.py')
        self.f.catalog.unlink()
        result=self.launcher('seç');self.assertEqual(result.returncode,2)
        self.assertNotIn(str(self.f.root).encode(),result.stderr)

    def test_bundle_inventory_stops_before_exhausting_iterator(self):
        class Entries:
            count=0
            def __enter__(self): return self
            def __exit__(self,*args): pass
            def __iter__(self): return self
            def __next__(self):
                self.count+=1
                if self.count>3: raise AssertionError('unbounded enumeration')
                return type('Entry',(),{'name':str(self.count)})()
        entries=Entries()
        with mock.patch.object(os,'scandir',return_value=entries):
            with self.assertRaises(M['Reject']): M['exact_files'](self.destination,{'a':b'a'})
        self.assertEqual(entries.count,2)

    def test_missing_corrupt_selector_bootstrap_quiet_subprocess(self):
        owned=self.f.root/'bootstrap';owned.mkdir(mode=0o700)
        manager=owned/SCRIPT.name;shutil.copyfile(SCRIPT,manager)
        helper=owned/'derleyici-secici.py'
        for kind in ('missing','corrupt','empty','truncated','invalid_exports'):
            if kind=='corrupt': helper.write_text('raise RuntimeError("PRIVATE_BOOTSTRAP_CANARY")')
            if kind=='empty': helper.write_bytes(b'')
            if kind=='truncated': helper.write_text('class Reject(ValueError): pass\n')
            if kind=='invalid_exports': helper.write_text('Reject = object\nrequire = 1\nregular_bytes = None\n')
            result=subprocess.run([sys.executable,'-I',str(manager),'import','--PRIVATE_ARG'],capture_output=True,timeout=10)
            self.assertEqual(result.returncode,2)
            self.assertEqual(result.stdout,b'')
            self.assertEqual(result.stderr,'Yerel araç işlemi reddedildi.\n'.encode())
            self.assertNotIn(str(owned).encode(),result.stderr)

    def test_duplicate_inventory_observations_reject_without_exhaustion(self):
        class Entries:
            count=0
            def __enter__(self): return self
            def __exit__(self,*args): pass
            def __iter__(self): return self
            def __next__(self):
                self.count+=1
                if self.count>2: raise AssertionError('unbounded repeated enumeration')
                return type('Entry',(),{'name':'dil'})()
        # Publication and authenticated source reader both count observations,
        # not only accumulated distinct names. Duplicate is rejected immediately.
        entries=Entries()
        with mock.patch.object(os,'scandir',return_value=entries):
            with self.assertRaises(M['Reject']): M['exact_files'](self.destination,{'dil':b'a','dillsp':b'b'})
        self.assertEqual(entries.count,2)
        entries=Entries();real=os.scandir
        def controlled(path):
            return entries if not isinstance(path,int) and Path(path).resolve()==self.bundle.resolve() else real(path)
        with mock.patch.object(os,'scandir',side_effect=controlled):
            with self.assertRaises(M['Reject']):
                with M['S']['authenticated_snapshot'](self.pin,self.f.store,self.f.catalog,self.f.publisher,
                        self.f.tool,self.f.tool_sha,False,self.project,bundle_source=self.bundle): pass
        self.assertEqual(entries.count,2)

    def real_package(self):
        tools=F['Fixture'].__init__.__globals__['COMPILER_DIR']
        build=json.loads(subprocess.run([str(tools/'dil'),'sürüm','--json'],capture_output=True,check=True,timeout=10).stdout)
        self.actual_build=build
        date=build['source_date'] if build['source_date']!='unknown' else F['DATE']
        epoch=int(datetime.strptime(date,'%Y-%m-%d').replace(tzinfo=timezone.utc).timestamp())
        # Existing A identity is removed only from this synthetic test catalog/store.
        self.f.trust['releases']=[]
        self.pin,self.bundle,self.record=self.f.package('REAL',version=build['version'],channel=build['channel'],
              source_sha=build['source_sha'] if build['source_sha']!='unknown' else 'c'*40,source_date=date,epoch=epoch)
        for name in M['S']['binary_names'](self.pin): shutil.copyfile(tools/name,self.bundle/name)
        sums={name:sha((self.bundle/name).read_bytes()) for name in M['S']['binary_names'](self.pin)}
        (self.bundle/'SHA256SUMS').write_text(''.join(f'{sums[name]}  {name}\n' for name in sorted(sums)))
        self.f.sign(self.bundle/'SHA256SUMS')
        sbom=json.loads((self.bundle/'sbom.spdx.json').read_text())
        for node in sbom['@graph']:
            if node['type']=='software_File': node['verifiedUsing'][0]['hashValue']=sums[node['name']]
        (self.bundle/'sbom.spdx.json').write_text(json.dumps(sbom))
        provenance=json.loads((self.bundle/'provenance.intoto.json').read_text())
        for subject in provenance['subject']: subject['digest']={'sha256':sums[subject['name']]}
        for byproduct in provenance['predicate']['runDetails']['byproducts']: byproduct['digest']={'sha256':sha((self.bundle/'SHA256SUMS').read_bytes())}
        (self.bundle/'provenance.intoto.json').write_text(json.dumps(provenance))
        self.record['file_hashes']={name:sha((self.bundle/name).read_bytes()) for name in M['S']['META']}
        self.f.save_catalog()
        return tools

    def test_actual_signed_debug_pair_dependency_lock_and_lsp_stdio(self):
        tools=self.real_package()
        for name in ('app','dep'):
            result=subprocess.run([str(tools/'dil'),'yeni',name],cwd=self.f.root,capture_output=True,timeout=10)
            self.assertEqual(result.returncode,0,result.stderr)
        self.project=self.f.root/'app'
        (self.project/'derleyici.kilit.json').write_text(json.dumps(self.pin))
        self.do_import(pilot=True);M['install_launcher'](*self.params(),pilot=True)
        # Scaffold creates a lock; remove explicitly to exercise legacy no-dependency mode.
        (self.project/'proje.kilit').unlink(missing_ok=True)
        self.assertFalse((self.project/'proje.kilit').exists())
        pin_before=(self.project/'derleyici.kilit.json').read_bytes()
        source_before=(self.project/'proje.dil').read_bytes()
        result=self.launcher('çalıştır','denetle');self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual((self.project/'proje.dil').read_bytes(),source_before)
        # Actual LSP initialize/shutdown/exit, not process-start-only smoke.
        requests=[{'jsonrpc':'2.0','id':1,'method':'initialize','params':{'capabilities':{}}},
                  {'jsonrpc':'2.0','id':2,'method':'shutdown'},
                  {'jsonrpc':'2.0','method':'exit'}]
        frames=b''.join(b'Content-Length: '+str(len(data)).encode()+b'\r\n\r\n'+data
                         for data in [json.dumps(item).encode() for item in requests])
        result=self.launcher('lsp',input=frames)
        self.assertEqual(result.returncode,0,result.stderr)
        remaining=result.stdout;responses=[]
        while remaining:
            headers,body=remaining.split(b'\r\n\r\n',1)
            size=int(headers.split(b':',1)[1]);responses.append(json.loads(body[:size]));remaining=body[size:]
        self.assertEqual([response['id'] for response in responses],[1,2])
        self.assertIn('capabilities',responses[0]['result'])
        self.assertEqual(responses[0]['result']['serverInfo']['zeeBuild'],self.actual_build)
        self.assertIsNone(responses[1]['result'])
        manifest=self.project/'proje.dil'
        self.assertIn('yerel_bağımlılıklar boş liste olsun',manifest.read_text())
        manifest.write_text(manifest.read_text().replace('yerel_bağımlılıklar boş liste olsun','yerel_bağımlılıklar "../dep" listesi olsun'))
        result=self.launcher('çalıştır','denetle');self.assertNotEqual(result.returncode,0)
        self.assertIn(b'P008',result.stderr+result.stdout);self.assertFalse((self.project/'proje.kilit').exists())
        result=subprocess.run([str(tools/'dil'),'kilitle',str(self.project)],capture_output=True,timeout=10)
        self.assertEqual(result.returncode,0,result.stderr)
        lock_before=(self.project/'proje.kilit').read_bytes()
        source_before=(self.project/'proje.dil').read_bytes()
        all_app_sources={p.relative_to(self.project).as_posix():p.read_bytes() for p in self.project.rglob('*.dil')}
        result=self.launcher('çalıştır','denetle');self.assertEqual(result.returncode,0,result.stderr)
        dep_source=self.f.root/'dep/ana.dil'
        with dep_source.open('a') as file: file.write('\n')
        result=self.launcher('çalıştır','denetle');self.assertNotEqual(result.returncode,0)
        self.assertIn(b'P008',result.stderr+result.stdout)
        self.assertEqual((self.project/'proje.kilit').read_bytes(),lock_before)
        self.assertEqual((self.project/'proje.dil').read_bytes(),source_before)
        self.assertEqual((self.project/'derleyici.kilit.json').read_bytes(),pin_before)
        self.assertFalse((self.f.root/'A-main').exists())
        self.assertEqual({p.relative_to(self.project).as_posix():p.read_bytes() for p in self.project.rglob('*.dil')},all_app_sources)
        print('native-proof '+json.dumps({'build':self.actual_build,'lsp_initialize_shutdown':True,
              'no_dependency_missing_lock_accepted':True,'dependency_missing_lock_P008':True,
              'valid_dependency_lock_accepted':True,'changed_dependency_source_P008':True,
              'pin_lock_app_source_bytes_preserved':True}),file=sys.stderr)


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--compiler-dir',type=Path,required=True)
    args=parser.parse_args()
    F['Fixture'].__init__.__globals__['COMPILER_DIR']=args.compiler_dir.resolve()
    unittest.main(argv=[sys.argv[0]],verbosity=2)
