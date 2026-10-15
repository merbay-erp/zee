import argparse,hashlib,json,os,socket,subprocess,tempfile
from pathlib import Path
parser=argparse.ArgumentParser(description='Actual quiet native listener FD boundary negatives')
parser.add_argument('--framework-root',type=Path,required=True);parser.add_argument('--dil',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
ARGS=parser.parse_args();ROOT=ARGS.framework_root.resolve(strict=True);OUTPUT=ARGS.output.resolve()
if OUTPUT.exists():parser.error('--output must be fresh')
OUTPUT.mkdir(parents=True,mode=0o700)
binary=ARGS.dil.resolve(strict=True);canary=b'PRIVATE_K259_REVIEW_CANARY';cases=[]
build=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5))
with tempfile.TemporaryDirectory(prefix='zee-native-alias-') as tmp:
 source=Path(tmp)/'source.dil';source.write_bytes(canary+b' !!! malformed\n')
 for mode in ['stderr_alias','stdout_alias','opposite_control_pipe_ends','metric_ready_alias','udp','not_listening','blocking_listener','wildcard_listener','wrong_port','closed_listener','stdio_listener','wrong_ready_access','wrong_activate_access','ayar_before_stderr_alias','profil_before_stdout_alias','ayar_profil_before_role_alias']:
  listener=socket.socket(socket.AF_INET,socket.SOCK_STREAM,socket.IPPROTO_TCP);listener.bind(('127.0.0.1',0));listener.listen();listener.setblocking(False)
  if mode=='not_listening':listener.close();listener=socket.socket();listener.bind(('127.0.0.1',0));listener.setblocking(False)
  if mode=='udp':listener.close();listener=socket.socket(socket.AF_INET,socket.SOCK_DGRAM);listener.bind(('127.0.0.1',0));listener.setblocking(False)
  if mode=='wildcard_listener':listener.close();listener=socket.socket();listener.bind(('0.0.0.0',0));listener.listen();listener.setblocking(False)
  if mode=='blocking_listener':listener.setblocking(True)
  rr,rw=os.pipe();ar,aw=os.pipe();activate=rr if mode in ['opposite_control_pipe_ends','ayar_profil_before_role_alias'] else (aw if mode=='wrong_activate_access' else ar)
  ready=rr if mode=='wrong_ready_access' else rw
  received=9999 if mode=='closed_listener' else (0 if mode=='stdio_listener' else listener.fileno())
  port=(listener.getsockname()[1]-1 if listener.getsockname()[1]==65535 else listener.getsockname()[1]+1) if mode=='wrong_port' else listener.getsockname()[1]
  extra=['--web-metrik-fd',str(rw)] if mode=='metric_ready_alias' else []
  argv=[str(binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(port),'--web-listener-fd',str(received),'--web-ready-fd',str(ready),'--web-activate-fd',str(activate),'--web-generation','a'*32,*extra,str(source)]
  config_prefix=[]
  if mode=='ayar_before_stderr_alias':config_prefix=['--ayar',str(source)]
  if mode=='profil_before_stdout_alias':config_prefix=['--profil','PRIVATE_K259_CONFIG_CANARY']
  if mode=='ayar_profil_before_role_alias':config_prefix=['--ayar',str(source),'--profil','PRIVATE_K259_CONFIG_CANARY']
  argv[2:2]=config_prefix
  result=subprocess.run(argv,stdin=subprocess.DEVNULL,stdout=rw if mode in ['stdout_alias','profil_before_stdout_alias'] else subprocess.DEVNULL,stderr=rw if mode in ['stderr_alias','ayar_before_stderr_alias'] else subprocess.PIPE,pass_fds=tuple(set([listener.fileno(),rw,ready,activate])),timeout=5)
  os.close(rw);data=os.read(rr,8192)
  assert result.returncode==2 and data==b'' and not result.stderr and canary not in data,(mode,result.returncode,len(data),result.stderr)
  assert listener.fileno()>=0 and listener.getsockname()[1]>0
  cases.append({'name':mode,'exit':2,'control_bytes':0,'stderr_bytes':0,'canary_in_control':False,'parent_listener_preserved':True})
  for fd in [rr,ar,aw]:os.close(fd)
  listener.close()
(OUTPUT/'proof.json').write_text(json.dumps({'status':'passed','build':build,'runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':cases},indent=2)+'\n')
print('actual quiet borrowedFD negative16 passed')
