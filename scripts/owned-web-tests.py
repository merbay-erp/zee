#!/usr/bin/env python3
"""K271/K286: owned CLI/HTTP/PG; optional actual Zee source coverage."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import selectors
import shutil
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import time
from html.parser import HTMLParser
from types import SimpleNamespace

SCHEMA = 'zee-owned-web-tests-1'
CANCELLED = False
END = float('inf')


class Rejected(Exception):
    pass


class MaskedParser(argparse.ArgumentParser):
    def error(self, message):
        raise Rejected('arguments')


def need(value, reason='contract'):
    if CANCELLED or time.monotonic() >= END:
        raise Rejected('cancelled_or_deadline')
    if not value:
        raise Rejected(reason)


def guarded_file(path, maximum, consume):
    path = Path(path)
    before = path.lstat()
    need(stat.S_ISREG(before.st_mode) and before.st_size <= maximum, 'regular_file_bound')
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    pin = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns)
    total = 0
    try:
        opened = os.fstat(descriptor)
        need(stat.S_ISREG(opened.st_mode) and pin(opened) == pin(before), 'regular_file_identity')
        while True:
            need(True, 'file_deadline')
            block = os.read(descriptor, min(65536, maximum - total + 1))
            if not block:
                break
            total += len(block)
            need(total <= maximum, 'regular_file_bound')
            consume(block)
        need(total == opened.st_size and pin(opened) == pin(os.fstat(descriptor)) == pin(path.lstat()), 'regular_file_identity')
    finally:
        os.close(descriptor)


def regular_bytes(path, maximum):
    result = bytearray()
    guarded_file(path, maximum, result.extend)
    return bytes(result)


def sha(path, maximum=64 * 1024 * 1024):
    value = hashlib.sha256()
    guarded_file(path, maximum, value.update)
    return value.hexdigest()


def object_pairs(pairs):
    result = {}
    for key, value in pairs:
        need(key not in result, 'duplicate_key')
        result[key] = value
    return result


def decode_json(data):
    return json.loads(data.decode('utf-8'), object_pairs_hook=object_pairs)


def load(path, maximum=65536):
    return decode_json(regular_bytes(path, maximum))


def keys(value, allowed, required=()):
    need(type(value) is dict and set(required) <= set(value) <= set(allowed), 'keys')


def local(root, name):
    need(type(name) is str and name and '\\' not in name, 'path')
    p = Path(name)
    need(not p.is_absolute() and p.parts and p.as_posix() == name and len(name) <= 512 and len(p.parts) <= 16 and all(x not in ('..', '.') for x in p.parts), 'path')
    current = root
    for part in p.parts:
        current = current / part
        need(not current.is_symlink(), 'path')
    need(current.is_file() or current.is_dir(), 'path')
    return current


def validate(root, suite):
    keys(suite, ('schema', 'inventory', 'cases'), ('schema', 'inventory', 'cases'))
    need(suite['schema'] == SCHEMA, 'schema')
    inventory = suite['inventory']
    need(type(inventory) is dict and 1 <= len(inventory) <= 128, 'inventory')
    total = 0
    for name, digest in inventory.items():
        path = local(root, name)
        need(path.is_file() and stat.S_ISREG(path.stat().st_mode) and path.stat().st_nlink == 1, 'regular_file')
        total += path.stat().st_size
        need(total <= 2 * 1024 * 1024 and re.fullmatch('[0-9a-f]{64}', digest) and sha(path) == digest, 'source_pin')
    cases = suite['cases']
    need(type(cases) is list and 1 <= len(cases) <= 8, 'case_bound')
    names = set()
    for case in cases:
        keys(case, ('id', 'project', 'database', 'seed', 'steps'), ('id', 'project', 'steps'))
        need(re.fullmatch('[a-z][a-z0-9_-]{0,63}', case['id']) and case['id'] not in names, 'case_id')
        names.add(case['id'])
        project = local(root, case['project'])
        need(project.is_dir(), 'project')
        entries = 0
        for path in project.rglob('*'):
            entries += 1
            need(entries <= 256 and len(path.relative_to(root).parts) <= 16, 'project_bound')
            need(not path.is_symlink() and (path.is_dir() or str(path.relative_to(root)) in inventory), 'project_inventory')
        need(case['project'] + '/proje.dil' in inventory and case['project'] + '/ana.dil' in inventory, 'project_inventory')
        need(case.get('seed') in (None, 'panel_synthetic_phc'), 'seed')
        if 'database' in case:
            keys(case['database'], ('bootstrap', 'observers'), ('bootstrap', 'observers'))
            for name in case['database'].values():
                need(name in inventory, 'sql_inventory')
            bootstrap = local(root, case['database']['bootstrap']).read_text()
            need('\\' not in bootstrap and len(bootstrap) <= 16384, 'sql_meta')
            observations = load(local(root, case['database']['observers']))
            need(type(observations) is dict and 1 <= len(observations) <= 16, 'observer')
            for name, sql in observations.items():
                need(re.fullmatch('[a-z_]{1,48}', name) and type(sql) is str and len(sql) <= 2048 and
                     sql.startswith('SELECT ') and ';' not in sql and '\\' not in sql, 'observer')
        steps = case['steps']
        need(type(steps) is list and 1 <= len(steps) <= 64, 'step_bound')
        tokens = set()
        for step in steps:
            if 'observe' in step:
                keys(step, ('observe', 'equals'), ('observe', 'equals'))
                need('database' in case and step['observe'] in observations and type(step['equals']) is int, 'observer')
                continue
            keys(step, ('method', 'path', 'jar', 'form', 'csrf', 'expect', 'capture_csrf'), ('method', 'path', 'jar', 'expect'))
            need(step['method'] in ('GET', 'POST') and re.fullmatch('/[A-Za-z0-9/_-]{0,127}', step['path']), 'request')
            need(re.fullmatch('[a-z]{1,24}', step['jar']), 'jar')
            need(step.get('csrf', 'same') in ('same', 'omit_origin', 'cross_origin'), 'csrf_origin')
            keys(step['expect'], ('status', 'snapshot', 'rotated'), ('status',))
            need(type(step['expect']['status']) is int and 100 <= step['expect']['status'] <= 599, 'status')
            if 'snapshot' in step['expect']:
                need(step['expect']['snapshot'] in inventory and local(root, step['expect']['snapshot']).stat().st_size <= 65536, 'snapshot')
            if 'rotated' in step['expect']:
                need(type(step['expect']['rotated']) is bool, 'rotated')
            form = step.get('form', {})
            need(type(form) is dict and len(form) <= 32, 'form')
            for key, value in form.items():
                need(re.fullmatch('[A-Za-z_][A-Za-z0-9_]{0,63}', key), 'form')
                if type(value) is str:
                    need(len(value.encode()) <= 2048, 'form')
                else:
                    keys(value, ('csrf', 'secret'))
                    need(len(value) == 1, 'form_ref')
                    if 'csrf' in value:
                        need(value['csrf'] in tokens, 'csrf_ref')
                    else:
                        need(case.get('seed') == 'panel_synthetic_phc' and value['secret'] == 'panel_password', 'secret_ref')
            if 'capture_csrf' in step:
                name = step['capture_csrf']
                need(type(name) is str and re.fullmatch('[a-z]{1,24}', name) and name not in tokens, 'csrf_capture')
                tokens.add(name)
    return suite


class HiddenToken(HTMLParser):
    def __init__(self):
        super().__init__()
        self.tokens = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'input' and attrs.get('name') == '_csrf':
            value = attrs.get('value', '')
            need(re.fullmatch('[0-9a-f]{64}', value), 'csrf_html')
            self.tokens.append(value)
            need(len(self.tokens) <= 32, 'csrf_bound')


class Kit:
    def __init__(self, owner, binary, pg, work, executable_pins):
        self.owner_module, self.binary, self.pg, self.work = owner, binary, pg, work
        self.pins = executable_pins
        self.proof = None
        self.command_owner = owner.Supervisor(SimpleNamespace(project=work, stop_grace=1), None)
        self.commands = 0
        self.coverage_output = None

    def command(self, argv, env=None, timeout=15, limit=512 * 1024, input_bytes=None):
        if self.proof is not None:
            self.proof['current_operation'] = str(argv[1]) if Path(argv[0]) == self.binary else Path(argv[0]).name
        need(input_bytes is None or type(input_bytes) is bytes and len(input_bytes) <= 256, 'stdin_bound')
        if input_bytes is None:
            child = self.command_owner.spawn([str(x) for x in argv], capture=True, environment=env)
        else:
            child = subprocess.Popen([str(x) for x in argv], cwd=self.work, env=env or self.owner_module.safe_environment(), stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, close_fds=True, start_new_session=True)
            self.command_owner.children.append(child)
        data = bytearray()
        deadline = min(END, time.monotonic() + timeout)
        selector = None
        try:
            if input_bytes is not None:
                os.set_blocking(child.stdin.fileno(), False)
                need(os.write(child.stdin.fileno(), input_bytes) == len(input_bytes), 'stdin_write')
                child.stdin.close()
            selector = selectors.DefaultSelector()
            selector.register(child.stdout, selectors.EVENT_READ)
            while selector.get_map():
                need(time.monotonic() < deadline, 'command_deadline')
                for key, _ in selector.select(min(.05, max(0, deadline - time.monotonic()))):
                    block = os.read(key.fileobj.fileno(), 4096)
                    need(time.monotonic() < deadline, 'command_deadline')
                    if not block:
                        selector.unregister(key.fileobj)
                    else:
                        need(len(data) + len(block) <= limit, 'capture_bound')
                        data.extend(block)
            status = self.command_owner.exit_status(child)
            while status is None:
                need(time.monotonic() < deadline, 'command_deadline')
                time.sleep(.01)
                status = self.command_owner.exit_status(child)
            need(status == 0, 'command_failed')
            self.commands += 1
            return bytes(data)
        finally:
            if selector is not None:
                selector.close()
            try:
                self.command_owner.stop_child(child)
            finally:
                if child.stdin is not None:
                    child.stdin.close()

    def sql(self, port, database, role, sql):
        env = self.owner_module.safe_environment()
        env.update(PGCONNECT_TIMEOUT='2', PGOPTIONS='-c statement_timeout=3000 -c lock_timeout=2000', PSQL_HISTORY='/dev/null')
        return self.command([self.pg / 'psql', '-X', '-h', '127.0.0.1', '-p', str(port), '-U', role,
                             '-d', database, '-At', '-v', 'ON_ERROR_STOP=1', '-c', sql], env, timeout=5, limit=65536).decode().strip()

    def cluster(self, directory):
        data = directory / 'data'
        self.command([self.pg / 'initdb', '-D', data, '--no-locale', '--encoding=UTF8', '--auth=trust', '-U', 'postgres'], timeout=25)
        # Foreground postgres remains the registered held leader; no pg_ctl PID adoption.
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        server_owner = self.owner_module.Supervisor(SimpleNamespace(project=directory, stop_grace=2), None)
        child = server_owner.spawn([str(self.pg / 'postgres'), '-D', str(data), '-h', '127.0.0.1', '-p', str(port), '-k', str(directory),
                                    '-c', 'max_connections=16', '-c', 'shared_buffers=16MB'])
        try:
            deadline = min(END, time.monotonic() + 10)
            while True:
                need(server_owner.exit_status(child) is None and time.monotonic() < deadline, 'pg_start')
                try:
                    self.sql(port, 'postgres', 'postgres', 'SELECT 1')
                    break
                except Rejected:
                    need(time.monotonic() < deadline, 'pg_start')
                    time.sleep(.03)
            self.sql(port, 'postgres', 'postgres', 'CREATE ROLE fixture_owner LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS; CREATE ROLE app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS; CREATE ROLE observer LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS;')
            self.sql(port, 'postgres', 'postgres', 'ALTER ROLE observer SET default_transaction_read_only=on')
            self.sql(port, 'postgres', 'postgres', 'CREATE DATABASE owned_case OWNER fixture_owner')
            identity = self.sql(port, 'postgres', 'postgres', 'SELECT system_identifier::text FROM pg_control_system()')
            need(identity.isascii() and identity.isdecimal(), 'cluster_identity')
            return server_owner, port, identity
        except BaseException:
            server_owner.cleanup()
            raise

    def case(self, root, case, index, build, proof):
        directory = self.work / ('case' + str(index))
        directory.mkdir(mode=0o700)
        project = directory / 'project'
        shutil.copytree(local(root, case['project']), project)
        for path in project.rglob('*'):
            if path.is_file():
                need(not path.is_symlink() and sha(path) == proof['fixture_inventory'][case['project'] + '/' + str(path.relative_to(project))], 'copied_project_pin')
        worker_owner = self.owner_module.Supervisor(SimpleNamespace(project=project, stop_grace=1), None)
        pg_owner = None
        env = self.owner_module.safe_environment()
        jars, tokens = {}, {}
        secret_values = {}
        observations = {}
        cluster_identity = None
        try:
            if 'database' in case:
                pgdir = directory / 'pg'
                pgdir.mkdir(mode=0o700)
                pg_owner, pgport, cluster_identity = self.cluster(pgdir)
                self.sql(pgport, 'owned_case', 'fixture_owner', local(root, case['database']['bootstrap']).read_text())
                self.sql(pgport, 'owned_case', 'fixture_owner', 'GRANT USAGE ON SCHEMA public TO app, observer; GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app; GRANT SELECT ON ALL TABLES IN SCHEMA public TO observer;')
                observations = load(local(root, case['database']['observers']))
                need(self.sql(pgport, 'owned_case', 'observer', 'SHOW transaction_read_only') == 'on', 'observer_readonly')
                # Existing explicit loopback development profile, no TLS acceptance claim.
                declaration = project / 'proje.dil'
                text = declaration.read_text()
                target = 'postgresql://127.0.0.1:5432/owned_case'
                need(text.count(target) == 1, 'db_target_binding')
                declaration.write_text(text.replace(target, f'postgresql://127.0.0.1:{pgport}/owned_case'))
                proof.setdefault('owned_project_bindings', []).append({'id': case['id'], 'declaration_sha256': sha(declaration), 'binding': 'owned_loopback_port_only'})
                env['ZEE_K271_DATABASE_URL'] = f'host=127.0.0.1 port={pgport} user=app dbname=owned_case sslmode=disable'
            if case.get('seed') == 'panel_synthetic_phc':
                secret_values['panel_password'] = secrets.token_hex(24)
                phc = self.command([self.binary, 'parola-özeti', '--stdin'], input_bytes=secret_values['panel_password'].encode(), limit=1024)
                need(phc.startswith(b'$argon2id$'), 'synthetic_hash')
                private = project / 'panel-phc'
                with private.open('xb') as stream:
                    stream.write(phc.rstrip(b'\n'))
                private.chmod(0o600)
            for command in ('kilitle', 'denetle', 'dene'):
                self.command([self.binary, command, project], env)
            with socket.socket() as sock:
                sock.bind(('127.0.0.1', 0))
                port = sock.getsockname()[1]
            worker_args = [str(self.binary), 'çalıştır']
            coverage_path = None
            if self.coverage_output is not None:
                coverage_path = self.coverage_output / ('coverage-' + case['id'] + '.json')
                worker_args += ['--kapsam', str(coverage_path)]
            worker_args += ['--web-proxy', 'https://fixture.example', '--web-worker-port', str(port), str(project)]
            child = worker_owner.spawn(worker_args, environment=env)
            until = min(END, time.monotonic() + 10)
            while True:
                need(worker_owner.exit_status(child) is None and time.monotonic() < until, 'worker_start')
                try:
                    first = exchange(port, 'GET', '/', {}, '', 'same')
                    need(first[0] == 200 and first[3] == build, 'worker_identity')
                    break
                except (OSError, ValueError):
                    need(time.monotonic() < until, 'worker_start')
                    time.sleep(.02)
            observations_done = []
            for step_index, step in enumerate(case['steps']):
                proof['current_step'] = step_index
                if 'observe' in step:
                    actual = self.sql(pgport, 'owned_case', 'observer', observations[step['observe']])
                    need(actual == str(step['equals']), 'db_observer_mismatch')
                    observations_done.append({'name': step['observe'], 'value': int(actual)})
                    continue
                jar = jars.setdefault(step['jar'], '')
                fields = {key: value if type(value) is str else tokens[value['csrf']] if 'csrf' in value else secret_values[value['secret']]
                          for key, value in step.get('form', {}).items()}
                status, body, cookie, identity = exchange(port, step['method'], step['path'], fields, jar, step.get('csrf', 'same'))
                need(identity == build and status == step['expect']['status'], 'http_expectation')
                if cookie is not None:
                    jars[step['jar']] = cookie
                if 'rotated' in step['expect']:
                    need(bool(cookie and cookie != jar) == step['expect']['rotated'], 'cookie_rotation')
                if 'capture_csrf' in step:
                    parser = HiddenToken()
                    parser.feed(body.decode())
                    distinct = set(parser.tokens)
                    need(len(distinct) == 1, 'csrf_capture')
                    tokens[step['capture_csrf']] = next(iter(distinct))
                if 'snapshot' in step['expect']:
                    need(body == local(root, step['expect']['snapshot']).read_bytes(), 'snapshot_mismatch')
            result = {'id': case['id'], 'passed': True, 'requests': sum('method' in x for x in case['steps']), 'observations': observations_done, 'cluster_system_identifier': cluster_identity, 'observer_readonly': cluster_identity is not None}
            if coverage_path is not None:
                child.send_signal(signal.SIGTERM)
                until = min(END, time.monotonic() + 5)
                while worker_owner.exit_status(child) is None:
                    need(time.monotonic() < until, 'coverage_worker_exit')
                    time.sleep(.01)
                need(worker_owner.exit_status(child) == 0, 'coverage_worker_exit')
                worker_owner.stop_child(child)
                coverage = load(coverage_path, 2 * 1024 * 1024)
                need(coverage.get('schema') == 'zee-source-coverage-1' and coverage.get('complete') is True and coverage.get('execution_success') is True and coverage.get('compiler_build') == build, 'coverage_identity')
                snapshots = coverage.get('sources')
                need(type(snapshots) is dict and 1 <= len(snapshots) <= 256, 'coverage_snapshots')
                for name, pin in snapshots.items():
                    if name.startswith('gömülü:'):
                        need(proof.get('unit', {}).get('embedded_sources', {}).get(name) == pin, 'coverage_embedded_snapshot')
                    else:
                        path = Path(name)
                        need(path.is_absolute() and project in path.parents and path.suffix == '.dil' and sha(path) == pin, 'coverage_snapshot')
                        original = case['project'] + '/' + str(path.relative_to(project))
                        need(proof['fixture_inventory'].get(original) == pin, 'coverage_fixture_binding')
                result['source_coverage'] = {'path': str(coverage_path), 'sha256': sha(coverage_path), 'product': coverage['product'], 'tests': coverage['tests'], 'sources': snapshots}
            return result
        finally:
            try:
                worker_owner.cleanup()
            finally:
                try:
                    if pg_owner is not None:
                        pg_owner.cleanup()
                finally:
                    proof['cleanup'].append({'id': case['id'], 'worker_groups_empty': not worker_owner.children, 'pg_groups_empty': pg_owner is None or not pg_owner.children})


def exchange(port, method, path, form, cookie, origin):
    import urllib.parse
    deadline = min(END, time.monotonic() + 3)
    body = urllib.parse.urlencode(form).encode()
    need(len(body) <= 8192, 'request_bound')
    headers = 'Host: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n'
    if cookie:
        headers += 'Cookie: ' + cookie + '\r\n'
    if method == 'POST':
        if origin != 'omit_origin':
            headers += 'Origin: https://' + ('fixture.example' if origin == 'same' else 'cross.invalid') + '\r\n'
        headers += 'Content-Type: application/x-www-form-urlencoded\r\nContent-Length: ' + str(len(body)) + '\r\n'
    with socket.create_connection(('127.0.0.1', port), timeout=max(.001, deadline - time.monotonic())) as stream:
        def remaining():
            need(time.monotonic() < deadline, 'http_deadline')
            stream.settimeout(deadline - time.monotonic())
        remaining()
        stream.sendall((method + ' ' + path + ' HTTP/1.1\r\n' + headers + '\r\n').encode() + body)
        data = bytearray()
        while True:
            remaining()
            block = stream.recv(4096)
            need(time.monotonic() < deadline, 'http_deadline')
            if not block:
                break
            need(len(data) + len(block) <= 81920, 'response_bound')
            data.extend(block)
            if b'\r\n\r\n' not in data:
                need(len(data) <= 16384, 'header_bound')
    header, body = bytes(data).split(b'\r\n\r\n', 1)
    need(len(header) <= 16384 and len(body) <= 65536, 'response_bound')
    lines = header.decode('ascii').split('\r\n')
    need(re.fullmatch(r'HTTP/1\.[01] [1-5][0-9]{2} [ -~]*', lines[0]), 'http_status')
    fields = {}
    cookie = None
    for line in lines[1:]:
        name, value = line.split(':', 1)
        name, value = name.lower(), value.strip()
        need(re.fullmatch('[a-z0-9-]+', name) and all(32 <= ord(c) <= 126 for c in value), 'http_header')
        if name == 'set-cookie':
            need(cookie is None and len(value) <= 4096, 'cookie_bound')
            cookie = value.split(';', 1)[0]
        else:
            need(name not in fields, 'http_duplicate')
            fields[name] = value
    need(fields.get('content-length') == str(len(body)) and fields.get('connection', '').lower() == 'close' and 'transfer-encoding' not in fields, 'http_frame')
    build = json.loads(fields.get('x-zee-build', '{}'), object_pairs_hook=object_pairs)
    need(time.monotonic() < deadline, 'http_deadline')
    return int(lines[0].split(' ')[1]), body, cookie, build


def main(argv=None):
    global END, CANCELLED
    END, CANCELLED = time.monotonic() + 180, False
    def cancel(number, frame):
        global CANCELLED
        CANCELLED = True
    for number in (signal.SIGTERM, signal.SIGINT):
        signal.signal(number, cancel)
    parser = MaskedParser(allow_abbrev=False)
    for name in ('fixture-root', 'suite', 'dil', 'postgres-bin-dir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    for name in ('suite-sha256', 'binary-sha256', 'expected-source'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--candidate', action='store_true')
    parser.add_argument('--source-coverage', action='store_true')
    parser.add_argument('--unit-receipt', type=Path)
    args = parser.parse_args(argv)
    need(not args.source_coverage or args.unit_receipt is not None, 'coverage_requires_unit_bridge')
    root = args.fixture_root.resolve(strict=True)
    binary = args.dil.resolve(strict=True)
    pg = args.postgres_bin_dir.resolve(strict=True)
    need(os.name == 'posix' and os.geteuid() != 0 and hasattr(os, 'WNOWAIT'), 'platform')
    suite_bytes = regular_bytes(args.suite, 65536)
    need(re.fullmatch('[0-9a-f]{40}', args.expected_source) and hashlib.sha256(suite_bytes).hexdigest() == args.suite_sha256 and sha(binary) == args.binary_sha256, 'identity_pin')
    suite = validate(root, decode_json(suite_bytes))
    output = args.output
    need(not output.exists() and not output.is_symlink() and output.parent.is_dir(), 'output')
    output = output.parent.resolve(strict=True) / output.name
    need(root not in output.parents and output.parent.stat().st_uid == os.geteuid(), 'output')
    for name in ('initdb', 'postgres', 'psql'):
        need((pg / name).is_file(), 'pg_binary')
    support = Path(__file__).with_name('worker-supervisor.py')
    framework = Path(__file__).resolve().parent.parent
    source_files = {'scripts/owned-web-tests.py': sha(__file__, 1024 * 1024), 'scripts/worker-supervisor.py': sha(support, 1024 * 1024)}
    executable_pins = {str(binary): sha(binary), **{str(pg / name): sha(pg / name) for name in ('initdb', 'postgres', 'psql')}}
    spec = importlib.util.spec_from_file_location('owned_kit_owner', support)
    owner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owner)
    output.mkdir(mode=0o700)
    proof = {'schema': SCHEMA, 'status': 'running', 'cases': [], 'cleanup': [], 'sources': source_files, 'executables': executable_pins,
             'suite_sha256': args.suite_sha256, 'fixture_inventory': suite['inventory'], 'candidate': args.candidate}
    kit = None
    try:
        with tempfile.TemporaryDirectory(prefix='zee-kit-', dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp') as temp:
            kit = Kit(owner, binary, pg, Path(temp).resolve(), executable_pins)
            kit.proof = proof
            kit.coverage_output = output if args.source_coverage else None
            if args.unit_receipt is not None:
                unit = load(args.unit_receipt, 16384)
                need(unit.get('schema') == 'zee-dene-unit-before-native-1' and unit.get('passed') is True, 'unit_receipt')
                proof['unit'] = unit
                proof['unit_receipt_sha256'] = sha(args.unit_receipt, 16384)
            build = json.loads(kit.command([binary, 'sürüm', '--json'], limit=4096))
            need(build.get('schema') == 'zee-build-v1' and build.get('source_sha') == args.expected_source and (args.candidate or build.get('dirty') is False), 'build_identity')
            proof['build'] = build
            if args.unit_receipt is not None:
                need(proof['unit'].get('compiler_build') == build and proof['unit'].get('python_sha256') == sha(Path(sys.executable).resolve()), 'unit_build_runtime')
                proof['executables'][str(Path(sys.executable).resolve())] = proof['unit']['python_sha256']
            version = kit.command([pg / 'postgres', '--version'], limit=512).decode().strip()
            need(version == 'postgres (PostgreSQL) 16.15', 'pg_version')
            proof['postgres_version'] = version
            for index, case in enumerate(suite['cases']):
                proof['current_case'] = case['id']
                proof['cases'].append(kit.case(root, case, index, build, proof))
            cluster_ids = [item['cluster_system_identifier'] for item in proof['cases'] if item['cluster_system_identifier'] is not None]
            need(len(cluster_ids) == len(set(cluster_ids)), 'cluster_isolation')
            need(all(sha(name) == digest for name, digest in executable_pins.items()), 'executable_changed')
            need(all(sha(framework / name, 1024 * 1024) == digest for name, digest in source_files.items()), 'source_changed')
            suite_end = regular_bytes(args.suite, 65536)
            need(suite_end == suite_bytes, 'suite_changed')
            validate(root, decode_json(suite_end))
            need(json.loads(kit.command([binary, 'sürüm', '--json'], limit=4096)) == build, 'end_identity')
            if args.unit_receipt is not None:
                need(sha(Path(sys.executable).resolve()) == proof['unit']['python_sha256'], 'python_changed')
            proof.update(status='passed', source_pins_unchanged=True, owned_cleanup=True)
    except BaseException as error:
        proof['status'] = 'failed'
        proof['failure_type'] = type(error).__name__
        proof['reason'] = str(error) if type(error) is Rejected else 'execution_failed'
        raise Rejected(proof['reason']) from None
    finally:
        try:
            if kit is not None:
                kit.command_owner.cleanup()
        except BaseException:
            proof.update(status='failed', failure_type='CleanupFailure', reason='cleanup_failed')
            raise Rejected('cleanup_failed') from None
        finally:
            proof['command_groups_empty'] = kit is None or not kit.command_owner.children
            proof['owned_cleanup'] = proof['command_groups_empty'] and all(item['worker_groups_empty'] and item['pg_groups_empty'] for item in proof['cleanup'])
            if not proof['owned_cleanup']:
                proof.update(status='failed', failure_type='CleanupFailure', reason='cleanup_failed')
            (output / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n')


if __name__ == '__main__':
    try:
        main()
    except (Exception, KeyboardInterrupt) as error:
        print(json.dumps({'schema': SCHEMA, 'status': 'rejected', 'type': type(error).__name__}), file=sys.stderr)
        raise SystemExit(2)
