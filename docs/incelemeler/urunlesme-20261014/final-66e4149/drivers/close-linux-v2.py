import json,runpy,time
from pathlib import Path
E=Path(__file__).resolve().parents[1];R=Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014');H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));S='66e4149e53451111d153f47283c219079a8d1cab';H['clean_exact'](R,S)
for name in ['s3-linux-delivery-unzip-controller.json','s3-linux-install-content-controller.json']:
 v=json.loads((E/name).read_text());assert v['status']=='passed' and v['owned_group_closed_and_reaped']
O=E/'s3-66e4149-linux-end-v2';assert not O.exists();O.mkdir(mode=0o700)
home=Path('/private/tmp/zk276-hih4chab/lima');i=home/'k276';disk=i/'disk'
def ownership():
 assert (i.stat().st_dev,i.stat().st_ino,i.stat().st_uid)==(16777232,422274626,501)
 assert (disk.stat().st_dev,disk.stat().st_ino,disk.stat().st_size,disk.stat().st_uid)==(16777232,422274644,17179869184,501)
 assert H['digest'](i/'lima.yaml')=='ac827e1be51749d62ceefb0e9cb8bdce1257a10f96e0def065ad6506e85854f9'
ownership();env=H['controlled_env']();env['LIMA_HOME']=str(home)
def cmd(args,label,timeout=30):return H['run'](args,env=env,timeout=timeout,limit=1024**2,log_path=O/(label+'.log'))
assert json.loads(cmd(['/opt/homebrew/bin/limactl','list','k276','--json'],'before'))['status']=='Running'
code="""import json,runpy,shutil
from pathlib import Path
B=Path('/home/mustafaerbay.guest/zee-k276/service-acceptance-66e4149e5345-1792057016809668000');R=B/'source';H=runpy.run_path(str(R/'scripts/lifecycle_core.py'));E=H['controlled_env']();E['PATH']='/home/mustafaerbay.guest/zee-k276/tools/git-2.56.0/bin:/usr/bin:/bin:/usr/sbin:/sbin';H['clean_exact'](R,'66e4149e53451111d153f47283c219079a8d1cab')
latch=B/'k282-native/unknown-state/inflight.json';h=H['digest'](latch);assert h=='9390eaecfc326878a771e4c18441303e130163e5ed1718353c95cb01103440c4'
units={}
for name in ['zee-dispatch.service','zee-dispatch.timer','zee-http.service']:
 raw=H['run'](['systemctl','--user','show',name,'--property=LoadState','--property=ActiveState','--property=ControlGroup'],env=E,timeout=10,limit=5000).decode();v=dict(x.split('=',1) for x in raw.splitlines());assert v['LoadState']=='not-found' and v['ActiveState']=='inactive' and v.get('ControlGroup','')=='';v['ControlGroup_reported']='ControlGroup' in v;units[name]=v
proof={'status':'passed','source_sha':'66e4149e53451111d153f47283c219079a8d1cab','source_end_clean':True,'owned_units':units,'unknown_inflight_latch_sha256':h,'latch_retained':True,'sql_replayed':False,'guest_free_bytes':shutil.disk_usage(B).free,'debug_binary_sha256':H['digest'](R/'compiler/target/debug/dil'),'release_binary_sha256':H['digest'](B/'linux-delivery-unzip-path/binaries/dil'),'PID_adoption':False,'foreign_unit_reset':False}
print(json.dumps(proof))
"""
raw=cmd(['/opt/homebrew/bin/limactl','shell','--workdir=/tmp','k276','--','python3','-I','-B','-c',code],'guest-end',60);guest=json.loads(raw);assert guest['status']=='passed';(O/'GUEST-END.json').write_text(json.dumps(guest,indent=2)+'\n')
ownership();start=time.monotonic();cmd(['/opt/homebrew/bin/limactl','stop','k276'],'stop',90)
after=json.loads(cmd(['/opt/homebrew/bin/limactl','list','k276','--json'],'after'));assert after['status']=='Stopped';ownership();H['clean_exact'](R,S)
proof={'status':'passed','source_sha':S,'state':'Stopped','issued_LIMA_HOME':str(home),'issued_name':'k276','instance_device_inode':[16777232,422274626],'disk_device_inode_size':[16777232,422274644,17179869184],'disk_and_latch_retained':True,'guest_end_sha256':H['digest'](O/'GUEST-END.json'),'original_owned_guest_campaigns_closed':True,'normal_namespace_shutdown':True,'PID_adoption':False,'stop_seconds':time.monotonic()-start,'commands':{p.name:H['digest'](p) for p in O.glob('*.log')}}
(O/'VM-STOP.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof))
