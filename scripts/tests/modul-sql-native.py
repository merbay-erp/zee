#!/usr/bin/env python3
"""Fresh owned PG, native Zee CRUD/migrations and independent readonly observer."""
import argparse
import contextlib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import stat
import socket
import sys
import tempfile
import time
from unittest.mock import patch
p=argparse.ArgumentParser(allow_abbrev=False)
for name in ['framework-root','dil','postgres-bin-dir','output']:p.add_argument('--'+name,type=Path,required=True)
p.add_argument('--sha',required=True)
p.add_argument('--binary-sha256',required=True)
p.add_argument('--candidate',action='store_true',help='explicit dirty/historical-binary candidate evidence, never clean acceptance')
a=p.parse_args();R=a.framework_root.resolve(strict=True)
assert a.framework_root == R and len(a.sha)==40 and all(c in '0123456789abcdef' for c in a.sha)
assert len(a.binary_sha256)==64 and all(c in '0123456789abcdef' for c in a.binary_sha256)
assert a.output.is_absolute() and a.output.parent.resolve(strict=True)==a.output.parent
parent=a.output.parent.stat(follow_symlinks=False)
assert stat.S_ISDIR(parent.st_mode) and parent.st_uid==os.geteuid() and parent.st_mode&0o077==0
a.output.mkdir(mode=0o700)
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
M=load('sql_modules',R/'scripts/modul-sql-yoneticisi.py');H=M.BASE['H']
TLS=load('owned_pg',R/'scripts/postgresql-tls-ci.py')
roles=['scripts/modul-sql-yoneticisi.py','scripts/modul-yoneticisi.py','scripts/lifecycle_core.py','scripts/tests/modul-sql-native.py','scripts/tests/modul-sql-test.py','scripts/postgresql-tls-ci.py','scripts/postgresql-tls11-control.py','golden/module-sql/news.json','golden/module-sql/stock.json']
pins={n:M.digest(M.BASE['snapshot'](R/n)) for n in roles}
proof={'schema':'zee-module-sql-native-tests-1','status':'running','cases':[],'source_files':[{'path':n,'sha256':v} for n,v in pins.items()]}
raw=[];cluster=None;work=None;cleanup=[];oldsignals={}
campaign_started=time.monotonic();campaign_deadline=campaign_started+600;cleaning=False
original_run=H['run']
def bounded_run(argv,**kw):
    left=campaign_deadline-time.monotonic()
    if not cleaning:assert left>0,'global campaign deadline'
    requested=kw.pop('timeout',30)
    return original_run(argv,timeout=requested if cleaning else min(requested,left),**kw)
H['run']=bounded_run

def file_pin(path):
    path=Path(path);fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd);assert stat.S_ISREG(before.st_mode) and before.st_size<=134217728
        h=hashlib.sha256();seen=0
        while True:
            block=os.read(fd,65536)
            if not block:break
            assert time.monotonic()<campaign_deadline
            seen+=len(block);assert seen<=134217728;h.update(block)
        after=os.fstat(fd);current=path.stat(follow_symlinks=False)
        key=lambda x:(x.st_dev,x.st_ino,x.st_size,x.st_mtime_ns,x.st_ctime_ns)
        assert key(before)==key(after)==key(current)
        return h.hexdigest()
    finally:os.close(fd)
pgpins={n:file_pin(a.postgres_bin_dir/n) for n in ['postgres','initdb','pg_ctl','psql']}
proof['postgres_binary_pins']=pgpins
proof['mode']='candidate' if a.candidate else 'clean-exact'

def run(argv,**kw):
    result=H['run'](argv,timeout=kw.pop('timeout',30),limit=65536,**kw)
    raw.append({'operation':str(Path(str(argv[0])).name),'output':result.decode('utf8','replace')})
    return result.decode()
TLS.run=run
for sig in [signal.SIGTERM,signal.SIGINT]:oldsignals[sig]=signal.signal(sig,lambda *_:H.__setitem__('CANCELLED',True))
def record(name):
    proof['cases'].append({'name':name,'passed':True})
    (a.output/'progress.json').write_text(json.dumps({'status':'running','completed_cases':proof['cases']},indent=2)+'\n')
