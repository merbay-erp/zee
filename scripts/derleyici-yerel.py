#!/usr/bin/env python3
"""K270: explicit POSIX local bundle publication and owned project launcher."""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import stat
import sys
import tempfile

SELECTOR = Path(__file__).resolve().with_name('derleyici-secici.py')
try:
    S = runpy.run_path(str(SELECTOR))
    Reject = S['Reject']
    require = S['require']
    regular_bytes = S['regular_bytes']
    if not isinstance(Reject, type) or not issubclass(Reject, ValueError) or not callable(require) or not callable(regular_bytes):
        raise TypeError('invalid bootstrap exports')
except Exception:
    print('Yerel araç işlemi reddedildi.', file=sys.stderr)
    raise SystemExit(2) from None
PROFILE = 'zee-local-compiler-tools-posix-1'


class PublicationError(Reject):
    def __init__(self, published):
        self.published = published
        super().__init__('publication state')


def owned_directory(path):
    path = Path(path)
    info = path.lstat()
    require(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid() and
            not info.st_mode & 0o022, 'owned directory required')
    return path


def identity(info):
    return info.st_dev, info.st_ino


@contextmanager
def exclusive_lock(parent, name):
    lock = parent / name
    fd = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600)
    acquired = identity(os.fstat(fd))
    def still_owned():
        try:
            info = lock.lstat()
            return stat.S_ISREG(info.st_mode) and identity(info) == acquired
        except FileNotFoundError:
            return False
    try:
        yield still_owned
    finally:
        # Keep the acquired inode alive until this identity check and unlink.
        # Hostile same-UID namespace races are outside the owned-store boundary.
        try:
            if still_owned():
                lock.unlink()
        finally:
            os.close(fd)


