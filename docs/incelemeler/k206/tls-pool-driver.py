from pathlib import Path
import subprocess,os,time,uuid,tempfile,json,hashlib
name='zee-k206-tls-'+uuid.uuid4().hex
image='postgres@sha256:efedf3595f1d6f415c08568ba171029bf54052e754cc9f030e3f2412b21f3d67'
def run(args,**kwargs):return subprocess.run(args,check=True,capture_output=True,**kwargs).stdout
created=False
start=time.monotonic()
with tempfile.TemporaryDirectory(prefix='zee-k206-tls-') as temp:
 try:
  root=Path(temp);crt=root/'server.crt';key=root/'server.key';ca=root/'ca.crt';cakey=root/'ca.key';csr=root/'server.csr';ext=root/'server.ext'
  run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-subj','/CN=zee-isolated-ca','-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,keyCertSign,cRLSign','-keyout',str(cakey),'-out',str(ca)])
  run(['openssl','req','-new','-newkey','rsa:2048','-nodes','-subj','/CN=localhost','-keyout',str(key),'-out',str(csr)])
  ext.write_text('basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n')
  run(['openssl','x509','-req','-in',str(csr),'-CA',str(ca),'-CAkey',str(cakey),'-CAcreateserial','-days','1','-extfile',str(ext),'-out',str(crt)])
  key.chmod(0o600)
  run(['docker','run','--detach','--rm','--name',name,'--publish','127.0.0.1::5432','--env','POSTGRES_HOST_AUTH_METHOD=trust','--entrypoint','sh',image,'-c','while [ ! -f /tmp/zee-server.key ]; do sleep 0.2; done; exec docker-entrypoint.sh postgres -c ssl=on -c ssl_cert_file=/tmp/zee-server.crt -c ssl_key_file=/tmp/zee-server.key']);created=True
  run(['docker','cp',str(crt),name+':/tmp/zee-server.crt']);run(['docker','cp',str(key),name+':/tmp/zee-server.key'])
  run(['docker','exec',name,'chown','postgres:postgres','/tmp/zee-server.key','/tmp/zee-server.crt'])
  for _ in range(80):
   ready=subprocess.run(['docker','exec',name,'pg_isready','-U','postgres'],capture_output=True)
   if ready.returncode==0:break
   time.sleep(.25)
  else:raise ValueError('Isolated TLS server not ready')
  address=run(['docker','port',name,'5432/tcp']).decode().strip();port=address.rsplit(':',1)[1]
  env=dict(os.environ,ZEE_POSTGRES_SAHA_HEDEFI=f'postgresql://localhost:{port}/postgres',ZEE_POSTGRES_SAHA_URL=f'host=localhost port={port} user=postgres dbname=postgres sslmode=require',ZEE_POSTGRES_SAHA_URL_TLS_CA_PEM=ca.read_text(),ZEE_POSTGRES_SAHA_UZUN_SOAK='1')
  command=['cargo','test','--locked','--lib','postgresql::havuz::testler::saha_profili_gercek_tls_exhaustion_ve_stale_recoveryyi_olcer','--','--exact','--nocapture']
  proc=subprocess.run(command,env=env,capture_output=True,timeout=500)
  transcript=(proc.stdout+proc.stderr).decode();Path('target/k206-tls-long-test.log').write_text(transcript)
  result={'source':run(['git','rev-parse','HEAD']).decode().strip(),'source_clean':not run(['git','status','--porcelain']).strip(),'image':image,'server_version':run(['docker','exec',name,'postgres','--version']).decode().strip(),'test':command[4],'exit':proc.returncode,'elapsed_seconds':round(time.monotonic()-start,3),'ca_fingerprint_sha256':hashlib.sha256(ca.read_bytes()).hexdigest(),'boundary':'Own synthetic loopback TLS container, ephemeral CA/key deleted, long native pool test: checkout exhaustion, stale replacement, active lease 300s plus 30s maintenance. Concurrent process-specific K-207 RSS soak; not user production cluster.'}
  Path('target/k206-tls-long.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result,ensure_ascii=False))
  if proc.returncode:raise ValueError('Isolated TLS test failed; inspect ignored transcript')
 finally:
  if created:run(['docker','rm','--force',name])
