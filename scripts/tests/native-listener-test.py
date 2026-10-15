from pathlib import Path
import argparse,concurrent.futures,hashlib,importlib.util,json,os,selectors,signal,socket,subprocess,tempfile,time
parser=argparse.ArgumentParser(description='Actual Unix owned native listener acceptance')
parser.add_argument('--framework-root',type=Path,required=True);parser.add_argument('--dil',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
parser.add_argument('--metrics-composition',action='store_true',help='Also test merged native metrics/listener option ordering and closed reader')
ARGS=parser.parse_args();ROOT=ARGS.framework_root.resolve(strict=True);OUTPUT=ARGS.output.resolve()
if OUTPUT.exists():parser.error('--output must be fresh')
OUTPUT.mkdir(parents=True,mode=0o700)
HOST_PATH=ROOT/'scripts/native-listener-host.py';LEADER=ROOT/'scripts/tests/fixtures/native-listener/leader-native-fixture.py';REPRODUCER=ROOT/'dogfood/zee-cekirdek/socket-devri'
spec=importlib.util.spec_from_file_location('host',HOST_PATH);HOST=importlib.util.module_from_spec(spec);spec.loader.exec_module(HOST)
def require(ok,label):
 if not ok:raise AssertionError(label)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 binary=ARGS.dil.resolve(strict=True);build=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5));revision='4'*40
 proof={'schema':'zee-native-inherited-listener-stage-1','status':'running','source_acceptance':'actual given binary; clean status derives from BuildJSON, not assumed','build':build,'binary_sha256':sha(binary),'cases':[]}
 proof['reproducer_hashes']={name:sha(REPRODUCER/name) for name in ['ana.dil','proje.dil']}
 repro=subprocess.run([str(binary),'denetle',str(REPRODUCER)],capture_output=True,timeout=10);require(repro.returncode==0,'canonical B6 reproducer denetle')
 proof['integration_sources']={str(f.relative_to(ROOT)):sha(f) for f in [HOST_PATH,LEADER,ROOT/'scripts/worker-supervisor.py',ROOT/'scripts/native-capabilities.py']}
 proof['integration_sources']['scripts/tests/native-listener-test.py']=sha(Path(__file__).resolve())

 def passed(name,observation):proof['cases'].append({'name':name,'passed':True,'observations':observation});print(name+' ... passed',flush=True)
 with tempfile.TemporaryDirectory(prefix='zee-owned-listener-') as tmp:
  project=Path(tmp).resolve();payload=json.dumps({'schema':'zee-worker-readiness-1','application_revision':revision,'ready':True},separators=(',',':'))
  (project/'proje.dil').write_text('proje "native-listener-fixture" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu", "dosya-yazma", "web-oturumu" listesi olsun\n')
  (project/'ana.dil').write_text('argümanlar komut satırından gelenler olsun\nreceipt argümanların ilki olsun\nreceipt dosyasına '+json.dumps(payload)+' yaz\n8080 kapısında sunucu başlat\nGET "/hazir" adresine istek geldiğinde\n    "ok" yanıtını gönder\nGET "/slow" adresine istek geldiğinde\n    "entered" yaz\n    3 saniye bekle\n    "completed" yanıtını gönder\n')
  with (project/'ana.dil').open('a') as stream:stream.write('GET "/csrf" adresine istek geldiğinde\n    csrf belirteci yanıtını gönder\nPOST "/login" adresine istek geldiğinde\n    herkese açık\n    "owned" kullanıcısını "test" rolüyle oturuma al\n    "logged" yanıtını gönder\nGET "/panel" adresine istek geldiğinde\n    oturum gerekli\n    "panel" yanıtını gönder\nPOST "/logout" adresine istek geldiğinde\n    oturum gerekli\n    oturumu kapat\n    "revoked" yanıtını gönder\nGET "/limited" adresine istek geldiğinde\n    "limited" yanıtını gönder\n')
  check=subprocess.run([str(binary),'denetle',str(project)],capture_output=True,timeout=10);require(check.returncode==0,'fixture denetle '+check.stderr.decode())
  proof['fixture_hashes']={name:sha(project/name) for name in ['ana.dil','proje.dil']}
  host=HOST.Host(binary,project,build,revision,sha(binary));gens=[];foreign=subprocess.Popen(['python3','-I','-c','import time;time.sleep(60)'],start_new_session=True)
  def connect(path='/hazir',client='127.0.0.1'):
   s=socket.create_connection(('127.0.0.1',host.port),timeout=5);s.settimeout(5);s.sendall(('GET '+path+' HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for='+client+';proto=https;host=fixture.example\r\nConnection: close\r\n\r\n').encode());return s
  def response(s):
   data=bytearray()
   while True:
    b=s.recv(4096)
    if not b:break
    data.extend(b);require(len(data)<16384,'response bound')
   s.close();return bytes(data)
  def request():return response(connect())
  try:
   original=os.fstat(host.listener.fileno()).st_ino;g=host.spawn();gens.append(g);ready=host.ready(g)
   s=connect();s.settimeout(.2);blocked=False
   try:s.recv(1)
   except socket.timeout:blocked=True
   require(blocked,'no HTTP accept before activation');s.settimeout(5);host.activate(g);reply=response(s);require(reply.startswith(b'HTTP/1.1 200 ') and reply.endswith(b'ok'),'native valid inherited listener actual HTTP')
   passed('owned_tcp_listener_no_accept_before_exact_activation',{'pid_matched':ready['pid']==g['child'].pid,'nonce_matched':True,'build_matched':True,'fresh_generation_receipt':True,'pre_activation_response':False,'http200':True,'stable_socket_inode':os.fstat(host.listener.fileno()).st_ino==original})
   bad=host.spawn();gens.append(bad);host.ready(bad);host.activate(bad,b'ACTIVATE '+b'0'*32+b'\n');host.await_exit(bad,3);require(request().endswith(b'ok'),'old worker stays usable after bad newnonce');host.stop(bad);gens.remove(bad)
   passed('wrong_new_generation_nonce_does_not_activate_old_stays_serving',{'old_http200':True,'new_exited':True})
   for variant in ['emptyEOF','extra_bytes','duplicate_activation','oversize']:
    invalid=host.spawn();gens.append(invalid);host.ready(invalid);valid=('ACTIVATE '+invalid['nonce']+'\n').encode();message={'emptyEOF':b'','extra_bytes':valid+b'extra','duplicate_activation':valid+valid,'oversize':b'x'*128}[variant]
    host.activate(invalid,message);host.await_exit(invalid,3);host.stop(invalid);gens.remove(invalid);require(request().endswith(b'ok'),'old native worker preserved after invalid control '+variant)
    passed('actual_activation_'+variant+'_refused_without_old_generation_loss',{'new_exited':True,'old200':True,'unreaped_until_group_cleanup':True})
   # App receipt freshness and revision cannot be borrowed from another generation.
   new=host.spawn();gens.append(new);host.ready(new);old_bytes=new['receipt'].read_bytes();os.utime(new['receipt'],ns=(1,1));rejected=False
   try:host.activate(new)
   except HOST.OWNER.Reject:rejected=True
   require(rejected,'old receipt rejected');new['receipt'].write_bytes(old_bytes);host.activate(new)
   slow=connect('/slow');sel=selectors.DefaultSelector();os.set_blocking(g['child'].stdout.fileno(),False);sel.register(g['child'].stdout,selectors.EVENT_READ);deadline=time.monotonic()+5;data=bytearray()
   # Either worker may own slow request; identify actual admitted handler from bounded stdout.
   sel.register(new['child'].stdout,selectors.EVENT_READ);os.set_blocking(new['child'].stdout.fileno(),False);admitted=None
   while admitted is None:
    require(time.monotonic()<deadline,'actual slow entry')
    for key,_ in sel.select(.1):
     b=os.read(key.fd,4096);data.extend(b)
     if b'entered\n' in b:admitted=g if key.fd==g['child'].stdout.fileno() else new
   sel.close();start=time.monotonic();admitted['child'].send_signal(signal.SIGTERM);reply=response(slow);require(reply.startswith(b'HTTP/1.1 200 ') and reply.endswith(b'completed'),'issued successful accept drains response on TERM');host.await_exit(admitted,3);host.stop(admitted);gens.remove(admitted);g=new if admitted is g else g
   require(request().endswith(b'ok') and foreign.poll() is None,'remaining generation and foreign live')
   passed('ready_overlap_old_receipt_refused_admitted_stream_finishes_term',{'old_receipt_rejected':True,'completed200':True,'elapsed_seconds':round(time.monotonic()-start,3),'foreign_survived':True,'stable_socket_inode':os.fstat(host.listener.fileno()).st_ino==original})
   # Fixed-capacity requests below shared quota: every outcome counted, no retry.
   outcomes=[]
   for n in range(20):outcomes.append(request().split(b'\r\n',1)[0].decode())
   require(all(x.startswith('HTTP/1.1 200 ') for x in outcomes),'bounded legitimate load statuses')
   passed('bounded_20request_native_overlap_profile_no_retry',{'requests':20,'http200':20,'5xx':0,'refused':0,'retry':0})
   def http(path,method='GET',cookie=None,csrf=None,client='127.0.0.1'):
    body='';headers='Host: fixture.example\r\nForwarded: for='+client+';proto=https;host=fixture.example\r\nConnection: close\r\n'
    if cookie:headers+='Cookie: __Host-zee-oturum='+cookie+'\r\n'
    if csrf:
     body='_csrf='+csrf;headers+='Origin: https://fixture.example\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: '+str(len(body))+'\r\n'
    sock=socket.create_connection(('127.0.0.1',host.port),timeout=5);sock.settimeout(5);sock.sendall((method+' '+path+' HTTP/1.1\r\n'+headers+'\r\n'+body).encode());raw=response(sock);head,body=raw.split(b'\r\n\r\n',1);status=int(head.split(b' ')[1]);newcookie=None
    for line in head.split(b'\r\n'):
     if line.lower().startswith(b'set-cookie: __host-zee-oturum='):newcookie=line.split(b'=',1)[1].split(b';',1)[0].decode()
    return status,body.decode(),newcookie
   def retire(old):old['child'].send_signal(signal.SIGTERM);host.await_exit(old,3);host.stop(old);gens.remove(old)
   status,csrf,guest=http('/csrf');require(status==200 and len(csrf)==64 and guest,'actual native guest csrf')
   status,_,cookie=http('/login','POST',guest,csrf);require(status==200 and cookie,'actual native session login status'+str(status))
   require(http('/panel',cookie=cookie)[0]==200,'authenticated old generation')
   new=host.spawn();gens.append(new);host.ready(new);host.activate(new);retire(g);g=new
   require(http('/panel',cookie=cookie)[0]==200,'session shared to new generation')
   status,csrf,_=http('/csrf',cookie=cookie);require(status==200,'authenticated csrf')
   require(http('/logout','POST',cookie,csrf)[0]==200 and http('/panel',cookie=cookie)[0]==401,'native revoke old cookie')
   new=host.spawn();gens.append(new);host.ready(new);host.activate(new);require(http('/panel',cookie=cookie)[0]==401,'revoked overlap');retire(g);g=new;require(http('/panel',cookie=cookie)[0]==401,'revoked after restart')
   state=project/'.zee/web-durumu-v1.json';require(state.exists(),'actual shared JSON state');passed('actual_native_session_revoke_shared_across_generation_restart',{'login200':True,'next_generation_panel200':True,'revoked_overlap401':True,'revoked_restart401':True,'backend':'atomicJSONfile','state_mode':oct(state.stat().st_mode&0o777),'no_cookie_recorded':True})
   attempts=0
   while attempts<200:
    status,_,_=http('/limited',client='127.0.0.9');attempts+=1
    if status==429:break
    require(status==200,'quota fixture statuses')
   require(status==429,'actual bounded quota exhaustion')
   new=host.spawn();gens.append(new);host.ready(new);host.activate(new);require(http('/limited',client='127.0.0.9')[0]==429,'quota preserved overlap');retire(g);g=new;require(http('/limited',client='127.0.0.9')[0]==429,'quota preserved new generation only')
   passed('actual_shared_json_quota_exhaustion_survives_overlap_restart',{'requests_until_first429':attempts,'overlap429':True,'new_generation429':True,'quota_reset':False,'backend':'atomicJSONfile'})
   outcomes=[];waves=0;started=time.monotonic()
   for n in range(1000):
    if n and n%200==0:
     new=host.spawn();gens.append(new);host.ready(new);host.activate(new);retire(g);g=new;waves+=1
    client='127.1.'+str(n//250)+'.'+str(n%250+1);status,_,_=http('/hazir',client=client);outcomes.append(status)
   require(outcomes.count(200)==1000,'all legitimate capacity-bounded unique fixture clients received200')
   passed('actual_1000_requests_four_owned_rollovers_same_tcp_listener',{'requests':1000,'http200':1000,'5xx':sum(500<=x<=599 for x in outcomes),'refusals':0,'retries':0,'rollovers':waves,'concurrency':1,'elapsed_seconds':round(time.monotonic()-started,3),'quota_profile':'1000 distinct synthetic loopback client identities; no quota raise/reset','stable_listener_inode':os.fstat(host.listener.fileno()).st_ino==original})
   receipt_dir=host.generation_root/'leader-fixture';receipt_dir.mkdir(mode=0o700);owned=host.owner.spawn(['python3','-I','-B',str(LEADER),str(binary),str(project),str((receipt_dir/'ready.json').relative_to(project))],capture=True)
   os.set_blocking(owned.stdout.fileno(),False);marker=bytearray();deadline=time.monotonic()+5
   while b'owned-native-descendant-activated\n' not in marker:
    require(time.monotonic()<deadline,'actual native descendant activated in owned group')
    try:marker.extend(os.read(owned.stdout.fileno(),4096))
    except BlockingIOError:time.sleep(.02)
   while host.owner.exit_status(owned) is None:
    require(time.monotonic()<deadline,'owned fixture leader exits');time.sleep(.02)
   require(host.owner.exit_status(owned)==9 and owned.returncode is None,'WNOWAIT retains dead owned leader')
   # Native descendant keeps inherited stdout open after leader exit; no EOF.
   persisted=False
   try:os.read(owned.stdout.fileno(),1)
   except BlockingIOError:persisted=True
   require(persisted,'actual native descendant remains alive with inherited group pipe')
   started=time.monotonic();host.owner.stop_child(owned);require(owned not in host.owner.children and foreign.poll() is None,'owned group cleaned before wait foreign survived')
   passed('exited_owned_leader_real_native_descendant_cleanup_wnowait_foreign_survives',{'leader_exit':9,'unreaped_until_cleanup':True,'actual_native_descendant_pipe_persisted':True,'owned_group_cleaned':True,'foreign_survived':True,'cleanup_seconds':round(time.monotonic()-started,3)})
   if ARGS.metrics_composition:
    for first in [True,False]:
     combined=host.spawn(metrics=True,metric_first=first);gens.append(combined);host.ready(combined);host.activate(combined);retire(g);g=combined
     require(http('/hazir',client='127.2.0.1')[0]==200,'combined listener/metrics HTTP')
     frames=bytearray();deadline=time.monotonic()+2
     while time.monotonic()<deadline:
      try:
       chunk=os.read(combined['metric'],4096)
       if not chunk:break
       frames.extend(chunk);require(len(frames)<=65536,'bounded metric fixture read')
      except BlockingIOError:
       if frames:break
       time.sleep(.02)
     records=[HOST.OWNER.decode_json(line) for line in frames.splitlines()];require(records and all(record.get('schema')=='zee-native-metrics-1' for record in records),'actual native metric records')
     passed('combined_metric_option_'+('before' if first else 'after')+'_listener_options',{'http200':True,'native_metric_records':len(records),'source_build_matched':True,'one_shared_listener':True})
    combined=host.spawn(metrics=True,metric_first=True,closed_metric_reader=True);gens.append(combined);host.ready(combined);host.activate(combined);retire(g);g=combined
    require(http('/hazir',client='127.2.0.2')[0]==200,'closed metric reader does not invalidate listener ownership constructor')
    passed('combined_metric_epipe_before_listener_constructor_no_stale_descriptor_recheck',{'http200':True,'metric_reader_closed_before_spawn':True,'ready_build_matched':True,'no_closed_metric_fd_adoption':True})
   # Actual app argv oracle bypasses no CLI layer: valid owned FDs precede source,
   # new protocol flags after source must remain ordinary application data.
   oracle=project/'argv-oracle.dil';oracle.write_text('gelenler komut satırından gelenler olsun\ngelenlerdeki her argüman için\n    argüman yaz\n')
   app_args=['APP_FIRST','--web-generation','APP_GENERATION','--web-metrik-fd','APP_METRIC','--ayar','APP_AYAR','--profil','APP_PROFILE','APP_LAST']
   for metric_first in ([False,True] if ARGS.metrics_composition else [False]):
    rr,rw=os.pipe();ar,aw=os.pipe();mr,mw=os.pipe();os.set_blocking(mw,False);inherited=[host.listener.fileno(),rw,ar]
    argv=[str(binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(host.port),'--web-listener-fd',str(host.listener.fileno()),'--web-ready-fd',str(rw),'--web-activate-fd',str(ar),'--web-generation','a'*32]
    if ARGS.metrics_composition:
     location=2 if metric_first else len(argv);argv[location:location]=['--web-metrik-fd',str(mw)];inherited.append(mw)
    try:
     child=host.owner.spawn([*argv,str(oracle),*app_args],capture=True,passed_fds=tuple(inherited));os.set_blocking(child.stdout.fileno(),False);output=bytearray();deadline=time.monotonic()+5
     while host.owner.exit_status(child) is None:
      require(time.monotonic()<deadline,'app argv oracle bound')
      try:output.extend(os.read(child.stdout.fileno(),4096))
      except BlockingIOError:time.sleep(.01)
      require(len(output)<=4096,'app argv oracle stdout bound')
     while True:
      part=os.read(child.stdout.fileno(),4096)
      if not part:break
      output.extend(part);require(len(output)<=4096,'app argv oracle stdout bound')
     status=host.owner.exit_status(child);host.owner.stop_child(child)
     require(status==0 and output==('\n'.join(app_args)+'\n').encode(),'actual CLI appargv preserves after-source data and omits protocol values')
    finally:
     for fd in [rr,rw,ar,aw,mr,mw]:os.close(fd)
   passed('actual_cli_application_argv_oracle_protocol_prefix_removed_data_suffix_preserved',{'argv_count':len(app_args),'metric_order_variants':2 if ARGS.metrics_composition else 1,'exact_stdout_matched':True,'no_rate_baseline_claim':True})
   # A malformed project config must reject before source executes/server starts.
   badproject=project/'bad-config';badproject.mkdir();(badproject/'ana.dil').write_text('"PRIVATE_K259_SOURCE_EXEC_CANARY" yaz\n');(badproject/'proje.dil').write_text('proje "bad-config" olsun\nsürüm "0.1.0" olsun\ngiriş "ana.dil" olsun\n');(badproject/'zee.ayar').write_text('{PRIVATE_K259_CONFIG_CANARY malformed')
   rr,rw=os.pipe();ar,aw=os.pipe()
   argv=[str(binary),'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(host.port),'--web-listener-fd',str(host.listener.fileno()),'--web-ready-fd',str(rw),'--web-activate-fd',str(ar),'--web-generation','a'*32,str(badproject)]
   try:
    rejected=subprocess.run(argv,stdin=subprocess.DEVNULL,capture_output=True,pass_fds=(host.listener.fileno(),rw,ar),timeout=5);os.close(rw);rw=None;control=os.read(rr,4096)
    require(rejected.returncode==2 and rejected.stdout==b'' and control==b'' and b'PRIVATE_K259' not in rejected.stderr and b'zee.ayar:' in rejected.stderr,'malformed zee.ayar rejects before appsource/control readiness')
   finally:
    for fd in [rr,rw,ar,aw]:
     if fd is not None:os.close(fd)
   passed('malformed_project_zee_ayar_rejected_before_source_control_or_admission',{'exit':2,'stdout_bytes':0,'control_bytes':0,'private_config_or_source_canary_leaked':False,'app_source_executed':False})
   proof['metrics_composition_required']=ARGS.metrics_composition
   proof['status']='passed'
  finally:
   for g in gens:
    try:host.stop(g)
    except Exception:pass
   host.cleanup();foreign.terminate();foreign.wait(timeout=3);proof['owned_workers_closed']=True;(OUTPUT/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
if __name__=='__main__':main()
