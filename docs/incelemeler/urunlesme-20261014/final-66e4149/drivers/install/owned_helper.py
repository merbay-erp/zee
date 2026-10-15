#!/usr/bin/env python3
"""External owned local delivery; unsigned, never publishes or generates keys."""
import argparse,hashlib,importlib.util,json,os,re,selectors,shutil,signal,subprocess,sys,time,tomllib
from pathlib import Path
MAX_CAPTURE=8*1024*1024
CANCELLED=False
class Reject(Exception):pass
def require(value,reason):
    if not value:raise Reject(reason)
def digest(path):
    h=hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda:stream.read(1024*1024),b''):h.update(block)
    return h.hexdigest()
def held_status(child):
    require(child.returncode is None,'lost child authority')
    status=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
    return None if status is None else (status.si_status if status.si_code==os.CLD_EXITED else -status.si_status)
def stop_owned(child,grace=3):
    deadline=time.monotonic()+grace
    held_status(child)
    def retry():
        left=deadline-time.monotonic();require(left>0,'owned cleanup permission deadline');time.sleep(min(.001,left))
    def alive():
        while True:
            try:os.killpg(child.pid,0);return True
            except ProcessLookupError:return False
            except PermissionError:
                if held_status(child) is not None:return False
                retry()
    def send(number):
        while True:
            try:os.killpg(child.pid,number);return
            except ProcessLookupError:return
            except PermissionError:
                if not alive():return
                retry()
    if alive():send(signal.SIGTERM)
    while alive() and time.monotonic()<deadline:time.sleep(min(.02,max(0,deadline-time.monotonic())))
    if alive():send(signal.SIGKILL)
    return child.wait(timeout=2)
def controlled_env():
    env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
    env.pop('ZEE_SURUM_TANI_DIZINI',None)
    env.update({'GIT_TERMINAL_PROMPT':'0','GIT_CONFIG_GLOBAL':'/dev/null','GIT_CONFIG_NOSYSTEM':'1'})
    return env
def run(argv,cwd=None,timeout=30,limit=MAX_CAPTURE,env=None,log_path=None):
    require(not CANCELLED,'cancelled')
    child=subprocess.Popen([str(x) for x in argv],cwd=cwd,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True,close_fds=True,env=controlled_env() if env is None else env)
    sel=None;out=bytearray();deadline=time.monotonic()+timeout
    try:
        sel=selectors.DefaultSelector();sel.register(child.stdout,selectors.EVENT_READ)
        while sel.get_map():
            require(not CANCELLED and time.monotonic()<deadline,'command deadline/cancelled')
            for key,_ in sel.select(min(.1,max(0,deadline-time.monotonic()))):
                block=os.read(key.fileobj.fileno(),4096)
                if not block:sel.unregister(key.fileobj)
                else:require(len(out)+len(block)<=limit,'command capture budget');out.extend(block)
        status=held_status(child)
        while status is None:
            require(not CANCELLED and time.monotonic()<deadline,'terminal command deadline/cancelled');time.sleep(.01);status=held_status(child)
        require(status==0,'command nonzero')
        return bytes(out)
    finally:
        try:
            if sel is not None:sel.close()
            if log_path is not None:Path(log_path).write_bytes(out)
        finally:
            try:stop_owned(child)
            finally:child.stdout.close()
def git(repo,*args):return run(['git','-C',repo,*args]).decode().strip()
def clean_exact(repo,sha):
    require(git(repo,'rev-parse','HEAD')==sha,'HEAD mismatch')
    require(not git(repo,'status','--porcelain','--untracked-files=all'),'dirty repository')
def preflight(repo,sha,out,check_toolchain=True):
    require(bool(re.fullmatch('[0-9a-f]{40}',sha)),'exact source SHA required')
    require(Path(repo).is_dir(),'repository missing');repo=Path(repo).resolve(strict=True)
    require(not Path(out).is_symlink(),'output symlink');require(not Path(out).exists(),'output exists')
    require(Path(out).parent.is_dir(),'output parent missing');out=Path(out).parent.resolve(strict=True)/Path(out).name
    require(repo!=out and repo not in out.parents,'output must be external')
    require(os.name=='posix' and hasattr(os,'WNOWAIT'),'POSIX held ownership required')
    clean_exact(repo,sha);require(git(repo,'rev-parse','--is-shallow-repository')=='false','shallow repository')
    require(git(repo,'rev-parse',sha+'^{commit}')==sha,'commit identity')
    alt=Path(git(repo,'rev-parse','--git-path','objects/info/alternates'));alt=alt if alt.is_absolute() else repo/alt
    require(not alt.exists(),'object alternates')
    required=['scripts/surum-artefakti.sh','scripts/temiz-kaynak-arsivi.sh','scripts/kaynak-ci-esleme.py','scripts/rust-arac-zinciri.sh','compiler/rust-toolchain.toml']
    require(all((repo/x).is_file() for x in required),'required source helper missing')
    require(tomllib.loads((repo/'compiler/rust-toolchain.toml').read_text())['toolchain']['channel']=='1.93.1','Rust pin mismatch')
    if check_toolchain:
        tools=run(['rustup','toolchain','list']).decode();require(any(x.split()[0].startswith('1.93.1-') for x in tools.splitlines() if x.split()),'installed Rust1.93.1 required')
    return repo,out,{x:digest(repo/x) for x in required}
