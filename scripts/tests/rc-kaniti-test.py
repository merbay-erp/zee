#!/usr/bin/env python3
"""Synthetic protocol fixtures only: no actual ASAN/Miri campaign acceptance."""
import copy,hashlib,importlib.util,json,os,subprocess,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('rc_evidence',ROOT/'scripts/rc-kaniti.py');V=importlib.util.module_from_spec(spec);spec.loader.exec_module(V)

def put(p,data):
    p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data if isinstance(data,bytes) else data.encode())

def command(name,argv,log,elapsed=1801):return {'name':name,'argv':argv,'exit':0,'elapsed_seconds':elapsed,'supervisor_failure':None,'passed':True,'log_sha256':V.digest(log)}

class Protocol(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='zee-rc-protocol-only-');self.home=Path(self.temp.name).resolve();self.root=self.home/'repo';self.root.mkdir();self.c=self.home/'campaign';self.c.mkdir();self.bundle=self.home/'bundle'
        put(self.root/'scripts/rc-kampanyasi.py',(ROOT/'scripts/rc-kampanyasi.py').read_bytes())
        for name in ('rc-kaniti.py','guvenlik-kapisi.sh'):put(self.root/'scripts'/name,(ROOT/'scripts'/name).read_bytes())
        put(self.root/'docs/fuzz-rc-gecmisi-v1.tsv',b'# deliberately no current source rows\n')
        put(self.root/'compiler/fixture-only',b'not a compiler')
        for target in V.TARGETS:put(self.root/('compiler/fuzz/corpus/'+target+'/seed'),b'synthetic-fixture')
        for name in ('conformance-korugu.sh','semantic-regresyon-korugu.sh','core-freeze-korugu.sh'):put(self.root/'scripts'/name,b'exit 0\n')
        def git(*a):return subprocess.check_output(['git','-C',str(self.root),*a],stderr=subprocess.DEVNULL)
        git('init','-q');git('add','.');git('-c','user.name=Protocol Test','-c','user.email=protocol@example.invalid','commit','-qm','synthetic protocol source')
        git('-c','user.name=Protocol Test','-c','user.email=protocol@example.invalid','commit','--allow-empty','-qm','synthetic protocol candidate')
        self.git=git;self.source=V.identity(self.root);self.producer=V.digest((self.root/'scripts/rc-kampanyasi.py').read_bytes())
        self.proof={'schema':'zee-local-asan-miri-campaign-1','source_sha':self.source['sha'],'source_unchanged':True,'producer_path':'scripts/rc-kampanyasi.py','producer_sha256':self.producer,'platform':{'system':'Darwin','release':'fixture-kernel','os_version':'fixture-os','machine':'arm64','processor':'fixture CPU','ram_bytes':1024},'host_target':'aarch64-apple-darwin','run_origin':'local','run_id':'123','run_attempt':1,'result':'passed-local-technical-campaign','tools':{'toolchain':'nightly-2026-08-31','cargo_fuzz':'cargo-fuzz 0.13.2','rustc':'rustc fixture-only\nhost: aarch64-apple-darwin\ncommit-hash: 90850177249efe0321573c569aec5d12b257f8d6','miri':'miri 0.1.0 (9085017724 2026-08-30)'},'maximum_concurrent_fuzz_targets':2,'builds':{},'targets':[],'miri':[]}
        for k in ('source_start','source_end','original_source_start','original_source_end'):self.proof[k]=self.source
        for t,maximum in V.TARGETS.items():
            build=b'rustc --cfg sanitizer=address\n';put(self.c/('logs/asan-build-'+t+'.log'),build);self.proof['builds'][t]=dict(command(t,['cargo','+nightly-2026-08-31','fuzz','build','--verbose','--sanitizer','address','--target','aarch64-apple-darwin','--target-dir',str(self.c/'fuzz-target'),t],build,2),explicit_sanitizer_flag=True)
            pref=self.c/'targets'/t;log=b'Done 40 runs in 1801 second(s)\nstat::number_of_executed_units: 40\n';put(pref/'run.log',log)
            seed=b'synthetic-fixture';put(pref/'corpus/seed',seed)
            headers={'target':t,'corpus':t,'source_commit':self.source['sha'],'run_id':'123','run_attempt':'1','toolchain':'nightly-2026-08-31','cargo_fuzz':'0.13.2','run_origin':'local','seed_sha256':'relative_path'}
            manifest=('# zee-fuzz-corpus-artifact-1\n'+''.join('# '+k+'\t'+v+'\n' for k,v in headers.items())+V.digest(seed)+'\tseed\n').encode();put(pref/'manifest.tsv',manifest);put(pref/'initial-manifest.tsv',manifest);put(pref/'manifest-verification.log',b'fixture verified\n')
            result={'target':t,'source_commit':self.source['sha'],'run_id':'123','run_attempt':'1','run_origin':'local','requested_seconds':'1800','sanitizer':'address','exit_code':'0'};put(pref/'sonuc.tsv','# zee-fuzz-rc-sonucu-1\n'+''.join(k+'\t'+v+'\n' for k,v in result.items()))
            record={'target':t,'max_len':maximum,'asan_runtime_marker':True,'manifest_verified':True,'manifest_sha256':V.digest(manifest),'final_seeds':1,'initial_seeds':1,'binary_sha256':'1'*64,'command':command(t,['cargo','+nightly-2026-08-31','fuzz','run','--sanitizer','address','--target','aarch64-apple-darwin','--target-dir',str(self.c/'fuzz-target'),t,str(pref/'corpus'),'--','-dict='+str(self.c/'checkout/compiler/fuzz/dictionaries/zee.dict'),'-max_total_time=1800','-max_len='+str(maximum),'-timeout=5','-print_final_stats=1','-artifact_prefix='+str(pref/'crashes')+'/'],log),'stats':{'passed':True,'sanitizer':'address','sanitizer_finding':False,'libfuzzer_failure':False,'input_timeout':False,'crash_artifacts':[],'requested_seconds':1800,'actual_seconds':1801,'executions':40}}
            self.proof['targets'].append(record);put(pref/'result.json',json.dumps(record))
        for group in V.MIRI:
            log=b'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured;\n';put(self.c/('logs/miri-'+group+'.log'),log);self.proof['miri'].append(dict(command('miri-'+group,['cargo','+nightly-2026-08-31','miri','test','--locked','--lib',group+'::testler::'],log,1),actual_tests=2))
        self.save()
    def save(self):put(self.c/'campaign-proof.json',json.dumps(self.proof))
    def tearDown(self):self.temp.cleanup()
    def export(self):return V.export(self.root,self.c,self.bundle)
    def check(self,receipt):
        m,d=V.bundle_snapshot(self.bundle,receipt['manifest_sha256']);return V.validate(self.root,V.identity(self.root),m,d)
    def rejected(self):
        self.save()
        with self.assertRaises((V.Invalid,KeyError)):self.export()
    def test_synthetic_protocol_fixture_roundtrip_not_campaign(self):
        r=self.export();self.assertEqual(self.check(r)['targets'],4)
        p=subprocess.run([sys.executable,'-I','-B',str(ROOT/'scripts/rc-kaniti.py'),'check','--framework-root',str(self.root),'--bundle',str(self.bundle),'--sha256',r['manifest_sha256']],capture_output=True,timeout=10);self.assertEqual(p.returncode,0,p.stderr)
    def test_other_clean_commit_not_source_equivalent(self):
        r=self.export();self.git('-c','user.name=Test','-c','user.email=x@example.invalid','commit','--allow-empty','-qm','other SHA same tree')
        with self.assertRaises(V.Invalid):self.check(r)
    def test_dirty_original_and_changed_producer_rejected(self):
        put(self.root/'untracked',b'x')
        with self.assertRaises(V.Invalid):self.export()
        (self.root/'untracked').unlink();self.proof['producer_sha256']='0'*64;self.rejected()
    def test_missing_duplicate_target_and_miri(self):
        for field in ('targets','miri'):
            saved=copy.deepcopy(self.proof[field]);self.proof[field]=saved[:-1];self.rejected();self.proof[field]=saved
        self.proof['targets'][1]=copy.deepcopy(self.proof['targets'][0]);self.rejected()
    def sync_target(self,index=0):
        rec=self.proof['targets'][index];put(self.c/('targets/'+rec['target']+'/result.json'),json.dumps(rec));self.save()
    def test_wrong_build_target_and_run_operation_rejected(self):
        self.proof['builds']['lexer_parser']['argv'][-1]='morfoloji';self.rejected()
        self.proof['builds']['lexer_parser']['argv'][-1]='lexer_parser';self.proof['targets'][0]['command']['argv'][3]='build';self.sync_target();self.rejected()
    def test_conflicting_duplicate_fuzz_options_rejected(self):
        self.proof['targets'][0]['command']['argv'].append('-max_total_time=1');self.sync_target();self.rejected()
    def test_dictionary_and_cache_paths_bind_same_campaign_root(self):
        rec=self.proof['targets'][0];original=rec['command']['argv'][13];rec['command']['argv'][13]='-dict=relative/checkout/compiler/fuzz/dictionaries/zee.dict';self.sync_target();self.rejected();rec['command']['argv'][13]=original
        self.proof['builds']['lexer_parser']['argv'][10]='/foreign/cache';rec['command']['argv'][9]='/foreign/cache';self.sync_target();self.rejected()
    def test_cross_target_root_and_lexical_path_escapes_rejected(self):
        rec=self.proof['targets'][1];original=copy.deepcopy(rec['command']['argv']);root='/other-campaign';rec['command']['argv'][9]=root+'/fuzz-target';rec['command']['argv'][11]=root+'/targets/morfoloji/corpus';rec['command']['argv'][13]='-dict='+root+'/checkout/compiler/fuzz/dictionaries/zee.dict';rec['command']['argv'][-1]='-artifact_prefix='+root+'/targets/morfoloji/crashes/';self.proof['builds']['morfoloji']['argv'][10]=root+'/fuzz-target';self.sync_target(1);self.rejected()
        rec['command']['argv']=original;rec['command']['argv'][11]=str(self.c)+'/./targets/morfoloji/corpus';self.sync_target(1);self.rejected()
        for value in ('/a/../campaign/fuzz-target','//campaign/fuzz-target','/a//campaign/fuzz-target','relative/fuzz-target'):
            with self.assertRaises(V.Invalid):V.recorded_path(value)
    def test_initial_manifest_count_and_source_lineage_rejected(self):
        p=self.c/'targets/lexer_parser/initial-manifest.tsv';saved=p.read_bytes();p.write_bytes(b'not a manifest\n');self.rejected();p.write_bytes(saved)
        self.proof['targets'][0]['initial_seeds']=2;self.sync_target();self.rejected();self.proof['targets'][0]['initial_seeds']=1;self.sync_target()
        p.write_bytes(saved.replace(V.digest(b'synthetic-fixture').encode(),b'0'*64));self.rejected()
    def test_deep_json_and_bad_binary_digest_or_concurrency_rejected(self):
        with self.assertRaises(V.Invalid):V.decode(('['*40+'0'+']'*40).encode())
        self.proof['targets'][0]['binary_sha256']='invalid';self.sync_target();self.rejected();self.proof['targets'][0]['binary_sha256']='1'*64;self.sync_target()
        self.proof['maximum_concurrent_fuzz_targets']=4;self.rejected()
    def test_environment_cannot_be_unknown_or_platform_claim_swapped(self):
        environment=copy.deepcopy(self.proof['platform'])
        for key,value in [('processor',''),('ram_bytes',None),('system','Windows'),('os_version','')]:
            self.proof['platform']=dict(environment,**{key:value});self.rejected()
        self.proof['platform']=environment;self.proof['tools']['toolchain']='nightly';self.rejected()
    def test_duration_exit_sanitizer_and_deadline(self):
        record=self.proof['targets'][0]
        for path,bad in [('actual_seconds',1799),('requested_seconds',3601),('sanitizer','none'),('input_timeout',True),('executions',0)]:
            saved=record['stats'][path];record['stats'][path]=bad;self.rejected();record['stats'][path]=saved
        record['command']['elapsed_seconds']=2413;self.rejected()
    def test_raw_log_and_result_disagreement(self):
        put(self.c/'targets/lexer_parser/run.log',b'AddressSanitizer:DEADLYSIGNAL\n');self.rejected()
    def test_miri_zero_ignored_wrong_selection(self):
        rec=self.proof['miri'][0];rec['argv'][-1]='other::';self.rejected();rec['argv'][-1]='ondalik::testler::'
        log=b'test result: ok. 0 passed; 0 failed; 1 ignored;\n';put(self.c/'logs/miri-ondalik.log',log);rec.update(actual_tests=0,log_sha256=V.digest(log));self.rejected()
    def test_raw_seed_tamper_and_crash_entry(self):
        put(self.c/'targets/lexer_parser/corpus/seed',b'tampered');self.rejected()
        put(self.c/'targets/lexer_parser/crashes/crash-file',b'bad');self.rejected()
    def test_bundle_manifest_digest_and_unlisted_file(self):
        r=self.export()
        with self.assertRaises(V.Invalid):V.bundle_snapshot(self.bundle,'0'*64)
        put(self.bundle/'extra',b'x')
        with self.assertRaises(V.Invalid):self.check(r)
    def test_symlink_leaf_ancestor_and_fifo_never_followed(self):
        r=self.export();p=self.bundle/'logs/miri-ondalik.log';p.unlink();p.symlink_to(self.c/'logs/miri-ondalik.log')
        with self.assertRaises((V.Invalid,OSError)):self.check(r)
        alias=self.home/'alias';alias.symlink_to(self.home,target_is_directory=True)
        with self.assertRaises(OSError):V.open_directory(alias/'bundle')
        p.unlink();os.mkfifo(p)
        with self.assertRaises((V.Invalid,OSError)):self.check(r)
    def test_export_parent_alias_cannot_write_source(self):
        alias=self.home/'source-alias';alias.symlink_to(self.root,target_is_directory=True)
        with self.assertRaises(V.Invalid):V.export(self.root,self.c,alias/'injected-evidence')
        self.assertFalse((self.root/'injected-evidence').exists());self.assertEqual(V.identity(self.root),self.source)
    def test_entry_budget_stops_stream_before_enumerating_rest(self):
        from contextlib import contextmanager
        from types import SimpleNamespace
        observed=[]
        @contextmanager
        def scan(_):
            def entries():
                for number in range(100000000):
                    observed.append(number)
                    yield SimpleNamespace(name='entry-'+str(number))
            yield entries()
        with patch.object(V,'MAX_FILES',3),patch.object(V.os,'scandir',side_effect=scan):
            with self.assertRaises(V.Invalid):V.inventory(0)
        self.assertEqual(observed,[0,1,2,3])
    def test_duplicate_json_and_path_traversal_rejected(self):
        with self.assertRaises(V.Invalid):V.decode(b'{"schema":1,"schema":2}')
        for name in ('../x','/tmp/x','a/../b','a//b','a\\b'):
            with self.assertRaises(V.Invalid):V.relative(name)
    def test_source_changed_at_postcheck(self):
        r=self.export();m,d=V.bundle_snapshot(self.bundle,r['manifest_sha256']);changed=dict(self.source,sha='0'*40)
        with patch.object(V,'identity',return_value=changed):
            with self.assertRaises(V.Invalid):V.validate(self.root,self.source,m,d)
    def test_bundle_snapshot_keeps_checked_bytes_if_original_mutates(self):
        r=self.export();m,d=V.bundle_snapshot(self.bundle,r['manifest_sha256']);p=self.bundle/'campaign-proof.json';p.chmod(0o600);p.write_bytes(b'bad')
        self.assertEqual(V.validate(self.root,self.source,m,d)['status'],'validated-integrity-and-policy')
        with self.assertRaises(V.Invalid):self.check(r)

