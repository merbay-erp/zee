#!/usr/bin/env python3
"""Actual ephemeral Ed25519 + native Zee request/reply; operator-owned exact PDF runtime."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile

p=argparse.ArgumentParser(allow_abbrev=False)
for name in ['framework-root','dil','verifier','python','output']:p.add_argument('--'+name,type=Path,required=True)
a=p.parse_args();R=a.framework_root.resolve(strict=True);a.output.mkdir(mode=0o700)
spec=importlib.util.spec_from_file_location('pdf_plugin',R/'scripts/pdf-plugin-host.py');M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)
proof={'schema':'zee-pdf-plugin-native-tests-1','status':'running','cases':[]}
source_roles=['golden/pdf-host/assets/base.pdf','golden/pdf-host/assets/font.ttf','golden/pdf-host/assets/product.png','golden/pdf-host/assets/LICENSE-DejaVu.txt','golden/pdf-host/assets/COPYRIGHT-DejaVu.txt','scripts/pdf-plugin-host.py','scripts/tests/pdf-plugin-native.py','scripts/worker-supervisor.py','scripts/derleyici-secici.py','scripts/pdf-katalog-host.py','golden/pdf-plugin/ana.dil']
pins={n:M.digest(M.bounded(R/n,1048576)) for n in source_roles}
private=tempfile.TemporaryDirectory(prefix='zee-pdf-plugin-native-');T=Path(private.name);owner=M.Owned(R,T);owner.deadline=M.time.monotonic()+180
raw=[]
def command(argv,limit=16384):return owner.command(argv,30,limit)
def host(package,trust,source,output):
    argv=[sys.executable,'-I','-B',str(R/'scripts/pdf-plugin-host.py'),'--framework-root',str(R),'--package',str(package),'--trust',str(trust),'--trust-sha256',M.digest(trust.read_bytes()),'--dil',str(a.dil),'--dil-sha256',M.digest(M.bounded(a.dil,67108864)),'--verifier',str(a.verifier),'--verifier-sha256',M.digest(M.bounded(a.verifier,67108864)),'--python',str(a.python),'--source',str(source),'--expected-source',build['source_sha'],'--output',str(output)]
    label=source.parent.name
    bindings=a.output/'public-bindings'/label
    for name,file in [('plugin.json',package/'plugin.json'),('plugin.json.zee-imza',package/'plugin.json.zee-imza'),('trust.json',trust)]:shutil.copyfile(file,bindings/name)
    data=command(['/bin/sh','-c','"$@" 2>&1; code=$?; printf "\\nPDF_EXIT:%s\\n" "$code"','owned-pdf-status',*argv])
    body,tail=data.rsplit(b'\nPDF_EXIT:',1);code=int(tail.strip());value=json.loads(body)
    raw.append({'exit':code,'metadata':value});return code,value
try:
    build=json.loads(command([a.dil,'sürüm','--json']));proof['build']=build;proof['binary_sha256']=M.digest(M.bounded(a.dil,67108864));proof['runtime_sha256']=M.digest(M.bounded(a.python.resolve(strict=True),67108864));proof['verifier_sha256']=M.digest(M.bounded(a.verifier,67108864));proof['framework_head']=command(['git','-C',R,'rev-parse','HEAD']).decode().strip()
    key=T/'synthetic.key';command([a.dil,'anahtar','üret',key]);os.chmod(key,0o600)
    key2=T/'other.key';command([a.dil,'anahtar','üret',key2]);os.chmod(key2,0o600)
    def fixture(number,backend=None):
        root=T/str(number);root.mkdir(mode=0o700);package=root/'package';package.mkdir(mode=0o700);(package/'assets').mkdir()
        for name in M.FILES:
            data=(R/'scripts/pdf-katalog-host.py').read_bytes() if name=='backend.py' else (R/'golden/pdf-host'/name).read_bytes()
            if name=='backend.py' and backend is not None:data=backend
            (package/name).write_bytes(data)
        manifest={'schema':'zee-pdf-plugin-1','purpose':M.PURPOSE,'libraries':M.LIBRARIES,'files':{n:M.digest((package/n).read_bytes()) for n in sorted(M.FILES)}}
        mf=package/'plugin.json';mf.write_text(json.dumps(manifest,sort_keys=True)+'\n')
        sig=command([a.verifier,'imzala','--anahtar',key,mf]);(package/'plugin.json.zee-imza').write_bytes(sig);publisher=json.loads(sig)['anahtar_kimligi']
        trust=root/'trust.json';trust.write_text(json.dumps({'schema':'zee-pdf-plugin-trust-1','purpose':M.PURPOSE,'publisher':publisher,'manifest_sha256':M.digest(mf.read_bytes()),'python_sha256':proof['runtime_sha256']})+'\n')
        bindings=a.output/'public-bindings'/str(number);bindings.mkdir(parents=True)
        for name, file in [('plugin.json',mf),('plugin.json.zee-imza',package/'plugin.json.zee-imza'),('trust.json',trust)]:shutil.copyfile(file,bindings/name)
        source=root/'source.dil';source.write_bytes((R/'golden/pdf-plugin/ana.dil').read_bytes());return root,package,trust,source
    root,package,trust,source=fixture('positive');code,value=host(package,trust,source,root/'pdf');assert code==0 and value['status']=='completed'
    inspection=json.loads(command([a.python,'-I','-B','-c','import sys,json;from pypdf import PdfReader;p=PdfReader(sys.argv[1]);r=p.pages[1]["/Resources"];fonts=[x.get_object() for x in r["/Font"].get_object().values()];embedded=any("/FontFile2" in x["/FontDescriptor"].get_object() for x in fonts if "/FontDescriptor" in x);images=sum(x.get_object().get("/Subtype")=="/Image" for x in r["/XObject"].get_object().values());print(json.dumps({"pages":len(p.pages),"text":"\\n".join(x.extract_text() for x in p.pages),"embedded_font":embedded,"images":images}))',root/'pdf/catalog.pdf']))
    assert inspection['pages']==3 and inspection['embedded_font'] and inspection['images']>=1
    proof['PDF_inspection']={k:inspection[k] for k in ['pages','embedded_font','images']}
    text=inspection['text']
    for expected in ['Anonim ürün kataloğu','Örnek ürün - ŞĞİıÇÖÜ','OWNED BASE COVER','OWNED BASE BACK COVER']:assert expected in text
    shutil.copytree(root/'pdf',a.output/'pdf');proof['cases'].append({'name':'actual-signed-Zee-request-receipt-continuation-and-PDF-text-pages','passed':True,'pages':3})
    for kind in ['purpose','key','signature','schema','size','runtime','double','unknown']:
        backend=None
        if kind in {'double','unknown'}:
            backend=(R/'scripts/pdf-katalog-host.py').read_text().replace('results = produce(job, args.fixture_root.resolve(strict=True))','with (args.output.parent / "invocations").open("a") as count: count.write("1\\n")\n    results = produce(job, args.fixture_root.resolve(strict=True))')
            if kind=='unknown':backend=backend.replace("'status': 'produced'","'status': 'unknown'")
            backend=backend.encode()
        root,package,trust,source=fixture(kind,backend)
        if kind=='purpose':
            value=json.loads(trust.read_text());value['purpose']='zee-compiler-release';trust.write_text(json.dumps(value))
        elif kind=='key':
            sig=command([a.verifier,'imzala','--anahtar',key2,package/'plugin.json']);(package/'plugin.json.zee-imza').write_bytes(sig)
        elif kind=='signature':
            value=json.loads((package/'plugin.json.zee-imza').read_text());value['ed25519']='0'*128;(package/'plugin.json.zee-imza').write_text(json.dumps(value))
        elif kind=='schema':source.write_text(source.read_text().replace('zee-pdf-plugin-request-1','unknown-request'))
        elif kind=='size':source.write_text(('"'+'x'*16385+'" diye sor\n'))
        elif kind=='runtime':
            value=json.loads(trust.read_text());value['python_sha256']='0'*64;trust.write_text(json.dumps(value))
        elif kind=='double':source.write_text(source.read_text().replace('"PDF_PLUGIN_DONE" yaz','cerceve yaz'))
        code,value=host(package,trust,source,root/'pdf');assert code==2 and value['status']=='rejected'
        if kind in {'double','unknown'}:
            assert (root/'invocations').read_text()=='1\n' and (root/'pdf/catalog.pdf').is_file()
            assert value['effect']==('produced_client_unconfirmed' if kind=='double' else 'unknown')
        else:assert not (root/'pdf').exists()
        proof['cases'].append({'name':'actual-reject-'+kind,'passed':True,'effect':value['effect']})
    assert pins=={n:M.digest(M.bounded(R/n,1048576)) for n in source_roles}
    assert json.loads(command([a.dil,'sürüm','--json']))==build
    assert M.digest(M.bounded(a.dil,67108864))==proof['binary_sha256'] and M.digest(M.bounded(a.verifier,67108864))==proof['verifier_sha256'] and M.digest(M.bounded(a.python.resolve(strict=True),67108864))==proof['runtime_sha256']
    assert command(['git','-C',R,'rev-parse','HEAD']).decode().strip()==proof['framework_head']
    proof.update(status='passed',source_pins_unchanged=True,source_files=[{'path':n,'sha256':v} for n,v in pins.items()],case_count=len(proof['cases']),failed=0)
except BaseException as error:
    proof.update(status='failed',failure_type=type(error).__name__)
    raise
finally:
    try:owner.close();proof['owned_groups_closed']=True
    except BaseException:proof.update(status='failed',owned_groups_closed=False);raise
    finally:
        private.cleanup();proof['synthetic_private_keys_removed']=not T.exists();(a.output/'raw-metadata.json').write_text(json.dumps(raw,indent=2)+'\n');(a.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