def observed(sql):
    return run(cluster.psql+['--no-psqlrc','--dbname',f'host=127.0.0.1 port={cluster.port} user=postgres dbname=postgres sslmode=require','--tuples-only','--no-align','--set','ON_ERROR_STOP=1','--command','BEGIN READ ONLY; '+sql+'; COMMIT;']).strip().splitlines()[1:-1]
def reject(call):
    try:call()
    except (M.Rejected,M.BASE['Reject'],H['Reject'],OSError,BlockingIOError):return
    raise AssertionError('expected closed admission')
def request(port,path,deadline):
    deadline=min(campaign_deadline,deadline)
    assert deadline>time.monotonic()
    with socket.create_connection(('127.0.0.1',port),timeout=min(.5,deadline-time.monotonic())) as s:
        s.sendall(('GET '+path+' HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n').encode());out=b''
        while True:
            left=deadline-time.monotonic();assert left>0
            s.settimeout(min(.5,left));block=s.recv(4096)
            assert time.monotonic()<deadline
            if not block:return out
            out+=block;assert len(out)<=65536
try:
    assert run(['git','-C',R,'rev-parse','HEAD']).strip()==a.sha
    if not a.candidate:H['clean_exact'](R,a.sha)
    assert file_pin(a.dil)==a.binary_sha256
    build=json.loads(run([a.dil,'sürüm','--json']));proof['build']=build;proof['binary_sha256']=file_pin(a.dil)
    assert build['schema']=='zee-build-v1'
    if not a.candidate:assert build['source_sha']==a.sha and build['dirty'] is False
    proof['source_start']={'head':a.sha,'clean':not bool(run(['git','-C',R,'status','--porcelain','--untracked-files=all']).strip())}

    proof['framework_head']=run(['git','-C',R,'rev-parse','HEAD']).strip()
    with contextlib.ExitStack() as stack:
        td=stack.enter_context(tempfile.TemporaryDirectory(prefix='zms-',dir=Path('/tmp').resolve(strict=True)));work=Path(td).resolve();os.chmod(work,0o700)
        certdir=work/'pg';certdir.mkdir(mode=0o700)
        ca,wrong,cert,key=TLS.certificates(certdir)
        (certdir/'pg_hba.conf').write_text('local all all trust\nhostssl all all 127.0.0.1/32 trust\nhostnossl all all 0.0.0.0/0 reject\n')
        cluster=TLS.LocalCluster(certdir,cert,key,a.postgres_bin_dir)
        def close_cluster():
            global cleaning
            cleaning=True
            previous_cancel=H['CANCELLED'];H['CANCELLED']=False
            handlers={sig:signal.signal(sig,lambda *_:None) for sig in [signal.SIGTERM,signal.SIGINT]}
            try:cluster.close();cleanup.append('owned-pg-closed')
            except BaseException:cleanup.append('owned-pg-cleanup-failed');raise
            finally:
                if (certdir/'server.log').is_file():(a.output/'postgres.log').write_bytes(M.BASE['snapshot'](certdir/'server.log'))
                H['CANCELLED']=previous_cancel
                for sig,handler in handlers.items():signal.signal(sig,handler)
        stack.callback(close_cluster);cluster.start()
        proof['postgres_version']=cluster.version
        assert cluster.version=='postgres (PostgreSQL) 16.15'
        env={'ZEE_MODULE_DATABASE_URL':f'host=localhost port={cluster.port} user=postgres dbname=postgres sslmode=require','ZEE_MODULE_DATABASE_URL_TLS_CA_PEM':ca.read_text()}
        with patch.dict(os.environ,env):
            with socket.socket() as listener:listener.bind(('127.0.0.1',0));port=listener.getsockname()[1]
            m=M.Manager(work/'application',a.dil.resolve(strict=True),{'schema':'zee-sql-module-profile-1','target':f'postgresql://localhost:{cluster.port}/postgres','environment':'ZEE_MODULE_DATABASE_URL'});m.init(port)
            news=json.loads((R/'golden/module-sql/news.json').read_text());stock=json.loads((R/'golden/module-sql/stock.json').read_text())
            m.transition('up',news);m.transition('up',stock)
            assert set(m.ready()['modules'])=={'news','stock'}
            assert observed('SELECT count(*) FROM public._zee_gocleri')==['2']
            record('two-generic-schemas-native-DDL-and-project-registry')
            hostile={'id':"item'); DROP TABLE victim;--",'title':'<script>ŞĞİı</script>','published':True}
            assert m.crud('news','create',hostile)['affected']==1
            assert m.crud('news','read',{'id':hostile['id']})==[{'id':hostile['id'],'title':hostile['title'],'published':'true'}]
            m.crud('stock','create',{'id':'stock1','order':'Owned stock','quantity':17,'enabled':False})
            assert m.crud('stock','list',{})==[{'id':'stock1','order':'Owned stock','quantity':'17','enabled':'false'}]
            record('actual-parameterized-create-read-list-two-types-and-injection-data')
            m.crud('stock','update',{'id':'stock1','order':'Updated','quantity':-4,'enabled':True})
            assert observed('SELECT "order"||\':\'||quantity::text||\':\'||enabled::text FROM public.zee_mod_stock')==['Updated:-4:true']
            deleted=m.crud('stock','delete',{'id':'stock1'});assert deleted['affected']==1
            assert m.crud('stock','delete',{'id':'missing'})['affected']==0
            assert observed('SELECT count(*) FROM public.zee_mod_stock')==['0']
            record('actual-update-delete-independent-readonly-observer')
            child=H['subprocess'].Popen([sys.executable,'-I','-B',R/'scripts/modul-sql-yoneticisi.py','--root',m.root,'--binary',a.dil,'launch'],stdin=H['subprocess'].DEVNULL,stdout=H['subprocess'].DEVNULL,stderr=H['subprocess'].DEVNULL,start_new_session=True,close_fds=True)
            try:
                deadline=time.monotonic()+15
                while True:
                    try:page=request(port,'/news',deadline);break
                    except OSError:
                        assert H['held_status'](child) is None and time.monotonic()<deadline;time.sleep(.03)
                assert b'200 OK' in page and 'ŞĞİı'.encode() in page and b'DROP TABLE victim' in page
                assert b'<script>' not in page and b'&lt;script&gt;' in page and b'<pre>' in page
                assert b'Anonim haber|Anonim stok' in request(port,'/moduller',deadline)
                reject(lambda:m.transition('down','news'))
            finally:H['stop_owned'](child)
            assert not list((m.control/'active').iterdir());record('actual-HTTP-module-list-and-live-host-transition-lock')
            m.transition('down','news')
            assert observed('SELECT count(*) FROM public.zee_mod_news_archive')==['1']
            reject(lambda:m.crud('news','list',{}));reject(m.launch)
            m.transition('up','news')
            assert m.crud('news','read',{'id':hostile['id']})[0]['title']==hostile['title']
            assert observed('SELECT count(*) FROM public._zee_gocleri')==['4']
            record('real-down-archive-up-reapply-data-preserved-ledger-four')
            before=M.BASE['snapshot'](m.control/'sql-state.json');reject(lambda:m.transition('up','news'));reject(lambda:m.transition('up',news));assert M.BASE['snapshot'](m.control/'sql-state.json')==before
            migration=m.sqlroot/'gocler/000001_module.sql';original=migration.read_bytes();migration.write_bytes(original+b'-- drift\n')
            reject(lambda:m.crud('news','list',{}));migration.write_bytes(original)
            m.native(b'"MIGRATION_NOOP" yaz\n',{n:M.BASE['snapshot'](m.sqlroot/'gocler'/n) for n in m.state()['migrations']},'göçür')
            assert observed('SELECT count(*) FROM public._zee_gocleri')==['4']
            record('repeat-namespace-applied-noop-and-source-drift-reject-without-mutation')
            original_native=m.native
            def unknown(source,*args,**kw):
                result=original_native(source,*args,**kw)
                if b'INSERT INTO public.zee_mod_stock' in source:raise H['Reject']('controlled post-commit receipt loss')
                return result
            with patch.object(m,'native',side_effect=unknown):reject(lambda:m.crud('stock','create',{'id':'unknown','order':'Observed once','quantity':1,'enabled':True}))
            pending_raw=M.BASE['snapshot'](m.control/'sql-write-pending.json')
            reject(lambda:m.crud('stock','create',{'id':'unknown','order':'Observed once','quantity':1,'enabled':True}))
            assert observed("SELECT count(*) FROM public.zee_mod_stock WHERE id='unknown'")==['1']
            rows=[{'id':'unknown','order':'Observed once','quantity':'1','enabled':'true'}]
            review={'schema':'zee-sql-module-write-review-1','pending_sha256':M.digest(pending_raw),'observed':rows,'decision':'accept-observed-state-no-replay'}
            reject(lambda:m.reconcile_write(dict(review,pending_sha256='0'*64)))
            m.reconcile_write(review);assert m.crud('stock','read',{'id':'unknown'})==rows
            record('actual-committed-write-lost-receipt-persistent-block-readonly-reviewed-no-replay')
            original_publish=m.publish
            with patch.object(m,'publish',side_effect=OSError('controlled after database commit before filesystem projection')):reject(lambda:m.transition('down','stock'))
            assert (m.control/'sql-pending.json').exists();reject(lambda:m.crud('news','list',{}))
            m.reconcile_sql();assert m.state()['modules']['stock']['state']=='down'
            assert observed("SELECT count(*) FROM public.zee_mod_stock_archive WHERE id='unknown'")==['1']
            record('actual-migration-committed-filesystem-fault-readonly-ledger-reconcile')
            # A foreign collision is rejected by transactional CREATE, never adopted or dropped.
            run(cluster.psql+['--no-psqlrc','--dbname',env['ZEE_MODULE_DATABASE_URL'],'--set','ON_ERROR_STOP=1','--command','CREATE TABLE public.zee_mod_collision (foreign_marker text); INSERT INTO public.zee_mod_collision VALUES (\'retained\');'])
            bad={'schema':'zee-sql-module-1','id':'collision','label':'Collision','fields':[{'name':'title','type':'text'}]}
            reject(lambda:m.transition('up',bad));assert (m.control/'sql-pending.json').exists()
            reject(m.reconcile_sql);reject(lambda:m.crud('news','list',{}))
            assert observed('SELECT foreign_marker FROM public.zee_mod_collision')==['retained']
            assert observed('SELECT count(*) FROM public._zee_gocleri')==['5']
            record('real-DDL-collision-rollback-ledger-unchanged-pending-retained-no-adoption')
        assert pins=={n:M.digest(M.BASE['snapshot'](R/n)) for n in roles}
        assert file_pin(a.dil)==proof['binary_sha256'] and json.loads(run([a.dil,'sürüm','--json']))==build
        assert run(['git','-C',R,'rev-parse','HEAD']).strip()==proof['framework_head']
        assert pgpins=={n:file_pin(a.postgres_bin_dir/n) for n in pgpins}
        if not a.candidate:H['clean_exact'](R,a.sha)
        proof['source_end']={'head':a.sha,'clean':not bool(run(['git','-C',R,'status','--porcelain','--untracked-files=all']).strip())}
        proof.update(status='passed',case_count=len(proof['cases']),failed=0,source_end_unchanged=True,binary_end_unchanged=True,unknown_write_replay_count=0)
except BaseException as error:
    proof.update(status='failed',failure_type=type(error).__name__)
    raise
finally:
    for sig,handler in oldsignals.items():signal.signal(sig,handler)
    proof['elapsed_seconds']=time.monotonic()-campaign_started
    proof['cleanup']=cleanup;proof['temporary_private_root_removed']=work is not None and not work.exists()
    (a.output/'raw.json').write_text(json.dumps(raw,indent=2,ensure_ascii=False)+'\n')
    (a.output/'proof.json').write_text(json.dumps(proof,indent=2,ensure_ascii=False)+'\n')
