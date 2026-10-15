#!/usr/bin/env python3
"""Actual installed stdio LSP; outer reviewed owner reserves the process group."""
import argparse,hashlib,json,os,selectors,subprocess,time
from pathlib import Path
def frames(values):
    result=b''
    for value in values:
        payload=json.dumps(value,separators=(',',':')).encode();result+=b'Content-Length: '+str(len(payload)).encode()+b'\r\n\r\n'+payload
    return result
def decode(raw):
    values=[]
    while raw:
        head,separator,body=raw.partition(b'\r\n\r\n');assert separator
        lines=head.split(b'\r\n');assert len(lines)==1 and lines[0].startswith(b'Content-Length: ')
        length=int(lines[0][16:]);assert 0<length<=65536 and len(body)>=length
        values.append(json.loads(body[:length]));raw=body[length:]
    return values
def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();assert not a.output.exists();binary=a.binary.resolve(strict=True);pin=hashlib.sha256(binary.read_bytes()).hexdigest()
    child=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,close_fds=True);sel=None;buffers={};end=time.monotonic()+7
    try:
        child.stdin.write(frames([{'jsonrpc':'2.0','id':1,'method':'initialize','params':{'processId':os.getpid(),'rootUri':None,'capabilities':{}}},{'jsonrpc':'2.0','method':'initialized','params':{}},{'jsonrpc':'2.0','id':2,'method':'shutdown'},{'jsonrpc':'2.0','method':'exit'}]));child.stdin.close()
        sel=selectors.DefaultSelector()
        for name,pipe in [('stdout',child.stdout),('stderr',child.stderr)]:sel.register(pipe,selectors.EVENT_READ,name);buffers[name]=bytearray()
        while sel.get_map():
            assert time.monotonic()<end
            for key,_ in sel.select(.05):
                block=os.read(key.fileobj.fileno(),4096)
                if not block:sel.unregister(key.fileobj)
                else:buffers[key.data].extend(block);assert len(buffers[key.data])<=65536
        code=child.wait(timeout=max(.001,end-time.monotonic()));assert code==0 and not buffers['stderr'];replies=decode(bytes(buffers['stdout']));assert len(replies)==2 and {r['id'] for r in replies}=={1,2}
        byid={r['id']:r for r in replies};assert byid[1]['jsonrpc']=='2.0' and isinstance(byid[1]['result']['capabilities'],dict) and byid[2]['result'] is None
        assert hashlib.sha256(binary.read_bytes()).hexdigest()==pin
        a.output.write_text(json.dumps({'schema':'zee-owned-installed-lsp-smoke-1','passed':True,'binary_sha256':pin,'exit':code,'stderr_bytes':0,'response_ids':[1,2],'raw_replies':replies,'request_boundary':'shutdown and exit omit absent params; null is not structured JSON-RPC params'},indent=2)+'\n')
    finally:
        if sel is not None:sel.close()
        if child.returncode is None:
            child.kill();child.wait(timeout=2)
        for pipe in (child.stdin,child.stdout,child.stderr):
            if not pipe.closed:pipe.close()
if __name__=='__main__':main()
