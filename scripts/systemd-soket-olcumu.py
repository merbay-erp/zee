#!/usr/bin/env python3
"""K-343: çok işçi grubunun gerçek systemd soket etkinleştirmesiyle yeniden
başlatılmasında istek kaybını ve kuyrukta bekleme süresini ölçer.

Birim şablonları docs/cok-isci-systemd-soketi.md içindeki iki ```ini
bloğundan okunur (belgelenen şablon = ölçülen şablon). Linux + systemd +
parolasız sudo ister (CI ubuntu-24.04). Başarısız istek varsa çıkış 1.
"""
import argparse
import getpass
import json
import os
import re
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

KOK = Path(__file__).resolve().parents[1]
BIRIM = "zee-k343-olcum"
ORIGIN = "https://panel.example"


def sudo(*komut):
    subprocess.run(["sudo", *komut], check=True)


def sablonlar():
    metin = (KOK / "docs/cok-isci-systemd-soketi.md").read_text(encoding="utf-8")
    bloklar = re.findall(r"```ini\n(.*?)```", metin, flags=re.S)
    if len(bloklar) != 2:
        raise SystemExit("belgede tam iki ini bloğu (socket, service) bekleniyor")
    return bloklar


def bos_kapi():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def istek(kapi, kimlik):
    basla = time.monotonic()
    try:
        with socket.create_connection(("127.0.0.1", kapi), timeout=5) as s:
            s.settimeout(60)
            s.sendall(
                f"GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for={kimlik};proto=https;host=panel.example\r\nConnection: close\r\n\r\n".encode()
            )
            veri = b""
            while True:
                parca = s.recv(4096)
                if not parca:
                    break
                veri += parca
        tamam = veri.startswith(b"HTTP/1.1 200 ")
        neden = veri[9:12].decode("ascii", "replace") if veri else "bos-yanit"
        # K-353: ret gövdesi kök nedeni söyler (yalnız durum kodu yetmedi).
        govde = veri.partition(b"\r\n\r\n")[2][:200].decode("utf-8", "replace")
    except OSError as hata:
        tamam, neden, govde = False, type(hata).__name__, str(hata)[:200]
    return basla, time.monotonic() - basla, tamam, neden, govde


def calisiyor_mu():
    durum = subprocess.run(["systemctl", "is-active", f"{BIRIM}.service"],
                           capture_output=True, text=True).stdout.strip()
    return durum in ("active", "activating", "reloading", "deactivating")


def hazir_bekle(kapi, kimlik, neden):
    # Hizmet çöktüyse 60 sn beklemeden çık; tanıyı çağıran yazdırır.
    son = time.monotonic() + 60
    while not istek(kapi, kimlik)[2]:
        if not calisiyor_mu():
            time.sleep(0.5)
            if not calisiyor_mu():
                raise SystemExit(f"{neden}: hizmet etkin değil (çöktü ya da başlamadı)")
        if time.monotonic() > son:
            raise SystemExit(f"{neden}: 60 sn içinde hazır olmadı")
        time.sleep(0.02)


