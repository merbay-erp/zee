#!/usr/bin/env python3
"""Run a bounded loopback PostgreSQL SQL-intent pilot; no SMTP/HTTP/file handler."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import selectors
import secrets
import shutil
import signal
import stat
import subprocess
import tempfile
import time
from types import SimpleNamespace


STAGE = Path(__file__).resolve().parent

def support_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

OWNER = support_module('owned_stage_supervisor', STAGE / 'worker-supervisor.py')
CREDENTIAL = support_module('owned_stage_credential', STAGE / 'credential-baslat.py')

class JsonLog(OWNER.JsonLog):
    def emit(self, event, **fields):
        super().emit(event, schema='zee-sql-intent-dispatcher-1', **fields)

OPS = {'reap', 'claim', 'begin', 'apply', 'renew', 'fail'}
FILES = ('ana.dil', 'proje.dil', 'gocler/0001_kuyruk.sql', 'gocler/0002_kira_sahibi.sql', 'gocler/0003_sabit_niyet.sql')
class Uncertain(Exception): pass

def require(ok):
    if not ok: raise Uncertain('fixed_profile_refused')

def private_bytes(path, limit):
    before = path.lstat()
    require(stat.S_ISREG(before.st_mode) and not before.st_mode & 0o022 and before.st_uid in {0, os.geteuid()} and before.st_size <= limit)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        opened = os.fstat(fd)
        key = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns)
        require(key(before) == key(opened))
        with os.fdopen(fd, 'rb') as stream:
            fd = None
            data = stream.read(limit + 1)
            require(key(opened) == key(os.fstat(stream.fileno())) == key(path.lstat()) and len(data) == opened.st_size)
        return data
    finally:
        if fd is not None: os.close(fd)

def project_snapshot(project):
    return {name: private_bytes(project / name, 256 * 1024) for name in FILES}

def project_digest(files):
    record = '\n'.join(name + ':' + hashlib.sha256(files[name]).hexdigest() for name in FILES)
    return hashlib.sha256(record.encode()).hexdigest()

class Dispatcher:
    def __init__(self, binary, project, env, owner, log=None):
        self.binary, self.project, self.env, self.owner = binary, project, env, owner
        self.log = log or JsonLog()
        self.group_owner = OWNER.Supervisor(SimpleNamespace(project=project, dil=binary, stop_grace=.5), self.log)
        self.children = self.group_owner.children
        self.stopped = False
        self.applied = self.heartbeat_acks = 0
    def signal(self, number, frame): self.stopped = True
    def emit(self, event, **fields):
        self.log.emit(event, **fields)
    def start(self, op, lease=None, token='unused'):
        require(op in OPS and not self.stopped)
        job, token, generation = lease if lease else ('0', token, '0')
        require(re.fullmatch(r'[0-9]{1,20}', job) is not None and re.fullmatch(r'[0-9]{1,20}', generation) is not None and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', token) is not None)
        child = self.group_owner.spawn([str(self.binary), 'çalıştır', str(self.project), op, job, token, generation, self.owner],
                                       capture=True, environment=self.env)
        # K246 owns the new session leader; WNOWAIT holds its PID until group cleanup.
        os.set_blocking(child.stdout.fileno(), False)
        return child
    def terminate(self, child):
        require(child in self.children)
        self.group_owner.stop_child(child)
    def finish(self, child, timeout, heartbeat=None):
        deadline = time.monotonic() + timeout
        renew_at = time.monotonic() + 5
        output = bytearray()
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(child.stdout, selectors.EVENT_READ)
                while True:
                    if self.stopped or time.monotonic() >= deadline: raise Uncertain('operation_interrupted')
                    for key, _ in selector.select(.05):
                        part = os.read(key.fd, 257)
                        if part:
                            output.extend(part)
                            require(len(output) <= 256)
                        else:
                            selector.unregister(key.fileobj)
                    status = self.group_owner.exit_status(child)
                    if status is not None and status != 0: raise Uncertain('owned_leader_failed')
                    if status is not None and not selector.get_map(): break
                    if heartbeat is not None and time.monotonic() >= renew_at:
                        require(self.query('renew', heartbeat) == '1')
                        self.heartbeat_acks += 1
                        renew_at = time.monotonic() + 5
            # All nonzero exits incl C027, malformed output, timeout and signals pause
            # admission. Never turn them into an automatic fail/retry write.
            require(status == 0)
            require(re.fullmatch(rb'(?:[0-9]{1,20}(?::[0-9]{1,20})?|bos|reddedildi)\n', output) is not None)
            return bytes(output).decode('ascii').strip()
        finally:
            self.terminate(child)
    def query(self, op, lease=None, token='unused'):
        return self.finish(self.start(op, lease, token), 5)
    def tick(self):
        require(self.query('reap').isdigit())
        token = 'lease-' + secrets.token_hex(16)
        claim = self.query('claim', token=token)
        if claim == 'bos': self.emit('empty'); return
        require(re.fullmatch(r'[0-9]{1,20}:[0-9]{1,20}', claim) is not None)
        job, generation = claim.split(':'); lease = job, token, generation
        require(self.query('begin', lease) == '1')
        result = self.finish(self.start('apply', lease), 120, heartbeat=lease)
        if result == '1':
            self.applied += 1; self.emit('done', applied=self.applied)
        elif result == 'reddedildi':
            # Exit0 explicit result after outer rollback acknowledgement only.
            require(self.query('fail', lease) == '1'); self.emit('known_sql_failure')
        else: raise Uncertain('operation_result')
    def cleanup(self):
        self.group_owner.cleanup()
        self.env.clear()

class MaskedParser(argparse.ArgumentParser):
    def error(self, message):
        raise Uncertain('arguments_rejected')

def main():
    parser = MaskedParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--dil', type=Path, required=True)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--expected-source', required=True)
    parser.add_argument('--expected-binary-hash', required=True)
    parser.add_argument('--expected-project-hash', required=True)
    parser.add_argument('--owned-pg-port', type=int, required=True)
    parser.add_argument('--owned-ca', type=Path, required=True)
    parser.add_argument('--credential-root', type=Path, required=True)
    parser.add_argument('--credential-env', action='append', required=True)
    parser.add_argument('--owner', required=True)
    parser.add_argument('--ticks', type=int, default=20)
    args = parser.parse_args()
    require(re.fullmatch(r'[0-9a-f]{40}', args.expected_source) is not None and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', args.owner) is not None and 1 <= args.owned_pg_port <= 65535 and 1 <= args.ticks <= 100)
    data = private_bytes(args.dil, 128 * 1024 * 1024); require(hashlib.sha256(data).hexdigest() == args.expected_binary_hash)
    files = project_snapshot(args.project); require(project_digest(files) == args.expected_project_hash)
    expected_target = 'veritabanı_hedefi "postgresql://localhost:' + str(args.owned_pg_port) + '/postgres" olsun'
    require(expected_target in files['proje.dil'].decode('utf8'))
    with tempfile.TemporaryDirectory(prefix='zee-sql-dispatcher-private-') as tmp:
        directory = Path(tmp); binary = directory / 'dil'; binary.write_bytes(data); binary.chmod(0o500)
        project = directory / 'project'; project.mkdir(mode=0o700)
        for name, value in files.items():
            target = project / name; target.parent.mkdir(mode=0o700, parents=True, exist_ok=True); target.write_bytes(value); target.chmod(0o400)
        log = JsonLog()
        identity_owner = OWNER.Supervisor(SimpleNamespace(project=project, dil=binary, stop_grace=.5), log)
        try:
            build = identity_owner.query_json('--json')
        finally:
            identity_owner.cleanup()
        require(build.get('schema') == 'zee-build-v1' and build.get('source_sha') == args.expected_source and build.get('dirty') is False)
        credential_args = ['--credential-root', str(args.credential_root), '--dil', str(binary)]
        for binding in args.credential_env: credential_args.extend(['--env', binding])
        credential_args.extend(['--', 'çalıştır', str(project)])
        credential_root, bindings, _ = CREDENTIAL.parse_arguments(credential_args)
        mapped = CREDENTIAL.child_environment(credential_root, bindings)
        names = {name for name, _ in bindings}
        require('ZEE_JOB_CI_URL' in names and 'ZEE_JOB_CI_URL_TLS_CA_PEM' not in names)
        env = OWNER.safe_environment()
        env.update({name: mapped[name] for name in names})
        # CA is bounded public trust configuration, not a multiline credential.
        env['ZEE_JOB_CI_URL_TLS_CA_PEM'] = private_bytes(args.owned_ca, 65536).decode('ascii')
        mapped.clear()
        expected_dsn = f'host=localhost port={args.owned_pg_port} user=postgres dbname=postgres sslmode=require'
        require(env['ZEE_JOB_CI_URL'] == expected_dsn)
        dispatcher = Dispatcher(binary, project, env, args.owner, log=log)
        previous = {number: signal.signal(number, dispatcher.signal) for number in (signal.SIGTERM, signal.SIGINT)}
        try:
            for _ in range(args.ticks):
                if dispatcher.stopped: raise Uncertain('stopped')
                dispatcher.tick(); time.sleep(.1)
            dispatcher.emit('finished', applied=dispatcher.applied, heartbeat_acks=dispatcher.heartbeat_acks, log_drops=dispatcher.log.dropped, forced_kills=dispatcher.group_owner.metrics['forced_kills'])
        except Uncertain:
            dispatcher.emit('paused_uncertain'); return 3
        finally:
            dispatcher.cleanup()
            for number, handler in previous.items(): signal.signal(number, handler)
    return 0
if __name__ == '__main__':
    try: raise SystemExit(main())
    except (Uncertain, OWNER.Reject, CREDENTIAL.Rejected, ValueError, OSError, subprocess.SubprocessError):
        JsonLog().emit('rejected')
        raise SystemExit(2)
