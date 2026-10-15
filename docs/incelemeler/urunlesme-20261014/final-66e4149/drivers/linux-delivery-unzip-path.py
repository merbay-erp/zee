import argparse,json,runpy,time
from pathlib import Path
p=argparse.ArgumentParser()
for n in ['repo','native-out','out']:p.add_argument('--'+n,type=Path,required=True)
p.add_argument('--sha',required=True)
a=p.parse_args();R=a.repo.resolve(strict=True);H=runpy.run_path(str(R/'scripts/lifecycle_core.py'))
H['clean_exact'](R,a.sha)
n=json.loads((a.native_out/'CONTROLLER.json').read_text());assert n['source_sha']==a.sha
retry=a.native_out.parent/'s3-66e4149-linux-http-retry'
r=json.loads((retry/'CONTROLLER.json').read_text());assert r['status']=='passed' and r['source_sha']==a.sha and r['guest_leaf']==n['guest_leaf']
for directory,name,count in [(a.native_out,'K282',4),(retry,'K283',7)]:
 receipt=json.loads((directory/(name+'-RECEIPT.json')).read_text());proofpath=directory/(name+'-proof.json');proof=json.loads(proofpath.read_text())
 assert receipt['status']=='passed' and receipt['source_sha']==a.sha and receipt['outer_owned_handles_closed']
 assert H['digest'](proofpath)==receipt['proof_sha256']
 assert proof['source_sha']==a.sha and proof['status']=='passed' and len(proof['cases'])==count and all(x['passed'] for x in proof['cases'])
 assert proof['binary_end_equal'] and proof['source_pins_end_equal'] and proof['postgres_binary_pins_end_equal'] and proof['owned_units_released'] and proof['temporary_cleanup_complete'] and proof['cleanup_errors']==[]
 for name,digest in proof['source_pins_start'].items():assert H['digest'](R/name)==digest

leaf=n['guest_leaf'];assert leaf.startswith('/home/mustafaerbay.guest/zee-k276/service-acceptance-'+a.sha[:12]+'-')
assert not a.out.exists() and a.out.is_absolute() and R not in a.out.parents
a.out.mkdir(mode=0o700);(a.out/'binaries').mkdir(mode=0o700)
home=Path('/private/tmp/zk276-hih4chab/lima');i=home/'k276';d=i/'disk'
assert (i.stat().st_dev,i.stat().st_ino)==(16777232,422274626)
assert (d.stat().st_dev,d.stat().st_ino,d.stat().st_size)==(16777232,422274644,17179869184)
assert H['digest'](i/'lima.yaml')=='ac827e1be51749d62ceefb0e9cb8bdce1257a10f96e0def065ad6506e85854f9'
env=H['controlled_env']();env['LIMA_HOME']=str(home)
script=Path(__file__).parent.parent/'local-deliverable-plan-stage/deliverable.py'
assert H['digest'](script)=='509106d70e5e35c1142594c98df022b01e689966415bdd94a8f67feecb8587d8'
receipt={'source_sha':a.sha,'guest_leaf':leaf,'status':'running','helper_sha256':H['digest'](script),'started_ns':time.time_ns()}
def cmd(argv,label,timeout=30):
    return H['run'](argv,env=env,timeout=timeout,limit=8*1024*1024,log_path=a.out/(label+'.log'))
def remote(code,label,timeout=30):
    return cmd(['/opt/homebrew/bin/limactl','shell','--workdir=/tmp','k276','--','python3','-I','-B','-c',code],label,timeout)
try:
    assert json.loads(cmd(['/opt/homebrew/bin/limactl','list','k276','--json'],'vm-list'))['status']=='Running'
    remote('from pathlib import Path; import shutil,os; p=Path('+repr(leaf)+'); assert p.resolve()==p and p.stat().st_uid==os.geteuid()==501 and shutil.disk_usage(p).free>4*1024**3; assert not (p/"deliverable-unzip-path.py").exists() and not (p/"linux-delivery-unzip-path").exists()','preflight')
    remote("from pathlib import Path; import hashlib; p=Path('/home/mustafaerbay.guest/zee-k276/tools/unzip-owned/usr/bin/unzip'); assert hashlib.sha256(p.read_bytes()).hexdigest()=='104cb83abadfb24c8cb09818021604b8d6ec17c3d5dad330268df4e985e2f65d'",'unzip-pin')
    cmd(['/opt/homebrew/bin/limactl','copy',script,'k276:'+leaf+'/deliverable-unzip-path.py'],'copy-helper',60)
    code='''from pathlib import Path
import hashlib,runpy
R=Path('/home/mustafaerbay.guest/zee-k276');B=Path(LEAF)
P=B/'deliverable-unzip-path.py';assert hashlib.sha256(P.read_bytes()).hexdigest()=='509106d70e5e35c1142594c98df022b01e689966415bdd94a8f67feecb8587d8'
H=runpy.run_path(str(B/'source/scripts/lifecycle_core.py'));E=H['controlled_env']()
E.update(PATH=str(R/'tools/git-2.56.0/bin')+':'+str(R/'tools/unzip-owned/usr/bin')+':'+str(R/'ru-cargo/bin')+':/usr/bin:/bin:/usr/sbin:/sbin',RUSTUP_HOME=str(R/'ruhome'),CARGO_HOME=str(R/'cargo-home'),RUSTUP_TOOLCHAIN='1.93.1',CARGO_BUILD_JOBS='1',CARGO_NET_OFFLINE='true',TMPDIR='/tmp')
for key in list(E):
 if key.startswith(('ZEE_','CARGO_PROFILE_')) or key in ['CARGO_TARGET_DIR','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:E.pop(key)
H['run'](['/usr/bin/python3','-I','-B',P,'--repo',B/'source','--sha',SHA,'--out',B/'linux-delivery-unzip-path'],env=E,cwd=B,timeout=4000,limit=8*1024*1024,log_path=B/'linux-delivery-unzip-path-controller.log')
'''.replace('LEAF',repr(leaf)).replace('SHA',repr(a.sha))
    remote(code,'campaign',4300)
    remote('from pathlib import Path; print((Path('+repr(leaf)+')/"linux-delivery-unzip-path/DELIVERY.json").read_text())','delivery-receipt')
    value=json.loads((a.out/'delivery-receipt.log').read_text());assert value['status']=='passed' and value['source_sha']==a.sha
    (a.out/'DELIVERY.json').write_text(json.dumps(value,indent=2)+'\n')
    for name in ['dil','dillsp','GIT_SHA','SHA256SUMS','sbom.spdx.json','provenance.intoto.json']:
        cmd(['/opt/homebrew/bin/limactl','copy','k276:'+leaf+'/linux-delivery-unzip-path/binaries/'+name,a.out/'binaries'/name],'copy-'+name,120)
    assert all(H['digest'](a.out/'binaries'/name)==h for name,h in value['binary_sha256'].items())
    H['clean_exact'](R,a.sha);receipt['status']='passed'
except BaseException as error:
    receipt.update(status='failed',failure_type=type(error).__name__)
    raise
finally:
    receipt['finished_ns']=time.time_ns();(a.out/'CONTROLLER.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'status':receipt['status'],'source_sha':a.sha}))
