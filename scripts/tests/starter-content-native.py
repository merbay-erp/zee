#!/usr/bin/env python3
"""Owned fresh PG + HTTPS starter workflow; explicit historical candidate mode."""
import argparse
import contextlib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import socket
import ssl
import struct
import sys
import tempfile
import time
import urllib.parse
from unittest.mock import patch
import zlib
p=argparse.ArgumentParser(allow_abbrev=False)
for n in ['framework-root','dil','postgres-bin-dir','output']:p.add_argument('--'+n,type=Path,required=True)
for n in ['sha','binary-sha256']:p.add_argument('--'+n,required=True)
p.add_argument('--candidate',action='store_true');a=p.parse_args();R=a.framework_root
assert R.resolve(strict=True)==R and a.output.parent.resolve(strict=True)==a.output.parent

def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
M=load('starter',R/'scripts/starter-content-host.py');M.helper();H=M.H
M.directory(a.output.parent);a.output.mkdir(mode=0o700)
roles=['scripts/starter-content-host.py','scripts/starter-content-reconcile.py','scripts/icerik-paneli-olustur.py','scripts/tests/starter-content-test.py','scripts/tests/starter-content-native.py','scripts/lifecycle_core.py','scripts/postgresql-tls-ci.py','scripts/postgresql-tls11-control.py','sablonlar/icerik-paneli/ana.dil','sablonlar/icerik-paneli/alan.dil','sablonlar/icerik-paneli/proje.dil','sablonlar/icerik-paneli/gocler/0001_content.sql','sablonlar/icerik-paneli/.gitignore','docs/versioned/0.8/icerik-paneli.md']
pins={n:M.sha(M.bounded(R/n,262144)) for n in roles}
TLS=load('owned_pg',R/'scripts/postgresql-tls-ci.py')
start=time.monotonic();deadline=start+600;cleaning=False;original_run=H['run'];raw=[];cleanup=[];host=None;private=None;command_counter=0
proof={'schema':'zee-content-starter-native-tests-1','status':'running','mode':'candidate' if a.candidate else 'clean-exact','cases':[],'source_files':[{'path':n,'sha256':v} for n,v in pins.items()]}
def bounded_run(argv,**kwargs):
    left=deadline-time.monotonic();assert cleaning or left>0
    requested=kwargs.pop('timeout',30)
    return original_run(argv,timeout=requested if cleaning else min(requested,left),**kwargs)
H['run']=bounded_run

def command(argv,**kwargs):
    global command_counter
    command_counter+=1
    sensitive_output=kwargs.pop('sensitive_output',False)
    if not sensitive_output:kwargs['log_path']=a.output/f'command-{command_counter:03d}.log'
    out=H['run'](argv,timeout=kwargs.pop('timeout',30),limit=1048576,**kwargs)
    # Never retain PHC/credential command output or request/cookie data.
    if not sensitive_output:raw.append({'tool':Path(str(argv[0])).name,'stdout_sha256':M.sha(out),'bytes':len(out)})
    return out
TLS.run=lambda argv,**kw:command(argv,**kw).decode()
def record(name):
    proof['cases'].append({'name':name,'passed':True});(a.output/'progress.json').write_text(json.dumps({'status':'running','cases':proof['cases']},indent=2)+'\n')
def pg_pin():return {n:M.sha(M.bounded(a.postgres_bin_dir/n,134217728)) for n in ['postgres','initdb','pg_ctl','psql']}
pgpins=None

