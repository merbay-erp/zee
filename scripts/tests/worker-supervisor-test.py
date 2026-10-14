#!/usr/bin/env python3
"""K-246 strict synthetic negatives and two actual native Zee workers."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import selectors
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
import unittest
sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "scripts/worker-supervisor.py"
LAUNCHER = ROOT / "dogfood/zeedili-com/scripts/yerel-baslat.sh"
REVISION = "2" * 40
FAKE_SHA = "1" * 40
CANARY = "synthetic-secret-canary-K246"
SYNTHETIC_ENV = "ZEE_K246_SYNTHETIC_KEY"
NATIVE = None
NODE = shutil.which("node")


def port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def listening(value):
    try:
        with socket.create_connection(("127.0.0.1", value), timeout=.1):
            return True
    except OSError:
        return False


def gate(path, ready, revision=REVISION):
    value = {"schema": "zee-worker-readiness-1", "application_revision": revision, "ready": ready}
    temporary = path.with_name("gate.next")
    temporary.write_text(json.dumps(value))
    temporary.chmod(0o600)
    temporary.replace(path)


FAKE_SOURCE = r'''import json,os,signal,socket,sys,time
from pathlib import Path
base=Path(__file__).parent
record={"schema":"zee-build-v1","version":"0.8.0-dev","source_sha":"1"*40,"source_date":"2026-10-14","channel":"development","dirty":False}
if sys.argv[1]=="sürüm":
    (base/"query.called").write_text(str(os.getpid()))
    mode=(base/"query-mode").read_text() if (base/"query-mode").exists() else "normal"
    if mode=="sleep": time.sleep(10)
    if mode=="large": print("x"*100000);sys.exit(0)
    if mode=="bad": print("not-json");sys.exit(0)
    if mode=="dirty": record["dirty"]=True
    print(json.dumps(record));sys.exit(0)
project=Path(sys.argv[-1]);mode=(project/"mode").read_text()
p=int(sys.argv[sys.argv.index("--web-worker-port")+1])
(project/("worker-"+str(p)+".pid")).write_text(str(os.getpid()))
(project/("worker-"+str(p)+".env.json")).write_text(json.dumps(sorted(os.environ)))
g=project/"readiness.json"
if mode!="stale-gate" and g.is_file() and not g.is_symlink():
    data=g.read_bytes();tmp=project/("gate-"+str(os.getpid())+".next");tmp.write_bytes(data);tmp.chmod(0o600);tmp.replace(g)
if mode=="crash":sys.exit(7)
if mode=="stuck":signal.signal(signal.SIGTERM,signal.SIG_IGN)
if mode=="secret":
    os.write(1,b"synthetic-secret-canary-K246"*50000)
    os.write(2,b"synthetic-secret-canary-K246"*50000+b"\xff")
s=socket.socket();s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1);s.bind(("127.0.0.1",p));s.listen(8)
while True:
    c,_=s.accept()
    with c:
        c.settimeout(1)
        try:
            request=c.recv(4096)
            if not request:continue
            if mode=="delay":time.sleep(2)
            body=b"ok"
            status="503 Unavailable" if mode=="503" else "200 OK"
            build=dict(record)
            if mode=="forged":build["source_sha"]="3"*40
            response=("HTTP/1.1 "+status+"\r\nContent-Length: 2\r\nX-Zee-Build: "+json.dumps(build)+"\r\nConnection: close\r\n\r\n").encode()+body
            c.sendall(response)
        except (OSError,TimeoutError):pass
'''


class Running:
    def __init__(self, argv, env=None):
        self.child = subprocess.Popen(argv, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.child.stdout, selectors.EVENT_READ)
        self.buffer = bytearray()
        self.events = []

    def until(self, predicate, timeout=8, after=0):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if any(predicate(event) for event in self.events[after:]):
                return next(event for event in reversed(self.events[after:]) if predicate(event))
            for key, _ in self.selector.select(.05):
                chunk = os.read(key.fileobj.fileno(), 4096)
                if not chunk:
                    self.selector.unregister(key.fileobj)
                    continue
                self.buffer.extend(chunk)
                while b"\n" in self.buffer:
                    line, _, rest = self.buffer.partition(b"\n")
                    self.buffer = bytearray(rest)
                    self.events.append(json.loads(line))
            if self.child.poll() is not None and not self.selector.get_map():
                break
        error=AssertionError("expected bounded event missing")
        error.timeline=self.events[-128:]
        raise error

    def finish(self, stop=True):
        if stop and self.child.poll() is None:
            self.child.terminate()
        output, error = self.child.communicate(timeout=8)
        self.selector.close()
        pending = bytes(self.buffer) + output
        for line in pending.splitlines():
            self.events.append(json.loads(line))
        self.buffer.clear()
        if error:
            raise AssertionError("supervisor stderr must stay empty")
        return self.child.returncode


class Fixture:
    def __init__(self, native=False, mode="normal", ready=True, query="normal", extra=(), app_signal=False, credential=False):
        self.temporary = tempfile.TemporaryDirectory(prefix="zee-supervisor-k246-")
        self.root = Path(self.temporary.name).resolve()
        self.project = self.root / "project"
        self.project.mkdir(mode=0o700)
        self.ports = [port(), port()]
        while self.ports[1] == self.ports[0]:
            self.ports[1] = port()
        self.gate = self.project / "readiness.json"
        gate(self.gate, ready)
        if native:
            self.dil = NATIVE
            self.build = json.loads(subprocess.check_output([str(self.dil), "sürüm", "--json"], timeout=5))
            self.source = self.project / "program.dil"
            prefix = ''
            capabilities = ['ağ-sunucusu']
            metadata = ''
            if credential:
                prefix += 'parola "'+CANARY+'" olsun\nsifreli parolanın gizli şifrelenmişi olsun\nsonuç sifreli gizli çözmeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\ndeğilse\n    programı 1 ile bitir\n'
                capabilities.append('kriptografi')
                metadata = 'gizli_anahtar_değişkeni "ZEE_K246_SYNTHETIC_KEY" olsun\n'
            if ready or app_signal or credential:
                gate(self.gate, False)
                capabilities.append('dosya-yazma')
                payload = json.dumps({'schema':'zee-worker-readiness-1','application_revision':REVISION,'ready':True},separators=(',',':'))
                prefix += '"readiness.json" dosyasına '+json.dumps(payload)+' yaz\n'
            self.source.write_text(prefix+'8080 kapısında sunucu başlat\n\nGET "/hazir" adresine istek geldiğinde\n    "ok" yanıtını gönder\n\nGET "/yavas" adresine istek geldiğinde\n    3 saniye bekle\n    "bitti" yanıtını gönder\n')
            (self.project / "proje.dil").write_text('proje "fixture" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "program.dil" olsun\nyetkinlikler '+', '.join(json.dumps(x,ensure_ascii=False) for x in capabilities)+' listesi olsun\nağ_hedefleri boş liste olsun\nyerel_bağımlılıklar boş liste olsun\n'+metadata)
        else:
            self.dil = self.root / "fake-dil"
            self.dil.write_text("#!" + sys.executable + "\n" + FAKE_SOURCE)
            self.dil.chmod(0o700)
            (self.root / "query-mode").write_text(query)
            (self.project / "mode").write_text(mode)
            self.build = {"source_sha": FAKE_SHA}
        self.args = [sys.executable, "-I", "-B", str(TOOL), "--dil", str(self.dil),
                     "--project", str(self.project), "--origin", "https://fixture.example",
                     "--expected-source", self.build["source_sha"], "--application-revision", REVISION,
                     "--readiness-file", str(self.gate), "--health-path", "/hazir", "--health-body", "ok",
                     "--worker-port", str(self.ports[0]), "--worker-port", str(self.ports[1]),
                     "--startup-timeout", "3", "--probe-timeout", ".5", "--stop-grace", ".2", "--pilot"] + list(extra)
        if credential:
            credentials = self.root / 'credentials'
            credentials.mkdir(mode=0o700)
            key = credentials / 'key'
            key.write_text('a5'*32)
            key.chmod(0o400)
            self.args += ['--credential-root',str(credentials),'--credential-env',SYNTHETIC_ENV+'='+'key']
        self.running = None

    def start(self, env=None):
        self.running = Running(self.args, env)
        return self.running

    def close(self):
        if self.running is not None and self.running.child.poll() is None:
            self.running.finish()
        for value in self.ports:
            if listening(value):
                raise AssertionError("owned worker still listening after cleanup")
        self.temporary.cleanup()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


class Synthetic(unittest.TestCase):
    def rejection(self, **kwargs):
        with Fixture(**kwargs) as f:
            run = f.start()
            self.assertNotEqual(run.finish(False), 0)
            self.assertFalse(any(x.get("ready") is True for x in run.events))
            return run.events

    def test_application_gate_false_never_ready(self):
        self.rejection(ready=False)
        self.rejection(mode="stale-gate")

    def test_application_revision_wrong_never_ready(self):
        malformed = [json.dumps({'schema':'zee-worker-readiness-1','application_revision':'3'*40,'ready':True}),
                     '{"schema":"zee-worker-readiness-1","application_revision":"'+REVISION+'","ready":true,"future":1}',
                     '{"ready":true,"ready":false}', '['*10000+']'*10000]
        for value in malformed:
            with self.subTest(kind=value[:12]), Fixture() as f:
                f.gate.write_text(value)
                run = f.start()
                self.assertNotEqual(run.finish(False), 0)
                self.assertFalse(any(x.get("ready") is True for x in run.events))

    def test_fifo_gate_no_writer_is_bounded(self):
        with Fixture() as f:
            f.gate.unlink()
            os.mkfifo(f.gate, 0o600)
            run = f.start()
            self.assertNotEqual(run.finish(False), 0)

    def test_forged_health_build_rejected(self):
        self.rejection(mode="forged")

    def test_health_503_rejected(self):
        self.rejection(mode="503")
        # Real TCP peer delivers valid200 after2s; a5s per-probe allowance must
        # not extend the remaining .3s global startup budget.
        spec=importlib.util.spec_from_file_location('supervisor_deadline',TOOL)
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        with Fixture(mode='delay') as f:
            values=f.args[f.args.index(str(TOOL))+1:]
            values[values.index('--probe-timeout')+1]='5'
            a=module.arguments(values)
            owner=module.Supervisor(a,type('Quiet',(),{'emit':lambda *args,**kwargs:None})())
            try:
                owner.build=module.validate_build(owner.query_json('--json'),a)
                owner.spawn([str(f.dil),'çalıştır','--web-worker-port',str(f.ports[0]),str(f.project)])
                deadline=time.monotonic()+3
                while not listening(f.ports[0]) and time.monotonic()<deadline:time.sleep(.02)
                self.assertTrue(listening(f.ports[0]))
                begin=time.monotonic()
                self.assertFalse(owner.probe(f.ports[0],begin+.3))
                self.assertLess(time.monotonic()-begin,1.5)
            finally:
                owner.cleanup()

    def test_child_crash_cleans_other_child(self):
        self.rejection(mode="crash")

    def test_secret_dump_is_not_forwarded_or_blocking(self):
        with Fixture(mode="secret") as f:
            env=dict(os.environ,SHARED_SECRET=CANARY,HTTP_PROXY='https://'+CANARY+'.example',PYTHONPATH=str(f.root))
            run = f.start(env)
            run.until(lambda x: x.get("ready") is True)
            for value in f.ports:
                keys=json.loads((f.project/('worker-'+str(value)+'.env.json')).read_text())
                self.assertNotIn('SHARED_SECRET',keys)
                self.assertNotIn('HTTP_PROXY',keys)
                self.assertNotIn('PYTHONPATH',keys)
            self.assertEqual(run.finish(), 0)
            self.assertNotIn(CANARY, json.dumps(run.events))

    def test_stuck_worker_is_killed_and_reaped(self):
        with Fixture(mode="stuck") as f:
            run = f.start()
            run.until(lambda x: x.get("ready") is True)
            before = time.monotonic()
            self.assertEqual(run.finish(), 0)
            self.assertLess(time.monotonic() - before, 3)
            self.assertTrue(any(x.get("forced_kills", 0) >= 2 for x in run.events))

    def test_foreign_port_no_launch_or_adoption(self):
        with Fixture() as f, socket.socket() as foreign:
            foreign.bind(("127.0.0.1", f.ports[0]))
            foreign.listen(1)
            run = f.start()
            self.assertNotEqual(run.finish(False), 0)
            self.assertFalse((f.root / "query.called").exists())
            self.assertTrue(listening(f.ports[0]))
            foreign.close()

    def test_malformed_and_oversized_build_output_reject(self):
        for query in ["bad", "large"]:
            with self.subTest(query=query):
                events = self.rejection(query=query)
                self.assertFalse(any(x["event"] == "worker_started" for x in events))

    def test_unknown_arguments_and_url_health_target_reject(self):
        for extra in [("--private-value", CANARY), ("--health-path", "http://external.example/"),
                      ("--origin", "https://user:password@external.example")]:
            with self.subTest(extra=extra[0]):
                events = self.rejection(extra=extra)
                self.assertNotIn(CANARY, json.dumps(events))

    def test_signal_during_build_query_cleans_owned_query(self):
        with Fixture(query="sleep") as f:
            run = f.start()
            deadline = time.monotonic() + 3
            while not (f.root / "query.called").exists() and time.monotonic() < deadline:
                time.sleep(.02)
            self.assertTrue((f.root / "query.called").exists())
            query_pid = int((f.root / "query.called").read_text())
            self.assertEqual(run.finish(), 0)
            with self.assertRaises(ProcessLookupError):
                os.kill(query_pid, 0)

    def test_pipe_backpressure_does_not_block_logger(self):
        code = 'import importlib.util; s=importlib.util.spec_from_file_location("w",'+repr(str(TOOL))+');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);l=m.JsonLog();[l.emit("test",ready=False) for _ in range(100000)];assert l.dropped>0'
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", code], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(child.wait(timeout=4), 0)
        output, error = child.communicate()
        self.assertFalse(error)
        for line in output.splitlines():
            self.assertEqual(json.loads(line)["schema"], "zee-worker-supervisor-1")


class Native(unittest.TestCase):
    def test_real_two_worker_ready_and_term_cleanup(self):
        with Fixture(native=True,app_signal=True) as f:
            run = f.start()
            run.until(lambda x: x.get("ready") is True)
            self.assertEqual(sum(x["event"] == "worker_started" for x in run.events), 2)
            self.assertEqual(run.finish(), 0)

    def test_real_credential_composition_secret_stays_off_log(self):
        with Fixture(native=True,credential=True) as f:
            run=f.start()
            run.until(lambda x:x.get('ready') is True)
            self.assertEqual(run.finish(),0)
            self.assertNotIn(CANARY,json.dumps(run.events))

    def test_real_credential_leader_crash_kills_native_descendant(self):
        with Fixture(native=True,credential=True) as f:
            run=f.start()
            run.until(lambda x:x.get('ready') is True)
            leader=next(x['pid'] for x in run.events if x['event']=='worker_started')
            os.kill(leader,signal.SIGKILL)
            self.assertNotEqual(run.finish(False),0)
            self.assertFalse(any(listening(p) for p in f.ports))

    def test_real_application_gate_can_remove_readiness(self):
        with Fixture(native=True) as f:
            run = f.start()
            try:
                run.until(lambda x: x.get("ready") is True)
                after=len(run.events)
                gate(f.gate, False)
                event=run.until(lambda x: x.get("state") == "unready",after=after)
                self.assertFalse(event['application_ready'])
                self.assertEqual(event['healthy_workers'],2)
                self.assertTrue(all(listening(p) for p in f.ports))
                self.assertEqual(run.finish(), 0)
            except AssertionError as error:
                error.timeline=run.events[-128:]
                error.assertion_stage='post_gate_unready'
                raise

    def test_real_failed_gate_never_ready_despite_200_health(self):
        with Fixture(native=True, ready=False) as f:
            run = f.start()
            self.assertNotEqual(run.finish(False), 0)
            self.assertFalse(any(x.get("ready") is True for x in run.events))

    def test_real_child_crash_cleans_peer(self):
        with Fixture(native=True) as f:
            run = f.start()
            run.until(lambda x: x.get("ready") is True)
            worker = next(x["pid"] for x in run.events if x["event"] == "worker_started")
            os.kill(worker, signal.SIGKILL)
            self.assertNotEqual(run.finish(False), 0)

    def test_real_hup_same_port_restart(self):
        with Fixture(native=True) as f:
            run = f.start()
            run.until(lambda x: x.get("ready") is True)
            run.child.send_signal(signal.SIGHUP)
            run.until(lambda x: x.get("restart_total", 0) == 2)
            self.assertEqual(sum(x["event"] == "worker_started" for x in run.events), 4)
            self.assertEqual(run.finish(), 0)

    def test_real_inflight_request_can_be_interrupted(self):
        with Fixture(native=True) as f:
            run = f.start()
            run.until(lambda x: x.get("ready") is True)
            with socket.create_connection(("127.0.0.1", f.ports[0]), timeout=1) as s:
                s.sendall(b'GET /yavas HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=192.0.2.7;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n')
                time.sleep(.1)
                self.assertEqual(run.finish(), 0)
                s.settimeout(1)
                self.assertNotIn(b"bitti", s.recv(4096))


class Launcher(unittest.TestCase):
    def campaign(self, curl_ok=True, front_code=None, use_native_node=False):
        with tempfile.TemporaryDirectory(prefix="zee-local-launcher-k246-") as d:
            root = Path(d).resolve()
            binary = root / "bin"
            binary.mkdir()
            worker_port, foreign_port = port(), port()
            with socket.socket() as foreign:
                foreign.bind(("127.0.0.1", foreign_port));foreign.listen(1)
                dil = binary / "dil"
                dil.write_text("#!" + sys.executable + '\nimport os,socket,sys,time\nif sys.argv[1]!="çalıştır":sys.exit(0)\ns=socket.socket();s.bind(("127.0.0.1",int(os.environ["WORKER_PORT"])));s.listen(8)\nwhile True:time.sleep(1)\n')
                (binary / "curl").write_text('#!/bin/sh\nexit '+('0' if curl_ok else '1')+'\n')
                (binary / "node").write_text("#!" + sys.executable + '\nimport os,time\nopen(os.environ["FRONT_READY"],"w").write("ready")\n'+('raise SystemExit('+str(front_code)+')\n' if front_code is not None else 'while True:time.sleep(1)\n'))
                for path in binary.iterdir():path.chmod(0o700)
                (binary/'python3').symlink_to(sys.executable)
                front_port=port()
                if use_native_node:
                    (binary/'node').unlink()
                    (binary/'node').symlink_to(NODE)
                env=os.environ.copy();env.update(PATH=str(binary)+':/usr/bin:/bin',DIL_BIN=str(dil),WORKER_PORT=str(worker_port),FRONT_READY=str(root/'front.ready'))
                env.update(ZEE_ONKAPI_PORT=str(front_port),ZEE_ISCI_PORT=str(worker_port))
                child=subprocess.Popen(['/bin/bash',str(LAUNCHER)],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
                try:
                    if curl_ok and front_code is None:
                        deadline=time.monotonic()+3
                        front_ready=lambda:listening(front_port) if use_native_node else (root/'front.ready').exists()
                        while time.monotonic()<deadline and not (front_ready() and listening(worker_port)):time.sleep(.02)
                        self.assertTrue(front_ready() and listening(worker_port))
                        child.terminate()
                    result=child.wait(timeout=10)
                    self.assertEqual(result,143 if curl_ok and front_code is None else (front_code if curl_ok else 1))
                    self.assertFalse(listening(worker_port))
                    self.assertFalse(listening(front_port))
                    self.assertTrue(listening(foreign_port))
                finally:
                    try:os.killpg(child.pid,signal.SIGKILL)
                    except ProcessLookupError:pass
                    child.wait(timeout=3)

    def test_frontend_term_closes_both_owned_children(self):
        self.campaign()

    def test_startup_failure_cleans_worker_no_foreign_kill(self):
        self.campaign(curl_ok=False)

    def test_frontend_exit_code_preserved_and_worker_cleaned(self):
        self.campaign(front_code=7)

    def test_available_native_node_frontend_cleanup(self):
        self.campaign(use_native_node=bool(NODE))


class Result(unittest.TextTestResult):
    def addSuccess(self,test):
        if not hasattr(self,'success_names'):self.success_names=[]
        self.success_names.append(test.id())
        super().addSuccess(test)

    def addFailure(self,test,error):
        if not hasattr(self,'diagnostics'):self.diagnostics=[]
        self.diagnostics.append({'test':test.id(),'kind':type(error[1]).__name__,'stage':getattr(error[1],'assertion_stage','assertion'),'timeline':getattr(error[1],'timeline',[])})
        super().addFailure(test,error)

    def addError(self,test,error):
        if not hasattr(self,'diagnostics'):self.diagnostics=[]
        self.diagnostics.append({'test':test.id(),'kind':type(error[1]).__name__,'stage':getattr(error[1],'assertion_stage','exception'),'timeline':getattr(error[1],'timeline',[])})
        super().addError(test,error)

    def printErrors(self):
        # Failure names are enough for the gate; no fixture values or environment dump.
        for test, _ in self.errors + self.failures:
            self.stream.writeln("FAILED " + test.id())


def main():
    global NATIVE
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dil',required=True)
    args=parser.parse_args()
    original=Path(args.dil).resolve(strict=True)
    sources=[TOOL,Path(__file__).resolve(),LAUNCHER,ROOT/'scripts/credential-baslat.py']
    source_hashes=[{'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sources]
    if os.name != 'posix':
        raise SystemExit('POSIX native campaign required; no ignored acceptance')
    with tempfile.TemporaryDirectory(prefix='zee-supervisor-native-binary-') as d:
        NATIVE=Path(d).resolve()/'dil'
        before=original.stat()
        shutil.copyfile(original,NATIVE)
        after=original.stat()
        identity=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
        if identity(before)!=identity(after):raise SystemExit('native binary changed during private snapshot')
        NATIVE.chmod(0o700)
        binary_hash=hashlib.sha256(NATIVE.read_bytes()).hexdigest()
        record=json.loads(subprocess.check_output([str(NATIVE),'sürüm','--json'],timeout=5))
        suite=unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
        result=unittest.TextTestRunner(verbosity=1,resultclass=Result).run(suite)
        if record!=json.loads(subprocess.check_output([str(NATIVE),'sürüm','--json'],timeout=5)) or binary_hash!=hashlib.sha256(NATIVE.read_bytes()).hexdigest():raise SystemExit('private native snapshot changed')
    successes=getattr(result,'success_names',[])
    native_names=[name for name in successes if '.Native.' in name]
    composed=[name for name in native_names if 'credential_' in name]
    stable=source_hashes==[{'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sources]
    summary={'schema':'zee-worker-supervisor-tests-1','tests':result.testsRun,'failed':len(result.failures),'errors':len(result.errors),'ignored':len(result.skipped),'actual_native_workers':len(native_names)==8,'native_build':record,'native_binary_sha256':binary_hash,'native_binary_private_snapshot_unchanged':True,'sources_unchanged':stable,'native_node_fixture':bool(NODE) and any('test_available_native_node_frontend_cleanup' in name for name in successes),'credential_composition_native_crypto_and_group_cleanup':len(composed)==2,'source_files':source_hashes,'boundaries':['native application init readiness signal is not DB/migration readiness','inflight request may be interrupted','no P14 socket/zero-downtime acceptance','no credential DB/SMTP end-to-end or live staging acceptance']}
    if result.errors or result.failures:
        proof=Path(tempfile.gettempdir())/('zee-k246-failure-'+str(os.getpid())+'.json')
        proof.write_text(json.dumps({'summary':summary,'diagnostics':getattr(result,'diagnostics',[])},indent=2)+'\n')
    print(json.dumps(summary,separators=(',',':')))
    return 0 if result.wasSuccessful() and result.testsRun == 25 and not result.skipped and stable else 1


if __name__=='__main__':
    sys.exit(main())
