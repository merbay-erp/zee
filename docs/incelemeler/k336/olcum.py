"""K-336 gözlemsel ölçüm: tek komutla başlayan N işçili grupta eşzamanlılık 1 ve 4.

Kullanım: python3 -I -B olcum.py <dil-ikilisi> <çıktı.json> [tur]
Her senaryo yeni geçici kopyada, kendi portuyla çalışır; grup SIGTERM ile kapanır.
Gözlemseldir (ADR-045/049 tabanı değil); paylaşılan makinede hard eşik yoktur.
"""
import concurrent.futures as cf, hashlib, http.client, json, os, platform, shutil
import signal, socket, subprocess, sys, tempfile, time

DIL, CIKTI = os.path.abspath(sys.argv[1]), sys.argv[2]
TUR = int(sys.argv[3]) if len(sys.argv) > 3 else 3
KAYNAK = os.path.join(os.path.dirname(os.path.abspath(__file__)), "yuk.dil")


def bos_kapi():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def istek(kapi, i):
    c = http.client.HTTPConnection("127.0.0.1", kapi, timeout=60)
    t = time.perf_counter()
    c.request("GET", "/is", headers={"Host": "urun.test",
              "Forwarded": f"for=203.0.113.{i % 200 + 1};proto=https;host=urun.test"})
    r = c.getresponse(); r.read(); c.close()
    return r.status, time.perf_counter() - t


def yuk(kapi, adet, esz):
    t0 = time.perf_counter()
    with cf.ThreadPoolExecutor(esz) as ex:
        sonuc = list(ex.map(lambda i: istek(kapi, i), range(adet)))
    sure = time.perf_counter() - t0
    lat = sorted(x[1] for x in sonuc)
    return {"istek": adet, "eszamanlilik": esz, "istek_sn": round(adet / sure, 3),
            "p50_ms": round(lat[len(lat) // 2] * 1000, 1),
            "p95_ms": round(lat[int(len(lat) * .95)] * 1000, 1),
            "durumlar": sorted({x[0] for x in sonuc})}


def senaryo(isci):
    kok = tempfile.mkdtemp(prefix="zee-k336-")
    shutil.copy(KAYNAK, os.path.join(kok, "yuk.dil"))
    kapi = bos_kapi()
    p = subprocess.Popen([DIL, "çalıştır", "--web-proxy", "https://urun.test",
                          "--web-worker-port", str(kapi), "--web-isci-sayisi", str(isci),
                          "yuk.dil"], cwd=kok, stdout=subprocess.DEVNULL,
                         stderr=subprocess.DEVNULL)
    try:
        son = time.time() + 20
        while True:
            try:
                istek(kapi, 0); break
            except OSError:
                if time.time() > son: raise
                time.sleep(0.05)
        for i in range(8): istek(kapi, i)  # ısınma, kayda girmez
        turlar = [{"c1": yuk(kapi, 40, 1), "c4": yuk(kapi, 80, 4)} for _ in range(TUR)]
    finally:
        p.send_signal(signal.SIGTERM); kod = p.wait(60); shutil.rmtree(kok, True)
    return {"isci": isci, "cikis": kod, "turlar": turlar}


def medyan(xs):
    xs = sorted(xs); return xs[len(xs) // 2]


sonuclar = [senaryo(1), senaryo(4)]
dort = sonuclar[1]["turlar"]
oran = [t["c4"]["istek_sn"] / t["c1"]["istek_sn"] for t in dort]
cikti = {
    "schema": "zee-k336-olcum-1",
    "binary_sha256": hashlib.sha256(open(DIL, "rb").read()).hexdigest(),
    "build": subprocess.run([DIL, "sürüm", "--json"], capture_output=True, text=True).stdout.strip(),
    "platform": platform.platform(), "cpu_sayisi": os.cpu_count(),
    "python": platform.python_version(), "workload_sha256":
        hashlib.sha256(open(KAYNAK, "rb").read()).hexdigest(),
    "tur": TUR, "isinma_istek": 8, "senaryolar": sonuclar,
    "dort_isci_c4_c1_orani": [round(x, 3) for x in oran],
    "dort_isci_c4_c1_orani_medyan": round(medyan(oran), 3),
    "kabul_esigi": 3.5,
}
json.dump(cikti, open(CIKTI, "w"), ensure_ascii=False, indent=2)
print(json.dumps({k: cikti[k] for k in ("dort_isci_c4_c1_orani", "dort_isci_c4_c1_orani_medyan")}))
