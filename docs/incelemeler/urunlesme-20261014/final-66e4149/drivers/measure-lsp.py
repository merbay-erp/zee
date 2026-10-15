import hashlib,json,math,os,platform,runpy,time
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');S='66e4149e53451111d153f47283c219079a8d1cab';H=runpy.run_path(str(R/'scripts/lifecycle_core.py'))
# Run only after fuzz and dedicated native work release their original handles.
for n in ['s3-rc-controller.json','s3-retained-native-controller.json']:
 v=json.loads((E/n).read_text());assert v['status']=='passed' and v['owned_group_closed_and_reaped']
O=E/'s3-66e4149-lsp-measurement';assert not O.exists();O.mkdir(mode=0o700)
B=E/'s3-66e4149-common/lsp_artimli_testi-release';build=json.loads((B.parent/'LSP-PERF-BUILD.json').read_text());pin=H['digest'](B)
assert build['source_sha']==S and build['source_clean'] and pin==build['binary_sha256'] and build['profile']['opt_level']=='3' and not build['profile']['debug_assertions']
H['clean_exact'](R,S);start=time.monotonic();env=H['controlled_env']()
raw=H['run']([B,'yirmi_bin_satir_proje_duzenleme_olcumu','--exact','--ignored','--nocapture'],cwd=R/'compiler',env=env,timeout=120,limit=1024*1024,log_path=O/'raw-test-output.txt').decode()
rows=[]
for line in raw.splitlines():
 try:v=json.loads(line)
 except ValueError:continue
 if isinstance(v,dict) and v.get('schema')=='zee-lsp-edit-measurement-1':
  values=v['samples_ms'];assert len(values)==20 and all(type(x) in [int,float] and math.isfinite(x) and x>=0 for x in values)
  assert v['lines']==20081 and v['modules']==40 and v['warmup_edits']==5 and v['p95_ms']==sorted(values)[18] and v['max_ms']==max(values);rows.append(v)
assert [v['open_documents'] for v in rows]==[1,41] and '1 passed; 0 failed; 0 ignored' in raw
H['clean_exact'](R,S);assert H['digest'](B)==pin
cpu=H['run'](['sysctl','-n','machdep.cpu.brand_string'],timeout=10,limit=10000).decode().strip();ram=int(H['run'](['sysctl','-n','hw.memsize'],timeout=10,limit=10000));rust_path=H['run'](['rustup','which','--toolchain','1.93.1','rustc'],timeout=10,limit=10000).decode().strip();rustc=H['run']([rust_path,'-vV'],timeout=10,limit=10000).decode();assert rustc.startswith('rustc 1.93.1 ')
proof={'schema':'zee-exact-source-LSP-observation-1','source_sha':S,'source_begin_end_clean':True,'binary_sha256':pin,'build_receipt_sha256':H['digest'](B.parent/'LSP-PERF-BUILD.json'),'profile':build['profile'],'machine':{'os':platform.platform(),'cpu':cpu,'ram_bytes':ram,'cpu_count':os.cpu_count()},'rustc_vV':rustc,'measurements':rows,'warmup_semantics':'first five of25 alternating ranged edits discarded per cohort; next20 retained; open1 and open41 separately','p95_independently_recomputed':True,'shared_machine_observational':True,'hard_200ms_gate_applied':False,'fresh_graph_lock_loader_binding_type_and_serialized_diagnostics':True,'claims_excluded':['real-product or global SLA','editor transport/UI','lint opt-in latency','incremental HIR/type'],'elapsed_seconds':time.monotonic()-start,'raw_sha256':H['digest'](O/'raw-test-output.txt')}
(O/'MEASUREMENT.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'source_sha':S,'observations':[{'open':v['open_documents'],'p95_ms':v['p95_ms'],'max_ms':v['max_ms'],'cold_ms':v['cold_ms']} for v in rows]},ensure_ascii=False))
