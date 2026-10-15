import json
import re
import runpy
import signal
import sys
import threading
import time
from pathlib import Path

E = Path('/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme')
R = Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014')
S = '66e4149e53451111d153f47283c219079a8d1cab'
H = runpy.run_path(str(R / 'scripts/lifecycle_core.py'))

def install_cancellation(proof):
    def cancel(number, _frame):
        proof['cancellation_signal'] = number
        # runpy's returned mapping is not the functions' globals dictionary.
        H['run'].__globals__['CANCELLED'] = True
    for number in (signal.SIGTERM, signal.SIGINT):
        signal.signal(number, cancel)

if sys.argv[1:] == ['--self-test']:
    O = E / 's3-gate-v2-cancel-self-test'
    assert not O.exists()
    O.mkdir(mode=0o700)
    proof = {'schema': 'zee-owned-gate-cooperative-cancellation-test-1'}
    install_cancellation(proof)
    original_stop = H['stop_owned']
    def audited_stop(child, *args, **kwargs):
        result = original_stop(child, *args, **kwargs)
        proof['original_child_reaped'] = child.returncode is not None
        proof['original_child_exit'] = child.returncode
        return result
    H['run'].__globals__['stop_owned'] = audited_stop
    child_code = "import signal,sys,time;from pathlib import Path\ndef done(*_):\n Path(sys.argv[1]).write_text('owned SIGTERM cleanup\\n');sys.exit(0)\nsignal.signal(signal.SIGTERM,done);print('ready',flush=True);time.sleep(30)"
    timer = threading.Timer(0.5, lambda: signal.raise_signal(signal.SIGTERM))
    timer.start()
    try:
        H['run']([sys.executable, '-I', '-B', '-c', child_code, O / 'child-cleanup.txt'], timeout=5, log_path=O / 'partial.txt')
        raise AssertionError('cancellation did not interrupt the command')
    except H['Reject']:
        assert proof['cancellation_signal'] == signal.SIGTERM
        assert proof['original_child_reaped'] and proof['original_child_exit'] == 0
        assert (O / 'child-cleanup.txt').read_text() == 'owned SIGTERM cleanup\n'
        assert (O / 'partial.txt').read_text() == 'ready\n'
        proof.update(status='passed', partial_log_preserved=True, original_handle_only=True)
    finally:
        timer.cancel()
        timer.join()
        (O / 'PROOF.json').write_text(json.dumps(proof, indent=2) + '\n')
    print(json.dumps(proof))
    raise SystemExit(0)

assert not sys.argv[1:]
controller = json.loads((E / 's3-rc-controller.json').read_text())
assert controller['status'] == 'passed' and controller['owned_group_closed_and_reaped']
campaign = json.loads((E / 's3-66e4149-rc/campaign-proof.json').read_text())
assert campaign['result'] == 'passed-local-technical-campaign' and campaign['source_sha'] == S
assert json.loads((E / 's3-gate-v2-cancel-self-test/PROOF.json').read_text())['status'] == 'passed'
first = json.loads((E / 's3-rc-gate-controller.json').read_text())
assert first['status'] == 'failed' and first['cancellation_reason'] == 'disk-or-log-budget'
H['clean_exact'](R, S)
O = E / 's3-66e4149-rc-gate-v2'
assert not O.exists()
O.mkdir(mode=0o700)
bundle = E / 's3-66e4149-rc-bundle'
manifest = H['digest'](bundle / 'manifest.json')
assert manifest == '1dba993afeaccbe11b9daabffba5c477d1371f044c9ea27dbc44aafe392c6fa0'
env = H['controlled_env']()
env.update(RUSTUP_TOOLCHAIN='1.93.1', CARGO_BUILD_JOBS='1', CARGO_NET_OFFLINE='true', PYTHONDONTWRITEBYTECODE='1')
proof = {'schema': 'zee-exact-source-post-RC-gate-2', 'source_sha': S, 'status': 'running',
         'started_ns': time.time_ns(), 'source_begin_clean': True, 'commands': [],
         'prior_attempt': 's3-66e4149-rc-gate/CANCELLATION-REVIEW.json',
         'bundle_reused_unchanged': True, 'export_attempt': 's3-66e4149-rc-gate/export.txt',
         'cooperative_cancellation_test': 's3-gate-v2-cancel-self-test/PROOF.json'}
install_cancellation(proof)

def save():
    (O / 'GATE-RECEIPT.json').write_text(json.dumps(proof, indent=2) + '\n')

def run(name, argv, timeout):
    record = {'name': name, 'argv': list(map(str, argv)), 'status': 'running'}
    proof['commands'].append(record)
    save()
    print(name + ': started', flush=True)
    start = time.monotonic()
    try:
        raw = H['run'](argv, cwd=R, env=env, timeout=timeout, limit=16*1024**2, log_path=O/(name+'.txt'))
        record.update(exit=0, status='passed', seconds=time.monotonic()-start,
                      log_sha256=H['digest'](O/(name+'.txt')))
        print(name + ': passed', flush=True)
        return raw.decode()
    except BaseException as error:
        record.update(status='failed', failure_type=type(error).__name__, seconds=time.monotonic()-start)
        if (O/(name+'.txt')).exists():
            record['log_sha256'] = H['digest'](O/(name+'.txt'))
        raise
    finally:
        save()

try:
    raw = run('check-before', ['python3', '-I', '-B', R/'scripts/rc-kaniti.py', 'check', '--framework-root', R, '--bundle', bundle, '--sha256', manifest], 300)
    assert json.loads(raw)['status'] == 'validated-integrity-and-policy'
    raw = run('security-RC', ['bash', R/'scripts/guvenlik-kapisi.sh', '--surum-adayi', '--rc-kanit', bundle, '--rc-kanit-sha256', manifest], 2400)
    body = raw.split('-- Clippy ve tam test paketi', 1)[1].split('-- Yayımlanmış kanıt korukları', 1)[0]
    totals = [tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', body)]
    assert totals and all(t[1] == 0 for t in totals)
    H['clean_exact'](R, S)
    proof.update(status='passed', source_end_clean=True, source_end_sha=S, bundle_manifest_sha256=manifest,
                 bundle_files=len(json.loads((bundle/'manifest.json').read_text())['files']),
                 full_cargo_groups=len(totals), full_cargo_passed=sum(t[0] for t in totals),
                 full_cargo_failed=sum(t[1] for t in totals), full_cargo_ignored=sum(t[2] for t in totals),
                 export_check_source_exact=True)
except BaseException as error:
    proof.update(status='failed', failure_type=type(error).__name__)
    raise
finally:
    proof['finished_ns'] = time.time_ns()
    save()
print(json.dumps({k: proof[k] for k in ['status','source_sha','bundle_manifest_sha256','full_cargo_groups','full_cargo_passed','full_cargo_failed','full_cargo_ignored']}))
