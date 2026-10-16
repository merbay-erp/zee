#!/usr/bin/env python3
"""Prepare only. Twenty-three Campaign-body models; no real child, signal or socket.

Run later only with an explicitly reviewed --target support-source copy.
AST extraction omits all target module imports. Helper loading and every owner
are fake; this does not test Linux, PG, systemd or real process cleanup.
"""
import argparse, ast, contextlib, hashlib, json, os, re, shutil, signal, socket
import stat, subprocess, sys, tempfile, threading, time, types, unittest
from pathlib import Path
from unittest import mock

TARGET = None
WRITE_TEXT = Path.write_text
HELPER_TEXT = b'CANCELLED=False\n'


class Injected(Exception):
    pass


class Clock:
    def __init__(self): self.now = 100.0
    def monotonic(self): return self.now
    def sleep(self, seconds): self.now += seconds


class FakeChild:
    def __init__(self, owner, argv):
        self.owner, self.argv, self.returncode, self.reaped = owner, list(argv), None, False
    def close(self, code=0): self.returncode, self.reaped = code, True


class FakeResource:
    def __init__(self, callback=None, failure=False):
        self.callback, self.failure, self.closed = callback, failure, 0
    def close(self):
        self.closed += 1
        if self.callback: self.callback()
        if self.failure: raise Injected('fake owned resource cleanup')