def tani_topla():
    cikti = []
    for komut in (["systemctl", "status", "--no-pager", "-l", f"{BIRIM}.socket", f"{BIRIM}.service"],
                  ["journalctl", "--no-pager", "-n", "200", "-u", f"{BIRIM}.socket", "-u", f"{BIRIM}.service"]):
        r = subprocess.run(["sudo", *komut], capture_output=True, text=True)
        cikti.append(f"$ {' '.join(komut)}\n{r.stdout}{r.stderr}")
    return "\n".join(cikti)


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--dil", type=Path, required=True)
    a.add_argument("--cikti", type=Path, required=True)
    a.add_argument("--yeniden", type=int, default=5)
    a.add_argument("--istemci", type=int, default=4)
    args = a.parse_args()
    dil = args.dil.resolve(strict=True)
    args.cikti.mkdir(parents=True, exist_ok=True)
    kapi = bos_kapi()
    proje = Path(tempfile.mkdtemp(prefix="zee-k343-"))
    # Şablondaki ExecStart proje kökünü alır; gerçek ürün gibi proje.dil şart
    # (yoksa `dil` P006 ile çıkar ve hizmet hiç ayağa kalkmaz).
    (proje / "proje.dil").write_text(
        'proje "k343-olcum" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\n'
        'giriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu" listesi olsun\n',
        encoding="utf-8",
    )
    (proje / "ana.dil").write_text(
        '8080 kapısında sunucu başlat\n\nGET "/" adresine istek geldiğinde\n    "tamam" yanıtını gönder\n',
        encoding="utf-8",
    )
    degerler = {"@KAPI@": str(kapi), "@DIL@": str(dil), "@PROJE_KOKU@": str(proje),
                "@KULLANICI@": getpass.getuser(), "@ORIGIN@": ORIGIN}
    soket_sablonu, hizmet_sablonu = sablonlar()
    def doldur(metin):
        for anahtar, deger in degerler.items():
            metin = metin.replace(anahtar, deger)
        return metin.replace("zee-uygulama", BIRIM)
    birimler = {f"/run/systemd/system/{BIRIM}.socket": doldur(soket_sablonu),
                f"/run/systemd/system/{BIRIM}.service": doldur(hizmet_sablonu)}
    for yol, icerik in birimler.items():
        subprocess.run(["sudo", "tee", yol], input=icerik.encode(), stdout=subprocess.DEVNULL, check=True)
    rapor = {"schema": "zee-k343-systemd-olcum-1", "yeniden_baslatma": args.yeniden,
             "istemci": args.istemci, "birimler": {Path(k).name: v for k, v in birimler.items()}}
    kayit, dur, kilit = [], threading.Event(), threading.Lock()
    try:
        sudo("systemctl", "daemon-reload")
        sudo("systemctl", "start", f"{BIRIM}.socket")
        sudo("systemctl", "start", f"{BIRIM}.service")
        hazir_bekle(kapi, "203.0.113.30", "ilk başlatma")

        # Her istemci 2000 kimlik arasında döner: tek kimliğin uç nokta oran
        # sınırına (100/60 sn) takılması kayıp sanılmaz; anahtar sayısı da
        # varsayılan 32768 kapasitenin altında kalır.
        def istemci(n):
            sira = 0
            while not dur.is_set():
                no = n * 2000 + sira % 2000
                sira += 1
                sonuc = istek(kapi, f"198.18.{no // 250}.{no % 250 + 1}")
                with kilit:
                    kayit.append(sonuc)

        isler = [threading.Thread(target=istemci, args=(n,)) for n in range(args.istemci)]
        for t in isler:
            t.start()
        time.sleep(2)
        pencereler = []
        for _ in range(args.yeniden):
            basla = time.monotonic()
            sudo("systemctl", "restart", f"{BIRIM}.service")
            hazir_bekle(kapi, "203.0.113.31", "yeniden başlatma sonrası")
            # systemd varsayılanı StartLimitBurst=5 / StartLimitIntervalSec=10:
            # 10 sn'de altıncı başlatma birimi (ve soketi) start-limit-hit ile
            # düşürür. Başlatmalar en az 2.5 sn arayla yapılır.
            time.sleep(max(1.0, 2.5 - (time.monotonic() - basla)))
            pencereler.append((basla, time.monotonic()))
        dur.set()
        for t in isler:
            t.join()
        basarisiz = sum(1 for _, _, ok, _, _ in kayit if not ok)
        nedenler = {}
        ornekler = []
        ilk = pencereler[0][0] if pencereler else 0.0
        for t, sure, ok, neden, govde in kayit:
            if not ok:
                nedenler[neden] = nedenler.get(neden, 0) + 1
                if len(ornekler) < 10:
                    # Zaman ilk yeniden başlatmaya göredir; pencerelerle eşlenir.
                    ornekler.append({"t_sn": round(t - ilk, 3), "sure_ms": round(sure * 1000, 1),
                                     "neden": neden, "govde": govde})
        azami = [round(max((s for t, s, _, _, _ in kayit if a <= t <= b), default=0.0) * 1000, 1)
                 for a, b in pencereler]
        rapor.update({"istek": len(kayit), "basarisiz": basarisiz,
                      "basarisizlik_nedenleri": nedenler,
                      "basarisizlik_ornekleri": ornekler,
                      "yeniden_baslatma_pencereleri_sn": [[round(a - ilk, 3), round(b - ilk, 3)]
                                                          for a, b in pencereler],
                      "yeniden_baslatma_basina_en_uzun_istek_ms": azami,
                      "durum": "gecti" if basarisiz == 0 and len(kayit) > 0 else "kaldi"})
    except BaseException as hata:
        # Birimler silinmeden önce tanı: status + journal hem JSON'a hem stderr'e.
        rapor["hata"] = str(hata)
        rapor["tani"] = tani_topla()
        print(rapor["tani"], file=sys.stderr)
        raise
    finally:
        dur.set()
        subprocess.run(["sudo", "systemctl", "stop", f"{BIRIM}.service", f"{BIRIM}.socket"], check=False)
        for yol in birimler:
            subprocess.run(["sudo", "rm", "-f", yol], check=False)
        subprocess.run(["sudo", "systemctl", "daemon-reload"], check=False)
        (args.cikti / "systemd-soket-olcumu.json").write_text(json.dumps(rapor, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({k: rapor.get(k) for k in ("istek", "basarisiz", "basarisizlik_nedenleri", "basarisizlik_ornekleri", "yeniden_baslatma_basina_en_uzun_istek_ms", "durum")}, ensure_ascii=False))
    if rapor.get("durum") != "gecti":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
