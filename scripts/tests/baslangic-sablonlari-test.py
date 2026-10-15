import argparse,hashlib,importlib.util,json,os,re,secrets,shutil,signal,socket,subprocess,tempfile,time,types,urllib.parse
from pathlib import Path
p=argparse.ArgumentParser(allow_abbrev=False);p.add_argument('--framework-root',type=Path,required=True);p.add_argument('--template-root',type=Path);p.add_argument('--dil',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
r=a.framework_root.resolve(strict=True);templates=(a.template_root or r/'sablonlar').resolve(strict=True);binary=a.dil.resolve(strict=True);output=a.output.resolve();assert not output.exists();output.mkdir(mode=0o700,parents=True)
spec=importlib.util.spec_from_file_location('owner',r/'scripts/worker-supervisor.py');owner=importlib.util.module_from_spec(spec);spec.loader.exec_module(owner)
CANCELLED=False
def cancelled(number,frame):
 global CANCELLED
 CANCELLED=True
for number in (signal.SIGTERM,signal.SIGINT):signal.signal(number,cancelled)
def require(v,label):
 if CANCELLED:raise RuntimeError('test interrupted')
 if not v:raise AssertionError(label)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
proof={'schema':'zee-starter-acceptance-1','status':'running','cases':[],'cleanup_cohorts':[],'binary_sha256':sha(binary),'build':json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5)),'sources':[{'path':'scripts/worker-supervisor.py','sha256':sha(r/'scripts/worker-supervisor.py')},{'path':'scripts/tests/baslangic-sablonlari-test.py','sha256':sha(Path(__file__))}],'templates':[]}
if (r/'scripts/native-capabilities.py').exists():proof['sources'].append({'path':'scripts/native-capabilities.py','sha256':sha(r/'scripts/native-capabilities.py')})
proof['support_files']=[{'path':str(f.relative_to(templates)),'sha256':sha(f)} for f in [templates/'README.md',templates/'site/.gitignore',templates/'panel/.gitignore']]
def http(port,path,method='GET',fields=None,cookie=None):
 body=urllib.parse.urlencode(fields or {}).encode();headers='Host: starter.invalid\r\nForwarded: for=127.0.0.1;proto=https;host=starter.invalid\r\nConnection: close\r\n'
 if cookie:headers+='Cookie: '+cookie+'\r\n'
 if method=='POST':headers+='Origin: https://starter.invalid\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: '+str(len(body))+'\r\n'
 with socket.create_connection(('127.0.0.1',port),timeout=1) as s:
  s.settimeout(3);s.sendall((method+' '+path+' HTTP/1.1\r\n'+headers+'\r\n').encode()+body);data=bytearray();deadline=time.monotonic()+5
  while True:
   remaining=deadline-time.monotonic();require(remaining>0,'response deadline');s.settimeout(min(3,remaining));part=s.recv(4096);require(time.monotonic()<=deadline,'final response deadline')
   if not part:break
   data.extend(part);require(len(data)<=65536,'response budget')
 head,body=bytes(data).split(b'\r\n\r\n',1);status=int(head.split(b' ')[1]);newcookie=None
 for line in head.split(b'\r\n'):
  if line.lower().startswith(b'set-cookie:'):newcookie=line.split(b':',1)[1].strip().split(b';',1)[0].decode()
 require(time.monotonic()<=deadline,'response parse deadline');return status,body.decode(),newcookie

