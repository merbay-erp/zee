#!/usr/bin/env python3
"""K265 canonical local RC producer. Requires explicit stable-SHA dispatch."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import datetime
import hashlib
import json
import os
import platform
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import threading
import time

TOOLCHAIN = 'nightly-2026-08-31'
FUZZ_VERSION = 'cargo-fuzz 0.13.2'
TARGETS = [('lexer_parser', 65536), ('morfoloji', 128),
           ('http_istegi', 81920), ('wasm_abi', 4097)]
STOP = threading.Event()
RUN_DIR = None
ACTIVE_PROOF = None


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()


def write(path, value):
    with path.open('x', encoding='utf-8') as output:
        json.dump(value, output, ensure_ascii=False, indent=2)
        output.write('\n')


def query(args, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, timeout=30)
    require(result.returncode == 0, 'read-only tool/source query failed')
    return result.stdout.decode().strip()


def source(repo, sha):
    actual = query(['git', '-C', str(repo), 'rev-parse', 'HEAD'])
    status = query(['git', '-C', str(repo), 'status', '--porcelain', '--untracked-files=all', '--', '.'])
    require(actual == sha and not status, 'exact source must remain clean')
    return {'sha': actual, 'tree': query(['git', '-C', str(repo), 'rev-parse', 'HEAD^{tree}']), 'clean': True}


def observed_exit(child):
    # Keep the group leader unreaped until group cleanup; no numeric PID adoption.
    info = os.waitid(os.P_PID, child.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
    if info is None:
        return None
    return info.si_status if info.si_code == os.CLD_EXITED else -info.si_status


def group_signal(child, sig, exited):
    try:
        os.killpg(child.pid, sig)
    except ProcessLookupError:
        pass
    except PermissionError:
        # Darwin can report EPERM for the exited, held leader's empty group.
        require(exited, 'owned live group signal was denied')


def command(name, args, cwd, env, logfile, timeout):
    start = time.monotonic()
    reason = None
    if STOP.is_set():
        logfile.write_bytes(b'')
        return {'name': name, 'exit': None, 'elapsed_seconds': 0,
                'supervisor_failure': 'interrupted-before-start',
                'log_sha256': digest(logfile), 'passed': False}
    with logfile.open('xb') as output:
        child = subprocess.Popen(args, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                 stdout=output, stderr=subprocess.STDOUT,
                                 start_new_session=True, close_fds=True)
        try:
            while observed_exit(child) is None:
                if STOP.is_set():
                    reason = 'interrupted'
                    break
                if time.monotonic() - start > timeout:
                    reason = 'outer-deadline'
                    break
                if logfile.stat().st_size > 512 * 1024 * 1024:
                    reason = 'log-budget'
                    break
                time.sleep(.1)
        finally:
            exited = observed_exit(child) is not None
            group_signal(child, signal.SIGTERM, exited)
            if not exited:
                grace = time.monotonic() + 2
                while observed_exit(child) is None and time.monotonic() < grace:
                    time.sleep(.05)
            group_signal(child, signal.SIGKILL, observed_exit(child) is not None)
            code = child.wait(timeout=10)
    return {'name': name, 'argv': args, 'exit': code, 'elapsed_seconds': round(time.monotonic() - start, 3),
            'supervisor_failure': reason, 'log_sha256': digest(logfile),
            'passed': code == 0 and reason is None}


def fuzz_stats(text, command_result, crash_files):
    done = re.findall(r'Done\s+(\d+)\s+runs in\s+(\d+)\s+second', text)
    stats = re.findall(r'stat::number_of_executed_units:\s*(\d+)', text)
    sanitizer = bool(re.search(r'(?:ERROR|SUMMARY):\s*AddressSanitizer|AddressSanitizer:DEADLYSIGNAL', text))
    failure = bool(re.search(r'ERROR:\s*libFuzzer|libFuzzer:\s*deadly signal|SUMMARY:\s*UndefinedBehaviorSanitizer', text))
    timeout = bool(re.search(r'ERROR:\s*libFuzzer:\s*timeout|TIMEOUT|timeout-', text))
    actual = int(done[-1][1]) if done else None
    executions = int(done[-1][0]) if done else None
    passed = (command_result['passed'] and done and stats and actual >= 1800
              and actual <= 3600 and executions > 0 and int(stats[-1]) > 0
              and not sanitizer and not failure and not timeout and not crash_files)
    return {'requested_seconds': 1800, 'actual_seconds': actual, 'executions': executions,
            'sanitizer': 'address', 'sanitizer_finding': sanitizer,
            'libfuzzer_failure': failure, 'input_timeout': timeout,
            'crash_artifacts': crash_files, 'passed': bool(passed)}


def manifest(target, package, sha, run_id, name='manifest.tsv'):
    rows = []
    for path in sorted((package / 'corpus').rglob('*')):
        if path.is_file():
            relative = path.relative_to(package / 'corpus').as_posix()
            require(not path.is_symlink() and '\t' not in relative and '\n' not in relative, 'unsafe seed path')
            rows.append(digest(path) + '\t' + relative)
    headers = ['# zee-fuzz-corpus-artifact-1', '# target\t' + target, '# corpus\t' + target,
               '# source_commit\t' + sha, '# run_id\t' + run_id, '# run_attempt\t1',
               '# toolchain\t' + TOOLCHAIN, '# cargo_fuzz\t0.13.2', '# run_origin\tlocal',
               '# seed_sha256\trelative_path']
    with (package / name).open('x', encoding='utf-8') as output:
        output.write('\n'.join(headers + rows) + '\n')
    return len(rows)


def main():
    global RUN_DIR, ACTIVE_PROOF
    parser = argparse.ArgumentParser(allow_abbrev=False)
    parser.add_argument('--repo', required=True)
    parser.add_argument('--source-sha', required=True)
    parser.add_argument('--out', required=True)
    parser.add_argument('--execute-approved-stable-sha', action='store_true')
    args = parser.parse_args()
    require(args.execute_approved_stable_sha, 'explicit stable-SHA dispatch is required')
    require(sys.flags.isolated and sys.dont_write_bytecode, 'use Python -I -B')
    require(os.name == 'posix' and all(hasattr(os, name) for name in ('waitid', 'WNOWAIT', 'WEXITED')), 'owned POSIX process support required')
    require(re.fullmatch('[0-9a-f]{40}', args.source_sha), 'exact full source SHA required')
    repo = Path(args.repo).resolve(strict=True)
    out = Path(args.out)
    require(out.is_absolute() and not out.exists() and not out.is_symlink(), 'new absolute output directory required')
    out = out.resolve(strict=False)
    require(not out.is_relative_to(repo), 'evidence must stay outside source')
    require(platform.system() == 'Darwin' and platform.machine() == 'arm64', 'reviewed local producer requires macOS arm64')
    original = source(repo, args.source_sha)
    producer = Path(__file__).resolve()
    expected_producer = subprocess.check_output(['git', '-C', str(repo), 'show', args.source_sha + ':scripts/rc-kampanyasi.py'], timeout=30)
    require(hashlib.sha256(expected_producer).hexdigest() == digest(producer), 'producer must match exact source')
    out.mkdir(mode=0o700)
    RUN_DIR = out
    (out / 'logs').mkdir(mode=0o700)
    (out / 'tmp').mkdir(mode=0o700)
    clone = out / 'checkout'
    subprocess.run(['git', 'clone', '-q', '--local', '--no-hardlinks', str(repo), str(clone)], check=True)
    subprocess.run(['git', '-C', str(clone), 'checkout', '-q', args.source_sha], check=True)
    start_source = source(clone, args.source_sha)
    env = {key: value for key, value in os.environ.items()
           if key in {'PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME', 'SDKROOT', 'DEVELOPER_DIR'}}
    env.update({'CARGO_NET_OFFLINE': 'true', 'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
                'RUSTUP_TOOLCHAIN': TOOLCHAIN, 'RUSTC_WRAPPER': '', 'RUSTC_WORKSPACE_WRAPPER': '',
                'TMPDIR': str(out / 'tmp')})
    env['RUSTC'] = query(['rustup', 'which', '--toolchain', TOOLCHAIN, 'rustc'])
    require(query(['cargo', 'fuzz', '--version']) == FUZZ_VERSION, 'cargo-fuzz pin differs')
    tools = {'toolchain': TOOLCHAIN, 'rustc': query([env['RUSTC'], '-vV']), 'cargo_fuzz': FUZZ_VERSION,
             'miri': query(['cargo', '+' + TOOLCHAIN, 'miri', '--version'])}
    run_id = str(time.time_ns())
    proof = {'schema': 'zee-local-asan-miri-campaign-1', 'source_sha': args.source_sha,
             'source_start': start_source, 'original_source_start': original,
             'run_origin': 'local', 'run_id': run_id, 'run_attempt': 1, 'tools': tools,
             'producer_sha256': digest(producer), 'producer_path': 'scripts/rc-kampanyasi.py', 'host_target': 'aarch64-apple-darwin',
             'platform': {'system': platform.system(), 'release': platform.release(), 'os_version': platform.mac_ver()[0], 'machine': platform.machine(), 'processor': query(['sysctl', '-n', 'machdep.cpu.brand_string']), 'ram_bytes': int(query(['sysctl', '-n', 'hw.memsize']))},
             'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
             'maximum_concurrent_fuzz_targets': 2, 'targets': [], 'miri': [],
             'result': 'incomplete', 'remote_ci': None, 'signing': False, 'publication': False,
             'release_gate_closed': False, 'seed_imported': False,
             'workflow_retention_days': 90, 'local_artifact_uploaded': False}
    ACTIVE_PROOF = proof
    write(out / 'started-proof.json', proof)
    compiler = clone / 'compiler'
    target_dir = out / 'fuzz-target'
    common = ['cargo', '+' + TOOLCHAIN, 'fuzz']
    builds = {}
    for target, _ in TARGETS:
        build_log = out / ('logs/asan-build-' + target + '.log')
        build = command('asan-build-' + target, common + ['build', '--verbose', '--sanitizer', 'address',
                        '--target', 'aarch64-apple-darwin', '--target-dir', str(target_dir), target],
                        compiler, env, build_log, 1800)
        build['explicit_sanitizer_flag'] = 'sanitizer=address' in build_log.read_text(errors='replace')
        build['passed'] = build['passed'] and build['explicit_sanitizer_flag']
        builds[target] = build
        source(clone, args.source_sha)
    proof['builds'] = builds
    if any(record['passed'] for record in builds.values()):
        def run_target(item):
            target, maximum = item
            if not builds[target]['passed']:
                return {'target': target, 'stats': {'passed': False}, 'build_failure': True}
            package = out / 'targets' / target
            package.mkdir(parents=True, mode=0o700)
            for seed in (compiler / 'fuzz/corpus' / target).rglob('*'):
                require(not seed.is_symlink() and (seed.is_file() or seed.is_dir()), 'unsafe original seed path')
            shutil.copytree(compiler / 'fuzz/corpus' / target, package / 'corpus', symlinks=False)
            (package / 'crashes').mkdir()
            initial_count = manifest(target, package, args.source_sha, run_id, 'initial-manifest.tsv')
            binary = target_dir / 'aarch64-apple-darwin/release' / target
            symbols = query(['nm', str(binary)])
            require('asan_init' in symbols, 'built target ASan runtime marker missing')
            command_args = common + ['run', '--sanitizer', 'address', '--target', 'aarch64-apple-darwin',
                            '--target-dir', str(target_dir), target, str(package / 'corpus'), '--',
                            '-dict=' + str(compiler / 'fuzz/dictionaries/zee.dict'), '-max_total_time=1800',
                            '-max_len=' + str(maximum), '-timeout=5', '-print_final_stats=1',
                            '-artifact_prefix=' + str(package / 'crashes') + '/']
            result = command(target, command_args, compiler, env, package / 'run.log', 2400)
            final_count = manifest(target, package, args.source_sha, run_id)
            verification = command(target + '-seed-manifest', ['bash', str(clone / 'scripts/fuzz-korpus-artefakti-dogrula.sh'),
                                   target, str(package)], clone, env, package / 'manifest-verification.log', 60)
            crash_files = [{'path': path.name, 'sha256': digest(path)}
                           for path in sorted((package / 'crashes').iterdir()) if path.is_file()]
            stats = fuzz_stats((package / 'run.log').read_text(errors='replace'), result, crash_files)
            stats['passed'] = stats['passed'] and verification['passed']
            record = {'target': target, 'max_len': maximum, 'command': result, 'stats': stats,
                      'binary_sha256': digest(binary), 'asan_runtime_marker': True,
                      'initial_seeds': initial_count, 'final_seeds': final_count,
                      'manifest_verified': verification['passed'], 'manifest_sha256': digest(package / 'manifest.tsv')}
            with (package / 'sonuc.tsv').open('x') as output:
                output.write('# zee-fuzz-rc-sonucu-1\n')
                for key, value in [('target', target), ('source_commit', args.source_sha), ('run_id', run_id),
                                   ('run_attempt', 1), ('run_origin', 'local'), ('requested_seconds', 1800),
                                   ('sanitizer', 'address'), ('exit_code', result['exit'])]:
                    output.write(str(key) + '\t' + str(value) + '\n')
            write(package / 'result.json', record)
            return record
        with ThreadPoolExecutor(max_workers=2) as executor:
            futures = [(item[0], executor.submit(run_target, item)) for item in TARGETS]
            for target, future in futures:
                try:
                    proof['targets'].append(future.result())
                except Exception:
                    proof['targets'].append({'target': target, 'stats': {'passed': False}, 'driver_failure': True})
    else:
        proof['targets'] = [{'target': target, 'stats': {'passed': False}, 'build_failure': True}
                            for target, _ in TARGETS]
    for pattern in ('ondalik::testler::', 'zaman::testler::'):
        miri_env = dict(env, CARGO_TARGET_DIR=str(out / 'miri-target'))
        record = command('miri-' + pattern.split(':')[0], ['cargo', '+' + TOOLCHAIN, 'miri', 'test', '--locked', '--lib', pattern],
                         compiler, miri_env, out / ('logs/miri-' + pattern.split(':')[0] + '.log'), 1800)
        text = (out / ('logs/miri-' + pattern.split(':')[0] + '.log')).read_text(errors='replace')
        totals = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', text)
        record['actual_tests'] = int(totals[-1][0]) if totals else 0
        record['passed'] = bool(record['passed'] and totals and record['actual_tests'] > 0
                                and totals[-1][1:] == ('0', '0'))
        proof['miri'].append(record)
    proof['source_end'] = source(clone, args.source_sha)
    proof['original_source_end'] = source(repo, args.source_sha)
    proof['source_unchanged'] = proof['source_start'] == proof['source_end'] == proof['original_source_start'] == proof['original_source_end']
    proof['result'] = ('passed-local-technical-campaign' if all(x['passed'] for x in builds.values()) and len(proof['targets']) == 4
                       and all(x['stats']['passed'] for x in proof['targets']) and len(proof['miri']) == 2
                       and all(x['passed'] for x in proof['miri']) and proof['source_unchanged'] else 'failed')
    proof['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    write(out / 'campaign-proof.json', proof)
    print(json.dumps({'result': proof['result'], 'source_sha': args.source_sha,
                      'targets': len(proof['targets']), 'miri_groups': len(proof['miri'])}))
    return 0 if proof['result'] == 'passed-local-technical-campaign' else 1


if __name__ == '__main__':
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, lambda *_: STOP.set())
    try:
        sys.exit(main())
    except Exception as error:
        if RUN_DIR is not None:
            failed = dict(ACTIVE_PROOF or {'schema': 'zee-local-asan-miri-driver-failure-1'})
            failed.update({'result': 'failed-driver', 'driver_error_type': type(error).__name__,
                           'finished_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()})
            write(RUN_DIR / 'driver-failure.json', failed)
        sys.stderr.write('Local RC driver failed; available raw evidence retained.\n')
        sys.exit(1)
