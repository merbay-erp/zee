#!/usr/bin/env python3
"""K276 private inert carrier. Canonical delivery first; never activates binaries."""
import argparse,hashlib,json,os,platform,re,signal,stat,sys,types
from pathlib import Path
class Reject(ValueError):pass
def need(ok):
 if not ok:raise Reject('private-carrier-rejected')
HERE=Path(__file__).resolve().parent
P_SHA='3b1e487235987b53a8ef63811b72a98878816295be1d7cfb0d972794cd671b4d'
H_SHA='e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9'
def helper(name,pin):
 path=HERE/name;fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
 try:
  a=os.fstat(fd);need(stat.S_ISREG(a.st_mode) and a.st_size<=1048576 and not a.st_mode&0o022);data=bytearray()
  while True:
   b=os.read(fd,65536)
   if not b:break
   data.extend(b);need(len(data)<=1048576)
  z=os.fstat(fd);q=path.lstat();key=lambda v:(v.st_dev,v.st_ino,v.st_size,v.st_mtime_ns,v.st_ctime_ns);need(key(a)==key(z)==key(q) and hashlib.sha256(data).hexdigest()==pin)
 finally:os.close(fd)
 m=types.ModuleType(name);m.__file__=str(path);exec(compile(bytes(data),str(path),'exec'),m.__dict__);return m

def package_name(sha):need(type(sha) is str and re.fullmatch('[0-9a-f]{40}',sha));return 'zee-private-carrier-'+sha

def control(sha,arch):
 name=package_name(sha);need(arch in ('amd64','arm64'))
 return ('Package: '+name+'\nVersion: 0.0.0+git.'+sha+'\nArchitecture: '+arch+'\nSection: misc\nPriority: optional\nMaintainer: Private evaluation placeholder <noreply@invalid.invalid>\nDescription: Inert private Zee artifact carrier; no publisher or license claim\n No activation, PATH, service, upgrade scripts or public distribution.\n').encode()

def fresh_output(path,repo,delivery):
 path=Path(path);need(path.is_absolute() and not path.exists() and not path.is_symlink());parent=path.parent.resolve(strict=True);need(parent==path.parent and parent.stat().st_uid==os.geteuid() and not parent.stat().st_mode&0o022)
 for root in (repo,delivery):need(path!=root and root not in path.parents and path not in root.parents)
 return path

