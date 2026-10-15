#!/usr/bin/env python3
"""K279: immutable v0.7 inputs, explicit K086 migration, no legacy rollback claim."""
import argparse, hashlib, importlib.util, json, os, re, signal, stat, tempfile, time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / 'compiler/tests/fixtures/onceki-kucuk-surum-v07'
HELPER = ROOT / 'scripts/lifecycle_core.py'
HELPER_SHA = 'e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9'

def require(value):
    if not value:
        raise ValueError('previous-minor-profile-rejected')

def bounded(path, maximum):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        before = os.fstat(fd)
        require(stat.S_ISREG(before.st_mode) and before.st_size <= maximum)
        data = bytearray()
        while len(data) <= maximum:
            part = os.read(fd, min(8192, maximum + 1 - len(data)))
            if not part:
                break
            data.extend(part)
        key = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        require(len(data) <= maximum and key(before) == key(os.fstat(fd)) == key(path.lstat()))
        return bytes(data)
    finally:
        os.close(fd)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def pairs(rows):
    result = {}
    for key, value in rows:
        require(key not in result)
        result[key] = value
    return result

def fixtures():
    raw = bounded(FIXTURES / 'manifest.json', 32768)
    m = json.loads(raw, object_pairs_hook=pairs)
    require(m['schema'] == 'zee-previous-minor-corpus-1' and m['source_count'] == len(m['cases']) == 33)
    require(m['historical_source'] == 'd08a947713ddc38d2163360c17d07fbffcb1213d')
    require(len(m['pure_runtime']) == 15)
    records = {}
    for row in m['cases']:
        name = row['file']
        require(re.fullmatch(r'[A-Za-z0-9_-]+\.dil', name) and name not in records)
        data = bounded(FIXTURES / name, 65536)
        require(digest(data) == row['sha256'])
        records[name] = data
    observed = set()
    with os.scandir(FIXTURES) as entries:
        for entry in entries:
            require(len(observed) < 34 and entry.name not in observed)
            observed.add(entry.name)
    require(observed == set(records) | {'manifest.json'})
    return raw, m, records

def main():
    p = argparse.ArgumentParser(allow_abbrev=False)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--expected-source', required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    require(re.fullmatch('[0-9a-f]{40}', a.expected_source))
    manifest_raw, m, records = fixtures()  # Validate all fixed sources before starting any compiler.
    helper = bounded(HELPER, 16384)
    require(digest(helper) == HELPER_SHA)
    owner = importlib.util.module_from_spec(importlib.util.spec_from_loader('owned', loader=None))
    exec(compile(helper, str(HELPER), 'exec'), owner.__dict__)
    def cancel(number, frame):
        owner.CANCELLED = True
    for number in (signal.SIGTERM, signal.SIGINT):
        signal.signal(number, cancel)
    binary = a.binary.resolve(strict=True)
    pin = owner.digest(binary)
    out = a.output.resolve()
    out.mkdir(mode=0o700)  # Fresh directory only; never overwrite an earlier receipt.
    # Linux retained-leader cleanup costs about 3s per successful command (52 commands).
    # This is only the test campaign budget; native 8s and owned cleanup stay unchanged.
    deadline = time.monotonic() + 240
    proof = {'schema': 'zee-previous-minor-tests-1', 'status': 'running', 'cases': [],
             'binary_sha256': pin, 'historical_source': m['historical_source'],
             'unmodified_corpus_all_compatible': False, 'legacy_rollback_accepted': False}
    # The fixed shell only observes exit status; native argv is passed as separate "$@" arguments.
    shell = '"$@"; zee_status=$?; printf \'\\nZEE_EXIT:%d\\n\' "$zee_status"'
    def command(name, argv):
        left = deadline - time.monotonic()
        require(left > 0)
        raw = owner.run(['/bin/sh', '-c', shell, 'zee-owned-status', *map(str, argv)],
                        cwd=out, timeout=min(8, left), limit=262144, log_path=out / (name + '.log'))
        match = re.search(rb'\nZEE_EXIT:([0-9]+)\n$', raw)
        require(match is not None)
        return int(match[1]), raw[:match.start()]
    try:
        code, raw = command('build', [binary, 'sürüm', '--json'])
        build = json.loads(raw, object_pairs_hook=pairs)
        require(code == 0 and build['schema'] == 'zee-build-v1' and build['source_sha'] == a.expected_source)
        proof['build'] = build
        for index, row in enumerate(m['cases']):
            code, raw = command('check-' + str(index), [binary, 'denetle', FIXTURES / row['file']])
            require(code == row['new_exit'])
            if row['file'] == m['migration']['file']:
                require(m['migration']['expected_diagnostic'].encode() in raw)
            proof['cases'].append({'kind': 'check', 'file': row['file'], 'exit': code, 'passed': True})
        for index, row in enumerate(m['pure_runtime']):
            code, raw = command('run-' + str(index), [binary, 'çalıştır', FIXTURES / row['file']])
            require(code == 0 and digest(raw) == row['combined_output_sha256'])
            proof['cases'].append({'kind': 'pure-runtime', 'file': row['file'], 'passed': True})
        migration = m['migration']
        with tempfile.TemporaryDirectory(prefix='zee-k279-migration-') as temporary:
            target = Path(temporary).resolve()
            (target / migration['file']).write_bytes(records[migration['file']])
            original = records[migration['unit']].decode()
            require(original.count(migration['old_line']) == 1)
            (target / migration['unit']).write_text(original.replace(migration['old_line'], migration['new_lines']))
            code, raw = command('migration-check', [binary, 'denetle', target / migration['file']])
            require(code == 0)
            code, raw = command('migration-runtime', [binary, 'çalıştır', target / migration['file']])
            require(code == 0 and digest(raw) == migration['combined_output_sha256'])
        proof['cases'].append({'kind': 'known-K086-migration', 'file': migration['file'], 'passed': True})
        code, raw = command('config-capability', [binary, 'sürüm', '--ayar-semasi'])
        capability = json.loads(raw, object_pairs_hook=pairs)
        require(code == 0 and capability == {'schema': 'zee-config-capability-1', 'config_schemas': ['zee-ayar-1'],
                'unknown_security': 'reject', 'version': build['version']})
        require(owner.digest(binary) == pin and fixtures()[0] == manifest_raw and bounded(HELPER, 16384) == helper)
        require(time.monotonic() <= deadline)
        proof.update(status='passed', case_count=49, source_pins_unchanged=True, owned_commands_closed=True)
    except BaseException as error:
        proof.update(status='failed', failure_type=type(error).__name__)
        raise
    finally:
        (out / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n')

if __name__ == '__main__':
    main()
