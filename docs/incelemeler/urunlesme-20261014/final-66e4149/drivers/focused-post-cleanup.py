import json, os, re, runpy, shutil, signal, time
from pathlib import Path
E=Path('/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme')
R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014')
S='66e4149e53451111d153f47283c219079a8d1cab'
H=runpy.run_path(str(R/'scripts/lifecycle_core.py'))
O=E/'s3-66e4149-focused-post-cleanup'; assert not O.exists(); O.mkdir(mode=0o700)
H['clean_exact'](R,S)
binary=R/'compiler/target/debug/dil'
proof=dict(source_sha=S,status='running',source_begin_clean=True,binary_begin_sha256=H['digest'](binary),test_threads=1,source_or_deadline_modified=False,started_ns=time.time_ns(),environment_change='User confirmed other-session cleanup completed before this attempt',free_bytes=shutil.disk_usage(R).free,load_average=os.getloadavg())
def save(): (O/'PROOF.json').write_text(json.dumps(proof,indent=2)+'\n')
def cancel(number,_frame):
    proof['cancellation_signal']=number
    H['run'].__globals__['CANCELLED']=True
for number in (signal.SIGTERM,signal.SIGINT): signal.signal(number,cancel)
env=H['controlled_env']();env.update(RUSTUP_TOOLCHAIN='1.93.1',CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true',PYTHONDONTWRITEBYTECODE='1')
save()
try:
    raw=H['run'](['cargo','test','--locked','--test','native_listener_testi','--test','worker_supervisor_testi','--','--test-threads=1'],cwd=R/'compiler',env=env,timeout=600,log_path=O/'focused.txt').decode()
    totals=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)]
    assert totals==[(3,0,0),(1,0,0)],totals
    H['clean_exact'](R,S)
    assert H['digest'](binary)==proof['binary_begin_sha256']
    proof.update(status='passed',passed=4,failed=0,ignored=0,binary_unchanged=True,source_end_clean=True)
except BaseException as error:
    proof.update(status='failed',failure_type=type(error).__name__)
    raise
finally:
    proof['finished_ns']=time.time_ns()
    if (O/'focused.txt').exists(): proof['log_sha256']=H['digest'](O/'focused.txt')
    save()
print(json.dumps(proof))
