import argparse,array,fcntl,hashlib,importlib.util,json,os,selectors,socket,subprocess,tempfile,termios,time
from pathlib import Path
from types import SimpleNamespace
ARGS=argparse.ArgumentParser();ARGS.add_argument('--framework-root',type=Path,required=True);ARGS.add_argument('--dil',type=Path,required=True);ARGS.add_argument('--output',type=Path,required=True);ARGS.add_argument('--fixture-source',type=Path);ARGS.add_argument('--expect',choices=('old-late-admission','rejected'),required=True);A=ARGS.parse_args();assert not A.output.exists();BINARY=A.dil.resolve(strict=True);TOOL=A.framework_root.resolve(strict=True)/'scripts/worker-supervisor.py';spec=importlib.util.spec_from_file_location('own',TOOL);S=importlib.util.module_from_spec(spec);spec.loader.exec_module(S);output=A.output.resolve();output.parent.mkdir(parents=True,exist_ok=True)
fixture_source=A.fixture_source.resolve(strict=True) if A.fixture_source is not None else None
fixture_bytes=fixture_source.read_bytes() if fixture_source is not None else '8080 kapısında sunucu başlat\nGET \"/hazir\" adresine istek geldiğinde\n    \"ok\" yanıtını gönder\n'.encode()
assert len(fixture_bytes)<=4096 and fixture_bytes.endswith(b'\n') and b'\r' not in fixture_bytes and b'\t' not in fixture_bytes
with tempfile.TemporaryDirectory(prefix='zee-activate-deadline-',dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp') as t:
 project=Path(t).resolve();(project/'proje.dil').write_text('proje "deadline" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu" listesi olsun\n');(project/'ana.dil').write_bytes(fixture_bytes);listener=socket.socket();listener.bind(('127.0.0.1',0));listener.listen();listener.setblocking(False);port=listener.getsockname()[1];rr,rw=os.pipe();ar,aw=os.pipe();owner=S.Supervisor(SimpleNamespace(project=project,stop_grace=.5),S.JsonLog());child=subprocess.Popen([str(BINARY),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(port),'--web-listener-fd',str(listener.fileno()),'--web-ready-fd',str(rw),'--web-activate-fd',str(ar),'--web-generation','a'*32,str(project)],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,pass_fds=(listener.fileno(),rw,ar),start_new_session=True);owner.children.append(child);os.close(rw);ready=b'';original_inode=os.fstat(listener.fileno()).st_ino;proof={'schema':'zee-native-listener-deadline-1','scope':'owned single generation activation EOF boundary; no longcampaign','binary_sha256':hashlib.sha256(BINARY.read_bytes()).hexdigest()}
 try:
  with selectors.DefaultSelector() as sel:
   sel.register(rr,selectors.EVENT_READ);end=time.monotonic()+3
   while time.monotonic()<end:
    if not sel.select(.1):continue
    chunk=os.read(rr,4096);ready+=chunk
    if not chunk:break
  assert json.loads(ready)['pid']==child.pid;start=time.monotonic();os.write(aw,b'A')
  # Observe the child actually consumed a prefix: Control.wait already started.
  end=time.monotonic()+1
  while True:
   count=array.array('i',[0]);fcntl.ioctl(ar,termios.FIONREAD,count,True)
   if count[0]==0:break
   assert time.monotonic()<end;time.sleep(.001)
  consumed=time.monotonic()-start;assert consumed<.05
  time.sleep(max(0,9.97-(time.monotonic()-start)));os.write(aw,('CTIVATE '+'a'*32+'\n').encode());remaining_frame_sent=time.monotonic()-start
  time.sleep(max(0,10.035-(time.monotonic()-start)));os.close(aw);aw=None;eof=time.monotonic()-start
  assert eof>consumed+10 # Later than maximum possible 10s deadline.
  reply=b''
  with socket.create_connection(('127.0.0.1',port),timeout=1) as c:
   c.settimeout(.7);c.sendall(b'GET /hazir HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n')
   try:
    while True:
     chunk=c.recv(4096)
     if not chunk:break
     reply+=chunk;assert len(reply)<8192
   except (socket.timeout,ConnectionResetError):pass
  proof.update(single_generation_only=True,owner_source_sha256=hashlib.sha256(TOOL.read_bytes()).hexdigest(),runner_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),fixture_hashes={n:hashlib.sha256((project/n).read_bytes()).hexdigest() for n in ['ana.dil','proje.dil']},build=json.loads(subprocess.check_output([str(BINARY),'sürüm','--json'])),prefix_consumed_elapsed=consumed,deadline_latest_upper_bound=consumed+10,remaining_frame_sent_elapsed=remaining_frame_sent,EOF_elapsed=eof,late_activation_HTTP200=reply.startswith(b'HTTP/1.1 200 ') and reply.endswith(b'ok'),WNOWAIT_status=owner.exit_status(child),response_bytes=len(reply),parent_listener_retained_during_probe=os.fstat(listener.fileno()).st_ino==original_inode)
 finally:
  if aw is not None:os.close(aw)
  owner.cleanup();child.stderr.close();listener.close();os.close(rr);os.close(ar)
 proof['owned_handles_reaped']=not owner.children;proof['closure_contract']='control failure maps to None and normalCLI0; no admission, terminal leader';proof['expected']=A.expect;proof['passed']=(proof['late_activation_HTTP200'] and proof['WNOWAIT_status'] is None) if A.expect=='old-late-admission' else (proof['response_bytes']==0 and proof['WNOWAIT_status']==0);assert hashlib.sha256(BINARY.read_bytes()).hexdigest()==proof['binary_sha256'];proof['fixture_source_sha256']=hashlib.sha256(fixture_bytes).hexdigest();assert fixture_source is None or fixture_source.read_bytes()==fixture_bytes;proof['sources']={str(p.relative_to(A.framework_root.resolve(strict=True))):hashlib.sha256(p.read_bytes()).hexdigest() for p in [TOOL,Path(__file__).resolve(),A.framework_root.resolve(strict=True)/'compiler/src/web_listener.rs',A.framework_root.resolve(strict=True)/'compiler/src/main.rs']};output.write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof));assert proof['passed'],'timingguard/expected leader/admission outcome failed; no acceptance inferred'
