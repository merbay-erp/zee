#!/usr/bin/env python3
"""Copy a bounded declared content starter; no new compiler command or installer."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

FILES=['ana.dil','alan.dil','proje.dil','gocler/0001_content.sql','.gitignore']
def main():
    M=None
    try:
        class Parser(argparse.ArgumentParser):
            def error(self,message):raise ValueError('arguments')
        p=Parser(allow_abbrev=False)
        for name in ['framework-root','root','dil']:p.add_argument('--'+name,type=Path,required=True)
        p.add_argument('--db-target',required=True)
        a=p.parse_args();framework=a.framework_root;root=a.root;binary=a.dil
        if framework.resolve(strict=True)!=framework or root.parent.resolve(strict=True)!=root.parent or binary.resolve(strict=True)!=binary:raise ValueError()
        spec=importlib.util.spec_from_file_location('content_starter_host',framework/'scripts/starter-content-host.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M);M.helper()
        M.directory(root.parent);M.need(not root.exists() and not root.is_symlink())
        M.need(re.fullmatch(r'postgresql://(?:localhost|127\.0\.0\.1):[0-9]{1,5}/[a-z][a-z0-9_]{0,31}',a.db_target))
        M.need(1024<=int(a.db_target.split(':')[-1].split('/')[0])<=65535)
        source=framework/'sablonlar/icerik-paneli'
        data={n:M.bounded(source/n,262144) for n in FILES};M.need(sum(map(len,data.values()))<=524288)
        old=b'postgresql://localhost:5432/zee_starter';M.need(data['proje.dil'].count(old)==1)
        data['proje.dil']=data['proje.dil'].replace(old,a.db_target.encode())
        runtime={n:M.bounded(framework/'scripts'/n,65536) for n in ['starter-content-host.py','starter-content-reconcile.py','lifecycle_core.py']}
        before=hashlib.sha256(M.bounded(binary,134217728)).hexdigest()
        os.umask(0o077)
        M.need(re.fullmatch('[a-z][a-z0-9-]{0,31}',root.name))
        M.H['run']([binary,'yeni',root.name],cwd=root.parent,timeout=20,limit=65536)
        M.need(hashlib.sha256(M.bounded(binary,134217728)).hexdigest()==before)
        M.directory(root);(root/'yonetim').mkdir(mode=0o700);(root/'media').mkdir(mode=0o700)
        for name,raw in data.items():
            path=root/name;path.parent.mkdir(parents=True,exist_ok=True,mode=0o700);path.write_bytes(raw)
        for name,raw in runtime.items():(root/'yonetim'/name).write_bytes(raw)
        # Existing CLI, fixed operations only. No arbitrary command from data.
        for command in ['kilitle','denetle','dene']:
            M.H['run']([binary,command,root],timeout=20,limit=65536)
            M.need(hashlib.sha256(M.bounded(binary,134217728)).hexdigest()==before)
        tracked=['ana.dil','alan.dil','proje.dil','proje.kilit','gocler/0001_content.sql']
        config={'schema':'zee-content-starter-1','root':list(M.directory(root)),'files':{n:M.sha(M.bounded(root/n,262144)) for n in tracked},'runtime':{n:M.sha(raw) for n,raw in runtime.items()}}
        (root/'starter-host.json').write_text(json.dumps(config,sort_keys=True,indent=2)+'\n')
        guide=framework/'docs/versioned/0.8/icerik-paneli.md'
        if guide.exists():(root/'yonetim/REHBER.md').write_bytes(M.bounded(guide,65536))
        sys.stdout.write('content-starter-created; database migration and private credentials remain explicit\n');return 0
    except BaseException:
        sys.stderr.write('content-starter-rejected; any fresh partial root is retained\n');return 2
if __name__=='__main__':sys.exit(main())
