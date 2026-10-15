#!/usr/bin/env python3
"""Owned local starter front: strict fixed routes plus immutable PNG bytes."""
import argparse
import contextlib
try:
    import fcntl
except ImportError:
    fcntl=None
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import secrets
import ssl
import signal
import socket
import stat
import sys
import tempfile
import time
import urllib.parse

PROFILE='zee-content-starter-https-posix-1'
STOPPED=False
H=None
class Rejected(ValueError):pass
class Visible(Rejected):pass
class Cancelled(Rejected):pass

def need(value):
    if STOPPED:raise Cancelled('cancelled')
    if not value:raise Rejected('rejected')

def sha(raw):return hashlib.sha256(raw).hexdigest()
def pairs(items):
    d={}
    for k,v in items:need(k not in d);d[k]=v
    return d

def decode(raw):
    need(len(raw)<=65536)
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=lambda _:(_ for _ in ()).throw(Rejected()))

def bounded(path,limit,private=False):
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd)
        need(stat.S_ISREG(before.st_mode) and before.st_nlink==1 and before.st_size<=limit)
        if private:need(before.st_uid==os.geteuid() and before.st_mode&0o077==0)
        parts=[];seen=0
        while True:
            part=os.read(fd,min(65536,limit+1-seen))
            if not part:break
            seen+=len(part);need(seen<=limit);parts.append(part)
        after=os.fstat(fd);current=Path(path).stat(follow_symlinks=False)
        key=lambda x:(x.st_dev,x.st_ino,x.st_size,x.st_mtime_ns,x.st_ctime_ns)
        need(key(before)==key(after)==key(current));return b''.join(parts)
    finally:os.close(fd)

def directory(path):
    path=Path(path);need(path.resolve(strict=True)==path)
    fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        a=os.fstat(fd);b=path.stat(follow_symlinks=False)
        need(stat.S_ISDIR(a.st_mode) and a.st_uid==os.geteuid() and a.st_mode&0o077==0 and (a.st_dev,a.st_ino)==(b.st_dev,b.st_ino))
        return a.st_dev,a.st_ino
    finally:os.close(fd)

def helper():
    global H
    p=Path(__file__).resolve().with_name('lifecycle_core.py');raw=bounded(p,16384)
    need(sha(raw)=='e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9')
    H={'__name__':'starter_owned_core','__file__':str(p)};exec(compile(raw,str(p),'exec'),H)
    need(callable(getattr(os,'waitid',None)) and hasattr(os,'WNOWAIT'))

GET={'/','/icerik','/giris','/admin'}
POST={'/giris-yap','/ekle','/guncelle','/sil','/medya-yukle','/medya-sil','/cikis'}
FORM_FIELDS={'/giris-yap':{'parola','_csrf'},'/ekle':{'id','title','body','_csrf'},'/guncelle':{'id','title','body','_csrf'},'/sil':{'id','_csrf'},'/medya-sil':{'source','png','_csrf'},'/cikis':{'_csrf'}}

def target(method,value):
    need(type(value)is str and len(value)<=512 and value.startswith('/') and '\\' not in value and '#' not in value and not any(ord(c)<33 or ord(c)>126 for c in value))
    path,sep,query=value.partition('?')
    need('%' not in path and '//' not in path)
    media=re.fullmatch(r'/media/([0-9a-f]{64})\.png',path)
    if media:
        need(method=='GET' and not sep);return path,{},media.group(1)
    need('.' not in path)
    need(path in (GET if method=='GET' else POST) and method in {'GET','POST'})
    expected={'id'} if path in {'/icerik','/medya-yukle'} else set()
    values={}
    if query:
        need(not re.search(r'%(?![0-9A-Fa-f]{2})',query))
        for k,v in urllib.parse.parse_qsl(query,keep_blank_values=True,strict_parsing=True,encoding='utf-8',errors='strict'):
            need(k not in values);values[k]=v
    need(set(values)==expected)
    if values:need(re.fullmatch('[a-z][a-z0-9_]{0,31}',values['id']))
    return path,values,None

