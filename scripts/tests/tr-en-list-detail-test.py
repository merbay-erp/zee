#!/usr/bin/env python3
"""K272 fixed complete TR/EN HTTP golden; no product adoption assertion."""
import argparse
import hashlib
from html.parser import HTMLParser
import importlib.util
import json
import os
from pathlib import Path
import shutil
import selectors
import signal
import time
import socket
import stat
import subprocess
import sys
import tempfile
from types import SimpleNamespace

SOURCES=('ana.dil','ceviri.dil','yerlesim.dil','rota.dil','proje.dil')
HELPERS=('native-listener-host.py','worker-supervisor.py','native-capabilities.py')
ROUTES=(('/tr/bildirimler','tr',0),('/tr/bildirimler/eposta','tr',1),('/tr/bildirimler/ozet','tr',2),
        ('/en/notifications','en',0),('/en/notifications/email','en',1),('/en/notifications/digest','en',2))
SNAPSHOTS=tuple(f'{lang}-{index}.html' for lang in ('tr','en') for index in range(3))+('xss-tr-1.html',)
XSS_TITLE='<script>alert(\'x\')</script> " autofocus onfocus="x &'
XSS_BODY='<img src=x onerror="alert(1)"> & \'body\''
LIMIT=65536
STOP=False
END=float('inf')
RUN_OWNER=None
LEGACY_SOURCES=('kaynak/ana.dil','kaynak/ceviri.dil','kaynak/sayfa.dil','proje.dil')
LEGACY_SNAPSHOTS=('en.html','tr.html','xss.html')


def require(value,label):
    if STOP or time.monotonic()>=END:raise ValueError('cancelled_or_deadline')
    if not value:raise ValueError(label)


def regular(path,limit):
    before=path.lstat();require(stat.S_ISREG(before.st_mode) and before.st_size<=limit,'input role')
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
    try:
        first=os.fstat(fd);require(stat.S_ISREG(first.st_mode) and (first.st_dev,first.st_ino,first.st_size,first.st_mtime_ns)==(before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns),'input identity')
        with os.fdopen(fd,'rb',closefd=False) as stream:data=stream.read(limit+1)
        final=os.fstat(fd);named=path.lstat();pin=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns)
        require(len(data)==first.st_size and len(data)<=limit and pin(first)==pin(final)==pin(named),'input snapshot')
        return data
    finally:os.close(fd)


def sha(data):return hashlib.sha256(data).hexdigest()


def files(root,names,limit):return {name:regular(root/name,limit) for name in names}


class Shape(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True);self.declarations=[];self.starts=[];self.ends=[];self.text=[]
    def handle_decl(self,decl):self.declarations.append(decl)
    def handle_starttag(self,tag,attrs):
        require(len(attrs)==len(dict(attrs)),'duplicate attribute')
        require(not any(name.startswith('on') for name,_ in attrs),'injected event attribute')
        self.starts.append((tag,dict(attrs)))
    def handle_endtag(self,tag):self.ends.append(tag)
    def handle_data(self,data):self.text.append(data)


def shape(body,lang,index,xss=False):
    parser=Shape();parser.feed(body.decode('utf-8'));parser.close()
    tags=[tag for tag,_ in parser.starts]
    require(parser.declarations==['doctype html'],'complete doctype')
    require(tags==['html','head','meta','meta','title','body','header','h1','nav','a','a','main','section','h2','ul','li','a','li','a','article','h2','p','footer','p'],'fixed full document tree')
    require(parser.starts[0][1]=={'lang':lang} and parser.starts[2][1]=={'charset':'utf-8'},'language charset')
    require(parser.ends[-2:]==['body','html'],'complete document ends')
    section=parser.starts[12][1];article=parser.starts[19][1]
    require(section==({'id':'liste','hidden':None} if index else {'id':'liste'}),'list visibility')
    require(article==({'id':'detay'} if index else {'id':'detay','hidden':None}),'detail visibility')
    links=[attrs['href'] for tag,attrs in parser.starts if tag=='a']
    require(links==['/tr/bildirimler','/en/notifications']+([r[0] for r in ROUTES if r[1]==lang and r[2]>0]),'fixed root-relative links')
    if xss:
        require(XSS_TITLE in parser.text and XSS_BODY in parser.text,'plain slot character data retained')
        require(parser.starts[7][1]=={'title':XSS_TITLE} and parser.starts[20][1]=={'title':XSS_TITLE},'quote remains inside title attribute')
    return {'tags':len(tags),'doctype':True,'charset':True,'fixed_links':True,'plain_xss':xss}


