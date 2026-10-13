from pathlib import Path
import json,os,re,subprocess,time,uuid
name='zee-k206-rss-'+uuid.uuid4().hex
image='postgres@sha256:efedf3595f1d6f415c08568ba171029bf54052e754cc9f030e3f2412b21f3d67'
def run(args,**kw):return subprocess.run(args,check=True,capture_output=True,**kw).stdout
result={'source':'9e80b692f9a0a37cdd8136783b42f2a0056f2478','image':image,'samples':[],
        'boundary':'Exploratory direct postgres streaming helper, not Zee production integration. Isolated synthetic loopback container; same driver versions as Cargo.lock. Concurrent K-207 soak. One sample, no warmup; RSS is total process, not a performance baseline or closed security finding.'}
created=False
try:
 run(['docker','run','--detach','--rm','--name',name,'--publish','127.0.0.1::5432','--env','POSTGRES_HOST_AUTH_METHOD=trust',image]);created=True
 for attempt in range(40):
  ready=subprocess.run(['docker','exec',name,'pg_isready','-U','postgres'],capture_output=True)
  if ready.returncode==0:break
  time.sleep(.25)
 else:raise ValueError('Temporary database did not become ready')
 address=run(['docker','port',name,'5432/tcp']).decode().strip()
 port=address.rsplit(':',1)[1]
 result['server_version']=run(['docker','exec',name,'postgres','--version']).decode().strip()
 env=dict(os.environ,ZEE_RSS_TARGET=f'postgresql://127.0.0.1:{port}/postgres',ZEE_RSS_URL=f'host=127.0.0.1 port={port} user=postgres dbname=postgres sslmode=disable options=\'-c statement_timeout=10000\'')
 binary=str(Path('target/release/zee-pg-stream-prototype').resolve())
 process=subprocess.run([binary,'contracts'],env=env,capture_output=True,timeout=60)
 out=process.stdout.decode();err=process.stderr.decode()
 result['contract_output']=out;result['exit']=process.returncode
 if process.returncode:raise ValueError('Contract comparison failed: '+err[:1000])

finally:
 if created:run(['docker','rm','--force',name])
Path('target/k206-contract-prototype.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False))
