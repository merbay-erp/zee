#!/usr/bin/env python3
"""K266: local owned-prefix actual installation acceptance; preparation is no acceptance."""
import argparse,hashlib,importlib.util,json,os,re,shutil,signal,sys,time
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
HELPER_SHA='509106d70e5e35c1142594c98df022b01e689966415bdd94a8f67feecb8587d8'
assert hashlib.sha256((HERE/'owned_helper.py').read_bytes()).hexdigest()==HELPER_SHA
spec=importlib.util.spec_from_file_location('owned_install_helper',HERE/'owned_helper.py');D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D)
SENTINEL=b'K266 foreign sentinel: preserve exactly\n'
def sha(value):D.require(isinstance(value,str) and re.fullmatch('[0-9a-f]{64}',value),'SHA256 argument');return value
def within(root,path):
    p=Path(path);D.require(not p.is_symlink(),'owned path symlink');resolved=p.resolve();D.require(resolved!=root and resolved.is_relative_to(root),'owned prefix confinement');return resolved
def artifact_check(delivery,expected_source,report_sha,hashes):
    delivery=Path(delivery).resolve(strict=True);report=delivery/'DELIVERY.json';D.require(not report.is_symlink() and report.is_file() and report.stat().st_size<=2*1024*1024 and D.digest(report)==report_sha,'delivery report pin/budget')
    data=json.loads(report.read_text());D.require(data['schema']=='zee-local-owned-delivery-1' and data['status']=='passed' and data['source_sha']==expected_source and data['signed'] is False and data['published'] is False,'canonical unsigned local delivery')
    D.require(data['binary_sha256']==hashes and data['build_identity']['source_sha']==expected_source and data['build_identity']['dirty'] is False,'delivery source/binary identities')
    seen=set()
    for row in data['files']:
        name=row['path'];D.require(isinstance(name,str) and name and not Path(name).is_absolute() and '..' not in Path(name).parts and name not in seen,'artifact relative file map');seen.add(name)
        path=(delivery/name).resolve(strict=True);D.require(path.is_relative_to(delivery) and path.is_file() and D.digest(path)==sha(row['sha256']),'artifact file hash/confinement')
    D.require({'binaries/dil','binaries/dillsp','binaries/SHA256SUMS','binaries/GIT_SHA','binaries/sbom.spdx.json','binaries/provenance.intoto.json'}<=seen,'canonical artifact dependencies')
    binaries=(delivery/'binaries').resolve(strict=True);D.require(binaries.is_relative_to(delivery),'artifact binary directory confinement')
    expected=''.join(hashes[n]+'  '+n+'\n' for n in ('dil','dillsp'))
    D.require((binaries/'SHA256SUMS').read_text()==expected and (binaries/'GIT_SHA').read_text()==expected_source+'\n','artifact checksums/source')
    D.require(not (binaries/'SHA256SUMS.zee-imza').exists(),'unsigned fixture required')
    D.require(all(D.digest(binaries/n)==h for n,h in hashes.items()),'expected binary pin')
    return delivery,binaries
def preflight(repo,source,out,delivery,report_sha,hashes):
    D.require(os.geteuid()!=0,'nonroot local installer owner required')
    repo,out,helpers=D.preflight(repo,source,out)
    D.require(out.parent.stat().st_uid==os.geteuid(),'output parent must belong to current owner')
    for name in ('scripts/kur.sh','scripts/kaldir.sh'):D.require((repo/name).is_file(),'installer helper missing');helpers[name]=D.digest(repo/name)
    delivery,binaries=artifact_check(delivery,source,report_sha,hashes)
    D.require(out!=delivery and delivery not in out.parents and out not in delivery.parents,'delivery/output alias')
    return repo,out,delivery,binaries,helpers
def manifest_check(data,bin_dir,hashes,source):
    lines=data.splitlines();D.require(lines[:2]==['# zee-kurulum-1','# dosya\tsha256\tkaynak_git_sha'],'installed manifest schema')
    rows=[x.split('\t') for x in lines[2:]];D.require(len(rows)==2 and all(len(r)==3 for r in rows),'installed manifest cardinality')
    expected={str(bin_dir/n):(h,source) for n,h in hashes.items()}
    D.require(len({r[0] for r in rows})==2 and {r[0]:(r[1],r[2]) for r in rows}==expected,'installed manifest exact owned paths/hash/source')
