#!/usr/bin/env python3
"""K-249: owned native CLI TR–EN and plain-text escaping campaign; no skip."""
import argparse
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import platform
import queue
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time

CASES = ('denetle', 'tr-en', 'dene', 'xss', 'unknown-language', 'unknown-key',
         'lang-attribute-injection')
SOURCES = ('kaynak/ana.dil', 'kaynak/ceviri.dil', 'kaynak/sayfa.dil', 'proje.dil')
SNAPSHOTS = ('en.html', 'tr.html', 'xss.html')
TEST_NAMES = (
    'sayfa: ceviri: TR ve EN sözlük değerleri deterministiktir',
    'sayfa: ceviri: Bilinmeyen dil ve çeviri anahtarı açıkça reddedilir',
    'sayfa: Plain slotlar text ve attribute bağlamında kaçırılır',
    'sayfa: Dil özniteliği whitelist dışında markup kabul etmez',
)
LIMIT = 65536


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        while True:
            chunk = stream.read(65536)
            if not chunk:
                break
            value.update(chunk)
    return value.hexdigest()


def inputs(root, names, maximum):
    result = []
    for name in names:
        path = root / name
        require(path.is_file() and not path.is_symlink(), 'Input must be a regular non-symlink file')
        require(0 < path.stat().st_size <= maximum, 'Input exceeds fixture bound')
        result.append({'path': name, 'sha256': digest(path)})
    return result


