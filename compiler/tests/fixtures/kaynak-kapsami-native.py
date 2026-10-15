"""K284 actual owned HTTP/source coverage; fixed fixture, no arbitrary suite/exec."""
import argparse,hashlib,importlib.util,json,os,selectors,signal,socket,stat,subprocess,sys,tempfile,time
from pathlib import Path
from types import SimpleNamespace
from urllib.parse import urlencode
END=time.monotonic()+150;CANCEL=False
class Rejected(Exception):pass
def need(v,reason):
 if CANCEL or time.monotonic()>=END:raise Rejected('cancelled_or_deadline')
 if not v:raise Rejected(reason)
def cancel(*_):
 global CANCEL
 CANCEL=True
def digest(path,limit=1024*1024):
 p=Path(path);before=p.lstat();need(stat.S_ISREG(before.st_mode) and before.st_size<=limit,'source_role')
 fd=os.open(p,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK);h=hashlib.sha256();total=0
 try:
  opened=os.fstat(fd);need((opened.st_dev,opened.st_ino,opened.st_size)==(before.st_dev,before.st_ino,before.st_size),'source_identity')
  while True:
   need(True,'hash_deadline');b=os.read(fd,65536)
   if not b:break
   total+=len(b);need(total<=limit,'source_bound');h.update(b)
  after=os.fstat(fd);need((after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns)==(before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns),'source_stable')
 finally:os.close(fd)
 return h.hexdigest()
def exchange(port,path,method='GET',cookie=None,form=None):
 deadline=min(END,time.monotonic()+3);body=urlencode(form or {}).encode();head=f'{method} {path} HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n'
 if cookie:head+='Cookie: __Host-zee-oturum='+cookie+'\r\n'
 if method=='POST':head+='Origin: https://fixture.example\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: '+str(len(body))+'\r\n'
 with socket.create_connection(('127.0.0.1',port),timeout=max(.001,deadline-time.monotonic())) as s:
  need(time.monotonic()<deadline,'http_deadline');s.settimeout(deadline-time.monotonic());s.sendall(head.encode()+b'\r\n'+body);raw=bytearray()
  while True:
   need(time.monotonic()<deadline,'http_deadline');s.settimeout(deadline-time.monotonic());b=s.recv(4096)
   if not b:break
   need(len(raw)+len(b)<=16384,'http_bound');raw.extend(b)
 need(time.monotonic()<deadline,'http_deadline');head,body=bytes(raw).split(b'\r\n\r\n',1);lines=head.decode().split('\r\n');status=int(lines[0].split()[1]);headers={}
 for line in lines[1:]:
  k,v=line.split(':',1);headers.setdefault(k.lower(),[]).append(v.strip())
 need(len(headers.get('content-length',[]))==1 and int(headers['content-length'][0])==len(body),'http_frame')
 cookie=None
 for value in headers.get('set-cookie',[]):
  if value.startswith('__Host-zee-oturum='):cookie=value.split('=',1)[1].split(';',1)[0]
 return status,body,cookie,json.loads(headers['x-zee-build'][0])