def image():
    def chunk(name,data):return struct.pack('!I',len(data))+name+data+struct.pack('!I',zlib.crc32(name+data)&0xffffffff)
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('!IIBBBBB',2,1,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\0\xff\0\0\xff\0\xff\0\xff'))+chunk(b'IEND',b'')

def main():
    global host,private,cleaning,pgpins
    old={sig:signal.signal(sig,lambda *_:H.__setitem__('CANCELLED',True)) for sig in [signal.SIGTERM,signal.SIGINT]}
    try:
        pgpins=pg_pin();proof['postgres_pins']=pgpins
        assert command(['git','-C',R,'rev-parse','HEAD']).decode().strip()==a.sha
        if not a.candidate:H['clean_exact'](R,a.sha)
        assert M.sha(M.bounded(a.dil,134217728))==a.binary_sha256
        build=json.loads(command([a.dil,'sürüm','--json']));assert build['schema']=='zee-build-v1' and build['source_sha']==a.sha
        if not a.candidate:assert build['dirty'] is False
        proof.update(build=build,binary_sha256=a.binary_sha256,framework_head=a.sha)
        with contextlib.ExitStack() as stack:
            td=stack.enter_context(tempfile.TemporaryDirectory(prefix='zcs-',dir=Path('/tmp').resolve(strict=True)));private=Path(td).resolve();private.chmod(0o700)
            directory=private/'pg';directory.mkdir(mode=0o700);ca,wrong,cert,key=TLS.certificates(directory)
            (directory/'pg_hba.conf').write_text('local all all trust\nhostssl all all 127.0.0.1/32 trust\nhostnossl all all 0.0.0.0/0 reject\n')
            cluster=TLS.LocalCluster(directory,cert,key,a.postgres_bin_dir)
            def close_pg():
                global cleaning
                cleaning=True;cancelled=H['CANCELLED'];H['CANCELLED']=False
                handlers={sig:signal.signal(sig,lambda *_:None) for sig in [signal.SIGTERM,signal.SIGINT]}
                try:cluster.close();cleanup.append('owned-pg-closed')
                finally:
                    if (directory/'server.log').is_file():(a.output/'postgres.log').write_bytes(M.bounded(directory/'server.log',1048576))
                    H['CANCELLED']=cancelled
                    for sig,h in handlers.items():signal.signal(sig,h)
            stack.callback(close_pg);cluster.start();assert cluster.version=='postgres (PostgreSQL) 16.15'
            def observer(sql):
                out=command(cluster.psql+['--no-psqlrc','--dbname',f'host=127.0.0.1 port={cluster.port} dbname=postgres user=postgres sslmode=require','--no-align','--tuples-only','--set','ON_ERROR_STOP=1','--command','BEGIN READ ONLY; '+sql+'; COMMIT;'])
                return out.decode().strip().splitlines()[1:-1]
            env={'ZEE_STARTER_DATABASE_URL':f'host=localhost port={cluster.port} dbname=postgres user=postgres sslmode=require','ZEE_STARTER_DATABASE_URL_TLS_CA_PEM':ca.read_text()}
            with patch.dict(os.environ,env):
                project=private/'site';command([sys.executable,'-I','-B',R/'scripts/icerik-paneli-olustur.py','--framework-root',R,'--root',project,'--dil',a.dil,'--db-target',f'postgresql://localhost:{cluster.port}/postgres'])
                assert (project/'proje.kilit').is_file();command([a.dil,'göçür',project]);record('creator-real-kilitle-denetle-dene-and-explicit-PG-migration')
                tls=private/'front-tls';tls.mkdir(mode=0o700);front_cert=tls/'cert.pem';front_key=tls/'key.pem'
                command(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-subj','/CN=localhost','-addext','subjectAltName=IP:127.0.0.1','-keyout',front_key,'-out',front_cert]);front_cert.chmod(0o600);front_key.chmod(0o600)
                with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
                authority=f'127.0.0.1:{port}';origin='https://'+authority
                argv=[sys.executable,'-I','-B',project/'yonetim/starter-content-host.py','--root',project,'--dil',a.dil,'--expected-source',a.sha,'--binary-sha256',a.binary_sha256,'--front-port',str(port),'--tls-cert',front_cert,'--tls-key',front_key]
                if a.candidate:argv+=['--candidate']
                def refused():
                    out=command(['/bin/sh','-c','"$@" 2>&1; code=$?; printf "\\nSTARTER_EXIT:%s\\n" "$code"','owned-starter-status',*argv])
                    assert out==b'starter-host-rejected\n\nSTARTER_EXIT:2\n'
                refused();record('missing-PHC-fixed-preflight-rejection')
                (project/'panel-phc').write_bytes(b'$argon2id$bad');(project/'panel-phc').chmod(0o600);refused();record('malformed-PHC-fixed-preflight-rejection')
                password=os.urandom(24).hex()
                # The only password-bearing command uses stdin through a retained owned wrapper.
                password_file=private/'password';password_file.write_text(password);password_file.chmod(0o600)
                phc=command(['/bin/sh','-c','exec "$1" parola-özeti --stdin < "$2"','owned-hash',a.dil,password_file],sensitive_output=True);(project/'panel-phc').write_bytes(phc.rstrip(b'\n'));(project/'panel-phc').chmod(0o600);password_file.unlink()
                context=ssl.create_default_context(cafile=str(front_cert))
                def http(path,method='GET',fields=None,cookie=None,body=None,extra=None):
                    end=min(deadline,time.monotonic()+12);data=urllib.parse.urlencode(fields or {}).encode() if body is None else body
                    headers={'Host':authority,'Connection':'close'}
                    if cookie:headers['Cookie']=cookie
                    if method=='POST':headers.update(Origin=origin,**{'Content-Type':'application/x-www-form-urlencoded' if body is None else 'application/octet-stream','Content-Length':str(len(data))})
                    headers.update(extra or {})
                    with socket.create_connection(('127.0.0.1',port),timeout=min(1,M.remaining(end))) as sock:
                        sock.settimeout(min(2,M.remaining(end)))
                        with context.wrap_socket(sock,server_hostname='127.0.0.1') as stream:
                            M.remaining(end);wire=(method+' '+path+' HTTP/1.1\r\n'+''.join(k+': '+v+'\r\n' for k,v in headers.items())+'\r\n').encode()+(data if method=='POST' else b'');M.send(stream,wire,end);out=bytearray()
                            while True:
                                part=M.recv(stream,16384,end)
                                if not part:break
                                out.extend(part);assert len(out)<=1048576
                    head,content=bytes(out).split(b'\r\n\r\n',1);status=int(head.split(b' ',2)[1]);cookies=[]
                    lengths=[int(x.split(b':',1)[1].strip()) for x in head.split(b'\r\n') if x.lower().startswith(b'content-length:')]
                    assert lengths==[len(content)]
                    if re.fullmatch(r'/media/[0-9a-f]{64}\.png',path) and status==200:
                        assert b'content-type: image/png' in head.lower() and b'x-content-type-options: nosniff' in head.lower()
                    for line in head.split(b'\r\n'):
                        if line.lower().startswith(b'set-cookie:'):cookies.append(line.split(b':',1)[1].strip().split(b';',1)[0].decode())
                    return status,content,cookies
                host=H['subprocess'].Popen(argv,stdin=H['subprocess'].DEVNULL,stdout=H['subprocess'].DEVNULL,stderr=H['subprocess'].PIPE,start_new_session=True,close_fds=True)
                def stop_host():
                    global host
                    if host is not None:
                        result=H['stop_owned'](host,grace=8);error=host.stderr.read(4096);host.stderr.close();host=None
                        proof['front_terminal']={'exit':result,'stderr_bytes':len(error),'stderr_sha256':M.sha(error)}
                        assert result==0 and not error;cleanup.append('owned-front-and-native-closed')
                stack.callback(stop_host)
                ready=time.monotonic()+15
                while True:
                    assert H['held_status'](host)is None and time.monotonic()<ready
                    try:
                        if http('/')[0]==200:break
                    except (OSError,ValueError):pass
                    time.sleep(.03)
                record('actual-HTTPS-owned-inherited-listener-ready-and-native-list')
                assert http('/admin')[0]==401
                status,body,cookies=http('/giris');assert status==200 and cookies;guest=cookies[0];csrf=re.search(rb'name=_csrf[^>]*value="([0-9a-f]{64})"',body).group(1).decode()
                assert http('/giris-yap','POST',{'parola':'wrong','_csrf':csrf},guest)[0]==303
                assert http('/admin',cookie=guest)[0]==401
                assert http('/giris-yap','POST',{'parola':password,'_csrf':'0'*64},guest)[0]==403
                assert http('/giris-yap','POST',{'parola':password,'_csrf':csrf},guest,extra={'Origin':'https://foreign.invalid'})[0]==400
                record('anonymous-wrong-password-origin-denied')
                status,_,cookies=http('/giris-yap','POST',{'parola':password,'_csrf':csrf},guest);assert status==303 and cookies and cookies[0]!=guest;session=cookies[0]
                status,body,_=http('/admin',cookie=session);assert status==200;csrf=re.search(rb'name=_csrf[^>]*value="([0-9a-f]{64})"',body).group(1).decode();record('actual-login-session-rotation-and-admin-CSRF')
                assert http('/ekle','POST',{'id':'badcsrf','title':'x','body':'x','_csrf':'0'*64},session)[0]==403
                assert not (project/'starter-pending.json').exists() and observer('SELECT count(*) FROM zee_starter_content')==['0']
                title='<script>owned-c6</script>"';body_value="'); DROP TABLE customer;--\n<script>owned-c6-body</script>"
                assert http('/ekle','POST',{'id':'news','title':title,'body':body_value,'_csrf':csrf},session)[0]==303
                status,detail,_=http('/icerik?id=news');assert status==200 and b'&lt;script&gt;owned-c6' in detail and title.encode() not in detail
                assert observer('SELECT count(*) FROM zee_starter_content')==['1'];record('actual-parameterized-content-create-detail-and-context-XSS')
                assert http('/guncelle','POST',{'id':'news','title':'Updated','body':body_value,'_csrf':csrf},session)[0]==303
                assert observer("SELECT title FROM zee_starter_content WHERE id='news'")==['Updated'];record('actual-content-update-independent-readonly-observer')
                binary=image();status,_,_=http('/medya-yukle?id=news','POST',cookie=session,body=binary,extra={'X-Zee-CSRF':csrf});assert status==200
                png_hash=observer("SELECT png_hash FROM zee_starter_media WHERE state='ready'")[0];assert re.fullmatch('[0-9a-f]{64}',png_hash)
                status,png,_=http('/media/'+png_hash+'.png');assert status==200 and hashlib.sha256(png).hexdigest()==png_hash and png.startswith(b'\x89PNG')
                assert b'/media/'+png_hash.encode()+b'.png' in http('/icerik?id=news')[1];record('actual-upload-image-conversion-immutable-binary-PNG-and-detail-wiring')
                def review(actions):
                    count=len(proof['cases']);observation=private/f'observation-{count}.json'
                    args=[sys.executable,'-I','-B',project/'yonetim/starter-content-reconcile.py','--root',project,'--dil',a.dil,'--expected-source',a.sha,'--binary-sha256',a.binary_sha256]
                    if a.candidate:args+=['--candidate']
                    command([*args,'--output',observation]);document=json.loads(observation.read_bytes())
                    value={'schema':'zee-content-starter-review-1','observation_sha256':M.sha(observation.read_bytes()),'decision':'accept-observed-state-no-replay','media_actions':actions};file=private/f'review-{count}.json';file.write_text(json.dumps(value));file.chmod(0o600)
                    command([*args,'--review',file]);assert not (project/'starter-pending.json').exists();return document
                assert http('/medya-yukle?id=news','POST',cookie=session,body=binary,extra={'X-Zee-CSRF':csrf})[0]==409
                assert (project/'starter-pending.json').exists() and M.sha(M.png_bytes(project,png_hash))==png_hash
                assert http('/guncelle','POST',{'id':'news','title':'must-not-run','body':'x','_csrf':csrf},session)[0]==400
                review([]);assert observer('SELECT count(*) FROM zee_starter_media')==['1'];record('duplicate-upload-no-clobber-persistent-admission-close-explicit-no-replay-review')
                for path,headers in [('/media/%2e%2e/x',{}),('/media/../x',{}),('/admin',{'Host':'foreign.invalid'}),('/icerik?id=news&id=other',{})]:assert http(path,extra=headers)[0]==400
                assert http('/ekle','POST',{'id':'../bad','title':'x','body':'x','_csrf':csrf},session)[0]==400
                assert not (project/'starter-pending.json').exists();record('actual-encoded-path-traversal-host-duplicate-and-form-boundary-negatives')
                invalid=b'not-an-image';source_hash=hashlib.sha256(invalid).hexdigest()
                assert http('/medya-yukle?id=news','POST',cookie=session,body=invalid,extra={'X-Zee-CSRF':csrf})[0]==422
                assert observer("SELECT state FROM zee_starter_media WHERE source_hash='"+source_hash+"'")==['preparing']
                observed=review([{'action':'discard-preparing','source_hash':source_hash,'png_hash':''}]);assert any(x['source_hash']==source_hash for x in observed['media'])
                assert observer('SELECT count(*) FROM zee_starter_media')==['1'];record('actual-bad-image-preparing-retained-explicit-metadata-only-review')
                assert http('/sil','POST',{'id':'news','_csrf':csrf},session)[0]==409;review([])
                source_hash=hashlib.sha256(binary).hexdigest();assert http('/medya-sil','POST',{'source':source_hash,'png':png_hash,'_csrf':csrf},session)[0]==303
                assert not (project/'media'/(png_hash+'.png')).exists() and observer('SELECT count(*) FROM zee_starter_media')==['0']
                assert http('/sil','POST',{'id':'news','_csrf':csrf},session)[0]==303 and observer('SELECT count(*) FROM zee_starter_content')==['0'];record('actual-media-delete-separate-SQL-finalization-then-content-delete')
                assert http('/cikis','POST',{'_csrf':csrf},session)[0]==303 and http('/admin',cookie=session)[0]==401;record('actual-logout-revoke')
                stop_host()
                assert any((project/'.zee/yuklemeler').iterdir());proof['orphan_boundary']='owned upload temporary artifacts retained until operator GC; not falsely allfiles-clean'
            assert pins=={n:M.sha(M.bounded(R/n,262144)) for n in roles}
            assert pg_pin()==pgpins and M.sha(M.bounded(a.dil,134217728))==a.binary_sha256 and json.loads(command([a.dil,'sürüm','--json']))==build
            assert command(['git','-C',R,'rev-parse','HEAD']).decode().strip()==a.sha
            if not a.candidate:H['clean_exact'](R,a.sha)
            proof.update(status='passed',case_count=len(proof['cases']),failed=0,source_end_unchanged=True,binary_end_unchanged=True)
    except BaseException as error:
        proof.update(status='failed',failure_type=type(error).__name__);raise
    finally:
        for sig,h in old.items():signal.signal(sig,h)
        proof.update(cleanup=cleanup,elapsed_seconds=time.monotonic()-start,private_root_removed=private is not None and not private.exists())
        (a.output/'raw-metadata.json').write_text(json.dumps(raw,indent=2)+'\n');(a.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
if __name__=='__main__':main()