def invoke(binary,args,cwd,timeout=15):
    deadline=min(END,time.monotonic()+timeout)
    child=subprocess.Popen([str(binary),*map(str,args)],cwd=cwd,env=RUN_OWNER_ENV(),stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,close_fds=True,start_new_session=True)
    # Handlers only set STOP, so Popen cannot be interrupted before registration.
    RUN_OWNER.children.append(child);data={'stdout':bytearray(),'stderr':bytearray()};selector=None
    try:
        selector=selectors.DefaultSelector()
        selector.register(child.stdout,selectors.EVENT_READ,'stdout');selector.register(child.stderr,selectors.EVENT_READ,'stderr')
        while selector.get_map():
            require(time.monotonic()<deadline,'native deadline')
            for key,_ in selector.select(min(.05,max(0,deadline-time.monotonic()))):
                block=os.read(key.fileobj.fileno(),4096);require(time.monotonic()<deadline,'native deadline')
                if not block:selector.unregister(key.fileobj)
                else:
                    target=data[key.data];require(len(target)+len(block)<=LIMIT,'native streaming capture bound');target.extend(block)
        status=RUN_OWNER.exit_status(child)
        while status is None:
            require(time.monotonic()<deadline,'native terminal deadline');time.sleep(.01);status=RUN_OWNER.exit_status(child)
        require(time.monotonic()<deadline,'native final deadline')
        return subprocess.CompletedProcess([],status,bytes(data['stdout']),bytes(data['stderr']))
    finally:
        if selector is not None:selector.close()
        try:RUN_OWNER.stop_child(child)
        finally:child.stdout.close();child.stderr.close()


def http(host,path):
    deadline=min(END,time.monotonic()+3)
    require(time.monotonic()<deadline,'HTTP deadline')
    with socket.create_connection(('127.0.0.1',host.port),timeout=deadline-time.monotonic()) as client:
        def remaining():
            require(time.monotonic()<deadline,'HTTP deadline');client.settimeout(deadline-time.monotonic())
        remaining();client.sendall(('GET '+path+' HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n').encode('ascii'))
        raw=bytearray()
        while True:
            remaining();part=client.recv(4096);require(time.monotonic()<deadline,'HTTP EOF deadline')
            if not part:break
            require(len(raw)+len(part)<=LIMIT,'HTTP response bound');raw.extend(part)
            if b'\r\n\r\n' not in raw:require(len(raw)<=16384,'HTTP header bound')
    head,body=bytes(raw).split(b'\r\n\r\n',1);require(len(head)<=16384,'HTTP header bound');lines=head.split(b'\r\n');status=int(lines[0].split(b' ')[1]);headers={}
    for line in lines[1:]:
        name,value=line.split(b':',1);name=name.lower();require(name not in headers,'duplicate HTTP header');headers[name]=value.strip()
    require(headers.get(b'connection',b'').lower()==b'close' and b'transfer-encoding' not in headers and headers.get(b'content-length')==str(len(body)).encode(),'HTTP content length')
    require(time.monotonic()<deadline,'HTTP parse deadline');return status,headers,body