class Harness:
    def __init__(self, case, target):
        self.case = case
        self.stack = contextlib.ExitStack()
        self.forbidden = []
        # Guard even calls swallowed by target BaseException cleanup handlers.
        for module, names in [(subprocess, ('Popen', 'run', 'call', 'check_call', 'check_output')),
                              (socket, ('socket', 'create_connection')),
                              (os, ('system', 'kill', 'killpg', 'fork', 'forkpty', 'posix_spawn', 'posix_spawnp',
                                    'spawnl', 'spawnle', 'spawnlp', 'spawnlpe', 'spawnv', 'spawnve', 'spawnvp', 'spawnvpe')),
                              (signal, ('signal', 'raise_signal', 'pthread_kill')),
                              (threading.Thread, ('start',))]:
            for name in names:
                if hasattr(module, name):
                    guard = self.stack.enter_context(mock.patch.object(module, name, side_effect=AssertionError('real operation forbidden: '+name)))
                    self.forbidden.append(guard)
        self.temp = tempfile.TemporaryDirectory(prefix='zee-k306-model-')
        self.root = Path(self.temp.name).resolve()
        self.repo, self.out = self.root/'repo', self.root/'output'
        self.repo.mkdir(); (self.repo/'fixed.txt').write_bytes(b'public modeled source\n')
        self.binary = self.root/'synthetic-binary'; self.binary.write_bytes(b'not executable\n')
        self.pg = self.root/'synthetic-pg'; self.pg.mkdir()
        for name in ('postgres', 'initdb', 'pg_ctl', 'psql'): (self.pg/name).write_bytes(b'not executable\n')
        self.clock = Clock()
        self.old = {signal.SIGTERM: object(), signal.SIGINT: object()}
        self.handlers = dict(self.old); self.captured = {}; self.children = []; self.calls = []; self.writes = []
        self.install_failure = False; self.on_clean = self.on_run = self.on_write = self.on_restore = None
        self.last_clean_had_handler = False; self.candidate = None
        self.primary = types.SimpleNamespace(CANCELLED=False, controlled_env=lambda: {})
        self.primary.run = lambda argv, **kw: self.run('normal', self.primary, argv, **kw)
        self.primary.clean_exact = self.clean_exact
        self.cleanup = types.ModuleType('fake_cleanup_owner')
        self.cleanup.CANCELLED = False
        self.cleanup.run = lambda argv, **kw: self.run('cleanup', self.cleanup, argv, **kw)
        self.fake_signal = types.SimpleNamespace(SIGTERM=signal.SIGTERM, SIGINT=signal.SIGINT,
                                                SIG_IGN=object(), signal=self.register)
        self.args = types.SimpleNamespace(repo=self.repo, dil=self.binary, out=self.out, sha='a'*40,
                                          binary_sha256=self.snapshot(self.binary), postgres_bin_dir=self.pg)
        timer = types.SimpleNamespace(D=self.primary, canonical=self.canonical, private_directory=lambda s: None,
                                      HELPER=hashlib.sha256(HELPER_TEXT).hexdigest())
        driver = types.SimpleNamespace(T=timer, snapshot=self.snapshot)
        util = types.SimpleNamespace(spec_from_loader=lambda *a, **kw: object(),
                                     module_from_spec=lambda spec: self.cleanup)
        raw = target.read_bytes()
        case.assertLessEqual(len(raw), 262144)
        tree = ast.parse(raw, filename=str(target))
        classes = [n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == 'Campaign']
        case.assertEqual(len(classes), 1)
        self.class_ast_sha256 = hashlib.sha256(ast.dump(classes[0], include_attributes=False).encode()).hexdigest()
        namespace = {'D': driver, 'HERE': self.root, 'Path': Path, 'hashlib': hashlib,
                     'json': json, 're': re, 'stat': stat, 'shutil': shutil, 'tempfile': tempfile,
                     'sys': types.SimpleNamespace(platform='linux'), 'os': types.SimpleNamespace(geteuid=lambda: 12345),
                     'signal': self.fake_signal, 'time': self.clock,
                     'importlib': types.SimpleNamespace(util=util), 'bounded_bytes': self.bounded_bytes,
                     'COMMON_SOURCES': ('fixed.txt',), 'DISPATCH_SOURCES': (), 'HTTP_SOURCES': ()}
        # Only class/function definitions are evaluated; module imports/main are excluded.
        exec(compile(ast.Module(body=classes, type_ignores=[]), str(target), 'exec'), namespace)
        self.Campaign = namespace['Campaign']
        def writer(path, text, *args, **kwargs):
            return self.write_text(path, text, *args, **kwargs)
        self.stack.enter_context(mock.patch.object(Path, 'write_text', writer))

    def canonical(self, path):
        path = Path(path)
        if not path.is_absolute() or path.resolve() != path: raise ValueError('modeled canonical path')
        return path

    def bounded_bytes(self, path):
        if path == self.root/'lifecycle_core.py': return HELPER_TEXT
        self.case.assertEqual(path, self.repo/'fixed.txt')
        return path.read_bytes()

    def snapshot(self, path, maximum=134217728):
        path = Path(path); self.case.assertTrue(path.is_relative_to(self.root))
        raw = path.read_bytes(); self.case.assertLessEqual(len(raw), maximum)
        return hashlib.sha256(raw).hexdigest()

    def register(self, number, handler):
        if callable(handler):
            if self.install_failure and number == signal.SIGINT: raise Injected('second handler install')
            self.captured[number] = handler
        old = self.handlers[number]; self.handlers[number] = handler
        if handler is self.old[number] and self.on_restore:
            callback, self.on_restore = self.on_restore, None; callback()
        return old

    def fire(self, number=signal.SIGTERM, captured=False):
        callback = self.captured.get(number) if captured else self.handlers[number]
        if callable(callback): callback(number, None); return True
        return False  # No actual OS signal; models absent/ignored registration only.

    def clean_exact(self, repo, sha):
        self.case.assertEqual((repo, sha), (self.repo, self.args.sha))
        self.last_clean_had_handler = callable(self.handlers[signal.SIGTERM])
        child = FakeChild('normal', ['modeled-git-clean']); self.children.append(child)
        try:
            if self.on_clean: self.on_clean()
            if self.primary.CANCELLED: raise Injected('modeled original query cancelled')
        finally: child.close(-15 if self.primary.CANCELLED else 0)

    def run(self, label, owner, argv, **kwargs):
        if owner.CANCELLED: raise Injected('fake owner cancelled before spawn')
        argv = [str(x) for x in argv]
        child = FakeChild(label, argv); self.children.append(child)
        self.calls.append({'owner': label, 'argv': argv, 'primary_cancelled': self.primary.CANCELLED,
                           'timeout': kwargs['timeout']})
        try:
            if argv[0] == 'git': data = b'fixed.txt\n'
            elif argv[0] == str(self.binary): data = json.dumps({'schema':'zee-build-v1','source_sha':self.args.sha,'dirty':False}).encode()
            elif argv[:3] == ['systemctl', '--user', 'is-system-running']: data = b'running\n'
            elif argv[:3] == ['systemctl', '--user', 'show']:
                data = b'not-found\n' if '--property=LoadState' in argv else b''
            elif argv[:3] in (['systemctl','--user','stop'], ['systemctl','--user','daemon-reload']): data = b''
            else: raise AssertionError('unexpected modeled command')
            kwargs['log_path'].write_bytes(data)
            if self.on_run: self.on_run(argv)
            if owner.CANCELLED: raise Injected('fake original command cancelled')
            return data
        finally: child.close(-15 if owner.CANCELLED else 0)

    def write_text(self, path, text, *args, **kwargs):
        result = WRITE_TEXT(path, text, *args, **kwargs)
        if path == self.out/'proof.json':
            value = json.loads(text); self.writes.append(value)
            if self.on_write:
                callback, self.on_write = self.on_write, None; callback()
        return result

    def make(self):
        self.candidate = self.Campaign.__new__(self.Campaign)
        self.Campaign.__init__(self.candidate, self.args, 'K282')
        return self.candidate

    def finished(self):
        campaign = self.make(); campaign.finish(); return campaign

    def exit(self): self.Campaign.__exit__(self.candidate, None, None, None)

    def assert_failed_receipt(self):
        self.case.assertEqual(self.candidate.proof['status'], 'failed')
        self.case.assertEqual(json.loads((self.out/'proof.json').read_text())['status'], 'failed')

    def assert_restored(self): self.case.assertEqual(self.handlers, self.old)

    def dispose(self):
        try:
            for guard in self.forbidden: guard.assert_not_called()
            self.case.assertTrue(all(c.reaped and c.returncode is not None for c in self.children))
        finally:
            self.stack.close(); self.temp.cleanup()