def form(path,raw):
    need(len(raw)<=16384 and path in FORM_FIELDS)
    text=raw.decode('ascii');need(not re.search(r'%(?![0-9A-Fa-f]{2})',text))
    values={}
    for k,v in urllib.parse.parse_qsl(text,keep_blank_values=True,strict_parsing=True,encoding='utf-8',errors='strict'):
        need(k not in values and '\x00' not in v);values[k]=v
    need(set(values)==FORM_FIELDS[path])
    need(re.fullmatch('[0-9a-f]{64}',values['_csrf']))
    if 'id' in values:need(re.fullmatch('[a-z][a-z0-9_]{0,31}',values['id']))
    if 'title' in values:need(1<=len(values['title'].encode())<=512 and len(values['body'].encode())<=8192)
    if path=='/medya-sil':need(all(re.fullmatch('[0-9a-f]{64}',values[n]) for n in ['source','png']))
    if 'parola' in values:need(1<=len(values['parola'].encode())<=1024)
    return urllib.parse.urlencode(values).encode()

def remaining(deadline):
    left=deadline-time.monotonic();need(left>0);return left

def send(stream,raw,deadline):
    view=memoryview(raw)
    while view:
        stream.settimeout(min(.5,remaining(deadline)));n=stream.send(view);need(n>0);view=view[n:]
    remaining(deadline)

def recv(stream,size,deadline):
    stream.settimeout(min(.5,remaining(deadline)));raw=stream.recv(size);remaining(deadline);return raw

def request(stream,authority,deadline):
    raw=bytearray()
    while b'\r\n\r\n' not in raw:
        part=recv(stream,min(4096,16385-len(raw)),deadline);need(part);raw.extend(part);need(len(raw)<=16384)
    header,body=bytes(raw).split(b'\r\n\r\n',1);lines=header.decode('ascii').split('\r\n');need(len(lines)<=33)
    first=lines.pop(0).split(' ');need(len(first)==3 and first[2]=='HTTP/1.1' and first[0] in {'GET','POST'})
    method,url,_=first;path,query,png=target(method,url);headers={}
    for line in lines:
        need(':' in line and not line.startswith((' ','\t')))
        name,value=line.split(':',1);name=name.lower();value=value.strip()
        need(re.fullmatch('[a-z][a-z0-9-]{0,39}',name) and name not in headers and len(value)<=4096 and all(32<=ord(x)<127 for x in value))
        headers[name]=value
    need(headers.get('host')==authority and not any(x in headers for x in ['transfer-encoding','expect','forwarded','x-forwarded-host','x-forwarded-proto','upgrade']))
    count=headers.get('content-length','0');need(re.fullmatch('0|[1-9][0-9]{0,6}',count));count=int(count)
    need(count<=1048576 and (method!='GET' or count==0) and len(body)<=count)
    if method=='POST':
        need(headers.get('origin')=='https://'+authority)
        kind=headers.get('content-type')
        if path=='/medya-yukle':need(kind=='application/octet-stream' and count>0 and re.fullmatch('[0-9a-f]{64}',headers.get('x-zee-csrf','')))
        else:need(kind=='application/x-www-form-urlencoded' and count<=16384)
    while len(body)<count:
        part=recv(stream,min(16384,count-len(body)),deadline);need(part);body+=part
    if method=='POST' and path!='/medya-yukle':body=form(path,body)
    return method,url,path,png,headers,body

def png_bytes(root,fingerprint):
    directory(root/'media')
    raw=bounded(root/'media'/(fingerprint+'.png'),1048576,True)
    need(raw.startswith(b'\x89PNG\r\n\x1a\n') and sha(raw)==fingerprint)
    need(len(raw)>=33 and raw[8:16]==b'\x00\x00\x00\rIHDR' and all(1<=int.from_bytes(raw[n:n+4],'big')<=320 for n in [16,20]) and raw[24:29]==bytes([8,6,0,0,0]))
    return raw

def syncdir(path):
    fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:os.fsync(fd)
    finally:os.close(fd)