def check_project(kind):
 with tempfile.TemporaryDirectory(prefix='zee-starter-') as temp:
  project=Path(temp).resolve()/'project';shutil.copytree(templates/kind,project)
  for name in ['ana.dil','proje.dil']:proof['templates'].append({'path':kind+'/'+name,'sha256':sha(project/name)})
  for command in ['kilitle','denetle','dene']:
   x=subprocess.run([str(binary),command,str(project)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=15);require(x.returncode==0,kind+' '+command)
  password=secrets.token_hex(16)
  if kind=='panel':
   h=subprocess.run([str(binary),'parola-özeti','--stdin'],input=password.encode(),capture_output=True,timeout=10);require(h.returncode==0 and len(h.stdout)<1024 and h.stdout.startswith(b'$argon2id$'),'native password hash')
   with (project/'panel-phc').open('xb') as f:f.write(h.stdout.rstrip(b'\n'))
   (project/'panel-phc').chmod(0o600)
  with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
  supervisor=owner.Supervisor(types.SimpleNamespace(project=project,stop_grace=2),None)
  try:
   child=supervisor.spawn([str(binary),'çalıştır','--web-proxy','https://starter.invalid','--web-worker-port',str(port),str(project)])
   deadline=time.monotonic()+10
   while True:
    require(supervisor.exit_status(child) is None,'live starter worker')
    try:status,body,_=http(port,'/')
    except (OSError,ValueError):
     require(time.monotonic()<deadline,'starter readiness');time.sleep(.02);continue
    require(status==200,'starter home');break
   if kind=='site':
    require(http(port,'/hakkinda')[0]==200 and '12' in http(port,'/obeb')[1] and http(port,'/missing')[0]==404,'site routes');proof['cases'].append({'name':'site_project_cli_and_native_routes','passed':True})
   else:
    require(http(port,'/yonet')[0]==401,'anonymous role denied')
    status,body,guest=http(port,'/giris');match=re.search(r'name=_csrf value="([0-9a-f]{64})"',body);require(status==200 and guest and match,'guest csrf');csrf=match.group(1)
    private=project/'panel-phc';valid=private.read_bytes();private.unlink()
    require(http(port,'/giris-yap','POST',{'parola':password,'_csrf':csrf},guest)[0]==303 and http(port,'/yonet',cookie=guest)[0]==401,'missing private hash cannot authenticate')
    private.write_bytes(b'invalid-PHC');private.chmod(0o600)
    require(http(port,'/giris-yap','POST',{'parola':password,'_csrf':csrf},guest)[0]==303 and http(port,'/yonet',cookie=guest)[0]==401,'invalid private hash cannot authenticate')
    private.write_bytes(valid);private.chmod(0o600)
    require(http(port,'/giris-yap','POST',{'parola':password},guest)[0]==403,'csrf guard')
    require(http(port,'/giris-yap','POST',{'parola':'incorrect','_csrf':csrf},guest)[0]==303,'wrong password no login')
    require(http(port,'/yonet',cookie=guest)[0]==401,'wrong password no role')
    status,_,authenticated=http(port,'/giris-yap','POST',{'parola':password,'_csrf':csrf},guest);require(status==303 and authenticated and authenticated!=guest,'password session rotation')
    status,body,_=http(port,'/yonet',cookie=authenticated);match=re.search(r'name=_csrf value="([0-9a-f]{64})"',body);require(status==200 and match,'authenticated panel');csrf=match.group(1)
    payload='<script>owned</script>';require(http(port,'/kaydet','POST',{'not':payload,'_csrf':csrf},authenticated)[0]==303,'note transaction');status,body,_=http(port,'/');require(status==200 and '&lt;script&gt;owned&lt;/script&gt;' in body and payload not in body,'stored content html escaped')
    require(http(port,'/cikis','POST',{'_csrf':csrf},authenticated)[0]==303 and http(port,'/yonet',cookie=authenticated)[0]==401,'logout revocation');proof['cases'].append({'name':'panel_project_native_password_csrf_rotation_xss_logout','passed':True})
  finally:
   supervisor.cleanup();proof['cleanup_cohorts'].append(kind)
 require(not supervisor.children,'owned cleanup')
try:
 for kind in ['site','panel']:check_project(kind)
 require(sha(binary)==proof['binary_sha256'],'binary unchanged')
 for record in proof['sources']:
  path=Path(__file__) if record['path']=='scripts/tests/baslangic-sablonlari-test.py' else r/record['path'];require(sha(path)==record['sha256'],'runtime source unchanged')
 for record in proof['templates']:require(sha(templates/record['path'])==record['sha256'],'template source unchanged')
 for record in proof['support_files']:require(sha(templates/record['path'])==record['sha256'],'support file unchanged')
 proof['sources_unchanged']=True;proof['status']='passed';proof['cleanup']='owned groups stopped and reaped';print('2 starter cohorts passed; owned cleanup complete')
except BaseException as error:
 proof['status']='failed';proof['failure_type']=type(error).__name__;raise
finally:
 (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
