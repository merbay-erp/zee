#!/usr/bin/env python3
"""K265 exact-source RC bundle export/check; integrity is not origin authentication."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys
import tempfile

TARGETS = {'lexer_parser':65536,'morfoloji':128,'http_istegi':81920,'wasm_abi':4097}
MIRI = ('ondalik','zaman')
MAX_FILE = 64 * 1024 * 1024
MAX_TOTAL = 256 * 1024 * 1024
MAX_FILES = 200000
SCHEMA = 'zee-rc-evidence-1'

class Invalid(ValueError): pass

def require(value, message):
    if not value: raise Invalid(message)

def digest(data): return hashlib.sha256(data).hexdigest()

def pairs(items):
    result={}
    for key,value in items:
        require(key not in result,'duplicate JSON field')
        result[key]=value
    return result

def decode(data):
    try:
        result=json.loads(data,object_pairs_hook=pairs,parse_constant=lambda _:(_ for _ in ()).throw(Invalid('nonfinite JSON')))
        pending=[(result,0)];nodes=0
        while pending:
            value,depth=pending.pop();nodes+=1;require(depth<=32 and nodes<=2000000,'JSON structure budget')
            if isinstance(value,dict):pending.extend((v,depth+1) for v in value.values())
            elif isinstance(value,list):pending.extend((v,depth+1) for v in value)
        return result
    except (UnicodeError,json.JSONDecodeError,RecursionError) as e:raise Invalid('malformed/bounded JSON') from e


def integer(v):return type(v) is int

def query(root,*args):
    p=subprocess.run(['git','-C',str(root),*args],capture_output=True,timeout=30)
    require(p.returncode==0,'source query failed')
    return p.stdout

def identity(root):
    require(query(root,'status','--porcelain','--untracked-files=all','--','.')==b'','source must be clean')
    return {'sha':query(root,'rev-parse','HEAD').decode().strip(),'tree':query(root,'rev-parse','HEAD^{tree}').decode().strip(),'clean':True}

def relative(name):
    require(isinstance(name,str) and name and len(name)<=4096 and '\\' not in name and not any(ord(c)<32 for c in name),'unsafe file path')
    p=PurePosixPath(name)
    require(not p.is_absolute() and all(x not in ('','.','..') for x in name.split('/')),'unsafe file path')
    return p.parts

def allowed(name):
    parts=relative(name)
    if name=='campaign-proof.json':return True
    if len(parts)==2 and parts[0]=='logs':return parts[1] in [f'asan-build-{t}.log' for t in TARGETS]+[f'miri-{p}.log' for p in MIRI]
    if len(parts)>=3 and parts[0]=='targets' and parts[1] in TARGETS:
        return (len(parts)==3 and parts[2] in ('run.log','result.json','manifest.tsv','initial-manifest.tsv','sonuc.tsv','manifest-verification.log')) or (len(parts)>=4 and parts[2]=='corpus')
    return False

def open_directory(path):
    path=Path(path)
    require(path.is_absolute(),'absolute directory required')
    fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY)
    try:
        for part in path.parts[1:]:
            require(part not in ('','.','..'),'unsafe directory')
            new=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=fd);os.close(fd);fd=new
        return fd
    except BaseException:
        os.close(fd);raise


def read_at(rootfd,name,limit):
    parts=relative(name);parent=os.dup(rootfd)
    try:
        for part in parts[:-1]:
            new=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent);os.close(parent);parent=new
        fd=os.open(parts[-1],os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=parent)
        try:
            before=os.fstat(fd);require(stat.S_ISREG(before.st_mode) and before.st_size<=limit,'file type/budget')
            data=bytearray()
            while True:
                chunk=os.read(fd,min(65536,limit+1-len(data)))
                if not chunk:break
                data.extend(chunk);require(len(data)<=limit,'file budget')
            after=os.fstat(fd)
            require((before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)==(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns),'file changed during read')
            require(len(data)==before.st_size,'short file')
            return bytes(data)
        finally:os.close(fd)
    finally:os.close(parent)

def write_at(rootfd,name,data):
    parts=relative(name);parent=os.dup(rootfd)
    try:
        for part in parts[:-1]:
            try:os.mkdir(part,mode=0o700,dir_fd=parent)
            except FileExistsError:pass
            child=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent);os.close(parent);parent=child
        fd=os.open(parts[-1],os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o400,dir_fd=parent)
        try:
            view=memoryview(data)
            while view:
                count=os.write(fd,view);require(count>0,'short export write');view=view[count:]
        finally:os.close(fd)
    finally:os.close(parent)


def inventory(rootfd):
    result=[];entries=[0]
    def walk(fd,prefix):
        require(prefix.count('/')<=32,'directory depth budget')
        names=[]
        with os.scandir(fd) as iterator:
            for entry in iterator:
                entries[0]+=1;require(entries[0]<=MAX_FILES,'entry count budget')
                names.append(entry.name)
        for name in sorted(names):
            path=prefix+name;relative(path)
            info=os.stat(name,dir_fd=fd,follow_symlinks=False)
            if stat.S_ISDIR(info.st_mode):
                child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=fd)
                try:walk(child,path+'/')
                finally:os.close(child)
            else:
                require(stat.S_ISREG(info.st_mode),'nonregular bundle entry')
                result.append(path);require(len(result)<=MAX_FILES,'file count budget')
    walk(rootfd,'');return result

def bundle_snapshot(bundle,expected):
    require(re.fullmatch('[0-9a-f]{64}',expected or ''),'manifest digest required')
    fd=open_directory(bundle)
    try:
        raw=read_at(fd,'manifest.json',32*1024*1024);require(digest(raw)==expected,'manifest digest differs')
        m=decode(raw);require(set(m)=={'schema','source','producer_sha256','origin','files'} and m['schema']==SCHEMA and m['origin']=='owned-local','manifest schema/origin')
        require(isinstance(m['files'],dict) and 0<len(m['files'])<=MAX_FILES,'file table')
        require(set(inventory(fd))==set(m['files'])|{'manifest.json'},'unlisted/missing entry')
        # Private byte snapshot, not path reopens after validation.
        data={};total=0
        for name,record in m['files'].items():
            require(allowed(name) and isinstance(record,dict) and set(record)=={'sha256','bytes'},'unexpected file record')
            require(integer(record['bytes']) and 0<=record['bytes']<=MAX_FILE,'file size')
            total+=record['bytes'];require(total<=MAX_TOTAL,'bundle budget')
            value=read_at(fd,name,min(record['bytes'],MAX_FILE));require(len(value)==record['bytes'] and digest(value)==record['sha256'],'raw file digest differs')
            data[name]=value
        require(read_at(fd,'manifest.json',32*1024*1024)==raw and set(inventory(fd))==set(m['files'])|{'manifest.json'},'bundle changed')
        return m,data
    finally:os.close(fd)

def command(record,log):
    require(isinstance(record,dict) and integer(record.get('exit')) and record.get('exit')==0 and record.get('passed') is True and record.get('supervisor_failure') is None,'command failed')
    require(record.get('log_sha256')==digest(log),'command/raw digest differs')
    elapsed=record.get('elapsed_seconds');require(type(elapsed) in (int,float) and 0<=elapsed<=2412,'command deadline')
    require(isinstance(record.get('argv'),list) and all(isinstance(x,str) for x in record['argv']),'command provenance')

def recorded_path(value):
    require(isinstance(value,str) and value.startswith('/') and not value.startswith('//'),'recorded absolute path required')
    require('\\' not in value and all(ord(c)>=32 for c in value),'unsafe recorded path')
    require(all(part not in ('','.','..') for part in value.split('/')[1:]),'noncanonical recorded path')
    return PurePosixPath(value)

def command_profile(argv,target,maximum,operation,campaign_root=None):
    prefix=['cargo','+nightly-2026-08-31','fuzz',operation]
    if operation=='build':
        require(len(argv)==12 and argv[:9]==prefix+['--verbose','--sanitizer','address','--target','aarch64-apple-darwin'] and argv[9]=='--target-dir' and argv[11]==target,'exact ASAN build argv')
        directory=recorded_path(argv[10]);require(campaign_root is not None and directory==campaign_root/'fuzz-target','canonical build directory')
        return campaign_root
    require(len(argv)==19 and argv[:9]==prefix+['--sanitizer','address','--target','aarch64-apple-darwin','--target-dir'] and argv[10]==target and argv[12]=='--','exact fuzz run argv')
    corpus=recorded_path(argv[11]);suffix=('targets',target,'corpus')
    require(len(corpus.parts)>len(suffix)+1 and corpus.parts[-3:]==suffix,'canonical target corpus')
    actual_root=corpus.parents[2]
    require(campaign_root is None or actual_root==campaign_root,'campaign root differs across targets')
    require(recorded_path(argv[9])==actual_root/'fuzz-target','canonical run directory')
    require(argv[13]=='-dict='+str(actual_root/'checkout/compiler/fuzz/dictionaries/zee.dict'),'canonical dictionary path')
    require(argv[14:]==['-max_total_time=1800','-max_len='+str(maximum),'-timeout=5','-print_final_stats=1','-artifact_prefix='+str(actual_root/'targets'/target/'crashes')+'/'],'fixed fuzz options')
    return actual_root


def seed_manifest(raw,target,proof,current):
    lines=raw.decode().splitlines();require(lines and lines[0]=='# zee-fuzz-corpus-artifact-1','corpus schema')
    headers={};seeds={}
    for line in lines[1:]:
        if line.startswith('# '):
            fields=line[2:].split('\t');require(len(fields)==2 and fields[0] not in headers,'corpus headers');headers[fields[0]]=fields[1]
        elif line:
            fields=line.split('\t');require(len(fields)==2 and fields[1] not in seeds and re.fullmatch('[0-9a-f]{64}',fields[0]),'seed record');relative(fields[1]);seeds[fields[1]]=fields[0]
    expected={'target':target,'corpus':target,'source_commit':current['sha'],'run_id':proof['run_id'],'run_attempt':'1','toolchain':'nightly-2026-08-31','cargo_fuzz':'0.13.2','run_origin':'local','seed_sha256':'relative_path'}
    require(headers==expected,'corpus provenance differs')
    return seeds

def source_seeds(root,sha,target):
    prefix='compiler/fuzz/corpus/'+target+'/'
    entries=query(root,'ls-tree','-r','-z',sha,'--',prefix).split(b'\0');result={}
    for entry in entries:
        if not entry:continue
        meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();path=name.decode()
        require(mode==b'100644' and kind==b'blob' and path.startswith(prefix),'source seed type')
        result[path[len(prefix):]]=digest(query(root,'cat-file','blob',oid.decode()))
    return result


def validate(root,current,m,data):
    require(isinstance(m['source'],dict) and set(m['source'])=={'sha','tree','clean'} and m['source'].get('clean') is True and m['source']==current,'exact source differs')
    producer=digest(query(root,'show',current['sha']+':scripts/rc-kampanyasi.py'))
    require(m['producer_sha256']==producer,'producer source differs')
    p=decode(data.get('campaign-proof.json',b''))
    require(p.get('schema')=='zee-local-asan-miri-campaign-1' and p.get('result')=='passed-local-technical-campaign','campaign incomplete')
    require(p.get('producer_path')=='scripts/rc-kampanyasi.py' and p.get('producer_sha256')==producer,'campaign producer differs')
    require(p.get('source_sha')==current['sha'] and p.get('source_unchanged') is True,'campaign source differs')
    for key in ('source_start','source_end','original_source_start','original_source_end'):require(isinstance(p.get(key),dict) and p[key].get('clean') is True and p[key]==current,'campaign clean source differs')
    require(p.get('run_origin')=='local' and integer(p.get('run_attempt')) and p.get('run_attempt')==1 and re.fullmatch('[0-9]+',p.get('run_id','')),'campaign origin')
    require(p.get('tools',{}).get('cargo_fuzz')=='cargo-fuzz 0.13.2' and 'commit-hash: 90850177249efe0321573c569aec5d12b257f8d6' in p.get('tools',{}).get('rustc','') and p.get('tools',{}).get('miri','').startswith('miri 0.1.0 (9085017724 2026-08-30)'),'tools differ')
    environment=p.get('platform',{})
    require(isinstance(environment,dict) and set(environment)=={'system','release','os_version','machine','processor','ram_bytes'},'environment fields')
    require(environment['system']=='Darwin' and environment['machine']=='arm64' and p.get('host_target')=='aarch64-apple-darwin','local host profile')
    require(all(isinstance(environment[k],str) and 0<len(environment[k])<=256 and all(ord(c)>=32 for c in environment[k]) for k in ('release','os_version','processor')),'environment missing')
    require(integer(environment['ram_bytes']) and environment['ram_bytes']>0,'RAM missing')
    require(p.get('tools',{}).get('toolchain')=='nightly-2026-08-31' and 'host: aarch64-apple-darwin' in p['tools']['rustc'],'toolchain/host differs')
    require(type(p.get('maximum_concurrent_fuzz_targets')) is int and p['maximum_concurrent_fuzz_targets']==2,'producer concurrency')
    records=p.get('targets',[]);require(len(records)==4 and {x.get('target') for x in records}==set(TARGETS),'four exact targets required')
    campaign_root=None
    for record in records:
        t=record['target'];prefix='targets/'+t+'/'
        require(record.get('max_len')==TARGETS[t] and record.get('asan_runtime_marker') is True and record.get('manifest_verified') is True,'target configuration')
        build=p.get('builds',{}).get(t,{});buildlog=data['logs/asan-build-'+t+'.log'];command(build,buildlog)
        require(isinstance(record.get('command'),dict),'target command required')
        campaign_root=command_profile(record['command'].get('argv',[]),t,TARGETS[t],'run',campaign_root)
        command_profile(build['argv'],t,TARGETS[t],'build',campaign_root)
        require(build.get('explicit_sanitizer_flag') is True and b'sanitizer=address' in buildlog,'ASAN build evidence')
        require(build['argv'][:4]==['cargo','+nightly-2026-08-31','fuzz','build'] and '--sanitizer' in build['argv'] and build['argv'][build['argv'].index('--sanitizer')+1]=='address','ASAN build command')
        require(re.fullmatch('[0-9a-f]{64}',record.get('binary_sha256','')),'target binary digest')
        log=data[prefix+'run.log'];command(record.get('command',{}),log);argv=record['command']['argv']
        command_profile(argv,t,TARGETS[t],'run',campaign_root)
        require(build['argv'][10]==argv[9],'build/run target directory differs')
        require(argv[:3]==['cargo','+nightly-2026-08-31','fuzz'] and '--sanitizer' in argv and argv[argv.index('--sanitizer')+1]=='address','ASAN run command')
        require(t in argv and '-timeout=5' in argv and '-print_final_stats=1' in argv and '-max_len='+str(TARGETS[t]) in argv,'fuzz arguments')
        s=record.get('stats',{});requested=s.get('requested_seconds');actual=s.get('actual_seconds');executions=s.get('executions')
        require(integer(requested) and 1800<=requested<=3600 and integer(actual) and requested<=actual<=3600 and integer(executions) and executions>0,'duration/executions')
        require('-max_total_time='+str(requested) in argv and record['command']['elapsed_seconds']>=actual-1,'duration argv/wall differs')
        require(s.get('passed') is True and s.get('sanitizer')=='address' and all(s.get(k) is False for k in ('sanitizer_finding','libfuzzer_failure','input_timeout')) and s.get('crash_artifacts')==[],'fuzz failure')
        text=log.decode('utf-8',errors='replace');done=re.findall(r'Done\s+(\d+)\s+runs in\s+(\d+)\s+second',text);units=re.findall(r'stat::number_of_executed_units:\s*(\d+)',text)
        require(done and units and tuple(map(int,done[-1]))==(executions,actual) and int(units[-1])==executions,'raw final stats differ')
        require(not re.search(r'AddressSanitizer:DEADLYSIGNAL|(?:ERROR|SUMMARY):\s*AddressSanitizer|ERROR:\s*libFuzzer|libFuzzer:\s*deadly signal|SUMMARY:\s*UndefinedBehaviorSanitizer|TIMEOUT|timeout-',text),'raw failure')
        require(decode(data[prefix+'result.json'])==record,'target result differs')
        require(digest(data[prefix+'manifest.tsv'])==record.get('manifest_sha256'),'corpus manifest differs')
        seeds=seed_manifest(data[prefix+'manifest.tsv'],t,p,current)
        initial=seed_manifest(data[prefix+'initial-manifest.tsv'],t,p,current)
        require(integer(record.get('initial_seeds')) and record['initial_seeds']==len(initial) and initial==source_seeds(root,current['sha'],t),'initial corpus/source lineage differs')
        actual_seeds={n[len(prefix+'corpus/'):]:digest(b) for n,b in data.items() if n.startswith(prefix+'corpus/')};require(seeds==actual_seeds and len(seeds)==record.get('final_seeds'),'corpus files differ')
        result={}
        for line in data[prefix+'sonuc.tsv'].decode().splitlines()[1:]:
            k,v=line.split('\t');require(k not in result,'duplicate result');result[k]=v
        require(result=={'target':t,'source_commit':current['sha'],'run_id':p['run_id'],'run_attempt':'1','run_origin':'local','requested_seconds':str(requested),'sanitizer':'address','exit_code':'0'},'raw result differs')
    miri=p.get('miri',[]);require(len(miri)==2 and {x.get('name') for x in miri}=={'miri-'+x for x in MIRI},'two Miri groups required')
    for rec in miri:
        group=rec['name'][5:];log=data['logs/miri-'+group+'.log'];command(rec,log)
        require(rec['argv']==['cargo','+nightly-2026-08-31','miri','test','--locked','--lib',group+'::testler::'],'Miri command differs')
        totals=re.findall(rb'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)
        require(len(totals)==1 and int(totals[0][0])>0 and totals[0][1:]==(b'0',b'0') and rec.get('actual_tests')==int(totals[0][0]),'Miri actual results')
    require(identity(root)==current,'source changed during validation')
    return {'schema':'zee-rc-evidence-check-1','source':current,'origin':'owned-local-reviewed-not-authenticated','targets':4,'miri_groups':2,'status':'validated-integrity-and-policy'}

def export(root,campaign,output):
    current=identity(root);require(output.resolve(strict=False)==output,'output ancestor symlink rejected');require(not output.exists() and output.is_absolute() and not output.is_relative_to(root),'fresh external bundle required')
    fd=open_directory(campaign)
    try:
        names=[n for n in inventory_campaign(fd) if allowed(n)];require('campaign-proof.json' in names,'campaign proof missing')
        proof=decode(read_at(fd,'campaign-proof.json',8*1024*1024));m={'schema':SCHEMA,'source':current,'producer_sha256':proof.get('producer_sha256'),'origin':'owned-local','files':{}};data={};total=0
        for n in names:
            b=read_at(fd,n,MAX_FILE);total+=len(b);require(total<=MAX_TOTAL,'bundle budget');data[n]=b;m['files'][n]={'sha256':digest(b),'bytes':len(b)}
        validate(root,current,m,data)
        parent=open_directory(output.parent)
        try:
            info=os.fstat(parent);require(info.st_uid==os.geteuid() and info.st_mode & 0o022 == 0,'private owned export parent required')
            os.mkdir(output.name,mode=0o700,dir_fd=parent)
            destination=os.open(output.name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent)
            try:
                for n,b in data.items():write_at(destination,n,b)
                raw=(json.dumps(m,sort_keys=True,indent=2)+'\n').encode();write_at(destination,'manifest.json',raw)
            finally:os.close(destination)
        finally:os.close(parent)
        require(identity(root)==current,'source changed during export')
        return {'status':'exported-not-release-gate-pass','manifest_sha256':digest(raw),'source':current}
    finally:os.close(fd)

def inventory_campaign(fd):
    # Campaign includes checkout/build caches; traverse only fixed evidence roots.
    result=['campaign-proof.json']
    for top in ('logs','targets'):
        child=os.open(top,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=fd)
        try:result.extend(top+'/'+p for p in inventory(child))
        finally:os.close(child)
    require(all(allowed(n) for n in result),'unexpected campaign evidence entry')
    return result

def main():
    options=[x for x in sys.argv[2:] if x.startswith('--')]
    require(len(options)==len(set(options)),'duplicate option')
    p=argparse.ArgumentParser(allow_abbrev=False);p.add_argument('mode',choices=('export','check'));p.add_argument('--framework-root',type=Path,required=True);p.add_argument('--campaign',type=Path);p.add_argument('--output',type=Path);p.add_argument('--bundle',type=Path);p.add_argument('--sha256');a=p.parse_args()
    require(sys.flags.isolated and sys.dont_write_bytecode,'use Python -I -B');root=a.framework_root.resolve(strict=True)
    if a.mode=='export':require(a.campaign is not None and a.output is not None and a.bundle is None and a.sha256 is None,'export arguments');result=export(root,a.campaign,a.output)
    else:
        require(a.bundle is not None and a.sha256 is not None and a.campaign is None and a.output is None,'check arguments');current=identity(root);m,data=bundle_snapshot(a.bundle,a.sha256);result=validate(root,current,m,data);result['manifest_sha256']=a.sha256
    print(json.dumps(result,sort_keys=True))

if __name__=='__main__':
    try:main()
    except (Invalid,OSError,subprocess.SubprocessError,KeyError,TypeError,ValueError,AttributeError,IndexError,RecursionError):
        sys.stderr.write('RC evidence rejected; exact source or complete immutable evidence required.\n');sys.exit(2)