class Cases(unittest.TestCase):
    def setUp(self):
        self.h = Harness(self, TARGET)
        self.addCleanup(self.h.dispose)

    def test_01_normal_finished(self):
        h=self.h; c=h.finished(); c.units['zee-owned.service']=True
        before=len(h.calls); h.exit()
        self.assertEqual(c.proof['status'],'passed'); h.assert_restored()
        self.assertFalse(c.stopped); self.assertFalse(h.primary.CANCELLED)
        self.assertEqual([r['argv'][2] for r in h.calls[before:]],['stop','show','daemon-reload','show'])

    def test_02_early_clean_cancel(self):
        h=self.h
        def cancel(): h.fire(); raise Injected('modeled interrupted first query')
        h.on_clean=cancel
        with self.assertRaises(Injected): h.make()
        self.assertTrue(h.last_clean_had_handler, 'structural: handler must precede first query')
        h.assert_failed_receipt(); h.assert_restored()
        self.assertTrue(h.primary.CANCELLED); self.assertEqual(h.calls,[])

    def test_03_partial_prepare_failure(self):
        h=self.h; resource=FakeResource()
        def fail(argv):
            if argv[0]=='git': h.candidate.resources.append(resource); raise Injected('partial prepare')
        h.on_run=fail
        with self.assertRaises(Injected): h.make()
        self.assertIs(h.candidate.resources[0],resource); self.assertEqual(resource.closed,1)
        h.assert_failed_receipt(); h.assert_restored()

    def test_04_first_handler_install_failure(self):
        h=self.h; h.install_failure=True
        with self.assertRaises(Injected): h.make()
        h.assert_restored(); self.assertEqual(h.children,[])
        self.assertFalse(h.out.exists())
        self.assertEqual(h.calls,[])
        self.assertIsNone(h.candidate.cleanup_owner)
        self.assertEqual((h.candidate.units,h.candidate.links,h.candidate.resources,h.candidate.clusters),({},[],[],[]))

    def test_05_cancel_before_close(self):
        h=self.h; c=h.finished(); resource=FakeResource(); c.resources.append(resource); h.fire(signal.SIGINT)
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored()
        self.assertTrue(c.stopped and h.primary.CANCELLED); self.assertEqual(resource.closed,1)

    def test_06_cancel_inside_unlink(self):
        h=self.h; c=h.finished(); c.units['zee-owned.service']=True
        h.on_run=lambda argv: h.fire() if argv[:3]==['systemctl','--user','stop'] else None
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored(); self.assertEqual(c.units,{})
        self.assertTrue(c.stopped and h.primary.CANCELLED)

    def test_07_cancel_inside_resource_close(self):
        h=self.h; c=h.finished(); resource=FakeResource(h.fire); c.resources.append(resource)
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored(); self.assertEqual(resource.closed,1)

    def test_08_cancel_during_proof_write(self):
        h=self.h; h.finished(); h.on_write=h.fire
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored()
        self.assertEqual([v['status'] for v in h.writes],['passed','failed'])

    def test_09_cancel_during_handler_restore(self):
        h=self.h; h.finished(); h.on_restore=lambda: h.fire(captured=True)
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored()

    def test_10_cancel_at_final_exit_check(self):
        h=self.h; c=h.finished(); real_close=c.close
        def close(): real_close(); h.fire(captured=True)
        with mock.patch.object(c,'close',side_effect=close),self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored()

    def test_11_expired_campaign_entry(self):
        h=self.h; c=h.finished(); deadline=c.deadline; h.clock.now=deadline+1
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored(); self.assertEqual(c.deadline,deadline)

    def test_12_expired_cleanup_publication(self):
        h=self.h; h.finished(); h.on_write=lambda: setattr(h.clock,'now',h.clock.now+301)
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored()

    def test_13_cleanup_failure(self):
        h=self.h; c=h.finished(); good=FakeResource(); bad=FakeResource(failure=True); c.resources.extend([good,bad])
        temp=h.root/'retained'; temp.mkdir(); info=temp.stat(); c.tempdirs.append((temp,(info.st_dev,info.st_ino)))
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored(); self.assertTrue(temp.exists())
        self.assertEqual((good.closed,bad.closed),(1,1))

    def test_14_cleanup_state_separation(self):
        h=self.h; c=h.finished(); c.units['zee-owned.service']=True; h.primary.CANCELLED=True
        before=len(h.children)
        with self.assertRaises((Injected,ValueError)): c.command('must-refuse',['systemctl','--user','start','foreign'])
        self.assertEqual(len(h.children),before); start=len(h.calls)
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); h.assert_restored(); self.assertTrue(h.primary.CANCELLED)
        cleanup=h.calls[start:]; self.assertEqual(len(cleanup),4)
        self.assertTrue(all(r['owner']=='cleanup' and r['primary_cancelled'] for r in cleanup))
        self.assertFalse(h.cleanup.CANCELLED)

    def test_15_finish_then_body_exception(self):
        h=self.h; c=h.finished(); resource=FakeResource(); c.resources.append(resource)
        error=Injected('body exception after finish')
        with self.assertRaises(Injected) as caught:
            with c: raise error
        self.assertIs(caught.exception,error)
        h.assert_failed_receipt(); h.assert_restored(); self.assertTrue(c.failed_body)
        self.assertEqual(resource.closed,1)
        self.assertTrue(all(v['status']=='failed' for v in h.writes))

    def test_16_repeated_close_cannot_erase_failure(self):
        h=self.h; c=h.finished(); resource=FakeResource(failure=True); c.resources.append(resource)
        with self.assertRaises(ValueError): h.exit()
        h.assert_failed_receipt(); resource.failure=False; before=len(h.children)
        c.close()
        h.assert_failed_receipt(); h.assert_restored()
        self.assertEqual(len(h.children),before)
        self.assertTrue(all(v['status']=='failed' for v in h.writes))