def invoke(binary, args, cwd):
    """Concurrent bounded pipe readers, one absolute deadline, kill/wait on error."""
    deadline = time.monotonic() + 15
    child = subprocess.Popen([str(binary), *args], cwd=cwd, stdin=subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    messages = queue.Queue()
    workers = []
    try:
        for name, pipe in (('stdout', child.stdout), ('stderr', child.stderr)):
            def read(name=name, pipe=pipe):
                try:
                    with pipe:
                        data = pipe.read(LIMIT + 1)
                    messages.put((name, data, None))
                except Exception:
                    messages.put((name, None, 'Native pipe reader failed'))
            worker = threading.Thread(target=read, name='k249-' + name)
            worker.start()
            workers.append(worker)
        outputs = {}
        while len(outputs) < 2:
            remaining = deadline - time.monotonic()
            require(remaining > 0, 'Native CLI deadline exceeded')
            try:
                name, data, error = messages.get(timeout=remaining)
            except queue.Empty:
                raise ValueError('Native CLI deadline exceeded') from None
            require(error is None and data is not None, 'Native pipe reader failed')
            require(len(data) <= LIMIT, 'Native CLI output exceeds 64 KiB')
            outputs[name] = data
        remaining = deadline - time.monotonic()
        require(remaining > 0, 'Native CLI deadline exceeded')
        code = child.wait(timeout=remaining)
        return code, outputs['stdout'], outputs['stderr']
    finally:
        if child.poll() is None:
            child.kill()
        child.wait(timeout=5)
        for worker in workers:
            worker.join(timeout=5)
        require(not any(worker.is_alive() for worker in workers), 'Native reader cleanup failed')
        child.stdout.close()
        child.stderr.close()


class Shape(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.tags = []
        self.text = []
        self.attributes = []

    def handle_starttag(self, tag, attrs):
        self.tags.append(tag)
        self.attributes.append(dict(attrs))
        require(len(attrs) == len(dict(attrs)), 'Duplicate HTML attribute')
        require(not any(name.lower().startswith('on') for name, _ in attrs), 'Executable event attribute')

    def handle_data(self, data):
        self.text.append(data)


def shape(value, language, slots):
    parser = Shape()
    parser.feed(value)
    parser.close()
    require(parser.tags == ['article', 'h1', 'p', 'button'], 'Plain slot created a new HTML node')
    require(parser.attributes == [
        {'lang': language, 'class': 'ayar-sayfasi'}, {'title': slots[0]},
        {'class': 'aciklama'}, {'type': 'button', 'aria-label': slots[2]}],
        'Plain slot changed HTML attribute structure')
    require(parser.text == list(slots), 'Plain slot changed character data')


def identity(binary, cwd):
    code, stdout, stderr = invoke(binary, ['sürüm', '--json'], cwd)
    require(code == 0 and stderr == b'', 'Native identity query failed')
    value = json.loads(stdout)
    require(value.get('schema') == 'zee-build-v1' and
            isinstance(value.get('dirty'), bool) and
            re.fullmatch(r'[a-f0-9]{40}', value.get('source_sha', '')) is not None,
            'Native identity schema invalid')
    return value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--dil', required=True)
    parser.add_argument('--project', required=True)
    parser.add_argument('--snapshots', required=True)
    parser.add_argument('--report')
    args = parser.parse_args()
    require(sys.flags.isolated == 1 and sys.dont_write_bytecode,
            'Run the fixture with Python -I -B')
    binary_source = Path(args.dil).resolve(strict=True)
    project_source = Path(args.project).resolve(strict=True)
    snapshots = Path(args.snapshots).resolve(strict=True)
    require(binary_source.is_file(), 'Explicit native binary required')
    require(sorted(str(p.relative_to(project_source)) for p in project_source.rglob('*.dil')) == list(SOURCES),
            'Expected exactly four project sources')
    before_sources = inputs(project_source, SOURCES, LIMIT)
    before_snapshots = inputs(snapshots, SNAPSHOTS, 16384)
    before_binary = digest(binary_source)
    outcomes = []
    with tempfile.TemporaryDirectory(prefix='zee-k249-native-') as temporary:
        owned = Path(temporary)
        binary = owned / ('dil.exe' if os.name == 'nt' else 'dil')
        shutil.copyfile(binary_source, binary)
        binary.chmod(0o500)
        require(digest(binary) == before_binary == digest(binary_source), 'Native snapshot race')
        project = owned / 'project'
        (project / 'kaynak').mkdir(parents=True)
        for name in SOURCES:
            shutil.copyfile(project_source / name, project / name)
            (project / name).chmod(0o400)
        require(inputs(project, SOURCES, LIMIT) == before_sources, 'Project snapshot differs')
        start_identity = identity(binary, owned)
        def case(name, command, expected, target=project):
            code, stdout, stderr = invoke(binary, [command, str(target)], owned)
            require(code == 0 and stderr == b'', 'Native case exit/stderr failure: ' + name)
            if expected is not None:
                require(stdout == expected, 'Native exact output differs: ' + name)
            outcomes.append({'name': name, 'status': 'passed', 'exit': code,
                             'stdout_sha256': hashlib.sha256(stdout).hexdigest(),
                             'stderr_empty': True})
            return stdout
        case('denetle', 'denetle', 'Denetim temiz: sözdizimi ve türler geçerli.\n'.encode())
        tr = (snapshots / 'tr.html').read_bytes()
        en = (snapshots / 'en.html').read_bytes()
        case('tr-en', 'çalıştır', tr + en)
        shape(tr.decode().rstrip('\n'), 'tr', ['Bildirim Tercihleri', 'Onaydan sonra görevi kuyruğa ekle.', 'Kaydet'])
        shape(en.decode().rstrip('\n'), 'en', ['Notification Preferences', 'Queue the task after approval.', 'Save'])
        tested = case('dene', 'dene', None).decode()
        expected_tests = ''.join('✓ ' + name + '\n' for name in TEST_NAMES) + '\n4 test: 4 geçti, 0 kaldı\n'
        require(tested == expected_tests, 'Exactly four source tests must pass; no skip')
        outcomes[-1]['source_tests'] = 4
        outcomes[-1]['failed'] = 0
        def probe(name, expression, expected):
            probe_project = owned / name
            (probe_project / 'kaynak').mkdir(parents=True)
            for source in ('ceviri.dil', 'sayfa.dil'):
                shutil.copyfile(project / 'kaynak' / source, probe_project / 'kaynak' / source)
            (probe_project / 'proje.dil').write_text(
                'proje "anonymous-probe" olsun\nsürüm "0.1.0" olsun\ngiriş "kaynak/ana.dil" olsun\nyetkinlikler boş liste olsun\n', encoding='utf-8')
            code = ('sayfa birimini kullan\nceviri birimini kullan\nsonuc ' + expression +
                    ' olsun\nsonuc başarılıysa\n    sonucun değeri yaz\ndeğilse\n    sonucun hatası yaz\n')
            (probe_project / 'kaynak/ana.dil').write_text(code, encoding='utf-8')
            return case(name, 'çalıştır', expected, probe_project)
        expression = '"tr" ve "<b>başlık</b> & \\"x\\"" ve "<script>alert(\'x\')</script>" ve "\\" autofocus onfocus=\\"alert(1)" ile sayfayiKur'
        xss = probe('xss', expression, (snapshots / 'xss.html').read_bytes())
        shape(xss.decode().rstrip('\n'), 'tr', ['<b>başlık</b> & "x"', "<script>alert('x')</script>", '" autofocus onfocus="alert(1)'])
        probe('unknown-language', '"fr" ve "baslik" ile ceviriyiAl', b'dil-desteklenmiyor\n')
        probe('unknown-key', '"tr" ve "missing" ile ceviriyiAl', b'ceviri-anahtari-yok\n')
        probe('lang-attribute-injection', '"tr\\" onmouseover=\\"alert(1)" ve "a" ve "b" ve "c" ile sayfayiKur', b'dil-desteklenmiyor\n')
        require(tuple(o['name'] for o in outcomes) == CASES, 'Exactly seven native cases required')
        end_identity = identity(binary, owned)
        require(start_identity == end_identity, 'Native identity changed')
        require(digest(binary) == before_binary, 'Private native snapshot changed')
        require(inputs(project, SOURCES, LIMIT) == before_sources, 'Owned project inputs changed')
    require(not owned.exists(), 'Owned native fixture cleanup failed')
    require(inputs(project_source, SOURCES, LIMIT) == before_sources and
            inputs(snapshots, SNAPSHOTS, 16384) == before_snapshots and
            digest(binary_source) == before_binary, 'Original inputs changed during campaign')
    report = {
        'schema': 'zee-tr-en-native-1', 'result': 'passed', 'case_count': 7,
        'skip_count': 0, 'source_test_count': 4, 'source_test_failed': 0,
        'cases': outcomes, 'native_binary_sha256': before_binary,
        'native_identity_start': start_identity, 'native_identity_end': end_identity,
        'source_files': before_sources, 'snapshots': before_snapshots,
        'host': {'system': platform.system(), 'architecture': platform.machine(),
                 'python_version': platform.python_version()},
        'isolated_python': True, 'bytecode_disabled': True,
        'html_structure_checked': True, 'owned_temp_removed': True,
        'source_unchanged': True, 'compiler_feature': False,
        'live_product_acceptance': False, 'physical_platform_acceptance': False,
    }
    data = json.dumps(report, ensure_ascii=False, sort_keys=True) + '\n'
    if args.report:
        with Path(args.report).open('x', encoding='utf-8') as output:
            output.write(data)
    sys.stdout.write(data)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError, UnicodeError) as error:
        sys.stderr.write('K249 native campaign failed: ' + str(error) + '\n')
        sys.exit(1)