def execute(repo,source,out,delivery,binaries,helpers,report_sha,hashes):
    D.clean_exact(repo,source);out.mkdir(mode=0o700);logs=out/'logs';logs.mkdir();env=D.controlled_env();env['CARGO_NET_OFFLINE']='true'
    pins={'driver':D.digest(Path(__file__)),'helper':D.digest(HERE/'owned_helper.py'),'smoke':D.digest(HERE/'lsp_smoke.py')};records=[];cases=[]
    proof={'schema':'zee-owned-prefix-install-acceptance-1','K':'K-266','source_sha':source,'status':'running','acceptance':False,'delivery_report_sha256':report_sha,'binary_sha256':hashes,'source_helpers':helpers,'external_start_sha256':pins,'cases':cases,'commands':records,'scope':'Actual unsigned local canonical artifact, owned prefixes only; no HOME/PATH/global install, network, keys or publication.'}
    def command(label,args,expected=0):
        log=logs/(label+'.log');start=time.monotonic();raw=D.run([sys.executable,'-I','-B','-c','import subprocess,sys; c=subprocess.call(sys.argv[1:]); print("__ZEE_INSTALL_EXIT__"+str(c),flush=True)',*map(str,args)],cwd=out,env=env,timeout=30,limit=1024*1024,log_path=log)
        match=re.search(rb'\n?__ZEE_INSTALL_EXIT__(-?[0-9]+)\n$',raw);D.require(match is not None,'owned command exit receipt');status=int(match[1]);records.append({'name':label,'argv':list(map(str,args)),'exit':status,'elapsed_seconds':time.monotonic()-start,'log':str(log.relative_to(out)),'log_sha256':D.digest(log)});D.require(status==expected,'unexpected installer exit');return raw[:match.start()]
    def prefix(name):
        parent=within(out,out/name);parent.mkdir(mode=0o700);b=within(out,parent/'bin');v=within(out,parent/'data');b.mkdir();v.mkdir()
        for p in (b/'foreign.sentinel',v/'foreign.sentinel'):p.write_bytes(SENTINEL)
        return b,v
    def foreign(b,v):D.require(all((p/'foreign.sentinel').read_bytes()==SENTINEL for p in (b,v)),'foreign sentinel changed')
    def empty(b,v):foreign(b,v);D.require(set(x.name for x in b.iterdir())=={'foreign.sentinel'} and set(x.name for x in v.iterdir())=={'foreign.sentinel'},'rejected/uninstalled prefix contains installed bytes/staging')
    def installed(b,v):
        foreign(b,v);D.require(all(D.digest(b/n)==h for n,h in hashes.items()),'installed binary mismatch');manifest_check((v/'kurulum-v1.tsv').read_text(),b,hashes,source)
    def install(label,artifact,b,v,status=0):return command(label,['bash',repo/'scripts/kur.sh','--artefakt',artifact,'--hedef',b,'--veri',v],status)
    def uninstall(label,v,status=0):return command(label,['bash',repo/'scripts/kaldir.sh','--veri',v],status)
    try:
        b,v=prefix('valid');install('valid-install',binaries,b,v);installed(b,v)
        identity=json.loads(D.run([b/'dil','sürüm','--json'],cwd=out,env=env,timeout=10,limit=65536,log_path=logs/'installed-build-json.log'));D.require(identity==json.loads((delivery/'DELIVERY.json').read_text())['build_identity'],'installed BuildJSON equality')
        smoke=out/'lsp-smoke.json';D.run([sys.executable,'-I','-B',HERE/'lsp_smoke.py','--binary',b/'dillsp','--output',smoke],cwd=out,env=env,timeout=15,limit=65536,log_path=logs/'lsp-smoke.log');D.require(json.loads(smoke.read_text())['passed'] is True,'actual installed LSP smoke');cases.append({'name':'valid-install-buildJSON-owned-LSP','passed':True})
        before=(v/'kurulum-v1.tsv').read_bytes();message=install('repeat-install-reject',binaries,b,v,1);D.require('Önce var olan kurulumu kaldır'.encode() in message,'repeat install reason');installed(b,v);D.require((v/'kurulum-v1.tsv').read_bytes()==before,'repeat install changed manifest');cases.append({'name':'repeat-install-refuses-no-overwrite','passed':True})
        with (b/'dil').open('ab') as stream:stream.write(b'K266-install-tamper')
        changed={n:D.digest(b/n) for n in hashes};message=uninstall('changed-installed-refuse',v,1);D.require('kurulumdan sonra değişmiş; hiçbir dosya silinmedi'.encode() in message,'changed uninstall reason');D.require((v/'kurulum-v1.tsv').read_bytes()==before and all((b/n).is_file() and D.digest(b/n)==h for n,h in changed.items()),'failed uninstall partially removed/changed files');foreign(b,v);cases.append({'name':'changed-installed-refuses-without-removal','passed':True})
        shutil.copyfile(binaries/'dil',b/'dil');installed(b,v);uninstall('valid-uninstall',v);empty(b,v);cases.append({'name':'manifest-uninstall-preserves-foreign','passed':True})
        message=uninstall('repeat-uninstall-reject',v,1);D.require('Kurulum manifesti yok:'.encode() in message,'repeat uninstall reason');empty(b,v);cases.append({'name':'repeat-uninstall-refuses-preserves-foreign','passed':True})
        install('reinstall',binaries,b,v);installed(b,v);uninstall('reinstall-uninstall',v);empty(b,v);cases.append({'name':'reinstall-uninstall-manifest-cycle','passed':True})
        for kind in ('binary','checksum'):
            copied=within(out,out/('tampered-'+kind));shutil.copytree(binaries,copied)
            if kind=='binary':
                with (copied/'dil').open('ab') as stream:stream.write(b'K266-artifact-tamper')
            else:(copied/'SHA256SUMS').write_text('0'*64+'  dil\n'+hashes['dillsp']+'  dillsp\n')
            tb,tv=prefix('reject-'+kind);message=install('tampered-'+kind+'-reject',copied,tb,tv,1);D.require('SHA-256 uyuşmuyor: dil'.encode() in message,'tampered artifact checksum reason');empty(tb,tv);cases.append({'name':'tampered-'+kind+'-refuses-no-installed-bytes','passed':True})
        D.clean_exact(repo,source);D.require({p:D.digest(repo/p) for p in helpers}==helpers,'source installer/helper changed');artifact_check(delivery,source,report_sha,hashes);D.require(pins=={'driver':D.digest(Path(__file__)),'helper':D.digest(HERE/'owned_helper.py'),'smoke':D.digest(HERE/'lsp_smoke.py')},'external runner changed')
        proof.update(status='passed',acceptance=True,source_end_clean=True,external_end_sha256=pins,original_artifact_unchanged=True)
    except BaseException as e:proof.update(status='failed',failure_type=type(e).__name__);raise
    finally:
        proof['files']=[{'path':str(p.relative_to(out)),'sha256':D.digest(p)} for p in sorted(out.rglob('*')) if p.is_file()];(out/'PROOF.json').write_text(json.dumps(proof,indent=2)+'\n')
    return proof
