"""Owned test-only leader: activates one real native descendant, then exits9."""
import json,os,secrets,socket,subprocess,sys,time
binary,project,receipt=sys.argv[1:]
s=socket.socket(socket.AF_INET,socket.SOCK_STREAM,socket.IPPROTO_TCP);s.bind(('127.0.0.1',0));s.listen();s.setblocking(False);rr,rw=os.pipe();ar,aw=os.pipe();nonce=secrets.token_hex(16)
child=subprocess.Popen([binary,'çalıştır','--web-proxy','https://fixture.example','--web-worker-port',str(s.getsockname()[1]),'--web-listener-fd',str(s.fileno()),'--web-ready-fd',str(rw),'--web-activate-fd',str(ar),'--web-generation',nonce,project,receipt],pass_fds=(s.fileno(),rw,ar),stdin=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
os.close(rw);os.close(ar);data=b''
while True:
 chunk=os.read(rr,4096)
 if not chunk:break
 data+=chunk
 if len(data)>4096:os._exit(8)
value=json.loads(data);assert value['pid']==child.pid
os.write(aw,('ACTIVATE '+nonce+'\n').encode());os.close(aw);os.close(rr);s.close();os.write(1,b'owned-native-descendant-activated\n');time.sleep(.2);os._exit(9)
