"""K294 actual own compiler/pipe regression, no network or PID adoption."""
import argparse,hashlib,importlib.util,json,os,selectors,signal,stat,subprocess,time
from pathlib import Path
from types import SimpleNamespace
SOURCE=b'"K294_OWNED_SOURCE_EXECUTED" yaz\n'
END=time.monotonic()+120
CANCEL=False

def need(value):
 if CANCEL or not value or time.monotonic()>=END:raise RuntimeError('contract')
def snapshot(path,limit):
 before=path.lstat();need(stat.S_ISREG(before.st_mode) and before.st_size<=limit)
 fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
 try:
  opened=os.fstat(fd);need((opened.st_dev,opened.st_ino,opened.st_size)==(before.st_dev,before.st_ino,before.st_size));data=bytearray()
  while True:
   need(True);block=os.read(fd,min(65536,limit-len(data)+1))
   if not block:break
   data.extend(block);need(len(data)<=limit)
  after=os.fstat(fd);need((after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns)==(before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns));return bytes(data)
 finally:os.close(fd)
def digest(path,limit=1024*1024):return hashlib.sha256(snapshot(path,limit)).hexdigest()
def main():
 def cancel(*_):
  global CANCEL
  CANCEL=True
 for number in (signal.SIGTERM,signal.SIGINT):signal.signal(number,cancel)
 parser=argparse.ArgumentParser(allow_abbrev=False)
 for n in ('framework-root','dil','output'):parser.add_argument('--'+n,type=Path,required=True)
 parser.add_argument('--fixture-source',type=Path);parser.add_argument('--expect',choices=('quiet','legacy-leak'),default='quiet');a=parser.parse_args()
 root=a.framework_root.resolve(strict=True);binary=a.dil.resolve(strict=True);out=a.output
 need(os.name=='posix' and os.geteuid()!=0 and hasattr(os,'WNOWAIT'));need(out.is_absolute() and out.parent.resolve()==out.parent and out.parent.stat().st_uid==os.geteuid() and not out.exists());out.mkdir(mode=0o700)
 supplied=snapshot(a.fixture_source,65536) if a.fixture_source else SOURCE;need(supplied==SOURCE)
 source=out/'minimal.dil';source.write_bytes(supplied)
 support=root/'scripts/worker-supervisor.py';spec=importlib.util.spec_from_file_location('metric_stdio_owner',support);M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M);owner=M.Supervisor(SimpleNamespace(project=out,stop_grace=2),None)
 files=[support,Path(__file__).resolve(),root/'compiler/src/main.rs',root/'compiler/src/web_listener.rs',root/'compiler/src/web_metrikleri.rs'];pins={str(p.relative_to(root)):digest(p) for p in files};pin=digest(binary,256*1024*1024)
 proof={'schema':'zee-native-metric-stdio-1','status':'running','expected':a.expect,'fixture_source_sha256':hashlib.sha256(supplied).hexdigest(),'binary_sha256':pin,'sources':pins,'cases':[]}
 def run(argv,aliased=None):
  rd,wr=os.pipe();os.set_blocking(rd,False);os.set_blocking(wr,False)
  child=None;selector=selectors.DefaultSelector();buffers={'stdout':bytearray(),'stderr':bytearray()};deadline=min(END,time.monotonic()+5)
  try:
   args=[str(binary),*argv(wr)]
   child=subprocess.Popen(args,cwd=out,env=M.safe_environment(),stdin=subprocess.DEVNULL,stdout=wr if aliased=='stdout' else subprocess.PIPE,stderr=wr if aliased=='stderr' else subprocess.PIPE,pass_fds=(wr,),close_fds=True,start_new_session=True);owner.children.append(child);os.close(wr);wr=-1
   for name in buffers:
    stream=getattr(child,name)
    if stream is not None:selector.register(stream,selectors.EVENT_READ,name)
   while selector.get_map():
    need(time.monotonic()<deadline)
    for key,_ in selector.select(.05):
     block=os.read(key.fd,4096)
     if not block:selector.unregister(key.fileobj)
     else:buffers[key.data].extend(block);need(len(buffers[key.data])<=4096)
   status=owner.exit_status(child)
   while status is None:need(time.monotonic()<deadline);time.sleep(.005);status=owner.exit_status(child)
   owner.stop_child(child);control=os.read(rd,4096)
   return status,bytes(buffers['stdout']),bytes(buffers['stderr']),control
  finally:
   selector.close()
   if child is not None and child in owner.children:owner.stop_child(child)
   if child is not None:
    for name in ('stdout','stderr'):
     stream=getattr(child,name)
     if stream is not None:stream.close()
   if wr>=0:os.close(wr)
   os.close(rd)
 try:
  status,stdout,stderr,_=run(lambda fd:['sürüm','--json']);need(status==0 and stderr==b'');proof['build']=json.loads(stdout)
  # Valid designated metadata writer and ordinary source prove this is not blanket rejection.
  status,stdout,stderr,control=run(lambda fd:['çalıştır','--web-metrik-fd',str(fd),'--web-proxy','https://fixture.example','--web-worker-port','18943',str(source)])
  need(status==0 and stdout==b'K294_OWNED_SOURCE_EXECUTED\n');proof['cases'].append({'name':'valid_metric_role_executes_actual_source','passed':True,'exit':status})
  variants=[('--web-metrik-fd','stderr','spaced')]
  if a.expect=='quiet':variants=[(role,stdio,form) for role in ('--web-listener-fd','--web-ready-fd','--web-activate-fd','--web-metrik-fd') for stdio in ('stdout','stderr') for form in ('spaced','equals','invalid-lateral')]
  for role,stdio,form in variants:
   def args(fd):
    field=[role,str(fd)] if form!='equals' else [role+'='+str(fd)]
    if form=='invalid-lateral':field=['--web-metrik-fd','999999999999999999999999999999',*field]
    return ['çalıştır',*field,str(source)]
   status,stdout,stderr,control=run(args,stdio);need(status==2 and stdout==b'' and stderr==b'')
   need(control==b'' if a.expect=='quiet' else len(control)>0)
   proof['cases'].append({'name':role+'-'+stdio+'-'+form,'passed':True,'exit':status,'control_bytes':len(control),'control_sha256':hashlib.sha256(control).hexdigest()})
  # A plain CLI failure remains diagnostic, rather than silently suppressing all stderr.
  status,stdout,stderr,_=run(lambda fd:['--K294_UNRECOGNIZED']);need(status==2 and stdout==b'' and len(stderr)>0);proof['cases'].append({'name':'ordinary_cli_error_remains_diagnostic','passed':True})
  need(digest(binary,256*1024*1024)==pin and all(digest(root/n)==h for n,h in pins.items()));proof.update(status='passed',source_pins_unchanged=True)
 except BaseException as e:proof.update(status='failed-preserved',failure_type=type(e).__name__)
 finally:
  owner.cleanup();proof['owned_handles_reaped']=not owner.children;(out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
 return 0 if proof['status']=='passed' and proof['owned_handles_reaped'] else 2
if __name__=='__main__':raise SystemExit(main())