def main(argv=None):
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--sha',required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--delivery',type=Path,required=True);p.add_argument('--delivery-sha256',required=True);p.add_argument('--dil-sha256',required=True);p.add_argument('--dillsp-sha256',required=True);p.add_argument('--preflight-only',action='store_true');a=p.parse_args(argv)
    D.CANCELLED=False
    def cancel(number,frame):D.CANCELLED=True
    for n in (signal.SIGTERM,signal.SIGINT):signal.signal(n,cancel)
    hashes={'dil':sha(a.dil_sha256),'dillsp':sha(a.dillsp_sha256)};report_sha=sha(a.delivery_sha256);repo,out,delivery,binaries,helpers=preflight(a.repo,a.sha,a.out,a.delivery,report_sha,hashes)
    if a.preflight_only:print(json.dumps({'status':'preflight-passed','acceptance':False,'installed':False}));return 0
    result=execute(repo,a.sha,out,delivery,binaries,helpers,report_sha,hashes);print(json.dumps({'status':result['status'],'acceptance':result['acceptance'],'source_sha':a.sha}));return 0
if __name__=='__main__':
    try:raise SystemExit(main())
    except (D.Reject,OSError,ValueError,KeyError) as e:print(json.dumps({'status':'rejected','type':type(e).__name__}),file=sys.stderr);raise SystemExit(2)