def main():
 for sig in (signal.SIGINT,signal.SIGTERM):signal.signal(sig,cancel)
 a=argparse.ArgumentParser(allow_abbrev=False);a.add_argument('--framework-root',type=Path,required=True);a.add_argument('--dil',type=Path,required=True);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 root=args.framework_root.resolve(strict=True);binary=args.dil.resolve(strict=True);out=args.output
 need(os.name=='posix' and os.geteuid()!=0 and hasattr(os,'WNOWAIT'),'platform')
 need(out.is_absolute() and out.parent.resolve()==out.parent and out.parent.stat().st_uid==os.geteuid() and not out.exists(),'fresh_owned_output');out.mkdir(mode=0o700)
 owner_path=root/'scripts/worker-supervisor.py';spec=importlib.util.spec_from_file_location('coverage_owner',owner_path);M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
 proof={'schema':'zee-source-coverage-native-1','status':'running','cases':[],'candidate_only':True};owners=[];failure=None
 fixture=root/'dogfood/zee-cekirdek/test-kapsami';names=('ana.dil','karar.dil','proje.dil','rota_a.dil','rota_b.dil')
 source_files=[owner_path,Path(__file__).resolve(),*[fixture/n for n in names],*[root/'compiler/src'/n for n in ('main.rs','lib.rs','cli/kapsam.rs','test_kapsami.rs','test_kapsami/envanter.rs','yorumlayici/cumle.rs','yorumlayici/ifade.rs','yorumlayici/web_istek.rs','yorumlayici/hir_gecisi.rs')]]
 pins={str(p.relative_to(root)):digest(p) for p in source_files};binary_pin=digest(binary,512*1024*1024);proof.update(sources=pins,binary_sha256=binary_pin)
 def owner(project):
  o=M.Supervisor(SimpleNamespace(project=project,stop_grace=2),None);owners.append(o);return o
 def command(o,argv,limit=65536):
  child=o.spawn([str(x) for x in argv],capture=True);sel=selectors.DefaultSelector();sel.register(child.stdout,selectors.EVENT_READ);data=bytearray();deadline=min(END,time.monotonic()+15)
  try:
   while sel.get_map():
    need(time.monotonic()<deadline,'command_deadline')
    for key,_ in sel.select(.05):
     b=os.read(key.fd,4096)
     if not b:sel.unregister(key.fileobj)
     else:need(len(data)+len(b)<=limit,'command_bound');data.extend(b)
   status=o.exit_status(child)
   while status is None:need(time.monotonic()<deadline,'command_terminal');time.sleep(.01);status=o.exit_status(child)
   need(status==0,'command_exit');return bytes(data)
  finally:sel.close();o.stop_child(child)
 def report(path):
  need(path.stat().st_size<=2*1024*1024,'report_bound');r=json.loads(path.read_bytes());need(r['complete'] is True and r['execution_success'] is True and r['schema']=='zee-source-coverage-1','report_complete');need(r['compiler_build']==proof['build'],'report_build')
  expected={str(project/n):pins[str((fixture/n).relative_to(root))] for n in ('ana.dil','karar.dil','rota_a.dil','rota_b.dil')};need(r['sources']==expected,'report_same_source_snapshots');need(r['product']['payda']==14,'source_denominator');need(len(r['edges'])==14,'all_edges');need(not any('0x' in str(e['id']) for e in r['edges']),'no_pointer_identity');return r
 try:
  project=out/'project';project.mkdir(mode=0o700)
  for n in names:
   data=(fixture/n).read_bytes();need(len(data)<=65536 and hashlib.sha256(data).hexdigest()==pins[str((fixture/n).relative_to(root))],'fixture_snapshot');(project/n).write_bytes(data)
  o=owner(project);proof['build']=json.loads(command(o,[binary,'sürüm','--json'],4096));command(o,[binary,'kilitle',project]);command(o,[binary,'denetle',project]);command(o,[binary,'dene','--kapsam',out/'dene.json',project]);dene=report(out/'dene.json');need(dene['product']['vurulan']==4,'test_reaches_four_product_edges');proof['cases'].append({'name':'actual_dene_product_missing_denominator','passed':True,'denominator':14,'hit':4,'missing':10})
  with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
  child=o.spawn([str(binary),'çalıştır','--kapsam',str(out/'native.json'),'--web-proxy','https://fixture.example','--web-worker-port',str(port),str(project)])
  until=min(END,time.monotonic()+10)
  while True:
   need(o.exit_status(child) is None and time.monotonic()<until,'native_start')
   try:
    status,_,_,build=exchange(port,'/hazir');need(status==200 and build==proof['build'],'native_build');break
   except (ConnectionError,TimeoutError,OSError):time.sleep(.02)
  statuses=[]
  def req(path,method='GET',cookie=None,form=None):
   v=exchange(port,path,method,cookie,form);need(v[3]==proof['build'],'response_identity');statuses.append(v[0]);return v
  need(req('/imported-a')[:2]==(200,b'a'),'imported_route_a')
  need(req('/imported-b')[:2]==(200,b'b'),'imported_route_b')
  need(req('/panel')[0]==401,'anonymous_panel')
  status,csrf,guest,_=req('/csrf');need(status==200 and len(csrf)==64 and guest,'real_csrf')
  need(req('/giris','POST',guest,{'jeton':'owned-fixture'})[0]==403,'missing_csrf')
  need(req('/giris','POST',guest,{'jeton':'wrong','_csrf':csrf.decode()})[0]==403,'source_false_branch')
  status,_,session,_=req('/giris','POST',guest,{'jeton':'owned-fixture','_csrf':csrf.decode()});need(status==200 and session and session!=guest,'real_login_rotation')
  need(req('/panel',cookie=session)[:2]==(200,b'panel'),'actual_helper_panel')
  status,csrf,_,_=req('/csrf',cookie=session);need(status==200,'authenticated_csrf')
  need(req('/cikis','POST',session,{'_csrf':csrf.decode()})[0]==200,'actual_logout')
  need(req('/panel',cookie=session)[0]==401,'actual_revocation')
  child.send_signal(signal.SIGTERM);until=min(END,time.monotonic()+5)
  while o.exit_status(child) is None:need(time.monotonic()<until,'native_term');time.sleep(.01)
  need(o.exit_status(child)==0,'native_exit');o.stop_child(child);native=report(out/'native.json');need(native['product']['vurulan']==6,'native_four_edges')
  missing=native['product']['eksik'];need(len(missing)==8 and all(e['vurus']==0 for e in native['edges'] if e['sahip']=='henuzSinanmayan'),'uncalled_procedure_stays_missing')
  login_line=next(i for i,l in enumerate((fixture/'ana.dil').read_text().splitlines(),1) if l.strip().startswith('jeton "owned-fixture"'))
  login=[e for e in native['edges'] if e['kaynak'].endswith('/ana.dil') and e['satir']==login_line];need(len(login)==2 and all(e['vurus']==1 for e in login),'actual_true_false_once_no_pre_guard_credit')
  for name,outcome in [('rota_a.dil',True),('rota_b.dil',False)]:
   edges=[e for e in native['edges'] if e['kaynak'].endswith('/'+name)]
   need(len(edges)==2 and all(e['satir']==3 for e in edges) and sum(e['vurus'] for e in edges)==1 and next(e['vurus'] for e in edges if e['sonuc'] is outcome)==1,'imported_origin_same_line_exact_hits')
  proof['cases'].append({'name':'actual_native_http_session_csrf_source_branches','passed':True,'requests':len(statuses),'statuses':statuses,'denominator':14,'hit':6,'missing':8,'login_true_false_hits':[1,1],'imported_same_line_origins':['rota_a.dil','rota_b.dil'],'no_tokens_or_bodies_recorded':True})
  # An existing FIFO is rejected before opening, rather than blocked by a writer.
  fifo=out/'fifo';os.mkfifo(fifo);start=time.monotonic();bad=o.spawn([str(binary),'dene','--kapsam',str(fifo),str(project)],capture=True)
  until=min(END,time.monotonic()+3)
  while o.exit_status(bad) is None:need(time.monotonic()<until,'fifo_no_block');time.sleep(.01)
  need(o.exit_status(bad)==2,'fifo_rejected');o.stop_child(bad)
  guard_names=['fifo_no_open']
  for name,argv in [('duplicate',['dene','--kapsam',str(out/'duplicate-1.json'),'--kapsam',str(out/'duplicate-2.json'),str(project)]),('symlink',['dene','--kapsam',str(out/'report-link'),str(project)])]:
   if name=='symlink':(out/'report-link').symlink_to(out/'dene.json')
   bad=o.spawn([str(binary),*argv],capture=True);until=min(END,time.monotonic()+3)
   while o.exit_status(bad) is None:need(time.monotonic()<until,'guard_terminal');time.sleep(.01)
   need(o.exit_status(bad)==2,'guard_rejected');o.stop_child(bad);guard_names.append(name)
  # Auto config must find the actual project, skipping the new value-bearing option.
  (project/'zee.ayar').write_text(json.dumps({'sema':'zee-ayar-1','surum':1,'ayarlar':{},'guvenlik':{'UNKNOWN_PRIVATE_CANARY':True}}))
  for order in range(2):
   report_path=out/('config-rejected-'+str(order)+'.json')
   argv=['çalıştır','--kapsam',str(report_path),str(project)] if order==0 else ['çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(port),'--kapsam',str(report_path),str(project)]
   bad=o.spawn([str(binary),*argv],capture=True);until=min(END,time.monotonic()+3)
   while o.exit_status(bad) is None:need(time.monotonic()<until,'config_terminal');time.sleep(.01)
   need(o.exit_status(bad)==2 and not report_path.exists(),'config_precedes_report');o.stop_child(bad)
  (project/'zee.ayar').unlink();guard_names.append('auto_config_both_orders')
  # Quiet control aliases reject before diagnostics and before opening coverage output.
  for fd_role in ('--web-ready-fd','--web-metrik-fd'):
   for stdio in ('stdout','stderr'):
    proof['current_guard']=fd_role+'-'+stdio
    read_fd,write_fd=os.pipe();os.set_blocking(read_fd,False);path=out/(fd_role[2:]+'-'+stdio+'.json')
    try:
     child=subprocess.Popen([str(binary),'dene','--kapsam',str(path),fd_role,str(write_fd),str(project)],cwd=project,env=M.safe_environment(),stdin=subprocess.DEVNULL,stdout=write_fd if stdio=='stdout' else subprocess.DEVNULL,stderr=write_fd if stdio=='stderr' else subprocess.DEVNULL,pass_fds=(write_fd,),start_new_session=True,close_fds=True);o.children.append(child);os.close(write_fd);write_fd=-1
     until=min(END,time.monotonic()+3)
     while o.exit_status(child) is None:need(time.monotonic()<until,'quiet_terminal');time.sleep(.01)
     need(o.exit_status(child)==2 and not path.exists(),'quiet_early_exit');o.stop_child(child);control=os.read(read_fd,4096);proof['quiet_control_bytes']=len(control);proof['quiet_control_sha256']=hashlib.sha256(control).hexdigest();need(control==b'','quiet_no_bytes')
    finally:
     if write_fd>=0:os.close(write_fd)
     os.close(read_fd)
  guard_names.append('ready_metric_stdio_aliases')
  # Nonzero application exit and failed tests must never look successful.
  for name,command_name,text,expected,complete in [('exit7','çalıştır','programı 7 ile bitir\n',7,True),('failed-test','dene','test "fails"\n    1 2 ye eşit olmalı\n',1,True),('parse-failure','dene','işlem\n',1,False)]:
   file=out/(name+'.dil');file.write_text(text);report_path=out/(name+'.json');bad=o.spawn([str(binary),command_name,'--kapsam',str(report_path),str(file)],capture=True);until=min(END,time.monotonic()+3)
   while o.exit_status(bad) is None:need(time.monotonic()<until,'failed_execution_terminal');time.sleep(.01)
   need(o.exit_status(bad)==expected,'actual_failed_exit');o.stop_child(bad);failed=json.loads(report_path.read_bytes());need(failed['complete'] is complete and (not complete or failed['execution_success'] is False),'failed_execution_not_green')
  guard_names.append('nonzero_failed_test_parse_not_green')

  proof['cases'].append({'name':'actual_report_ownership_config_order_and_quiet_guards','passed':True,'guards':guard_names,'elapsed_seconds':round(time.monotonic()-start,3)})
  proof['binary_sha256_end']=digest(binary,512*1024*1024);proof['sources_end']={n:digest(root/n) for n in pins};need(proof['binary_sha256_end']==binary_pin and proof['sources_end']==pins,'end_pins');proof['status']='passed-short-candidate'
 except BaseException as e:
  failure=type(e).__name__;proof.update(status='failed-preserved',failure_type=failure)
  if type(e) is Rejected:proof['reason']=str(e)
 finally:
  closures=[]
  for o in reversed(owners):
   try:o.cleanup();closures.append(not o.children)
   except BaseException:closures.append(False)
  proof['owned_groups_closed']=all(closures);proof['cleanup_groups']=closures
  if not all(closures):proof['status']='failed-preserved'
  (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
 return 0 if proof['status']=='passed-short-candidate' else 2
if __name__=='__main__':raise SystemExit(main())
