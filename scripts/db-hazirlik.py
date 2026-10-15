#!/usr/bin/env python3
"""K-251/K-274: owned once probe or explicitly finite repeated readiness checks."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import selectors
import signal
import shutil
import stat
import subprocess
import sys
import tempfile
import time

shared = runpy.run_path(str(Path(__file__).with_name('worker-supervisor.py')))
Reject, require = shared['Reject'], shared['require']
regular_bytes, decode_json = shared['regular_bytes'], shared['decode_json']
HEX40 = re.compile(r'[a-f0-9]{40}\Z')
HEX64 = re.compile(r'[a-f0-9]{64}\Z')
SOURCE_NAME = re.compile(r'(?:[a-zA-Z0-9_-]+/)*[a-zA-Z0-9_.-]+\Z')
MIGRATION_NAME = re.compile(r'[0-9]+_[a-zA-Z0-9_-]+\.sql\Z')


class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise Reject('input')


def file_key(path):
    s = path.lstat()
    return (s.st_dev, s.st_ino, s.st_mode, s.st_uid, s.st_nlink,
            s.st_size, s.st_mtime_ns, s.st_ctime_ns)


def sha(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(65536), b''):
            value.update(chunk)
    return value.hexdigest()


def canonical_directory(value):
    path = Path(value)
    require(path.is_absolute() and path.resolve(strict=True) == path, 'input')
    s = path.lstat()
    require(stat.S_ISDIR(s.st_mode) and s.st_uid in {0, os.geteuid()} and not s.st_mode & 0o022, 'input')
    return path


class Gate:
    def __init__(self, worker, path, revision):
        self.parent = canonical_directory(path.parent)
        require(self.parent.is_relative_to(worker) and path.suffix == '.json' and len(path.name) <= 128, 'gate')
        self.path, self.revision = path, revision
        self.fd = os.open(self.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
        self.lock = None
        try:
            self.lock = os.open(path.name+'.lock', os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, 0o600, dir_fd=self.fd)
            s = os.fstat(self.lock)
            require(stat.S_ISREG(s.st_mode) and s.st_uid in {0, os.geteuid()} and s.st_nlink == 1 and not s.st_mode & 0o077, 'gate')
            fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BaseException:
            self.close()
            raise

    def valid_parent(self):
        opened, current = os.fstat(self.fd), self.parent.lstat()
        key = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_uid)
        require(key(opened) == key(current) and self.parent.resolve(strict=True) == self.parent, 'gate')

    def write(self, ready):
        self.valid_parent()
        if self.path.exists() or self.path.is_symlink():
            s = self.path.lstat()
            require(stat.S_ISREG(s.st_mode) and s.st_uid in {0, os.geteuid()} and s.st_nlink == 1 and not s.st_mode & 0o022, 'gate')
        data = json.dumps({'schema': 'zee-worker-readiness-1', 'application_revision': self.revision, 'ready': ready}, separators=(',', ':')).encode()
        name = '.'+self.path.name+'.'+os.urandom(8).hex()+'.tmp'
        fd = os.open(name, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW | os.O_CLOEXEC, 0o600, dir_fd=self.fd)
        try:
            with os.fdopen(fd, 'wb') as stream:
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
            self.valid_parent()
            os.replace(name, self.path.name, src_dir_fd=self.fd, dst_dir_fd=self.fd)
            os.fsync(self.fd)
        finally:
            try:
                os.unlink(name, dir_fd=self.fd)
            except FileNotFoundError:
                pass

    def close(self):
        if self.lock is not None:
            os.close(self.lock)
            self.lock = None
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None


def child(binary, arguments, directory, deadline, cancelled=lambda: False):
    require(not cancelled(), 'cancelled')
    require(time.monotonic() < deadline, 'timeout')
    selector = None
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    process = subprocess.Popen([str(binary), *arguments], cwd=directory, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        selector = selectors.DefaultSelector()
        for name, pipe in [('stdout', process.stdout), ('stderr', process.stderr)]:
            os.set_blocking(pipe.fileno(), False)
            selector.register(pipe, selectors.EVENT_READ, name)
        while selector.get_map():
            require(not cancelled(), 'cancelled')
            require(time.monotonic() < deadline, 'timeout')
            for key, _ in selector.select(min(.1, max(0, deadline-time.monotonic()))):
                chunk = os.read(key.fileobj.fileno(), 4097)
                if not chunk:
                    selector.unregister(key.fileobj)
                else:
                    buffers[key.data].extend(chunk)
                    require(len(buffers[key.data]) <= 65536, 'output')
        while process.poll() is None:
            require(not cancelled(), 'cancelled')
            require(time.monotonic() < deadline, 'timeout')
            time.sleep(min(.05, max(0, deadline-time.monotonic())))
        require(not cancelled(), 'cancelled')
        require(time.monotonic() < deadline, 'timeout')
        return process.returncode, bytes(buffers['stdout']), bytes(buffers['stderr'])
    finally:
        try:
            if selector is not None:
                selector.close()
        finally:
            try:
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=5)
            finally:
                try:
                    process.stdout.close()
                finally:
                    process.stderr.close()


def probe(args, gate, cancelled=lambda: False):
    deadline = time.monotonic() + args.deadline
    project = canonical_directory(args.project)
    contract_path = Path(args.contract)
    require(contract_path.is_absolute() and contract_path.resolve(strict=True) == contract_path, 'input')
    data, _ = regular_bytes(contract_path)
    require(hashlib.sha256(data).hexdigest() == args.contract_sha256, 'input')
    contract = decode_json(data)
    require(isinstance(contract, dict) and set(contract) == {'schema', 'application_revision', 'files', 'migrations'} and contract['schema'] == 'zee-db-readiness-contract-1' and contract['application_revision'] == args.application_revision, 'input')
    rows, migrations = contract['files'], contract['migrations']
    require(isinstance(rows, list) and 3 <= len(rows) <= 32 and isinstance(migrations, list) and 1 <= len(migrations) <= 16, 'input')
    paths, source_bytes, keys = [], [], []
    for row in rows:
        require(isinstance(row, dict) and set(row) == {'path', 'sha256'} and isinstance(row['path'], str) and SOURCE_NAME.fullmatch(row['path']) and '..' not in Path(row['path']).parts and HEX64.fullmatch(row['sha256']), 'input')
        path = project/row['path']
        require(path.resolve(strict=True) == path and path not in paths, 'input')
        before = file_key(path)
        content, _ = regular_bytes(path)
        require(hashlib.sha256(content).hexdigest() == row['sha256'] and file_key(path) == before, 'input')
        paths.append(path); source_bytes.append(content); keys.append(before)
    require(sum(map(len, source_bytes)) <= 262144 and project/'proje.dil' in paths, 'input')
    actual = {str(p.relative_to(project)) for p in project.rglob('*') if p.is_file()}
    require(actual == {r['path'] for r in rows}, 'input')
    plans = []
    for migration in migrations:
        require(isinstance(migration, dict) and set(migration) == {'version','name','sha256'} and type(migration['version']) is int and migration['version'] > 0 and isinstance(migration['name'], str) and MIGRATION_NAME.fullmatch(migration['name']) and HEX64.fullmatch(migration['sha256']), 'input')
        require(int(migration['name'].split('_',1)[0]) == migration['version'], 'input')
        plans.append((migration['version'], migration['name'], migration['sha256']))
    require(plans == sorted(plans) and len({p[0] for p in plans}) == len(plans), 'input')
    expected = ('Migration planı: '+str(len(plans))+' uygulanmış, 0 bekleyen; SQL uygulanmadı.\n'+''.join(str(v)+'\tuygulanmış\t'+n+'\t'+h+'\n' for v,n,h in plans)).encode()
    binary = Path(args.dil)
    require(binary.is_absolute() and binary.resolve(strict=True) == binary and binary.is_file() and not binary.stat().st_mode & 0o022, 'input')
    require(binary.stat().st_size <= 134217728, 'input')
    binary_key, binary_hash = file_key(binary), sha(binary)
    with tempfile.TemporaryDirectory(prefix='zee-db-readiness-') as temporary:
        temporary = Path(temporary)
        copy = temporary/'dil'; shutil.copyfile(binary, copy); copy.chmod(0o500)
        require(sha(copy) == binary_hash and file_key(binary) == binary_key, 'input')
        snapshot = temporary/'project'; snapshot.mkdir(mode=0o700)
        for row, content in zip(rows, source_bytes):
            path = snapshot/row['path']; path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content); path.chmod(0o400)
        code, stdout, stderr = child(copy, ['sürüm','--json'], temporary, deadline, cancelled)
        build = decode_json(stdout)
        require(code == 0 and not stderr and isinstance(build, dict) and build.get('schema') == 'zee-build-v1' and build.get('source_sha') == args.expected_source and build.get('dirty') is False, 'identity')
        code, stdout, stderr = child(copy, ['göçür','--durum',str(snapshot)], temporary, deadline, cancelled)
        require(code == 0 and not stderr and stdout == expected, 'migration')
        code, stdout, stderr = child(copy, ['çalıştır',str(snapshot)], temporary, deadline, cancelled)
        require(code == 0 and not stderr and stdout == b'zee-db-ready-1\n', 'schema')
        require(sha(copy) == binary_hash and file_key(binary) == binary_key and sha(binary) == binary_hash and all(file_key(path) == key for path,key in zip(paths, keys)), 'input')
    require(not cancelled(), 'cancelled')
    gate.write(True)


def main(argv=None):
    parser = Parser(description=__doc__)
    for name in ['dil', 'project', 'worker-project', 'gate', 'expected-source', 'application-revision', 'contract', 'contract-sha256']:
        parser.add_argument('--'+name, required=True)
    parser.add_argument('--deadline', type=float, default=8)
    parser.add_argument('--rounds', type=int, default=1)
    parser.add_argument('--interval', type=float, default=2)
    parser.add_argument('--continuous', action='store_true')
    stopped, handlers = [False], {}
    def stop(signum, frame):
        stopped[0] = True
    gate, revision, reason, code = None, None, 'input', 2
    try:
        args = parser.parse_args(argv)
        require(os.name == 'posix' and HEX40.fullmatch(args.expected_source) and HEX40.fullmatch(args.application_revision) and HEX64.fullmatch(args.contract_sha256) and 0.1 <= args.deadline <= 15, 'input')
        require(1 <= args.rounds <= 1000 and 2 <= args.interval <= 60 and (not args.continuous or args.rounds == 1), 'input')
        if args.rounds > 1 or args.continuous:
            for number in (signal.SIGTERM, signal.SIGINT):
                handlers[number] = signal.signal(number, stop)
        revision = args.application_revision
        worker = canonical_directory(args.worker_project)
        path = Path(args.gate)
        require(path.is_absolute(), 'gate')
        gate = Gate(worker, path, revision)
        gate.write(False)
        index = 0
        while args.continuous or index < args.rounds:
            require(not stopped[0], 'cancelled')
            try:
                probe(args, gate, lambda: stopped[0])
            except Reject as error:
                gate.write(False)
                if (args.rounds == 1 and not args.continuous) or error.code not in {'migration', 'schema', 'timeout'}:
                    raise
            index += 1
            if args.continuous or index < args.rounds:
                end = time.monotonic() + args.interval
                while time.monotonic() < end and not stopped[0]:
                    time.sleep(min(.05, max(0, end-time.monotonic())))
        require(not stopped[0], 'cancelled')
        reason, code = ('ready' if args.rounds == 1 else 'completed'), 0
    except (Reject, OSError, ValueError, TypeError, RecursionError, subprocess.SubprocessError) as error:
        reason = error.code if isinstance(error, Reject) and error.code in {'input','gate','identity','migration','schema','timeout','output','cancelled'} else 'input'
        code = 1 if reason in {'migration','schema','timeout','output'} else 2
    finally:
        if gate is not None:
            try:
                if args.rounds > 1 or args.continuous or code != 0:
                    gate.write(False)
            except (Reject, OSError, ValueError, TypeError):
                reason, code = 'gate', 2
            finally:
                gate.close()
        for number, previous in handlers.items():
            signal.signal(number, previous)
    print(json.dumps({'schema':'zee-db-readiness-host-1','application_revision':revision,'ready':code == 0 and reason == 'ready','reason':reason}, separators=(',',':')))
    return code


if __name__ == '__main__':
    sys.exit(main())
