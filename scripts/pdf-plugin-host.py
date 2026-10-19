#!/usr/bin/env python3
"""K281: purpose-bound signed PDF plugin, one owned Zee stdio request/reply.
Trusted local code execution, not general FFI or a hostile-code/PDF sandbox.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import sys
import tempfile
import time
from types import SimpleNamespace

PURPOSE = 'zee-pdf-catalog-stdio-1'
FILES = {'backend.py', 'assets/base.pdf', 'assets/font.ttf', 'assets/product.png',
         'assets/LICENSE-DejaVu.txt', 'assets/COPYRIGHT-DejaVu.txt'}
LIBRARIES = {'reportlab': '4.4.9', 'pypdf': '6.10.0', 'Pillow': '12.3.0'}
HELPERS = {'derleyici-secici.py': 'b2103253fc59be2eeb27e6861e593d56202697f69519a2b158f9232326c07692',
           'worker-supervisor.py': '51f0d680d14c4c8a08dc60ab0946e82f2eceb7ead1760a6cc4cd7d14a0cd199e'}
BOOTSTRAP = 'import os,sys; f=int(sys.argv[1]); os.dup2(f,0); os.close(f); os.execv(sys.argv[2],sys.argv[2:])'

CANCELLED = False

class Rejected(Exception):
    pass

def need(value):
    if CANCELLED or not value:
        raise Rejected()

def digest(data):
    return hashlib.sha256(data).hexdigest()

def bounded(path, maximum):
    path = Path(path)
    before = path.lstat()
    need(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_size <= maximum)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    key = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
    try:
        initial = os.fstat(fd)
        need(key(initial) == key(before))
        data = bytearray()
        while len(data) <= maximum:
            part = os.read(fd, min(65536, maximum + 1 - len(data)))
            if not part:
                break
            data.extend(part)
        need(len(data) <= maximum and key(initial) == key(os.fstat(fd)) == key(path.lstat()))
        return bytes(data)
    finally:
        os.close(fd)

def pairs(items):
    out = {}
    for k, v in items:
        need(k not in out)
        out[k] = v
    return out

def decode(data):
    return json.loads(data, object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(Rejected()))

def pin(value):
    return type(value) is str and re.fullmatch('[0-9a-f]{64}', value) is not None

def request(data):
    need(len(data) <= 16384)
    obj = decode(data)
    need(type(obj) is dict and set(obj) == {'schema', 'job'} and
         obj['schema'] == 'zee-pdf-plugin-request-1' and type(obj['job']) is str)
    job = obj['job'].encode('utf-8')
    need(len(job) <= 8192)
    return job

def receipt(data, output):
    need(len(data) <= 4096)
    obj = decode(data)
    need(type(obj) is dict and set(obj) == {'schema', 'status', 'files'} and
         obj['schema'] == 'zee-owned-pdf-receipt-1' and obj['status'] == 'produced')
    need(type(obj['files']) is list and len(obj['files']) == 3)
    names = set()
    for row in obj['files']:
        need(type(row) is dict and set(row) == {'name', 'bytes', 'sha256'})
        name = row['name']
        need(type(name) is str and name in {'generated.pdf', 'selected.pdf', 'catalog.pdf'} and name not in names)
        names.add(name)
        need(type(row['bytes']) is int and 0 < row['bytes'] <= 1048576 and pin(row['sha256']))
        value = bounded(output / name, 1048576)
        need(len(value) == row['bytes'] and digest(value) == row['sha256'])
    return obj

def exact_names(directory, expected):
    seen = set()
    with os.scandir(directory) as entries:
        for entry in entries:
            need(len(seen) < len(expected) and entry.name not in seen)
            seen.add(entry.name)
    need(seen == expected)

def package(directory, trust_data):
    for path in [directory, directory / "assets"]:
        info = path.lstat()
        need(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid() and not info.st_mode & 0o022)
    exact_names(directory, {"plugin.json", "plugin.json.zee-imza", "backend.py", "assets"})
    exact_names(directory / "assets", {"base.pdf", "font.ttf", "product.png", "LICENSE-DejaVu.txt", "COPYRIGHT-DejaVu.txt"})
    trust = decode(trust_data)
    need(type(trust) is dict and set(trust) == {'schema', 'purpose', 'publisher', 'manifest_sha256', 'python_sha256'})
    need(trust['schema'] == 'zee-pdf-plugin-trust-1' and trust['purpose'] == PURPOSE and
         type(trust['publisher']) is str and re.fullmatch('sha256:[0-9a-f]{64}', trust['publisher']) and
         pin(trust['manifest_sha256']) and pin(trust['python_sha256']))
    raw = bounded(directory / 'plugin.json', 16384)
    need(digest(raw) == trust['manifest_sha256'])
    manifest = decode(raw)
    need(type(manifest) is dict and set(manifest) == {'schema', 'purpose', 'libraries', 'files'} and
         manifest['schema'] == 'zee-pdf-plugin-1' and manifest['purpose'] == PURPOSE and manifest['libraries'] == LIBRARIES)
    need(type(manifest['files']) is dict and set(manifest['files']) == FILES and all(pin(v) for v in manifest['files'].values()))
    files = {}
    for name in sorted(FILES):
        data = bounded(directory / name, 1048576)
        need(digest(data) == manifest['files'][name])
        files[name] = data
    signature = bounded(directory / 'plugin.json.zee-imza', 65536)
    return trust, raw, signature, files

class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise Rejected()

class Owned:
    def __init__(self, root, project):
        need(callable(getattr(os, "waitid", None)) and all(hasattr(os, name) for name in ("WNOWAIT", "WEXITED", "P_PID")))
        modules = {}
        for name, expected in HELPERS.items():
            data = bounded(root / 'scripts' / name, 262144)
            need(digest(data) == expected)
            module = {'__file__': str(root / 'scripts' / name), '__name__': 'pdf_private_helper'}
            exec(compile(data, module['__file__'], 'exec'), module)
            modules[name] = module
        self.environment = {'PATH': os.defpath, 'HOME': str(project), 'TMPDIR': str(project), 'LANG': 'C.UTF-8', 'PYTHONDONTWRITEBYTECODE': '1'}
        self.selector = modules['derleyici-secici.py']
        self.owner = modules['worker-supervisor.py']['Supervisor'](
            SimpleNamespace(project=project, stop_grace=1, runtime_metrics=False), SimpleNamespace(emit=lambda *a, **k: None))
        # Existing exact crypto helper, with its subprocess call routed through the retained owner.
        self.selector['subprocess'] = SimpleNamespace(run=lambda argv, **kw: SimpleNamespace(
            returncode=0 if self.command(argv, min(15, kw.get('timeout', 15)), 4096, kw.get('env')) is not None else 2))
        self.deadline = time.monotonic() + 60

    def spawn(self, argv, passed=(), environment=None):
        need(time.monotonic() < self.deadline)
        return self.owner.spawn([str(x) for x in argv], capture=True, passed_fds=passed, environment=self.environment)

    def read(self, child, maximum, line=False):
        data = bytearray()
        with selectors.DefaultSelector() as selector:
            selector.register(child.stdout, selectors.EVENT_READ)
            while selector.get_map():
                need(time.monotonic() < self.deadline)
                for key, _ in selector.select(min(.1, max(0, self.deadline-time.monotonic()))):
                    chunk = os.read(key.fileobj.fileno(), min(4096, maximum + 1 - len(data)))
                    if not chunk:
                        selector.unregister(key.fileobj)
                        break
                    data.extend(chunk)
                    need(len(data) <= maximum)
                    if line and b'\n' in data:
                        # A single request frame; extra data is a second invocation, never replayed.
                        need(data.endswith(b'\n') and data.count(b'\n') == 1)
                        need(time.monotonic() < self.deadline)
                        return bytes(data[:-1])
        need(not line and time.monotonic() < self.deadline)
        return bytes(data)

    def finish(self, child):
        while self.owner.exit_status(child) is None:
            need(time.monotonic() < self.deadline)
            time.sleep(.01)
        status = self.owner.exit_status(child)
        self.owner.stop_child(child)
        need(status == 0)

    def command(self, argv, timeout, maximum, environment=None):
        previous = self.deadline
        self.deadline = min(previous, time.monotonic()+timeout)
        child = self.spawn(argv, environment=environment)
        try:
            data = self.read(child, maximum)
            self.finish(child)
            return data
        finally:
            self.deadline = previous

    def close(self):
        self.owner.cleanup()

def invoke(args):
    need(os.name == 'posix' and pin(args.trust_sha256) and pin(args.dil_sha256) and pin(args.verifier_sha256))
    trust_data = bounded(args.trust, 16384)
    need(digest(trust_data) == args.trust_sha256)
    trust, manifest_raw, signature, files = package(args.package, trust_data)
    python = args.python.resolve(strict=True)
    python_before = bounded(python, 67108864)
    need(digest(python_before) == trust['python_sha256'])
    compiler = bounded(args.dil, 67108864)
    verifier = bounded(args.verifier, 67108864)
    need(digest(compiler) == args.dil_sha256 and digest(verifier) == args.verifier_sha256)
    source = bounded(args.source, 65536)
    parent = args.output.parent.lstat()
    need(stat.S_ISDIR(parent.st_mode) and parent.st_uid == os.getuid() and not parent.st_mode & 0o077)
    need(not args.output.exists() and not args.output.is_symlink())
    state = 'not_started'
    with tempfile.TemporaryDirectory(prefix='zee-pdf-plugin-') as tmp:
        private = Path(tmp)
        for name, value in files.items():
            target = private / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(value)
        for name, value in [('dil', compiler), ('source.dil', source)]:
            (private/name).write_bytes(value)
        (private/'dil').chmod(0o700)
        owner = Owned(args.framework_root.resolve(strict=True), private)
        writer = None
        try:
            owner.selector['verify_signed_bytes'](manifest_raw, signature, 'plugin.json', trust['publisher'], verifier)
            build = decode(owner.command([private/'dil', 'sürüm', '--json'], 10, 4096))
            need(build.get('schema') == 'zee-build-v1' and build.get('source_sha') == args.expected_source)
            r, writer = os.pipe()
            os.set_blocking(writer, False)
            try:
                child = owner.spawn([sys.executable, '-I', '-B', '-c', BOOTSTRAP, r,
                                     private/'dil', 'çalıştır', private/'source.dil'], passed=(r,))
            finally:
                os.close(r)
            job = request(owner.read(child, 16385, line=True))
            (private/'job.json').write_bytes(job)
            state = 'unknown'
            raw = owner.command([python, '-I', '-B', private/'backend.py', '--fixture-root', private,
                                 '--job', private/'job.json', '--output', args.output], 20, 4096)
            result = receipt(raw, args.output)
            state = 'produced_client_unconfirmed'
            response = json.dumps(result, ensure_ascii=True, separators=(',', ':')).encode()+b'\n'
            need(len(response) <= 4096)
            offset = 0
            while offset < len(response):
                need(time.monotonic() < owner.deadline)
                try:
                    offset += os.write(writer, response[offset:])
                except BlockingIOError:
                    time.sleep(.01)
            os.close(writer)
            writer = None
            need(owner.read(child, 64) == b'PDF_PLUGIN_DONE\n')
            owner.finish(child)
            after = decode(owner.command([private/'dil', 'sürüm', '--json'], 10, 4096))
            need(after == build and bounded(args.source, 65536) == source and
                 bounded(args.dil, 67108864) == compiler and bounded(python, 67108864) == python_before)
            return {'schema': 'zee-pdf-plugin-result-1', 'status': 'completed', 'build': build, 'files': result['files']}
        except Exception:
            raise Rejected(state) from None
        finally:
            if writer is not None:
                os.close(writer)
            try:
                owner.close()
            except Exception:
                raise Rejected(state) from None

def main():
    def cancelled(number, frame):
        global CANCELLED
        CANCELLED = True
    for number in (signal.SIGINT, signal.SIGTERM):
        signal.signal(number, cancelled)
    parser = Parser(allow_abbrev=False)
    for name in ['framework-root', 'package', 'trust', 'dil', 'verifier', 'python', 'source', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    for name in ['trust-sha256', 'dil-sha256', 'verifier-sha256', 'expected-source']:
        parser.add_argument('--'+name, required=True)
    args = parser.parse_args()
    need(re.fullmatch('[0-9a-f]{40}', args.expected_source))
    print(json.dumps(invoke(args), ensure_ascii=True))

if __name__ == '__main__':
    try:
        main()
    except (Exception, KeyboardInterrupt) as error:
        state = error.args[0] if isinstance(error, Rejected) and error.args and error.args[0] in {
            'not_started', 'unknown', 'produced_client_unconfirmed'} else 'not_started'
        print(json.dumps({'schema': 'zee-pdf-plugin-error-1', 'status': 'rejected', 'effect': state}), file=sys.stderr)
        sys.exit(2)