class GateRouting(unittest.TestCase):
    setUp=Protocol.setUp
    tearDown=Protocol.tearDown
    save=Protocol.save
    export=Protocol.export
    # Reuse actual temporary clean git repo and synthetic bundle, never cargo/fuzz.
    def invoke_gate(self,*args,swap_sha=None):
        tool=self.home/'tools';tool.mkdir(exist_ok=True);trace=self.home/'trace'
        put(tool/'cargo-deny',b'#!/bin/sh\nexit 0\n');(tool/'cargo-deny').chmod(0o700)
        put(tool/'cargo',b'#!/bin/sh\nif [ "$1 $2" = "deny --version" ]; then if [ -n "$RC_TEST_SWAP_SHA" ]; then git -C "$RC_TEST_REPO" checkout -q "$RC_TEST_SWAP_SHA"; fi; echo "cargo-deny 0.20.2"; exit 0; fi\nprintf "%s\n" "$*" >> "$RC_TEST_TRACE"\nexit 0\n');(tool/'cargo').chmod(0o700)
        env=dict(os.environ,PATH=str(tool)+os.pathsep+os.environ['PATH'],RC_TEST_TRACE=str(trace),RC_TEST_SWAP_SHA=swap_sha or '',RC_TEST_REPO=str(self.root))
        result=subprocess.run(['bash',str(self.root/'scripts/guvenlik-kapisi.sh'),*args],cwd=self.root,env=env,capture_output=True,timeout=15)
        return result,trace.read_text() if trace.exists() else ''
    def test_external_route_keeps_old_checks_and_fixed_default_rejects(self):
        receipt=self.export();result,trace=self.invoke_gate('--surum-adayi','--rc-kanit',str(self.bundle),'--rc-kanit-sha256',receipt['manifest_sha256']);self.assertEqual(result.returncode,0,result.stderr)
        self.assertIn('clippy --locked --all-targets -- -D warnings',trace);self.assertIn('test --locked --no-fail-fast',trace)
        result,_=self.invoke_gate('--surum-adayi');self.assertNotEqual(result.returncode,0)
    def test_source_pinned_before_first_tool_command(self):
        receipt=self.export();source=self.source['sha'];put(self.root/'different-tree',b'not fuzzed')
        self.git('add','.');self.git('-c','user.name=Protocol','-c','user.email=protocol@example.invalid','commit','-qm','other clean source tree')
        self.assertNotEqual(V.identity(self.root),self.source)
        result,_=self.invoke_gate('--surum-adayi','--rc-kanit',str(self.bundle),'--rc-kanit-sha256',receipt['manifest_sha256'],swap_sha=source)
        self.assertNotEqual(result.returncode,0);self.assertIn('erken'.encode(),result.stderr)
    def test_explicit_invalid_never_falls_back_and_invalid_modes(self):
        receipt=self.export()
        for args in [('--surum-adayi','--rc-kanit',str(self.bundle),'--rc-kanit-sha256','0'*64),('--surekli','--rc-kanit',str(self.bundle),'--rc-kanit-sha256',receipt['manifest_sha256']),('--surum-adayi','--rc-kanit',str(self.bundle)),('--surum-adayi','--rc-kanit','', '--rc-kanit',str(self.bundle),'--rc-kanit-sha256',receipt['manifest_sha256'])]:
            result,_=self.invoke_gate(*args);self.assertNotEqual(result.returncode,0)

if __name__=='__main__':unittest.main()
