from pathlib import Path
import json,os,re,subprocess,time,uuid,hashlib,platform
name='zee-k206-typed-frame-'+uuid.uuid4().hex
image='postgres@sha256:efedf3595f1d6f415c08568ba171029bf54052e754cc9f030e3f2412b21f3d67'
def run(args,**kw):return subprocess.run(args,check=True,capture_output=True,**kw).stdout
result={'source':run(['git','rev-parse','HEAD']).decode().strip(),'image':image,'samples':[],
        'boundary':'Native API internal audit helper; isolated synthetic data, loopback temporary container; RSS experiment is not a permanent performance baseline.'}
here=Path(__file__).resolve().parent
result['production_patch_active']=False
result['samples_per_case']=1
result['warmup_per_case']=0
result['environment']={'os':platform.platform(),'machine':platform.machine(),
 'rustc':run(['rustup','run','1.93.1','rustc','--version','--verbose']).decode(),
 'profile':'release, --locked --offline, temporary path-patched standalone probe'}
result['input_sha256']={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in
 [here/'backend.patch',here/'probe.rs',here/'probe-Cargo.lock',here/'probe-Cargo.toml']}
if run(['git','status','--porcelain']).strip():raise ValueError('Exact native evidence requires a clean source tree')
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
 binary=str(Path('target/release/zee-pg-typed-frame-probe').resolve())
 run(['docker','exec',name,'psql','-U','postgres','-v','ON_ERROR_STOP=1','-c',"CREATE TABLE zee_frame_probe(value text)"])
 result['binary_sha256']=hashlib.sha256(Path(binary).read_bytes()).hexdigest()
 for size,mode in [(1048576,"plain"),(16777216,"plain"),(33554432,"plain"),(67108864,"plain"),(67108864,"txn")]:
  process=subprocess.run(['/usr/bin/time','-l',binary,str(size),mode],env=env,capture_output=True,timeout=30)
  out=process.stdout.decode();err=process.stderr.decode()
  if process.returncode:raise ValueError('Audit helper failed: '+err[:300])
  rss=re.search(r'(\d+)\s+maximum resident set size',err)
  if not rss:raise ValueError('RSS measurement missing')
  result['samples'].append({'mode':mode,'row_bytes':size,'max_rss_bytes':int(rss[1]),'output':out,'exit':process.returncode})
 count=run(['docker','exec',name,'psql','-U','postgres','-At','-c','SELECT count(*) FROM zee_frame_probe']).decode().strip()
 if count!='0':raise ValueError('Disconnected transaction persisted its write')
 result['disconnected_transaction_persisted_rows']=int(count)
finally:
 if created:run(['docker','rm','--force',name])
result.update(source_clean=not run(['git','status','--porcelain']).strip(), boundary='Experimental local protocol fork, not shipped. Inactive typed guard in both backend parsers; asserted driver cause and disconnected transaction rollback. Synthetic PostgreSQL and one sample/no warmup process RSS.')
Path('target/k206-typed-frame-prototype.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False))
