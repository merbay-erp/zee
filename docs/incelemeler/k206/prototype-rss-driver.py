from pathlib import Path
import json,os,re,subprocess,time,uuid,hashlib
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
 for rows,size in [(1,1048576),(64,1048576),(1,67108864)]:
  process=subprocess.run(['/usr/bin/time','-l',binary,str(rows),str(size)],env=env,capture_output=True,timeout=30)
  out=process.stdout.decode();err=process.stderr.decode()
  if process.returncode:raise ValueError('Audit helper failed: '+err[:300])
  rss=re.search(r'(\d+)\s+maximum resident set size',err)
  if not rss:raise ValueError('RSS measurement missing')
  result['samples'].append({'rows':rows,'row_bytes':size,'max_rss_bytes':int(rss[1]),'output':out,'exit':process.returncode})
finally:
 if created:run(['docker','rm','--force',name])
result.update(source_clean=not run(['git','status','--porcelain']).strip(), build_profile='release', rustc=run(['rustc','--version']).decode().strip(), prototype_sources_sha256={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path('target/k206-stream-prototype/src/main.rs'),Path('target/k206-stream-prototype/src/contracts.rs'),Path('target/k206-stream-prototype/Cargo.toml'),Path('target/k206-stream-prototype/Cargo.lock')]})
Path('target/k206-memory-prototype.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False))
