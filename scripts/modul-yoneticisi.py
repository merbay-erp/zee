#!/usr/bin/env python3
"""Managed scaffold pilot. Trusted local code; no DB execution or sandbox."""
import argparse, base64, contextlib, hashlib, html, json, os, re, shutil
import signal, stat, subprocess, sys, tempfile, time, uuid
from pathlib import Path
try:
    import fcntl
except ImportError:
    fcntl = None
H=None
try:
    hp=Path(__file__).resolve().with_name('lifecycle_core.py')
    fd=os.open(hp,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        hs=os.fstat(fd); hb=b''
        if stat.S_ISREG(hs.st_mode) and hs.st_size<=16384:
            while len(hb)<=16384:
                chunk=os.read(fd,min(4096,16385-len(hb)))
                if not chunk:break
                hb+=chunk
        if stat.S_ISREG(hs.st_mode) and len(hb)<=16384 and hashlib.sha256(hb).hexdigest()=='e76fe65b6103fc4732aafc4468afc5bbf3d0699a0f79d7f74879165636ebb1a9':
            H={'__name__':'k273_lifecycle','__file__':str(hp)}; exec(compile(hb,str(hp),'exec'),H)
    finally: os.close(fd)
except (OSError,ValueError,ImportError): pass
MAX_BYTES = 262144
MAX_FILES = 64
HEX = re.compile(r'[0-9a-f]{64}\Z')
ID = re.compile(r'[a-z][a-z0-9_]{0,31}\Z')
class Reject(ValueError): pass
class Published(Reject): pass

def sha(b): return hashlib.sha256(b).hexdigest()
def require(c):
    if not c: raise Reject('rejected')
def pairs(p):
    d = {}
    for k,v in p:
        require(k not in d); d[k] = v
    return d

def decode(b):
    require(len(b) <= MAX_BYTES)
    try: return json.loads(b, object_pairs_hook=pairs, parse_constant=lambda _: (_ for _ in ()).throw(Reject()))
    except (ValueError, RecursionError, UnicodeError): raise Reject('record') from None

def encoded(v): return (json.dumps(v, ensure_ascii=False, sort_keys=True, separators=(',',':'))+'\n').encode()
def snapshot(p):
    p = Path(p)
    fd = os.open(p, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        before = os.fstat(fd)
        require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_uid == os.geteuid() and before.st_size <= MAX_BYTES)
        chunks=[]; size=0
        while True:
            b=os.read(fd,min(8192,MAX_BYTES+1-size))
            if not b: break
            size+=len(b); require(size<=MAX_BYTES); chunks.append(b)
        after=os.fstat(fd); path=os.stat(p,follow_symlinks=False)
        key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
        require(key(before)==key(after)==key(path)); return b''.join(chunks)
    finally: os.close(fd)

def syncdir(p):
    fd=os.open(p,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try: os.fsync(fd)
    finally: os.close(fd)

def write(p,b):
    p=Path(p); missing=[]; parent=p.parent
    while not parent.exists(): missing.append(parent); parent=parent.parent
    for parent in reversed(missing): parent.mkdir(mode=0o700)
    require(not p.is_symlink())
    tmp=p.parent/('.tmp-'+uuid.uuid4().hex)
    fd=os.open(tmp,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        view=memoryview(b)
        while view:
            n=os.write(fd,view); require(n>0); view=view[n:]
        os.fsync(fd)
    finally: os.close(fd)
    try: os.replace(tmp,p); syncdir(p.parent)
    finally:
        if tmp.exists(): tmp.unlink()

def rel(s):
    require(type(s) is str and s and len(s)<160)
    p=Path(s); require(not p.is_absolute() and p.as_posix()==s and all(x not in ('','.', '..') and not x.startswith('.') for x in s.split('/')))
    return s

def ver(s):
    require(type(s) is str and re.fullmatch(r'(0|[1-9][0-9]{0,3})\.(0|[1-9][0-9]{0,3})\.(0|[1-9][0-9]{0,3})',s)); return tuple(map(int,s.split('.')))

def metadata(m):
    require(type(m) is dict and set(m)=={'schema','id','version','core_min','core_max_exclusive','package_name','package_root','route_adapter','menu','content_type','migration_range','files'})
    require(m['schema']=='zee-modul-1' and type(m['id']) is str and ID.fullmatch(m['id']) and m['id'] not in {'kaydi','metadata','cekirdek'})
    require(m['package_name']==m['id'] and m['package_root']=='paket' and m['route_adapter']=='rota.dil')
    ver(m['version']); require(ver(m['core_min']) <= (0,1,0) < ver(m['core_max_exclusive']))
    menu=m['menu']; require(type(menu) is dict and set(menu)=={'slug','label','path'} and type(menu['slug']) is str and ID.fullmatch(menu['slug']) and menu['path']=='/'+menu['slug'])
    require(menu['path'] not in {'/moduller','/icerik-turleri'} and type(menu['label']) is str and 0<len(menu['label'])<=80 and all(ord(c)>=32 for c in menu['label']))
    require(type(m['content_type']) is str and ID.fullmatch(m['content_type']))
    r=m['migration_range']; require(type(r) is list and len(r)==2 and all(type(x) is int for x in r) and 1<=r[0]<=r[1]<=999999)
    files=m['files']; require(type(files) is dict and 3<=len(files)<=16 and {'paket/ana.dil','paket/proje.dil','rota.dil'}<=set(files))
    for name,h in files.items():
        rel(name); require(type(h) is str and HEX.fullmatch(h) and (name in {'paket/ana.dil','paket/proje.dil','rota.dil'} or re.fullmatch(r'sql/[0-9]{6}\.sql',name)))
        if name.startswith('sql/'): require(r[0]<=int(Path(name).stem)<=r[1])
    return m

def module(src):
    src=Path(src); require(src.resolve()==src and src.is_dir() and not src.is_symlink())
    m=decode(snapshot(src/'zee-modul.json'))
    metadata(m)
    files=m['files']; r=m['migration_range']
    data={}; total=0
    for name,h in files.items():
        rel(name); require(type(h) is str and HEX.fullmatch(h) and (name in {'paket/ana.dil','paket/proje.dil','rota.dil'} or re.fullmatch(r'sql/[0-9]{6}\.sql',name)))
        if name.startswith('sql/'): require(r[0]<=int(Path(name).stem)<=r[1])
        p=src/name
        for parent in p.parents:
            if parent==src: break
            require(not parent.is_symlink())
        b=snapshot(p); total+=len(b); require(total<=MAX_BYTES and sha(b)==h); data[name]=b
    # Initial profile admits no transitive dependencies or installer capabilities.
    manifest=data['paket/proje.dil'].decode()
    require(manifest == f'proje "{m["id"]}" olsun\nsürüm "{m["version"]}" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler boş liste olsun\nyerel_bağımlılıklar boş liste olsun\n')
    return m,data

class Manager:
    def __init__(self, root, binary):
        require(fcntl is not None and hasattr(os,'O_NOFOLLOW') and H is not None)
        self.root=Path(root); self.binary=Path(binary)
        require(self.root.resolve()==self.root and self.binary.resolve()==self.binary)
        self.binary_hash=sha(snapshot(self.binary)) if self.binary.stat().st_size<=MAX_BYTES else self.large_binary()
        self.control=self.root/'yonetim'; self.identity=None; self.directory_ids={}
    def large_binary(self):
        fd=os.open(self.binary,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
        try:
            before=os.fstat(fd); require(stat.S_ISREG(before.st_mode) and before.st_size<=134217728)
            h=hashlib.sha256(); observed=0
            while True:
                b=os.read(fd,65536)
                if not b: break
                observed+=len(b); require(observed<=134217728)
                h.update(b)
            after=os.fstat(fd); current=self.binary.stat(follow_symlinks=False)
            key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
            require(key(before)==key(after)==key(current)); return h.hexdigest()
        finally: os.close(fd)
    def directory(self,p):
        p=Path(p); require(p.resolve()==p); fd=os.open(p,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:
            st=os.fstat(fd); current=os.stat(p,follow_symlinks=False)
            require(stat.S_ISDIR(st.st_mode) and st.st_uid==os.geteuid() and st.st_mode&0o077==0 and (st.st_dev,st.st_ino)==(current.st_dev,current.st_ino))
            key=str(p); identity=(st.st_dev,st.st_ino)
            require(key not in self.directory_ids or self.directory_ids[key]==identity); self.directory_ids[key]=identity
        finally: os.close(fd)
    def anchor(self):
        s=self.root.stat(); require(stat.S_ISDIR(s.st_mode) and s.st_uid==os.geteuid() and s.st_mode&0o077==0 and not self.root.is_symlink())
        require(self.identity is None or self.identity==(s.st_dev,s.st_ino)); self.identity=(s.st_dev,s.st_ino)
        self.directory(self.control); self.directory(self.control/'active')
    @contextlib.contextmanager
    def locked(self, shared=False):
        self.anchor(); fd=os.open(self.control/'lock',os.O_RDWR|os.O_NOFOLLOW)
        try:
            fs=os.fstat(fd); ps=os.stat(self.control/'lock',follow_symlinks=False)
            require(stat.S_ISREG(fs.st_mode) and fs.st_uid==os.geteuid() and fs.st_mode&0o077==0 and fs.st_nlink==1 and (fs.st_dev,fs.st_ino)==(ps.st_dev,ps.st_ino))
            fcntl.flock(fd, (fcntl.LOCK_SH if shared else fcntl.LOCK_EX)|fcntl.LOCK_NB)
            ps=os.stat(self.control/'lock',follow_symlinks=False); require((fs.st_dev,fs.st_ino)==(ps.st_dev,ps.st_ino))
            self.anchor(); yield
        finally: os.close(fd)
    def noactive(self):
        require(not any((self.control/'active').iterdir()))
    def allowed(self,n):
        rel(n)
        require(n in {'ana.dil','proje.dil','proje.kilit','modul_kaydi.dil'} or re.fullmatch(r'modul_[a-z][a-z0-9_]{0,31}\.dil',n) or re.fullmatch(r'modules/[a-z][a-z0-9_]{0,31}/paket/(ana|proje)\.dil',n) or re.fullmatch(r'archive/[a-z][a-z0-9_]{0,31}/[0-9]{6}\.sql',n))
        p=self.root/n
        for parent in p.parents:
            if parent==self.root: break
            if parent.exists() or parent.is_symlink(): self.directory(parent)
    def record(self,r):
        require(type(r) is dict and set(r)=={'schema','root','modules','retired','files'} and r['schema']=='zee-managed-1' and type(r['root']) is list and all(type(x) is int for x in r['root']) and r['root']==list(self.identity))
        require(type(r['files']) is dict and len(r['files'])<=MAX_FILES and type(r['modules']) is dict and len(r['modules'])<=8 and type(r['retired']) is dict and len(r['retired'])<=64)
        require(not set(r['modules'])&set(r['retired']))
        expected={'ana.dil','proje.dil','proje.kilit','modul_kaydi.dil'}; entries=[]
        for group in ('modules','retired'):
            for i,m in r[group].items():
                metadata(m); require(i==m['id']); entries.append(m)
                for n,h in m['files'].items():
                    if n.startswith('sql/'): target='archive/'+i+'/'+n[4:]
                    elif group=='retired': continue
                    else: target='modul_'+i+'.dil' if n=='rota.dil' else 'modules/'+i+'/'+n
                    expected.add(target); require(r['files'].get(target)==h)
        for n,h in r['files'].items(): self.allowed(n); require(type(h) is str and HEX.fullmatch(h))
        require(set(r['files'])==expected)
        for index,m in enumerate(entries):
            for other in entries[:index]:
                require(m['menu']['slug']!=other['menu']['slug'] and m['content_type']!=other['content_type'])
                a,b=m['migration_range']; c,d=other['migration_range']; require(b<c or d<a)
        for n,b in self.render(r).items(): require(r['files'][n]==sha(b))
        return r
    def receipt(self, check=True):
        self.anchor(); r=self.record(decode(snapshot(self.control/'receipt.json')))
        if check:
            for name,h in r['files'].items(): require(sha(snapshot(self.root/name))==h)
        return r
    def ready(self):
        require(not (self.control/'journal.json').exists()); return self.receipt()
    def init(self, port):
        require(type(port) is int and 1024<=port<=65535 and not self.root.exists())
        self.root.mkdir(mode=0o700); self.control.mkdir(mode=0o700); (self.control/'active').mkdir(mode=0o700)
        write(self.control/'lock',b'')
        self.anchor()
        r={'schema':'zee-managed-1','root':list(self.identity),'modules':{},'retired':{},'files':{}}
        main=f'{port} kapısında sunucu başlat\nGET "/" adresine istek geldiğinde\n    "Anonim çekirdek" yanıtını gönder\nmodul_kaydi rotalarını kullan\n'.encode()
        data=self.render(r); data['ana.dil']=main
        data=self.validate(data)
        for n,b in data.items(): write(self.root/n,b)
        r['files']={n:sha(b) for n,b in data.items()}; write(self.control/'receipt.json',encoded(r)); syncdir(self.root)
    def render(self,r):
        ids=sorted(r['modules']); deps=', '.join('"modules/'+i+'/paket"' for i in ids)
        manifest='proje "yonetilen-uygulama" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu" listesi olsun\nyerel_bağımlılıklar '+(deps+' listesi' if ids else 'boş liste')+' olsun\n'
        registry=''.join('modul_'+i+' rotalarını kullan\n' for i in ids)
        for path,values in [('/moduller',[html.escape(r['modules'][i]['menu']['label'],quote=True) for i in ids]),('/icerik-turleri',[r['modules'][i]['content_type'] for i in ids])]:
            registry+=f'GET "{path}" adresine istek geldiğinde\n    '+json.dumps('|'.join(values),ensure_ascii=False)+' yanıtını gönder\n'
        return {'proje.dil':manifest.encode(),'modul_kaydi.dil':registry.encode()}
    def validate(self,data):
        with tempfile.TemporaryDirectory(prefix='k273-candidate-') as td:
            p=Path(td).resolve(); os.chmod(p,0o700)
            for n,b in data.items(): write(p/n,b)
            env={k:v for k,v in os.environ.items() if not k.startswith(('GIT_','ZEE_'))}; env.update(HOME=str(p),TMPDIR=str(p))
            for command in ('kilitle','denetle'):
                try:
                    require(self.large_binary()==self.binary_hash)
                    H['run']([str(self.binary),command,'.'],cwd=p,env=env,timeout=15,limit=65536)
                    require(self.large_binary()==self.binary_hash)
                except H['Reject']: raise Reject('candidate validation') from None
            data['proje.kilit']=snapshot(p/'proje.kilit')
        return data
    def publish(self,r,data):
        old=self.receipt(); require(len(data)<=MAX_FILES)
        r['files']={n:sha(b) for n,b in data.items()}
        operations=[]
        for n in sorted(set(old['files'])|set(data)):
            self.allowed(n); before=snapshot(self.root/n) if n in old['files'] else None
            if n not in old['files']: require(not (self.root/n).exists() and not (self.root/n).is_symlink())
            after=data.get(n)
            if before!=after: operations.append({'path':n,'old':None if before is None else base64.b64encode(before).decode(),'new':None if after is None else base64.b64encode(after).decode()})
        j={'schema':'zee-module-journal-1','root':list(self.identity),'operations':operations,'receipt':r}
        journal_bytes=encoded(j); require(len(journal_bytes)<=MAX_BYTES and len(encoded(r))<=MAX_BYTES)
        write(self.control/'journal.json',journal_bytes); self.finish(j)
    def preflight(self,j):
        self.anchor()
        require(type(j) is dict and set(j)=={'schema','root','operations','receipt'} and j['schema']=='zee-module-journal-1' and j['root']==list(self.identity))
        r=self.record(j['receipt']); current=self.receipt(check=False)
        require(r['files']['ana.dil']==current['files']['ana.dil'])
        require(type(j['operations']) is list and len(j['operations'])<=MAX_FILES)
        operations={}
        for op in j['operations']:
            require(type(op) is dict and set(op)=={'path','old','new'}); n=rel(op['path']); self.allowed(n)
            require(n!='ana.dil' and n not in operations)
            values=[]
            for field in ('old','new'):
                value=op[field]; require(value is None or type(value) is str)
                data=None if value is None else base64.b64decode(value,validate=True)
                require(data is None or len(data)<=MAX_BYTES); values.append(data)
            old,new=values
            require((new is None and n not in r['files']) or (new is not None and r['files'].get(n)==sha(new)))
            if n.startswith('archive/') and n in current['files']: require(new is not None and sha(new)==current['files'][n])
            operations[n]=(old,new)
        require(set(current['files'])|set(r['files'])==set(current['files'])|set(r['files'])|set(operations))
        for n in sorted(set(current['files'])|set(r['files'])):
            self.allowed(n); p=self.root/n; present=p.exists() or p.is_symlink(); data=snapshot(p) if present else None
            if n in operations: require(data in operations[n])
            else: require(data is not None and r['files'].get(n)==sha(data))
        return r,operations
    def finish(self,j):
        r,operations=self.preflight(j)
        for n,(before,after) in operations.items():
            self.anchor(); self.allowed(n); p=self.root/n
            current=snapshot(p) if p.exists() else None; require(current in (before,after))
            if after is None:
                if p.exists(): p.unlink(); syncdir(p.parent)
            else: write(p,after)
        write(self.control/'receipt.json',encoded(r)); self.receipt()
        (self.control/'journal.json').unlink()
        try: syncdir(self.control)
        except OSError: raise Published('publication-visible-durability-unconfirmed') from None
    def change(self,src=None,remove=None):
        with self.locked():
            self.noactive(); r=self.ready(); data={n:snapshot(self.root/n) for n in r['files']}
            if src is not None:
                m,files=module(src); i=m['id']; require(i not in r['modules'] and i not in r['retired'] and len(r['modules'])<8)
                for other in list(r['modules'].values())+list(r['retired'].values()):
                    require(m['menu']['slug']!=other['menu']['slug'] and m['menu']['path']!=other['menu']['path'] and m['content_type']!=other['content_type'])
                    a,b=m['migration_range']; c,d=other['migration_range']; require(b<c or d<a)
                for n,b in files.items():
                    target=('archive/'+i+'/'+n[4:]) if n.startswith('sql/') else ('modul_'+i+'.dil' if n=='rota.dil' else 'modules/'+i+'/'+n)
                    require(target not in data); data[target]=b
                r['modules'][i]=m
            else:
                require(type(remove) is str and remove in r['modules']); m=r['modules'].pop(remove); r['retired'][remove]=m
                data={n:b for n,b in data.items() if n!='modul_'+remove+'.dil' and not n.startswith('modules/'+remove+'/')}
            data.update(self.render(r)); data=self.validate(data); self.anchor(); self.publish(r,data)
    def reconcile(self):
        with self.locked():
            self.noactive(); self.anchor(); self.finish(decode(snapshot(self.control/'journal.json')))
    def reconcile_active(self, review):
        # Operator assertion bound to current markers; never PID discovery/adoption.
        with self.locked():
            self.ready(); r=decode(snapshot(Path(review)))
            require(type(r) is dict and set(r)=={'schema','root','markers','ownership'} and r['schema']=='zee-module-offline-review-1' and r['root']==list(self.identity) and r['ownership']=='owned-service-group-cleanup-confirmed')
            paths=[]
            for p in (self.control/'active').iterdir():
                require(len(paths)<16); paths.append(p)
            paths.sort(); require(paths and type(r['markers']) is dict)
            actual={p.name:sha(snapshot(p)) for p in paths}
            require(all(re.fullmatch(r'[0-9a-f]{32}\.json',p.name) for p in paths) and r['markers']==actual)
            write(self.control/'offline-review.json',encoded(r))
            for p in paths:
                self.anchor(); p.unlink()
            try: syncdir(self.control/'active')
            except OSError: raise Published('reconciliation-visible-durability-unconfirmed') from None
    def launch(self):
        with self.locked(shared=True):
            r=self.ready(); self.anchor()
            count=0
            for _ in (self.control/'active').iterdir():
                count+=1; require(count<16)
            marker=self.control/'active'/(uuid.uuid4().hex+'.json')
            self.anchor(); write(marker,encoded({'schema':'zee-active-1','root':list(self.identity),'binary':self.binary_hash,'projection':sha(encoded(r)),'created_ns':time.time_ns()}))
            child=None; stopping=False; old={}
            def stop(sig,frame):
                nonlocal stopping
                stopping=True
            try:
                for sig in (signal.SIGTERM,signal.SIGINT): old[sig]=signal.signal(sig,stop)
                require(not stopping and self.large_binary()==self.binary_hash)
                child=subprocess.Popen([str(self.binary),'çalıştır','--deneysel-web','.'],cwd=self.root,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,close_fds=True,start_new_session=False)
                while child.poll() is None and not stopping: time.sleep(.02)
                if stopping and child.poll() is None:
                    child.terminate()
                    try: child.wait(timeout=3)
                    except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=2)
                rc=child.wait(timeout=2); self.ready(); self.anchor()
                require(self.large_binary()==self.binary_hash)
                marker.unlink()
                try: syncdir(marker.parent)
                except OSError: raise Published('cleanup-visible-durability-unconfirmed') from None
                return rc
            finally:
                try:
                    if child is not None and child.poll() is None:
                        child.terminate()
                        try: child.wait(timeout=3)
                        except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=2)
                finally:
                    for sig,handler in old.items(): signal.signal(sig,handler)

def main():
    try:
        class QuietParser(argparse.ArgumentParser):
            def error(self,message): raise Reject('arguments')
        p=QuietParser(); p.add_argument('--root',required=True); p.add_argument('--binary',required=True)
        p.add_argument('command',choices=['init','install','remove','reconcile','reconcile-active','launch']); p.add_argument('value',nargs='?')
        a=p.parse_args(); m=Manager(a.root,a.binary)
        if a.command=='init': m.init(int(a.value))
        elif a.command=='install': m.change(src=Path(a.value))
        elif a.command=='remove': m.change(remove=a.value)
        elif a.command=='reconcile': m.reconcile()
        elif a.command=='reconcile-active': m.reconcile_active(a.value)
        else: return m.launch()
        return 0
    except Published: sys.stderr.write('module-publication-visible-durability-unconfirmed\n'); return 4
    except (OSError,ValueError,TypeError,KeyError,subprocess.SubprocessError): sys.stderr.write('module-operation-rejected\n'); return 2
if __name__=='__main__': sys.exit(main())