def clone(repo,path,sha):
    run(['git','clone','--no-hardlinks','--local',repo,path],timeout=120);git(path,'checkout','--detach',sha);clean_exact(path,sha)
def source_zip(repo,path,sha):
    run(['bash',repo/'scripts/temiz-kaynak-arsivi.sh',path],cwd=repo,timeout=120)
    require(path.is_file() and Path(str(path)+'.sha256').is_file(),'source archive absent')
    require(Path(str(path)+'.sha256').read_text()==digest(path)+'  '+path.name+'\n','source ZIP checksum')
    sys.dont_write_bytecode=True
    spec=importlib.util.spec_from_file_location('delivery_source_verifier',repo/'scripts/kaynak-ci-esleme.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    require(path.stat().st_size<=module.MAX_SOURCE_BYTES,'source ZIP byte bound')
    module.zip_hashes(path.read_bytes(),sha,module.archive_hashes(repo,sha))
def deliver(repo,sha,out,source_hashes):
    clean_exact(repo,sha);out.mkdir(mode=0o700);work=out/'owned-work';work.mkdir(mode=0o700);env=controlled_env();env['CARGO_NET_OFFLINE']='true';env['CARGO_TARGET_DIR']=str(work/'metadata-target');temporary=work/'temp';temporary.mkdir(mode=0o700);env['TMPDIR']=str(temporary)
    report={'schema':'zee-local-owned-delivery-1','source_sha':sha,'signed':False,'published':False,'status':'running','source_helpers':source_hashes}
    try:
        build=work/'build';clone(repo,build,sha)
        log=run(['bash',build/'scripts/surum-artefakti.sh','--cikti',out/'binaries'],cwd=build,timeout=3600,env=env,log_path=out/'build.log');clean_exact(build,sha)
        binary=out/'binaries';require((binary/'GIT_SHA').read_text()==sha+'\n','artifact GIT_SHA')
        sums={}
        for line in (binary/'SHA256SUMS').read_text().splitlines():
            match=re.fullmatch('([0-9a-f]{64})  (dil|dillsp)',line);require(match is not None,'binary SHA256SUMS format');require(match[2] not in sums,'duplicate binary checksum');sums[match[2]]=match[1]
        require(set(sums)=={'dil','dillsp'} and all(digest(binary/name)==h for name,h in sums.items()),'binary digest equality')
        require(not (binary/'SHA256SUMS.zee-imza').exists(),'default must remain unsigned')
        identity=json.loads(run([binary/'dil','sürüm','--json'],limit=65536));require(identity.get('schema')=='zee-build-v1' and identity.get('source_sha')==sha and identity.get('dirty') is False,'BuildJSON identity')
        sbom=json.loads((binary/'sbom.spdx.json').read_text());provenance=json.loads((binary/'provenance.intoto.json').read_text())
        require(provenance.get('_type')=='https://in-toto.io/Statement/v1' and provenance.get('predicateType')=='https://slsa.dev/provenance/v1','provenance schema')
        pred=provenance['predicate'];ext=pred['buildDefinition']['externalParameters'];require(ext['gitCommit']==sha and ext['rustc'].startswith('rustc 1.93.1 ') and ext['sourceDateEpoch']==int(git(build,'show','-s','--format=%ct',sha)),'provenance source/toolchain/epoch')
        require(pred['zeeTekrarUretim']['ikiTemizKlonEsit'] is True and isinstance(ext['buildEnvironment'],dict),'provenance two-build/environment')
        require({x['name']:x['digest']['sha256'] for x in provenance['subject']}==sums,'provenance binary subjects')
        products={x['name']:x['digest']['sha256'] for x in pred['runDetails']['byproducts']};require(products.get('SHA256SUMS')==digest(binary/'SHA256SUMS')==products.get('SHA256SUMS.ikinci-klon'),'two-build checksum evidence')
        graph=sbom['@graph'];require(sbom['@context']=='https://spdx.org/rdf/3.0.1/spdx-context.jsonld','SPDX context')
        require(any(x.get('type')=='software_Package' and x.get('name')=='dil' and x.get('software_downloadLocation','').endswith('@'+sha) for x in graph),'SBOM source root')
        require({x['name']:x['verifiedUsing'][0]['hashValue'] for x in graph if x.get('type')=='software_File'}==sums,'SBOM binary subjects')
        require(all(('-- '+name+' iki klonda eş: '+h).encode() in log for name,h in sums.items()),'observed two-build equality log')
        zip_paths=[]
        for name in ('zip_a','zip_b'):
            zrepo=work/name;clone(repo,zrepo,sha);zpath=zrepo/'dist'/('zee-kaynak-'+sha+'.zip');source_zip(zrepo,zpath,sha);require(git(zrepo,'rev-parse','HEAD')==sha,'source ZIP HEAD changed');zip_paths.append(zpath)
        require(digest(zip_paths[0])==digest(zip_paths[1]),'two source ZIPs differ')
        archive=out/zip_paths[0].name;shutil.copyfile(zip_paths[0],archive);shutil.copyfile(str(zip_paths[0])+'.sha256',str(archive)+'.sha256');source_zip_digest=digest(archive);require(source_zip_digest==digest(zip_paths[0]) and Path(str(archive)+'.sha256').read_text()==source_zip_digest+'  '+archive.name+'\n','copied ZIP checksum')
        for zpath in zip_paths:
            zpath.unlink();Path(str(zpath)+'.sha256').unlink();clean_exact(zpath.parent.parent,sha)
        zrepo=work/'zip_a';owned_branch='codex/local-deliverable-'+sha[:12];git(zrepo,'branch',owned_branch,sha);bundle=out/'zee.bundle';run(['git','-C',zrepo,'bundle','create',bundle,'HEAD','refs/heads/'+owned_branch],timeout=120)
        heads=git(zrepo,'bundle','list-heads',str(bundle)).splitlines();require(set(heads)=={sha+' HEAD',sha+' refs/heads/'+owned_branch},'bundle HEAD/branch')
        bare=work/'empty.git';run(['git','init','--bare',bare]);verification=run(['git','-C',bare,'bundle','verify',bundle]);(out/'bundle-verify.log').write_bytes(verification)
        restored=work/'restored';run(['git','clone','--branch',owned_branch,bundle,restored],timeout=120);clean_exact(restored,sha);require(git(restored,'symbolic-ref','--short','HEAD')==owned_branch,'restored branch')
        require(not (restored/'.git/objects/info/alternates').exists(),'restored alternates');fsck=run(['git','-C',restored,'fsck','--full'],timeout=120);(out/'fsck.log').write_bytes(fsck);require(git(restored,'rev-parse','HEAD^{tree}')==git(repo,'rev-parse',sha+'^{tree}'),'restored source tree')
        clean_exact(repo,sha);require({x:digest(repo/x) for x in source_hashes}==source_hashes,'source helper changed')
        report.update({'status':'passed','build_identity':identity,'binary_sha256':sums,'source_zip_sha256':source_zip_digest,'two_source_zip_equal':True,'bundle_head':sha,'bundle_branch':owned_branch,'bundle_sha256':digest(bundle),'restoration_fsck':True,'files':[{'path':str(p.relative_to(out)),'sha256':digest(p)} for p in sorted(out.rglob('*')) if p.is_file() and work not in p.parents]});shutil.rmtree(work)
    except BaseException as e:report['status']='failed';report['failure_type']=type(e).__name__;raise
    finally:(out/'DELIVERY.json').write_text(json.dumps(report,indent=2)+'\n')
    return report
def main(argv=None):
    parser=argparse.ArgumentParser();parser.add_argument('--repo',type=Path,required=True);parser.add_argument('--sha',required=True);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--preflight-only',action='store_true');args=parser.parse_args(argv)
    global CANCELLED
    CANCELLED=False
    def cancel(number,frame):
        global CANCELLED
        CANCELLED=True
    for number in (signal.SIGTERM,signal.SIGINT):signal.signal(number,cancel)
    repo,out,hashes=preflight(args.repo,args.sha,args.out)
    if args.preflight_only:clean_exact(repo,args.sha);print(json.dumps({'status':'preflight-passed','source_sha':args.sha,'executed_build_archive_bundle':False}));return 0
    result=deliver(repo,args.sha,out,hashes);print(json.dumps({'status':result['status'],'source_sha':args.sha,'signed':False,'published':False}));return 0
if __name__=='__main__':
    try:raise SystemExit(main())
    except (Reject,OSError,ValueError,subprocess.SubprocessError) as e:print(json.dumps({'status':'rejected','type':type(e).__name__}),file=sys.stderr);raise SystemExit(2)