def publish_private(path,raw):
    temp=path.parent/('.starter-'+secrets.token_hex(16))
    fd=os.open(temp,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        view=memoryview(raw)
        while view:
            count=os.write(fd,view);need(count>0);view=view[count:]
        os.fsync(fd)
    finally:os.close(fd)
    try:os.replace(temp,path);syncdir(path.parent)
    finally:
        if temp.exists():temp.unlink()

@contextlib.contextmanager
def mutation_lock(root):
    need(fcntl is not None);path=root/'starter-lock'
    fd=os.open(path,os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW|os.O_NONBLOCK,0o600)
    try:
        a=os.fstat(fd);b=path.stat(follow_symlinks=False)
        need(stat.S_ISREG(a.st_mode) and a.st_nlink==1 and a.st_uid==os.geteuid() and a.st_mode&0o077==0 and (a.st_dev,a.st_ino)==(b.st_dev,b.st_ino))
        fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
        b=path.stat(follow_symlinks=False);need((a.st_dev,a.st_ino)==(b.st_dev,b.st_ino))
        yield
    finally:os.close(fd)

def begin_effect(root,url,body):
    path=root/'starter-pending.json'
    need(not path.exists() and not path.is_symlink())
    value={'schema':'zee-content-starter-pending-1','root':list(directory(root)),'token':secrets.token_hex(16),'request_sha256':sha(url.encode()+b'\n'+body),'created_ns':time.time_ns()}
    raw=(json.dumps(value,sort_keys=True)+'\n').encode();publish_private(path,raw);return raw

def complete_effect(root,issued):
    path=root/'starter-pending.json';need(bounded(path,4096,True)==issued)
    path.unlink()
    try:syncdir(root)
    except OSError:raise Visible('known-completion-visible-durability-unconfirmed') from None

def response(stream,status,body,deadline,kind='text/plain; charset=utf-8'):
    head=f'HTTP/1.1 {status}\r\nContent-Length: {len(body)}\r\nContent-Type: {kind}\r\nX-Content-Type-Options: nosniff\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n'.encode()
    send(stream,head+body,deadline)

def proxy(stream,method,url,headers,body,port,deadline):
    # Issued native child only; operator cannot configure an arbitrary upstream.
    with socket.create_connection(('127.0.0.1',port),timeout=min(.5,remaining(deadline))) as upstream:
        forwarded={'Host':headers['host'],'Connection':'close','Forwarded':'for=127.0.0.1;proto=https;host="'+headers['host']+'"'}
        for name in ['cookie','content-type','origin','x-zee-csrf']:
            if name in headers:forwarded[name]=headers[name]
        forwarded['Content-Length']=str(len(body))
        wire=(method+' '+url+' HTTP/1.1\r\n'+''.join(k+': '+v+'\r\n' for k,v in forwarded.items())+'\r\n').encode()+body
        send(upstream,wire,deadline);out=bytearray()
        while True:
            part=recv(upstream,16384,deadline)
            if not part:break
            out.extend(part);need(len(out)<=1048576)
        head,content=bytes(out).split(b'\r\n\r\n',1);need(len(head)<=16384 and head.startswith(b'HTTP/1.1 '))
        lengths=[]
        for line in head.split(b'\r\n')[1:]:
            name,sep,value=line.partition(b':');need(sep)
            need(name.lower() not in {b'transfer-encoding',b'upgrade'})
            if name.lower()==b'content-length':lengths.append(int(value.strip()))
        need(lengths==[len(content)])
        return bytes(out)

def tls_material(cert,key):
    certificate=bounded(cert,16384,True);secret=bounded(key,16384,True)
    need(certificate.startswith(b'-----BEGIN CERTIFICATE-----') and secret.startswith((b'-----BEGIN PRIVATE KEY-----',b'-----BEGIN RSA PRIVATE KEY-----')) and b'ENCRYPTED' not in secret[:128])
    return certificate,secret

def run(root,binary,expected,binary_sha,front,cert,key,candidate=False):
    helper();need(type(front)is int and 1024<=front<=65535)
    identity=directory(root);directory(root/'media');directory(root/'yonetim')
    config=decode(bounded(root/'starter-host.json',65536,True))
    need(type(config)is dict and set(config)=={'schema','root','files','runtime'})
    need(config['schema']=='zee-content-starter-1' and config['root']==list(identity))
    files={'ana.dil','alan.dil','proje.dil','proje.kilit','gocler/0001_content.sql'}
    need(type(config['files'])is dict and set(config['files'])==files)
    def pins():
        need(directory(root)==identity)
        for name,h in config['files'].items():need(re.fullmatch('[0-9a-f]{64}',h) and sha(bounded(root/name,262144))==h)
        for name,h in config['runtime'].items():need(name in {'starter-content-host.py','starter-content-reconcile.py','lifecycle_core.py'} and sha(bounded(root/'yonetim'/name,65536))==h)
    need(set(config['runtime'])=={'starter-content-host.py','starter-content-reconcile.py','lifecycle_core.py'});pins()
    phc=bounded(root/'panel-phc',1024,True);need(re.fullmatch(rb'\$argon2id\$v=19\$m=[1-9][0-9]{0,7},t=[1-9][0-9]{0,3},p=[1-9][0-9]{0,3}\$[A-Za-z0-9+/]{11,86}\$[A-Za-z0-9+/]{22,86}',phc))
    need(re.fullmatch('[0-9a-f]{40}',expected) and re.fullmatch('[0-9a-f]{64}',binary_sha))
    raw=bounded(binary,134217728);need(sha(raw)==binary_sha)
    child=None
    with tempfile.TemporaryDirectory(prefix='zee-starter-native-') as td:
        p=Path(td);os.chmod(p,0o700);executable=p/'dil';executable.write_bytes(raw);executable.chmod(0o700)
        build=decode(H['run']([executable,'sürüm','--json'],timeout=10,limit=8192))
        need(build['schema']=='zee-build-v1' and build['source_sha']==expected)
        if not candidate:need(build['dirty'] is False)
        env={k:v for k,v in os.environ.items() if not k.startswith(('GIT_','ZEE_'))}
        for env_name in ['ZEE_STARTER_DATABASE_URL','ZEE_STARTER_DATABASE_URL_TLS_CA_PEM']:
            if env_name in os.environ:env[env_name]=os.environ[env_name]
        tls_cert,tls_key=tls_material(cert,key)
        cert_copy=p/'certificate.pem';key_copy=p/'key.pem';cert_copy.write_bytes(tls_cert);key_copy.write_bytes(tls_key);key_copy.chmod(0o600)
        context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);context.minimum_version=ssl.TLSVersion.TLSv1_2
        context.load_cert_chain(cert_copy,key_copy)
        owned=socket.socket(socket.AF_INET,socket.SOCK_STREAM,socket.IPPROTO_TCP)
        owned.bind(('127.0.0.1',0));owned.listen(8);owned.setblocking(False);worker=owned.getsockname()[1]
        ready_r,ready_w=os.pipe();activate_r,activate_w=os.pipe();nonce=secrets.token_hex(16)
        try:
            argv=[executable,'çalıştır','--web-proxy',f'https://127.0.0.1:{front}','--web-worker-port',str(worker),'--web-listener-fd',str(owned.fileno()),'--web-ready-fd',str(ready_w),'--web-activate-fd',str(activate_r),'--web-generation',nonce,root]
            try:
                child=H['subprocess'].Popen(argv,stdin=H['subprocess'].DEVNULL,stdout=H['subprocess'].DEVNULL,stderr=H['subprocess'].DEVNULL,start_new_session=True,close_fds=True,pass_fds=(owned.fileno(),ready_w,activate_r),env=env)
            finally:
                os.close(ready_w);os.close(activate_r)
            startup=time.monotonic()+10;data=bytearray();os.set_blocking(ready_r,False)
            with selectors.DefaultSelector() as selector:
                selector.register(ready_r,selectors.EVENT_READ)
                while True:
                    remaining(startup)
                    if not selector.select(min(.1,remaining(startup))):continue
                    part=os.read(ready_r,4097);data.extend(part);need(len(data)<=4096)
                    if not part:break
            value=decode(data)
            need(type(value)is dict and set(value)=={'schema','pid','generation','port','build'} and value=={'schema':'zee-native-listener-ready-1','pid':child.pid,'generation':nonce,'port':worker,'build':build})
            need(H['held_status'](child)is None);os.set_blocking(activate_w,False)
            frame=memoryview(('ACTIVATE '+nonce+'\n').encode())
            while frame:
                remaining(startup)
                try:count=os.write(activate_w,frame);need(count>0);frame=frame[count:]
                except (BlockingIOError,InterruptedError):time.sleep(min(.001,remaining(startup)))
            os.close(activate_w);activate_w=None
            with socket.socket() as listener:
                listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1);listener.bind(('127.0.0.1',front));listener.listen(8);listener.settimeout(.2)
                while not STOPPED:
                    need(H['held_status'](child)is None);pins()
                    try:stream,_=listener.accept()
                    except socket.timeout:continue
                    deadline=time.monotonic()+10
                    try:
                        stream.settimeout(min(.5,remaining(deadline)))
                        stream=context.wrap_socket(stream,server_side=True,do_handshake_on_connect=False)
                        stream.settimeout(min(2,remaining(deadline)));stream.do_handshake();remaining(deadline)
                    except (OSError,ValueError):stream.close();continue
                    with stream:
                        try:
                            method,url,path,png,headers,body=request(stream,f'127.0.0.1:{front}',deadline)
                            if png:response(stream,'200 OK',png_bytes(root,png),deadline,'image/png')
                            else:
                                unsafe=method=='POST' and path in {'/ekle','/guncelle','/sil','/medya-yukle','/medya-sil'}
                                with mutation_lock(root) if unsafe else contextlib.nullcontext():
                                    issued=begin_effect(root,url,body) if unsafe else None
                                    result=proxy(stream,method,url,headers,body,worker,deadline)
                                    status=int(result.split(b' ',2)[1])
                                    if issued is not None and (200<=status<400 or status in {401,403}):complete_effect(root,issued)
                                    send(stream,result,deadline)
                        except Visible:raise
                        except (OSError,ValueError,TypeError,KeyError):
                            try:response(stream,'400 Bad Request',b'starter-request-rejected',deadline)
                            except (OSError,ValueError):pass
        finally:
            try:
                if child is not None:H['stop_owned'](child)
            finally:
                for fd in [ready_r,activate_w]:
                    if fd is not None:os.close(fd)
                owned.close()
            if not STOPPED:pins();need(sha(bounded(binary,134217728))==binary_sha)

def main():
    try:
        class Parser(argparse.ArgumentParser):
            def error(self,message):raise Rejected('arguments')
        p=Parser(allow_abbrev=False)
        for name in ['root','dil','binary-sha256','expected-source']:p.add_argument('--'+name,required=True)
        p.add_argument('--front-port',type=int,required=True);p.add_argument('--tls-cert',type=Path,required=True);p.add_argument('--tls-key',type=Path,required=True);p.add_argument('--candidate',action='store_true')
        a=p.parse_args();root=Path(a.root);binary=Path(a.dil);need(root.resolve(strict=True)==root and binary.resolve(strict=True)==binary)
        def cancel(*_):
            global STOPPED
            STOPPED=True
            if H:H['CANCELLED']=True
        signal.signal(signal.SIGTERM,cancel);signal.signal(signal.SIGINT,cancel)
        os.umask(0o077)
        run(root,binary,a.expected_source,a.binary_sha256,a.front_port,a.tls_cert,a.tls_key,a.candidate);return 0
    except Cancelled:return 0
    except Visible:
        sys.stderr.write('starter-completion-visible-durability-unconfirmed\n');return 4
    except BaseException:
        sys.stderr.write('starter-host-rejected\n');return 2
if __name__=='__main__':sys.exit(main())
