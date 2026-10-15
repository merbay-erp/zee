"""K267 short actual dirty-candidate credential/native listener composition."""
import argparse,hashlib,importlib.util,json,os,selectors,socket,subprocess,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('host',ROOT/'scripts/native-listener-host.py');H=importlib.util.module_from_spec(spec);spec.loader.exec_module(H)
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def request(port):
    with socket.create_connection(('127.0.0.1',port),timeout=2) as s:
        s.settimeout(2);s.sendall(b'GET /hazir HTTP/1.1\r\nHost: fixture.example\r\nForwarded: for=127.0.0.1;proto=https;host=fixture.example\r\nConnection: close\r\n\r\n');data=b''
        while True:
            chunk=s.recv(4096)
            if not chunk:break
            data+=chunk;assert len(data)<8192
        assert data.startswith(b'HTTP/1.1 200 ') and data.endswith(b'ok');return len(data)
def main():
    p=argparse.ArgumentParser();p.add_argument('--dil',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();binary=a.dil.resolve(strict=True);out=a.output.resolve();assert not out.exists();out.mkdir(mode=0o700,parents=True);build=json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5));pin=sha(binary);cases=[];sources={str(p.relative_to(ROOT)):sha(p) for p in [ROOT/'scripts/credential-baslat.py',ROOT/'scripts/native-capabilities.py',ROOT/'scripts/native-listener-host.py',ROOT/'scripts/worker-supervisor.py',Path(__file__).resolve()]};proof={'schema':'zee-credential-listener-native-1','status':'running','build':build,'binary_sha256':pin,'sources':sources,'cases':cases,'scope':'actual short owned local candidate; Build dirty status recorded, not final clean acceptance'}
    try:
        with tempfile.TemporaryDirectory(prefix='zee-credential-listener-') as t:
            root=Path(t).resolve();project=root/'project';project.mkdir(mode=0o700);credentials=root/'credentials';credentials.mkdir(mode=0o700);(credentials/'key').write_text('a5'*32);(credentials/'key').chmod(0o400)
            (project/'proje.dil').write_text('proje "anonymous-credential-listener" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu", "dosya-yazma", "kriptografi" listesi olsun\ngizli_anahtar_değişkeni "ZEE_K267_SYNTHETIC_KEY" olsun\n')
            (project/'ana.dil').write_text('parola "K267_PLAINTEXT_CANARY" olsun\nsifreli parolanın gizli şifrelenmişi olsun\nsonuç sifreli gizli çözmeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\ndeğilse\n    programı 1 ile bitir\nargümanlar komut satırından gelenler olsun\nreceipt argümanların ilki olsun\nreceipt dosyasına "{\\"schema\\":\\"zee-worker-readiness-1\\",\\"application_revision\\":\\"4444444444444444444444444444444444444444\\",\\"ready\\":true}" yaz\n8080 kapısında sunucu başlat\nGET "/hazir" adresine istek geldiğinde\n    "ok" yanıtını gönder\n')
            proof['fixture_hashes']={name:sha(project/name) for name in ['ana.dil','proje.dil']}
            host=H.Host(binary,project,build,'4'*40,pin,H.CredentialRoute(credentials,('ZEE_K267_SYNTHETIC_KEY=key',)));gens=[]
            try:
                g=host.spawn(metrics=True);gens.append(g);ready=host.ready(g);assert ready['pid']==g['native_pid'] and ready['pid']!=g['child'].pid and g['startup'] is None
                with socket.create_connection(('127.0.0.1',host.port),timeout=2) as s:
                    s.settimeout(.15);s.sendall(b'GET /hazir HTTP/1.1\r\nHost: fixture.example\r\n\r\n')
                    try:data=s.recv(1)
                    except socket.timeout:data=None
                    assert data is None
                host.activate(g);request(host.port);cases.append({'name':'actual-credential-crypto-trusted-startup-ready-before-admission','passed':True,'launcher_pid':g['child'].pid,'native_pid':g['native_pid'],'trusted_startup_record':g['startup_record']})
                g2=host.spawn(metrics=True,metric_first=True);gens.append(g2);host.ready(g2);assert g2['nonce']!=g['nonce'] and g2['native_pid']!=g['native_pid'];host.activate(g2);request(host.port);host.stop(g);request(host.port);cases.append({'name':'actual-fresh-generation-overlap-then-owned-old-group-cleanup','passed':True})
                raw=bytearray();end=time.monotonic()+2
                while time.monotonic()<end:
                    try:block=os.read(g2['metric'],65536)
                    except BlockingIOError:block=b''
                    if block:raw.extend(block)
                    assert len(raw)<=65536
                    if raw and raw.endswith(b'\n'):
                        rows=[json.loads(line) for line in raw.splitlines()]
                        if any(r['kind']==0 and r['v'][1]>=1 for r in rows):break
                    time.sleep(.01)
                rows=[json.loads(line) for line in raw.splitlines()];assert rows and all(r['schema']=='zee-native-metrics-1' and len(r['v'])==(22 if r['kind']==0 else 8) for r in rows) and any(r['kind']==0 and r['v'][1]>=1 for r in rows)
                assert raw and len(raw)<=65536 and b'K267_PLAINTEXT_CANARY' not in raw and b'a5a5a5a5' not in raw;(out/'metric-raw.jsonl').write_bytes(raw);cases.append({'name':'actual-designated-metrics-both-orders-no-secret-metadata','passed':True,'bytes':len(raw)})
                host.stop(g2)
                g3=host.spawn();gens.append(g3);host.ready(g3);host.activate(g3,('ACTIVATE '+'b'*32+'\n').encode());assert host.await_exit(g3,2)==0
                with socket.create_connection(('127.0.0.1',host.port),timeout=1) as s:
                    s.settimeout(.15);s.sendall(b'GET /hazir HTTP/1.1\r\nHost: fixture.example\r\n\r\n')
                    try:data=s.recv(1)
                    except socket.timeout:data=b''
                    assert data==b''
                host.stop(g3);cases.append({'name':'actual-wrong-activation-generation-no-admission','passed':True})
                g4=host.spawn(metrics=True,closed_metric_reader=True);gens.append(g4);host.ready(g4);host.activate(g4);request(host.port);host.stop(g4);cases.append({'name':'actual-closed-metric-reader-remains-best-effort','passed':True})
                assert not host.owner.children
            finally:
                for g in gens:
                    if not g['closed']:host.stop(g)
                host.cleanup()
        assert sha(binary)==pin and json.loads(subprocess.check_output([str(binary),'sürüm','--json'],timeout=5))==build;assert all(sha(ROOT/path)==digest for path,digest in sources.items());proof['status']='passed';proof['owned_groups_reaped']=True
    finally:(out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
if __name__=='__main__':main()
