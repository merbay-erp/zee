#!/usr/bin/env python3
"""K276 preparation only: never executes payload, Docker, apt, ar or installer."""
import argparse,hashlib,json,os,platform,re,stat,struct,sys
from pathlib import Path
class Reject(ValueError):pass
LIMIT=134217728
ARCH={'amd64':(62,'x86_64','x86_64-unknown-linux-gnu'),'arm64':(183,'aarch64','aarch64-unknown-linux-gnu')}
REQUIRED={'binaries/dil','binaries/dillsp','binaries/SHA256SUMS','binaries/GIT_SHA','binaries/sbom.spdx.json','binaries/provenance.intoto.json','build.log'}
def need(c):
 if not c:raise Reject('package-input-rejected')
def pairs(rows):
 out={}
 for k,v in rows:need(k not in out);out[k]=v
 return out
def decode(b):
 try:return json.loads(b,object_pairs_hook=pairs,parse_constant=lambda _:(_ for _ in ()).throw(Reject()))
 except (ValueError,RecursionError,UnicodeError):raise Reject('record-rejected') from None
def hexstr(s,n):need(type(s) is str and re.fullmatch('[0-9a-f]{'+str(n)+'}',s));return s
def normal(name):
 need(type(name) is str and name and len(name)<256)
 need(not name.startswith('/') and all(x not in ('','.', '..') for x in name.split('/')) and '\\' not in name)
 return name
def file(path,limit=LIMIT,collect=False,prefix=False):
 path=Path(path);need(path.resolve(strict=True)==path)
 fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
 try:
  a=os.fstat(fd);need(stat.S_ISREG(a.st_mode) and a.st_nlink==1 and a.st_size<=limit and a.st_uid in {0,os.geteuid()} and not a.st_mode&0o022)
  h=hashlib.sha256();count=0;parts=[]
  while True:
   b=os.read(fd,min(65536,limit+1-count))
   if not b:break
   count+=len(b);need(count<=limit);h.update(b)
   if collect:parts.append(b)
   elif prefix and count==len(b):parts.append(b[:64])
  z=os.fstat(fd);p=path.lstat();key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
  need(key(a)==key(z)==key(p));return h.hexdigest(),b''.join(parts) if collect or prefix else None
 finally:os.close(fd)
def header(data,arch):
 need(arch in ARCH and len(data)>=64 and data[:4]==b'\x7fELF')
 need(data[4:7]==bytes([2,1,1]) and data[7] in (0,3))
 typ,machine,version=struct.unpack_from('<HHI',data,16);size=struct.unpack_from('<H',data,52)[0]
 need(typ in (2,3) and machine==ARCH[arch][0] and version==1 and size==64)
 return {'format':'ELF64-little','architecture':arch,'machine':machine,'type':typ}
def elf(path,arch):
 # Whole immutable snapshot hash/role first; only header parse is not executable validity.
 h,data=file(path,prefix=True)
 return h,header(data,arch)
def same_build(value,source):
 need(type(value) is dict and value.get('schema')=='zee-build-v1' and value.get('source_sha')==source and value.get('dirty') is False)
 need(type(value.get('version')) is str and re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:-dev)?',value['version']))
def delivery(path,source,report_hash,arch):
 path=Path(path);need(path.resolve(strict=True)==path and path.is_dir())
 rh,rb=file(path/'DELIVERY.json',4194304,True);need(rh==hexstr(report_hash,64));r=decode(rb);need(type(r) is dict)
 need(r.get('schema')=='zee-local-owned-delivery-1' and r.get('status')=='passed' and r.get('source_sha')==source and r.get('signed') is False and r.get('published') is False)
 need(r.get('two_source_zip_equal') is True and r.get('bundle_head')==source and r.get('restoration_fsck') is True);same_build(r['build_identity'],source)
 rows=r['files'];need(type(rows) is list and 7<=len(rows)<=64);mapping={}
 for row in rows:
  need(type(row) is dict and set(row)=={'path','sha256'});n=normal(row['path']);need(n not in mapping);mapping[n]=hexstr(row['sha256'],64)
 need(REQUIRED<=set(mapping))
 hashes=r['binary_sha256'];need(type(hashes) is dict and set(hashes)=={'dil','dillsp'})
 for n in REQUIRED:
  need(file(path/n,8388608 if n.endswith(('.json','.log')) else LIMIT)[0]==mapping[n])
 for n in ('dil','dillsp'):
  hexstr(hashes[n],64);need(mapping['binaries/'+n]==hashes[n]);need(elf(path/'binaries'/n,arch)[0]==hashes[n])
 sums=file(path/'binaries/SHA256SUMS',4096,True)[1];need(sums==''.join(hashes[n]+'  '+n+'\n' for n in ('dil','dillsp')).encode())
 need(file(path/'binaries/GIT_SHA',128,True)[1]==(source+'\n').encode())
 provenance=decode(file(path/'binaries/provenance.intoto.json',8388608,True)[1]);need(type(provenance) is dict);need(provenance.get('_type')=='https://in-toto.io/Statement/v1' and provenance.get('predicateType')=='https://slsa.dev/provenance/v1')
 pred=provenance['predicate'];ext=pred['buildDefinition']['externalParameters'];need(ext['gitCommit']==source and ext['platform']==ARCH[arch][2] and ext['rustc'].startswith('rustc 1.93.1 ') and type(ext['buildEnvironment']) is dict and pred['zeeTekrarUretim']['ikiTemizKlonEsit'] is True)
 need(type(provenance['subject']) is list and len(provenance['subject'])==2 and {x['name']:x['digest']['sha256'] for x in provenance['subject']}==hashes)
 byrows=pred['runDetails']['byproducts'];need(type(byrows) is list and len(byrows)==2);by={x['name']:x['digest']['sha256'] for x in byrows};need(by['SHA256SUMS']==by['SHA256SUMS.ikinci-klon']==mapping['binaries/SHA256SUMS'])
 graph=decode(file(path/'binaries/sbom.spdx.json',8388608,True)[1])['@graph'];need(type(graph) is list and len(graph)<=10000)
 need(all(type(x) is dict for x in graph));subjects=[x for x in graph if x.get('type')=='software_File'];need(len(subjects)==2 and {x['name']:x['verifiedUsing'][0]['hashValue'] for x in subjects}==hashes)
 log=file(path/'build.log',8388608,True)[1]
 for n,h in hashes.items():need(('-- '+n+' iki klonda eş: '+h).encode() in log)
 return {'schema':'zee-linux-packaging-preflight-1','status':'evidence-validated-only','source_sha':source,'architecture':arch,'binary_sha256':hashes,'delivery_sha256':rh,'payload_executed':False,'packaged':False,'installed':False,'container_executed':False}
def main():
 class Parser(argparse.ArgumentParser):
  def error(self,message):raise Reject('arguments')
 p=Parser();p.add_argument('--sha',required=True);p.add_argument('--delivery',type=Path,required=True);p.add_argument('--delivery-sha256',required=True);p.add_argument('--arch',choices=sorted(ARCH),required=True)
 a=p.parse_args();hexstr(a.sha,40);r=delivery(a.delivery,a.sha,a.delivery_sha256,a.arch)
 # A header+report does not authorize execution, package generation or architecture emulation.
 r['native_host_matches']=platform.system()=='Linux' and platform.machine()==ARCH[a.arch][1]
 print(json.dumps(r,separators=(',',':')));return 0
if __name__=='__main__':
 try:sys.exit(main())
 except (OSError,ValueError,TypeError,KeyError,IndexError,AttributeError,RecursionError):sys.stderr.write('linux-packaging-preflight-rejected\n');sys.exit(2)
