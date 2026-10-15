#!/usr/bin/env python3
"""Explicit owned observation/review; never automatically replay a business request."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import sys
import tempfile

M=None

def encode(value):return (json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n').encode()
def read_json(raw):
    M.need(len(raw)<=1048576)
    return json.loads(raw,object_pairs_hook=M.pairs,parse_constant=lambda _:(_ for _ in ()).throw(M.Rejected()))

class Review:
    def __init__(self,args):
        self.a=args;self.root=args.root;self.identity=M.directory(self.root)
        M.need(args.dil.resolve(strict=True)==args.dil)
        self.binary=M.bounded(args.dil,134217728);M.need(M.sha(self.binary)==args.binary_sha256)
        self.config=read_json(M.bounded(self.root/'starter-host.json',65536,True))
        M.need(self.config['schema']=='zee-content-starter-1' and self.config['root']==list(self.identity))
        self.pins()
        manifest=M.bounded(self.root/'proje.dil',262144).decode()
        targets=re.findall(r'^veritabanı_hedefi "(postgresql://(?:localhost|127\.0\.0\.1):[0-9]{1,5}/[a-z][a-z0-9_]{0,31})" olsun$',manifest,re.M)
        M.need(len(targets)==1);self.target=targets[0]
        self.temp=None;self.exe=None;self.deadline=M.time.monotonic()+90
    def pins(self):
        M.need(M.directory(self.root)==self.identity)
        expected={'ana.dil','alan.dil','proje.dil','proje.kilit','gocler/0001_content.sql'}
        M.need(type(self.config)is dict and set(self.config)=={'schema','root','files','runtime'} and set(self.config['files'])==expected)
        for name,h in self.config['files'].items():M.need(M.sha(M.bounded(self.root/name,262144))==h)
        M.need(set(self.config['runtime'])=={'starter-content-host.py','starter-content-reconcile.py','lifecycle_core.py'})
        for name,h in self.config['runtime'].items():M.need(M.sha(M.bounded(self.root/'yonetim'/name,65536))==h)
    def native(self,sql,params=(),write=False):
        root=self.temp/'query';root.mkdir(mode=0o700)
        try:
            manifest='proje "starter-review" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "veritabanı" listesi olsun\nveritabanı_hedefi '+json.dumps(self.target)+' olsun\nveritabanı_bağlantı_değişkeni "ZEE_STARTER_DATABASE_URL" olsun\nveritabanı_göçleri "gocler" olsun\n'
            (root/'proje.dil').write_text(manifest)
            (root/'gocler').mkdir(mode=0o700)
            values=', '.join(json.dumps(x,ensure_ascii=False) for x in params)+' listesi' if params else 'boş liste'
            body='parametreler '+values+' olsun\nsonuç '+json.dumps(sql)+' sorgusunu parametreler ile '+('değiştirmeyi' if write else 'okumayı')+' dene olsun\n'
            if write:source='eylem uygula\n    TamSayı sonucu döndürür\n'+''.join('    '+line+'\n' for line in body.splitlines())+'    sonuç döndür\nsonuç uygula olsun\nsonuç başarılıysa\n    sonucun değeri yaz\ndeğilse\n    "SQL_REJECTED" yaz\n'
            else:source=body+'sonuç başarılıysa\n    satirlar sonucun değeri olsun\n    satirların json metni yaz\ndeğilse\n    "SQL_REJECTED" yaz\n'
            (root/'ana.dil').write_text(source)
            env={k:v for k,v in os.environ.items() if not k.startswith(('GIT_','ZEE_'))}
            for key in ['ZEE_STARTER_DATABASE_URL','ZEE_STARTER_DATABASE_URL_TLS_CA_PEM']:
                if key in os.environ:env[key]=os.environ[key]
            for cmd in ['kilitle','çalıştır']:
                out=M.H['run']([self.exe,cmd,root],timeout=min(20,M.remaining(self.deadline)),limit=1048576,env=env)
            if write:M.need(out==b'1\n');return {'affected':1}
            return read_json(out)
        finally:
            import shutil
            shutil.rmtree(root)
    def observation(self):
        self.pins();raw=M.bounded(self.root/'starter-pending.json',4096,True);pending=read_json(raw)
        M.need(type(pending)is dict and set(pending)=={'schema','root','token','request_sha256','created_ns'})
        M.need(pending['schema']=='zee-content-starter-pending-1' and pending['root']==list(self.identity) and type(pending['created_ns'])is int and pending['created_ns']>0 and re.fullmatch('[0-9a-f]{32}',pending['token']) and re.fullmatch('[0-9a-f]{64}',pending['request_sha256']))
        ledger=self.native('SELECT ad::text AS ad,sha256::text AS sha256 FROM _zee_gocleri ORDER BY surum')
        M.need(ledger==[{'ad':'0001_content.sql','sha256':self.config['files']['gocler/0001_content.sql']}])
        contents=self.native('SELECT id::text AS id,title::text AS title,body::text AS body FROM zee_starter_content ORDER BY id LIMIT 101')
        media=self.native('SELECT source_hash::text AS source_hash,content_id::text AS content_id,png_hash::text AS png_hash,state::text AS state FROM zee_starter_media ORDER BY source_hash LIMIT 101')
        M.need(type(contents)is list and type(media)is list and len(contents)<=100 and len(media)<=100)
        for row in contents:M.need(type(row)is dict and set(row)=={'id','title','body'} and all(type(v)is str for v in row.values()))
        for row in media:M.need(type(row)is dict and set(row)=={'source_hash','content_id','png_hash','state'} and all(type(v)is str for v in row.values()) and row['state'] in {'preparing','ready','deleting'})
        M.need(M.bounded(self.root/'starter-pending.json',4096,True)==raw)
        return {'schema':'zee-content-starter-observation-1','root':list(self.identity),'pending_sha256':M.sha(raw),'project_sha256':M.sha(encode(self.config)),'binary_sha256':self.a.binary_sha256,'build':self.build,'ledger':ledger,'contents':contents,'media':media},raw
    def apply(self,review,observation,issued):
        M.need(type(review)is dict and set(review)=={'schema','observation_sha256','decision','media_actions'})
        M.need(review['schema']=='zee-content-starter-review-1' and review['observation_sha256']==M.sha(encode(observation)) and review['decision']=='accept-observed-state-no-replay' and type(review['media_actions'])is list and len(review['media_actions'])<=8)
        actions=[];seen=set()
        # Validate every requested role/path before any metadata mutation.
        for action in review['media_actions']:
            M.need(type(action)is dict and set(action)=={'action','source_hash','png_hash'})
            source=action['source_hash'];png=action['png_hash'];kind=action['action']
            M.need(type(source)is str and re.fullmatch('[0-9a-f]{64}',source) and source not in seen);seen.add(source)
            rows=[x for x in observation['media'] if x['source_hash']==source];M.need(len(rows)==1);row=rows[0]
            if kind=='mark-ready':
                M.need(row['state']=='preparing' and row['png_hash']=='' and re.fullmatch('[0-9a-f]{64}',png));M.png_bytes(self.root,png)
                sql="UPDATE zee_starter_media SET state='ready',png_hash=$2 WHERE source_hash=$1 AND state='preparing' AND png_hash=''"
            elif kind=='discard-preparing':
                M.need(row['state']=='preparing' and row['png_hash']=='' and png=='')
                sql="DELETE FROM zee_starter_media WHERE source_hash=$1 AND state='preparing' AND png_hash=''"
            else:
                M.need(kind=='complete-deleting' and row['state']=='deleting' and row['png_hash']==png and re.fullmatch('[0-9a-f]{64}',png))
                path=self.root/'media'/(png+'.png');M.directory(self.root/'media');M.need(not path.exists() and not path.is_symlink())
                sql="DELETE FROM zee_starter_media WHERE source_hash=$1 AND png_hash=$2 AND state='deleting'"
            actions.append((sql,[source,png] if kind!='discard-preparing' else [source]))
        M.publish_private(self.root/'starter-review.json',encode(review))
        for sql,params in actions:self.native(sql,params,True)
        self.pins();M.complete_effect(self.root,issued)
    def run(self):
        with tempfile.TemporaryDirectory(prefix='zee-starter-review-') as td:
            self.temp=Path(td).resolve();os.chmod(self.temp,0o700);self.exe=self.temp/'dil';self.exe.write_bytes(self.binary);self.exe.chmod(0o700)
            self.build=read_json(M.H['run']([self.exe,'sürüm','--json'],timeout=10,limit=8192))
            M.need(self.build['schema']=='zee-build-v1' and self.build['source_sha']==self.a.expected_source)
            if not self.a.candidate:M.need(self.build['dirty'] is False)
            with M.mutation_lock(self.root):
                observation,issued=self.observation()
                if self.a.review is None:
                    output=self.a.output;M.directory(output.parent);M.need(not output.exists() and not output.is_symlink())
                    fd=os.open(output,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
                    try:
                        raw=encode(observation);M.need(len(raw)<=1048576)
                        view=memoryview(raw)
                        while view:count=os.write(fd,view);M.need(count>0);view=view[count:]
                        os.fsync(fd)
                    finally:os.close(fd)
                    M.syncdir(output.parent)
                    sys.stdout.write('starter-observation-written '+M.sha(raw)+'\n')
                else:
                    self.apply(read_json(M.bounded(self.a.review,65536,True)),observation,issued)
                    sys.stdout.write('starter-review-completed; no business request replayed\n')
            self.pins();M.need(M.sha(M.bounded(self.a.dil,134217728))==self.a.binary_sha256)

def main():
    global M
    try:
        hp=Path(__file__).resolve().with_name('starter-content-host.py')
        spec=importlib.util.spec_from_file_location('owned_starter_host',hp);M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M);M.helper()
        class Parser(argparse.ArgumentParser):
            def error(self,message):raise M.Rejected('arguments')
        p=Parser(allow_abbrev=False)
        for name in ['root','dil']:p.add_argument('--'+name,type=Path,required=True)
        for name in ['expected-source','binary-sha256']:p.add_argument('--'+name,required=True)
        group=p.add_mutually_exclusive_group(required=True);group.add_argument('--output',type=Path);group.add_argument('--review',type=Path)
        p.add_argument('--candidate',action='store_true');a=p.parse_args();os.umask(0o077)
        def cancel(*_):M.STOPPED=True;M.H['CANCELLED']=True
        signal.signal(signal.SIGTERM,cancel);signal.signal(signal.SIGINT,cancel)
        Review(a).run();return 0
    except BaseException as error:
        visible=M is not None and isinstance(error,M.Visible)
        sys.stderr.write('starter-review-visible-durability-unconfirmed\n' if visible else 'starter-review-rejected; pending state retained unless visible completion was reported\n')
        return 4 if visible else 2
if __name__=='__main__':sys.exit(main())