def inventory(root,cap=64):
 root=Path(root);result={};seen=0;total=0;pending=[root]
 while pending:
  parent=pending.pop()
  with os.scandir(parent) as it:
   for entry in it:
    seen+=1;need(seen<=cap);q=Path(entry.path);s=entry.stat(follow_symlinks=False);need(not entry.is_symlink())
    if stat.S_ISDIR(s.st_mode):pending.append(q)
    else:
     need(stat.S_ISREG(s.st_mode) and s.st_nlink==1 and s.st_size<=134217728);fd=os.open(q,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
     try:
      a=os.fstat(fd);need(stat.S_ISREG(a.st_mode) and a.st_nlink==1 and a.st_size<=134217728 and (a.st_dev,a.st_ino)==(s.st_dev,s.st_ino));digest=hashlib.sha256();count=0
      while True:
       block=os.read(fd,65536)
       if not block:break
       count+=len(block);total+=len(block);need(count<=134217728 and total<=268435456);digest.update(block)
      z=os.fstat(fd);last=q.lstat();key=lambda x:(x.st_dev,x.st_ino,x.st_size,x.st_mtime_ns,x.st_ctime_ns);need(key(a)==key(z)==key(last));result[str(q.relative_to(root))]=(digest.hexdigest(),stat.S_IMODE(s.st_mode))
     finally:os.close(fd)
 return result

def emit(root,name,data,mode=0o644):
 q=root/name;q.parent.mkdir(parents=True,exist_ok=True);fd=os.open(q,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,mode)
 with os.fdopen(fd,'wb') as f:f.write(data)
 q.chmod(mode)

def build(a):
 need(platform.system()=='Linux' and os.geteuid()!=0 and platform.machine()=={'arm64':'aarch64','amd64':'x86_64'}[a.arch]);P=helper('linux-package-preflight.py',P_SHA);H=helper('lifecycle_core.py',H_SHA)
 # Cancellation unwinds reviewed run() cleanup rather than orphaning its group.
 def cancel(signum,frame):
  signal.signal(signal.SIGTERM,signal.SIG_IGN);signal.signal(signal.SIGINT,signal.SIG_IGN);raise SystemExit(128+signum)
 signal.signal(signal.SIGTERM,cancel);signal.signal(signal.SIGINT,cancel)
 repo=a.repo.resolve(strict=True);delivery=a.delivery.resolve(strict=True);need(repo==a.repo and delivery==a.delivery);package_name(a.sha);H.clean_exact(repo,a.sha)
 validated=P.delivery(delivery,a.sha,a.delivery_sha256,a.arch);out=fresh_output(a.out,repo,delivery)
 # Snapshot every payload BEFORE output mutation; source scripts must equal Git blobs.
 report_pin,report_bytes=P.file(delivery/'DELIVERY.json',4194304,True);need(report_pin==a.delivery_sha256);declared={x['path']:x['sha256'] for x in P.decode(report_bytes)['files']};payload={}
 for n in ('dil','dillsp','SHA256SUMS','GIT_SHA','sbom.spdx.json','provenance.intoto.json'):
  digest,b=P.file(delivery/'binaries'/n,134217728,True);need(digest==declared['binaries/'+n]);payload['binaries/'+n]=b
 need(sum(map(len,payload.values()))<=268435456)
 for name in ('kur.sh','kaldir.sh'):
  _,b=P.file(repo/'scripts'/name,1048576,True);expected=H.run(['git','-C',repo,'show',a.sha+':scripts/'+name],timeout=15,limit=1048576);need(b==expected);payload['scripts/'+name]=b
 H.clean_exact(repo,a.sha);need(P.delivery(delivery,a.sha,a.delivery_sha256,a.arch)==validated)
 epoch=H.run(['git','-C',repo,'show','-s','--format=%ct',a.sha],timeout=15,limit=128).decode().strip();need(epoch.isdecimal() and 0<int(epoch)<2**63)
 out.mkdir(mode=0o700);logs=out/'logs';logs.mkdir(mode=0o700);tree=out/'package-tree';tree.mkdir(mode=0o700);base='usr/share/zee-private-carrier/'+a.sha+'/'
 manifest={'schema':'zee-private-carrier-payload-1','source_sha':a.sha,'delivery_sha256':a.delivery_sha256,'files':[{'path':n,'sha256':hashlib.sha256(b).hexdigest()} for n,b in sorted(payload.items())],'private_evaluation_only':True,'automatic_activation':False}
 payload['CARRIER.json']=(json.dumps(manifest,sort_keys=True,indent=2)+'\n').encode()
 for name,data in payload.items():emit(tree,base+name,data)
 emit(tree,'DEBIAN/control',control(a.sha,a.arch))
 for q in sorted(tree.rglob('*'),reverse=True):
  if q.is_dir():q.chmod(0o755)
  os.utime(q,(int(epoch),int(epoch)),follow_symlinks=False)
 tree.chmod(0o755);os.utime(tree,(int(epoch),int(epoch)))
 expected=inventory(tree);env={k:v for k,v in os.environ.items() if k in {'HOME','USER','LOGNAME'}};env.update(PATH='/usr/bin:/bin',LANG='C.UTF-8',TZ='UTC',SOURCE_DATE_EPOCH=epoch);deb=out/(package_name(a.sha)+'.deb')
 H.run(['/usr/bin/dpkg-deb','--uniform-compression','-Zxz','-z6','--threads-max=2','--root-owner-group','--build',tree,deb],cwd=out,timeout=60,limit=1048576,env=env,log_path=logs/'build.log')
 H.run(['/usr/bin/dpkg-deb','--info',deb],cwd=out,timeout=15,limit=65536,env=env,log_path=logs/'info.log');H.run(['/usr/bin/dpkg-deb','--contents',deb],cwd=out,timeout=15,limit=65536,env=env,log_path=logs/'contents.log')
 extracted=out/'inspected';H.run(['/usr/bin/dpkg-deb','--raw-extract',deb,extracted],cwd=out,timeout=30,limit=65536,env=env,log_path=logs/'extract.log');need(inventory(extracted)==expected)
 H.clean_exact(repo,a.sha);need(P.delivery(delivery,a.sha,a.delivery_sha256,a.arch)==validated)
 receipt={'schema':'zee-private-carrier-build-1','status':'passed-build-inspection-only','source_sha':a.sha,'delivery_sha256':a.delivery_sha256,'package':package_name(a.sha),'architecture':a.arch,'deb_sha256':hashlib.sha256(deb.read_bytes()).hexdigest(),'deb_path':str(deb),'payload':manifest,'source_date_epoch':int(epoch),'exact_extracted_inventory':True,'no_maintainer_scripts':True,'installed':False,'activated':False,'published':False,'private_maintainer_placeholder':True,'logs':[{'path':str(q.relative_to(out)),'sha256':hashlib.sha256(q.read_bytes()).hexdigest()} for q in sorted(logs.iterdir())]}
 emit(out,'BUILD.json',(json.dumps(receipt,indent=2)+'\n').encode(),0o600);return receipt

def main():
 class Parser(argparse.ArgumentParser):
  def error(self,message):raise Reject('arguments')
 p=Parser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--sha',required=True);p.add_argument('--delivery',type=Path,required=True);p.add_argument('--delivery-sha256',required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--arch',choices=('amd64','arm64'),required=True);a=p.parse_args();r=build(a);print(json.dumps({'schema':r['schema'],'status':r['status'],'deb_sha256':r['deb_sha256']}));return 0
if __name__=='__main__':
 try:sys.exit(main())
 except Exception:sys.stderr.write('private-carrier-rejected\n');sys.exit(2)