def fsync_directory(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def write_files(stage, files, executables):
    for name, data in files.items():
        require(re.fullmatch(r'[A-Za-z0-9_.-]+', name) is not None, 'local filename required')
        with (stage / name).open('xb') as handle:
            os.chmod(stage / name, 0o700 if name in executables else 0o600)
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
    fsync_directory(stage)


def exact_files(directory, expected):
    names = set()
    observed = 0
    with os.scandir(directory) as entries:
        for entry in entries:
            require(observed < len(expected) and entry.name not in names, 'unexpected inventory')
            observed += 1
            names.add(entry.name)
    require(names == set(expected), 'unexpected inventory')
    return {name: regular_bytes(directory / name, max(len(data), 1)) for name, data in expected.items()}


def publish(parent, name, files, executables, idempotent):
    parent = owned_directory(parent)
    require(re.fullmatch(r'[A-Za-z0-9_.-]+', name) is not None, 'target component required')
    target = parent / name
    published = False
    with exclusive_lock(parent, '.' + name + '.lock') as still_owned:
        stage = Path(tempfile.mkdtemp(prefix='.' + name + '.staging-', dir=parent))
        try:
            write_files(stage, files, executables)
            require(still_owned(), 'publication lock replaced')
            if target.exists() or target.is_symlink():
                require(idempotent, 'target already exists')
                owned_directory(target)
                require(exact_files(target, files) == files, 'existing payload differs')
                return {'schema': 'zee-local-publication-1', 'published': False, 'existing_exact': True}
            os.rename(stage, target)
            published = True
            try:
                fsync_directory(parent)
            except OSError:
                # Publication already happened. Do not delete this target.
                raise PublicationError(True) from None
            return {'schema': 'zee-local-publication-1', 'published': True, 'existing_exact': False}
        except OSError:
            raise PublicationError(published) from None
        finally:
            if not published:
                shutil.rmtree(stage)


def import_bundle(project, source, store, trust, key, verifier, verifier_sha, pilot=False):
    require(os.name == 'posix', 'POSIX profile required')
    project = owned_directory(S['root_directory'](project))
    store = owned_directory(S['root_directory'](store))
    # The explicit source itself must not be a symlink. Parent path is operator-selected.
    source = owned_directory(Path(source).absolute())
    pin = S['load_json'](regular_bytes(project / S['PIN_FILE'], S['LIMITS']['pin']))
    with S['authenticated_snapshot'](pin, store, trust, key, verifier, verifier_sha, pilot, project,
                                     bundle_source=source) as (snapshot, _):
        files = {name: regular_bytes(snapshot / name, S['META_LIMITS'].get(name, S['LIMITS']['executable']))
                 for name in S['META'] | S['binary_names'](pin)}
        parent = store / pin['source_sha']
        try:
            parent.mkdir(mode=0o700)
            fsync_directory(store)
        except FileExistsError:
            pass
        owned_directory(parent)
        return publish(parent, pin['platform'], files, S['binary_names'](pin), True)


# Standalone bootstrap. Only the trusted selector is private-copied/executed;
# its current imports are stdlib-only. No project module or shell evaluation.
BOOTSTRAP = '''import hashlib,json,os,runpy,stat,sys,tempfile
from pathlib import Path
POLICY = __POLICY__
def read_snapshot(path,limit):
 p=Path(path); before=p.lstat()
 if not stat.S_ISREG(before.st_mode) or before.st_size>limit: raise ValueError()
 fd=os.open(p,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
 try:
  opened=os.fstat(fd)
  if not stat.S_ISREG(opened.st_mode) or (opened.st_dev,opened.st_ino,opened.st_size,opened.st_mtime_ns)!=(before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns): raise ValueError()
  with os.fdopen(fd,'rb',closefd=False) as f: data=f.read(limit+1)
  after=os.fstat(fd); named=p.lstat()
  if len(data)>limit or len(data)!=opened.st_size or (after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns)!=(opened.st_dev,opened.st_ino,opened.st_size,opened.st_mtime_ns) or (named.st_dev,named.st_ino,named.st_size,named.st_mtime_ns)!=(opened.st_dev,opened.st_ino,opened.st_size,opened.st_mtime_ns): raise ValueError()
  return data
 finally: os.close(fd)
def main():
 if not sys.flags.isolated or str(Path(sys.executable).resolve())!=POLICY['python']: raise ValueError()
 own=Path(__file__).resolve().parent
 project=Path(POLICY['project'])
 for p in (own,project):
  st=p.lstat()
  if not stat.S_ISDIR(st.st_mode) or st.st_uid!=os.getuid() or st.st_mode&0o022: raise ValueError()
 data=read_snapshot(own/'selector.py',1048576)
 if hashlib.sha256(data).hexdigest()!=POLICY['selector_sha256']: raise ValueError()
 mode=sys.argv[1] if len(sys.argv)>1 else 'çalıştır'
 if mode not in ('seç','çalıştır','lsp'): raise ValueError()
 tail=sys.argv[2:]
 if mode!='çalıştır' and tail: raise ValueError()
 with tempfile.TemporaryDirectory(prefix='zee-owned-selector-') as temporary:
  script=Path(temporary)/'selector.py';script.write_bytes(data);script.chmod(0o600)
  args=[POLICY['python'],'-I',str(script),mode,'--proje',POLICY['project'],'--depo',POLICY['store'],'--guven',POLICY['trust'],'--guven-anahtari',POLICY['key'],'--dogrulayici',POLICY['verifier'],'--dogrulayici-sha256',POLICY['verifier_sha256']]
  if POLICY['pilot']: args+=['--pilot']
  if mode=='çalıştır': args+=['--']+tail
  module=runpy.run_path(str(script))
  return module['main'](args[3:])
try: sys.exit(main())
except Exception:
 print('Yerel launcher reddedildi.',file=sys.stderr);sys.exit(2)
'''


def install_launcher(project, store, trust, key, verifier, verifier_sha, pilot=False):
    require(os.name == 'posix', 'POSIX profile required')
    project = owned_directory(S['root_directory'](project))
    store = owned_directory(S['root_directory'](store))
    pin = S['load_json'](regular_bytes(project / S['PIN_FILE'], S['LIMITS']['pin']))
    with S['authenticated_snapshot'](pin, store, trust, key, verifier, verifier_sha, pilot, project):
        pass
    interpreter = str(Path(sys.executable).resolve())
    require(not any(c.isspace() for c in interpreter), 'bootstrap path unsupported')
    selector = regular_bytes(SELECTOR, 1024 * 1024)
    policy = {'project': str(project), 'store': str(store), 'trust': str(Path(trust).absolute()),
              'key': key, 'verifier': str(Path(verifier).absolute()), 'verifier_sha256': verifier_sha,
              'python': interpreter, 'selector_sha256': hashlib.sha256(selector).hexdigest(), 'pilot': pilot}
    launcher = ('#!' + interpreter + ' -I\n' + BOOTSTRAP.replace('__POLICY__', repr(policy))).encode()
    files = {'zee': launcher, 'selector.py': selector}
    receipt = {'schema': 'zee-project-launcher-1', 'profile': PROFILE, 'uid': os.getuid(),
               'files': [{'name': name, 'sha256': hashlib.sha256(data).hexdigest()} for name, data in sorted(files.items())]}
    files['receipt.json'] = (json.dumps(receipt, indent=2) + '\n').encode()
    return publish(project, '.zee-araclar', files, {'zee'}, False)


class MaskedParser(argparse.ArgumentParser):
    def error(self, message):
        raise Reject('invalid arguments')
    def exit(self, status=0, message=None):
        raise Reject('argument exit')


def main(argv=None):
    parser = MaskedParser(description=__doc__, allow_abbrev=False, add_help=False)
    parser.add_argument('command', choices=('import', 'launcher'))
    for name in ('proje', 'depo', 'guven', 'guven-anahtari', 'dogrulayici', 'dogrulayici-sha256'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--kaynak')
    parser.add_argument('--pilot', action='store_true')
    raw = sys.argv[1:] if argv is None else argv
    keys = [a.split('=', 1)[0] for a in raw if a.startswith('--')]
    require(len(keys) == len(set(keys)), 'duplicate arguments')
    args = parser.parse_args(raw)
    params = (args.proje, args.depo, args.guven, args.guven_anahtari, args.dogrulayici, args.dogrulayici_sha256, args.pilot)
    if args.command == 'import':
        require(args.kaynak is not None, 'source required')
        result = import_bundle(params[0], args.kaynak, *params[1:])
    else:
        require(args.kaynak is None, 'source not allowed')
        result = install_launcher(*params)
    print(json.dumps(result))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except PublicationError as error:
        print(json.dumps({'schema': 'zee-local-publication-1', 'error': 'publication_failed',
                          'published': error.published}), file=sys.stderr)
        raise SystemExit(2)
    except (Exception, KeyboardInterrupt):
        print('Yerel araç işlemi reddedildi.', file=sys.stderr)
        raise SystemExit(2)