def owned_reset_campaign_class(path,driver):
    tree=ast.parse(path.read_bytes());original=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name=='Campaign')
    names={'need','stop','unlink','close','__exit__'}
    cls=ast.ClassDef(name='CampaignModel',bases=[],keywords=[],body=[n for n in original.body if isinstance(n,ast.FunctionDef) and n.name in names],decorator_list=[])
    env={'D':driver,'Path':Path,'time':time,'signal':signal,'json':json,'stat':stat,'shutil':shutil}
    exec(compile(ast.fix_missing_locations(ast.Module(body=[cls],type_ignores=[])),'actual-Campaign-methods','exec'),env)
    return env['CampaignModel']


class OwnedUnitResetModels(unittest.TestCase):
    def setUp(self):
        for obj,name in ((subprocess,'Popen'),(subprocess,'run'),(socket,'socket'),(socket,'create_connection'),(os,'kill'),(os,'killpg'),(signal,'signal')):
            patch=mock.patch.object(obj,name,side_effect=AssertionError('real action forbidden: '+name));guard=patch.start();self.addCleanup(patch.stop);self.addCleanup(guard.assert_not_called)
        self.tmp=tempfile.TemporaryDirectory(prefix='k306-unit-reset-model-');self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name).resolve(strict=True)
    def fixture(self,foreign=False,reset_error=False,not_inactive=False,empty=False,cancel_on_reset=False,replace_on_reset=False,unit_name='zee-dispatch.service',initial_state='failed',unloaded=False):
        testroot=self.root/str(len(list(self.root.iterdir())));testroot.mkdir();targetdir=testroot/'runtime';targetdir.mkdir()
        source=testroot/unit_name;source.write_bytes(b'[Service]\nExecStart=/synthetic/inert\n')
        link=targetdir/source.name;link.symlink_to(source);info=link.lstat()
        driver=types.SimpleNamespace(T=types.SimpleNamespace(D=types.SimpleNamespace(CANCELLED=False)),snapshot=lambda p,limit:hashlib.sha256(p.read_bytes()).hexdigest())
        klass=owned_reset_campaign_class(TARGET,driver);c=klass()
        c.units={link.name:True};c.links=[{'path':link,'target':source,'identity':(info.st_dev,info.st_ino),'sha256':driver.snapshot(source,65536)}]
        c.stopped=False;c.cleaning=False;c.deadline=time.monotonic()+120;c.cleanup_deadline=None
        c.commands=[];c.tempdirs=[];c.clusters=[];c.resources=[];c.previous={};c.finished=True;c.failed_body=False;c.output_created=False
        c.proof={'status':'running','cases':[{'name':n,'passed':True} for n in ('case1','case2','case3','case4')]}
        failed={link.name} if initial_state=='failed' else set();states={link.name:initial_state};calls=[];events=[]
        if foreign:failed.add('foreign.service');states['foreign.service']='failed'
        def system(label,*args,budget=25):
            calls.append((label,args));events.append(('system',args))
            if args[0]=='stop':return b''  # Stopped failed unit retains its failed result.
            if args[0]=='reset-failed':
                self.assertEqual(len(args),2);self.assertEqual(args[1],link.name)
                self.assertTrue(link.is_symlink());self.assertEqual(link.resolve(),source)
                if unloaded:raise ValueError('Unit '+link.name+' not loaded')
                if reset_error:raise ValueError('inert reset rejected')
                if cancel_on_reset:c.stopped=True;driver.T.D.CANCELLED=True
                failed.discard(args[1]);states[args[1]]='active' if not_inactive else 'inactive'
                if replace_on_reset:
                    alternate=testroot/'other.service';alternate.write_bytes(source.read_bytes());link.unlink();link.symlink_to(alternate)
                return b''
            if args[0]=='show':
                if '--property=ActiveState' in args:return states[args[1]].encode()+b'\n'
                if '--property=LoadState' in args:return b'loaded\n' if link.is_symlink() else b'not-found\n'
            if args[0]=='daemon-reload':self.assertFalse(link.is_symlink());return b''
            raise AssertionError('unexpected system command '+repr(args))
        def cgroup(name):events.append(('cgroup-empty',name));return not empty
        c.system=system;c.empty=cgroup
        return c,failed,calls,events,link,driver
    def test_candidate_resets_exact_owned_link_after_stop_and_cgroup_before_unlink(self):
        c,failed,calls,events,link,driver=self.fixture();cases=list(c.proof['cases']);c.__exit__(None,None,None)
        self.assertEqual(failed,set());self.assertEqual(c.proof['status'],'passed');self.assertEqual(c.proof['cases'],cases)
        self.assertFalse(link.is_symlink());self.assertFalse(c.units);self.assertFalse(c.links)
        resets=[args for _,args in calls if args[0]=='reset-failed'];self.assertEqual(resets,[('reset-failed','zee-dispatch.service')])
        self.assertLess(events.index(('cgroup-empty','zee-dispatch.service')),events.index(('system',resets[0])))
    def test_foreign_failed_unit_is_never_reset(self):
        c,failed,calls,events,link,driver=self.fixture(foreign=True);c.__exit__(None,None,None)
        self.assertEqual(failed,{'foreign.service'});self.assertEqual(c.proof['status'],'passed')
        self.assertFalse(any('foreign.service' in args for _,args in calls))
    def test_cgroup_or_owned_path_mismatch_prevents_reset(self):
        for mode in ('cgroup','hash','target','membership'):
            with self.subTest(mode=mode):
                c,failed,calls,events,link,driver=self.fixture(empty=mode=='cgroup')
                if mode=='hash':c.links[0]['sha256']='0'*64
                if mode=='target':c.links[0]['target']=self.root/'foreign.service'
                if mode=='membership':c.units.clear()
                with self.assertRaises(ValueError):c.unlink()
                self.assertFalse(any(args[0]=='reset-failed' for _,args in calls));self.assertTrue(link.is_symlink())
    def test_reset_failure_or_active_state_prevents_success_and_unlink(self):
        for mode in ('error','active','replacement'):
            with self.subTest(mode=mode):
                c,failed,calls,events,link,driver=self.fixture(reset_error=mode=='error',not_inactive=mode=='active',replace_on_reset=mode=='replacement')
                with self.assertRaisesRegex(ValueError,'owned cleanup failed'):c.__exit__(None,None,None)
                self.assertEqual(c.proof['status'],'failed');self.assertTrue(c.proof['cleanup_errors']);self.assertTrue(link.is_symlink());self.assertTrue(c.units)
    def test_cancelled_cleanup_remains_failed_after_exact_reset(self):
        c,failed,calls,events,link,driver=self.fixture(cancel_on_reset=True)
        with self.assertRaisesRegex(ValueError,'owned cleanup failed'):c.__exit__(None,None,None)
        self.assertEqual(c.proof['status'],'failed');self.assertTrue(c.proof['cancelled']);self.assertEqual(failed,set())
        self.assertFalse(link.is_symlink());self.assertFalse(c.units)


    def test_inactive_unloaded_owned_timer_skips_reset_and_releases(self):
        c,failed,calls,events,link,driver=self.fixture(unit_name='zee-dispatch.timer',initial_state='inactive',unloaded=True)
        c.__exit__(None,None,None)
        self.assertEqual(c.proof['status'],'passed');self.assertFalse(link.is_symlink());self.assertFalse(c.units);self.assertFalse(c.links)
        self.assertEqual(failed,set());self.assertFalse(any(args[0]=='reset-failed' for _,args in calls))
        states=[args for _,args in calls if args[0]=='show' and '--property=ActiveState' in args]
        self.assertEqual(states,[('show','zee-dispatch.timer','--property=ActiveState','--value')]*2)
        self.assertLess(events.index(('cgroup-empty','zee-dispatch.timer')),events.index(('system',states[0])))
    def test_active_or_unknown_owned_prestate_rejected_without_reset_or_unlink(self):
        for state in ('active','activating','deactivating','reloading','maintenance','unknown',''):
            with self.subTest(state=state):
                c,failed,calls,events,link,driver=self.fixture(initial_state=state)
                with self.assertRaisesRegex(ValueError,'owned cleanup failed'):c.__exit__(None,None,None)
                self.assertEqual(c.proof['status'],'failed');self.assertTrue(c.proof['cleanup_errors']);self.assertTrue(link.is_symlink());self.assertTrue(c.units)
                self.assertFalse(any(args[0]=='reset-failed' for _,args in calls))


if __name__=='__main__':
    parser=argparse.ArgumentParser(allow_abbrev=False); parser.add_argument('--target',type=Path,required=True)
    options,remaining=parser.parse_known_args(); TARGET=options.target.resolve(strict=True)
    unittest.main(argv=[sys.argv[0],*remaining])