def campaign(args):
    global END,RUN_OWNER,RUN_OWNER_ENV
    END=time.monotonic()+180
    require(os.name=='posix' and sys.flags.isolated and sys.dont_write_bytecode,'POSIX isolated profile')
    root=args.framework_root.resolve(strict=True);project=(args.project or root/'golden/tr-en-list-detail').resolve(strict=True)
    snapshots=(args.snapshots or root/'compiler/tests/fixtures/tr-en-list-detail').resolve(strict=True)
    output=args.output.absolute();require(not output.exists() and not output.is_symlink(),'fresh output required');output.mkdir(mode=0o700,parents=True)
    report={'schema':'zee-tr-en-list-detail-native-1','result':'running','owned_cleanup':False,'foreign_survived':False,'live_product_acceptance':False,'full_P08':False}
    try:
        source_bytes=files(project,SOURCES,LIMIT);snapshot_bytes=files(snapshots,SNAPSHOTS,16384);helpers=files(root/'scripts',HELPERS,1024*1024)
        runtime=regular(Path(__file__).resolve(),128*1024);legacy_runner=regular(root/'scripts/tests/tr-en-sayfa-test.py',128*1024)
        old_project=root/'dogfood/zee-cekirdek/tr-en';old_snapshots=root/'compiler/tests/fixtures/tr-en-sayfa'
        old_sources=files(old_project,LEGACY_SOURCES,LIMIT);old_snap=files(old_snapshots,LEGACY_SNAPSHOTS,16384)
        binary_source=args.dil.resolve(strict=True);binary_bytes=regular(binary_source,64*1024*1024);before_binary=sha(binary_bytes)
        outcomes=[];http_count=0;host_owners=[]
        report.update(binary_sha256=before_binary,runtime_sources=[{'path':'scripts/tests/tr-en-list-detail-test.py','sha256':sha(runtime)},{'path':'scripts/tests/tr-en-sayfa-test.py','sha256':sha(legacy_runner)}])
        with tempfile.TemporaryDirectory(prefix='zee-k272-native-') as temporary:
            owned=Path(temporary).resolve();binary=owned/'dil';binary.write_bytes(binary_bytes);binary.chmod(0o500)
            helper_dir=owned/'helpers';helper_dir.mkdir(mode=0o700)
            for name,data in helpers.items():(helper_dir/name).write_bytes(data)
            spec=importlib.util.spec_from_file_location('k272_owned_host',helper_dir/'native-listener-host.py');HOST=importlib.util.module_from_spec(spec);spec.loader.exec_module(HOST)
            RUN_OWNER=HOST.OWNER.Supervisor(SimpleNamespace(project=owned,dil=binary,stop_grace=2),None);RUN_OWNER_ENV=HOST.OWNER.safe_environment
            foreign_owner=HOST.OWNER.Supervisor(SimpleNamespace(project=owned,dil=binary,stop_grace=2),None)
            build_result=invoke(binary,['sürüm','--json'],owned);require(build_result.returncode==0 and not build_result.stderr,'build identity');build=json.loads(build_result.stdout)
            require(build['schema']=='zee-build-v1' and type(build['dirty']) is bool,'actual BuildJSON');report['build']=build
            foreign=foreign_owner.spawn([sys.executable,'-I','-B','-c','import time;time.sleep(180)'])
            def serve(name,changes=None):
                nonlocal http_count
                destination=owned/name;destination.mkdir(mode=0o700);data=dict(source_bytes)
                if changes:
                    text=data['yerlesim.dil'].decode();require(text.count('"E-posta bildirimleri"')==1 and text.count('"E-posta tercihlerini inceleyin."')==1,'fixed XSS fixture substitution')
                    text=text.replace('"E-posta bildirimleri"',json.dumps(XSS_TITLE,ensure_ascii=False)).replace('"E-posta tercihlerini inceleyin."',json.dumps(XSS_BODY,ensure_ascii=False));data['yerlesim.dil']=text.encode()
                for filename,raw in data.items():(destination/filename).write_bytes(raw)
                if not changes:
                    check=invoke(binary,['denetle',str(destination)],owned);require(check.returncode==0 and not check.stderr,'golden denetle');outcomes.append({'name':'denetle','passed':True})
                    tested=invoke(binary,['dene',str(destination)],owned);require(tested.returncode==0 and not tested.stderr and tested.stdout.endswith('5 test: 5 geçti, 0 kaldı\n'.encode()),'five source tests no skip');outcomes.append({'name':'dene5','passed':True})
                    formatted=invoke(binary,['biçimle','--denetle',str(destination)],owned);require(formatted.returncode==0 and not formatted.stderr,'canonical formatting')
                host=HOST.Host(binary,destination,build,'4'*40,before_binary);host_owners.append(host.owner);generation=None
                try:
                    generation=host.spawn();host.ready(generation);require(True,'native startup deadline');host.activate(generation)
                    for route,lang,index in ((ROUTES[1],) if changes else ROUTES):
                        status,headers,body=http(host,route);http_count+=1
                        require(status==200 and headers[b'content-type']==b'text/html; charset=utf-8','real HTTP HTML200');require(json.loads(headers[b'x-zee-build'])==build,'HTTP exact binary identity')
                        filename='xss-tr-1.html' if changes else f'{lang}-{index}.html';require(body==snapshot_bytes[filename],'exact full HTML snapshot')
                        outcomes.append({'name':filename,'passed':True,'status':status,'body_sha256':sha(body),'structure':shape(body,lang,index,bool(changes))})
                    if not changes:
                        for route in ('/de/bildirimler','/tr/bildirimler/kayip'):
                            status,_,body=http(host,route);http_count+=1;require(status==404,'unsupported locale/route404');outcomes.append({'name':route,'passed':True,'status':status,'body_sha256':sha(body)})
                    require(files(destination,SOURCES,LIMIT)==data,'owned project bytes unchanged')
                finally:
                    try:
                        if generation is not None:host.stop(generation)
                    finally:host.cleanup()
                require(HOST.OWNER.os.waitid(os.P_PID,foreign.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT) is None,'other owner sentinel survives')
            try:
                serve('normal');serve('xss',True);report['foreign_survived']=True
                # The preserved campaign receives the SAME private immutable binary and
                # private snapshots of its own exact helper/source/snapshot inputs.
                legacy=owned/'legacy';legacy.mkdir(mode=0o700);old_script=legacy/'tr-en-sayfa-test.py';old_script.write_bytes(legacy_runner)
                old_copy=legacy/'project';old_copy.mkdir();snap_copy=legacy/'snapshots';snap_copy.mkdir()
                for name,data in old_sources.items():
                    target=old_copy/name;target.parent.mkdir(exist_ok=True);target.write_bytes(data)
                for name,data in old_snap.items():(snap_copy/name).write_bytes(data)
                old_report=legacy/'preserved.json'
                old=invoke(sys.executable,['-I','-B',str(old_script),'--dil',str(binary),'--project',str(old_copy),'--snapshots',str(snap_copy),'--report',str(old_report)],owned,60)
                require(old.returncode==0 and not old.stderr,'existing K249 actual7+4 preserved');raw_prior=regular(old_report,LIMIT);prior=json.loads(raw_prior)
                require(prior['result']=='passed' and prior['case_count']==7 and prior['source_test_count']==4 and prior['skip_count']==0 and prior['source_unchanged'],'old exact campaign')
                require(prior['native_binary_sha256']==before_binary and prior['native_identity_start']==prior['native_identity_end']==build,'same immutable old binary identity')
                require(prior['source_files']==[{'path':name,'sha256':sha(data)} for name,data in old_sources.items()] and prior['snapshots']==[{'path':name,'sha256':sha(data)} for name,data in old_snap.items()],'old source snapshots exact')
                (output/'k249-preserved.json').write_bytes(raw_prior)
                require(json.loads(invoke(binary,['sürüm','--json'],owned).stdout)==build and sha(regular(binary,64*1024*1024))==before_binary,'post whole campaign binary')
            finally:
                try:foreign_owner.cleanup()
                finally:
                    try:RUN_OWNER.cleanup()
                    finally:report['owned_cleanup']=not foreign_owner.children and not RUN_OWNER.children and all(not owner.children for owner in host_owners)
            require(not foreign_owner.children and not RUN_OWNER.children,'registered command cleanup')
        require(not owned.exists(),'owned temporary removed')
        # These checks deliberately follow BOTH complete native campaigns.
        require(files(project,SOURCES,LIMIT)==source_bytes and files(snapshots,SNAPSHOTS,16384)==snapshot_bytes and files(root/'scripts',HELPERS,1024*1024)==helpers,'original input byte oracles')
        require(regular(Path(__file__).resolve(),128*1024)==runtime and regular(root/'scripts/tests/tr-en-sayfa-test.py',128*1024)==legacy_runner,'runtime source endpins')
        require(files(old_project,LEGACY_SOURCES,LIMIT)==old_sources and files(old_snapshots,LEGACY_SNAPSHOTS,16384)==old_snap and sha(regular(binary_source,64*1024*1024))==before_binary,'whole campaign original endpins')
        report.update(result='passed',cases=outcomes,http_requests=http_count,source_tests=5,old_fragment_native=7,old_fragment_source_tests=4,old_fragment_unchanged=True,
            source_files=[{'path':n,'sha256':sha(v)} for n,v in source_bytes.items()],snapshots=[{'path':n,'sha256':sha(v)} for n,v in snapshot_bytes.items()],
            helpers=[{'path':'scripts/'+n,'sha256':sha(v)} for n,v in helpers.items()],legacy_source_files=[{'path':'dogfood/zee-cekirdek/tr-en/'+n,'sha256':sha(v)} for n,v in old_sources.items()],
            legacy_snapshots=[{'path':'compiler/tests/fixtures/tr-en-sayfa/'+n,'sha256':sha(v)} for n,v in old_snap.items()],owned_cleanup=True,original_inputs_unchanged=True,native_platforms='actual current POSIX host only')
    except BaseException as error:
        report.update(result='failed',failure_type=type(error).__name__,reason='native_campaign_failed')
        raise
    finally:
        report['command_groups_empty']=RUN_OWNER is None or not RUN_OWNER.children
        if not report['command_groups_empty']:report.update(result='failed',reason='owned_cleanup_failed')
        (output/'proof.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'result':'passed','http_requests':http_count,'source_tests':5,'old_fragment':'7+4 unchanged'}))


class MaskedParser(argparse.ArgumentParser):
    def error(self,message):raise ValueError('arguments_rejected')


def main():
    global STOP
    STOP=False
    def cancel(number,frame):
        global STOP
        STOP=True
    for number in (signal.SIGTERM,signal.SIGINT):signal.signal(number,cancel)
    parser=MaskedParser(description=__doc__,allow_abbrev=False);parser.add_argument('--framework-root',type=Path,required=True);parser.add_argument('--dil',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--project',type=Path);parser.add_argument('--snapshots',type=Path)
    campaign(parser.parse_args())


if __name__=='__main__':
    try:main()
    except (Exception,KeyboardInterrupt):
        print('Fixed TR/EN native campaign failed.',file=sys.stderr);raise SystemExit(2)
