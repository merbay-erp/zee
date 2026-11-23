#!/usr/bin/env python3
"""K-376: adlı çoklu dinleyici devrini ve sd_notify'ı gerçek systemd ile ölçer.

Birim şablonları docs/adli-dinleyici-devri.md içindeki iki ```ini bloğundan
okunur (belgelenen şablon = ölçülen şablon). Dört adlı soket (iki IPv4, iki
V6ONLY IPv6) kurulur; hizmet Type=notify, ayrıcalıksız kullanıcı, boş
capability kümesi ve MemoryDenyWriteExecute altında başlar. Ardından iki
olumsuz koşu: bildirimde fazladan ad ve V6ONLY kapalı IPv6 soketi.
Linux + systemd + parolasız sudo ister (CI ubuntu-24.04). Başarısızlıkta çıkış 1.
"""
import argparse
import getpass
import json
import re
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

KOK = Path(__file__).resolve().parents[1]
ONEK = "zee-k376-olcum"
ORIGIN = "https://panel.example"
BIRIM_KOKU = Path("/run/systemd/system")


def sudo(*komut, kontrol=True):
    return subprocess.run(["sudo", *komut], check=kontrol, capture_output=True, text=True)


def sablonlar():
    metin = (KOK / "docs/adli-dinleyici-devri.md").read_text(encoding="utf-8")
    bloklar = re.findall(r"```ini\n(.*?)```", metin, flags=re.S)
    if len(bloklar) != 2:
        raise SystemExit("belgede tam iki ini bloğu (socket, service) bekleniyor")
    return bloklar


def bos_kapi(aile=socket.AF_INET, adres="127.0.0.1"):
    with socket.socket(aile, socket.SOCK_STREAM) as s:
        if aile == socket.AF_INET6:
            s.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 1)
        s.bind((adres, 0))
        return s.getsockname()[1]


def baglanir(aile, adres, kapi):
    try:
        with socket.socket(aile, socket.SOCK_STREAM) as s:
            s.settimeout(5)
            s.connect((adres, kapi))
        return True
    except OSError:
        return False


def http_200(kapi):
    try:
        with socket.create_connection(("127.0.0.1", kapi), timeout=5) as s:
            s.settimeout(30)
            s.sendall(b"GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.40;proto=https;host=panel.example\r\nConnection: close\r\n\r\n")
            veri = b""
            while True:
                parca = s.recv(4096)
                if not parca:
                    break
                veri += parca
        return veri.startswith(b"HTTP/1.1 200 ")
    except OSError:
        return False


def proje_yaz(kok, dinleyiciler):
    liste = ", ".join(f'"{ad} {adres}"' for ad, adres in dinleyiciler)
    (kok / "proje.dil").write_text(
        'proje "k376-olcum" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\n'
        'giriş "ana.dil" olsun\nyetkinlikler "ağ-sunucusu" listesi olsun\n'
        f"dinleyiciler {liste} listesi olsun\n",
        encoding="utf-8",
    )
    (kok / "ana.dil").write_text(
        '8080 kapısında sunucu başlat\n\nGET "/" adresine istek geldiğinde\n    "tamam" yanıtını gönder\n',
        encoding="utf-8",
    )


class Kurulum:
    """Bir koşunun birimlerini yazar, başlatır ve her durumda temizler."""

    def __init__(self, dil, proje, kapi, soketler, ipv6_kipi="ipv6-only"):
        self.soketler = soketler  # [(ad, ListenStream)]
        soket_sablonu, hizmet_sablonu = sablonlar()
        birim_adlari = " ".join(f"{ONEK}-{ad}.socket" for ad, _ in soketler)
        ortak = {"@KAPI@": str(kapi), "@DIL@": str(dil), "@PROJE_KOKU@": str(proje),
                 "@KULLANICI@": getpass.getuser(), "@ORIGIN@": ORIGIN,
                 "@SOKETLER@": birim_adlari}

        def doldur(metin, ek):
            for anahtar, deger in {**ortak, **ek}.items():
                metin = metin.replace(anahtar, deger)
            return metin.replace("zee-uygulama", ONEK)

        self.birimler = {}
        for ad, adres in soketler:
            icerik = doldur(soket_sablonu, {"@AD@": ad, "@ADRES@": adres})
            if ipv6_kipi != "ipv6-only":
                icerik = icerik.replace("BindIPv6Only=ipv6-only", f"BindIPv6Only={ipv6_kipi}")
            self.birimler[BIRIM_KOKU / f"{ONEK}-{ad}.socket"] = icerik
        self.birimler[BIRIM_KOKU / f"{ONEK}.service"] = doldur(hizmet_sablonu, {})

    def __enter__(self):
        for yol, icerik in self.birimler.items():
            subprocess.run(["sudo", "tee", str(yol)], input=icerik.encode(),
                           stdout=subprocess.DEVNULL, check=True)
        sudo("systemctl", "daemon-reload")
        for ad, _ in self.soketler:
            sudo("systemctl", "start", f"{ONEK}-{ad}.socket")
        return self

    def baslat(self):
        # Type=notify: start yalnız READY=1 gelince (ya da hata/zaman aşımı) döner.
        return sudo("systemctl", "start", f"{ONEK}.service", kontrol=False)

    def goster(self, *alanlar):
        r = sudo("systemctl", "show", f"{ONEK}.service", *(f"-p{a}" for a in alanlar))
        return dict(satir.split("=", 1) for satir in r.stdout.splitlines() if "=" in satir)

    def gunluk(self):
        # Yalnız bu koşunun çağrısı: önceki koşunun satırları karışmaz.
        kimlik = self.goster("InvocationID").get("InvocationID", "")
        if not kimlik:
            return ""
        return sudo("journalctl", "--no-pager", "-o", "cat",
                    f"_SYSTEMD_INVOCATION_ID={kimlik}", kontrol=False).stdout

    def __exit__(self, *_):
        birimler = [f"{ONEK}.service"] + [f"{ONEK}-{ad}.socket" for ad, _ in self.soketler]
        sudo("systemctl", "stop", *birimler, kontrol=False)
        sudo("systemctl", "reset-failed", *birimler, kontrol=False)
        for yol in self.birimler:
            sudo("rm", "-f", str(yol), kontrol=False)
        sudo("systemctl", "daemon-reload", kontrol=False)
        time.sleep(0.5)


def olumlu(dil, kok, rapor):
    kapi = bos_kapi()
    smtp = bos_kapi()
    imap = bos_kapi(socket.AF_INET6, "::1")
    soketler = [("smtp_v4", f"127.0.0.1:{smtp}"), ("smtp_v6", f"[::1]:{smtp}"),
                ("imap_v4", f"127.0.0.1:{imap}"), ("imap_v6", f"[::1]:{imap}")]
    proje = Path(tempfile.mkdtemp(prefix="zee-k376-", dir=kok))
    proje.chmod(0o755)
    proje_yaz(proje, soketler)
    sonuc = {"soketler": soketler}
    with Kurulum(dil, proje, kapi, soketler) as k:
        basla = time.monotonic()
        r = k.baslat()
        sonuc["start_rc"] = r.returncode
        sonuc["start_ms"] = round((time.monotonic() - basla) * 1000, 1)
        durum = k.goster("ActiveState", "SubState", "MainPID", "Type", "NotifyAccess")
        sonuc["durum"] = durum
        pid = durum.get("MainPID", "0")
        status = sudo("cat", f"/proc/{pid}/status", kontrol=False).stdout
        alan = {s.split(":", 1)[0]: s.split(":", 1)[1].split() for s in status.splitlines() if ":" in s}
        sonuc["uid"] = alan.get("Uid", ["?"])[0]
        sonuc["cap_eff"] = (alan.get("CapEff") or ["?"])[0]
        sonuc["cap_bnd"] = (alan.get("CapBnd") or ["?"])[0]
        sonuc["baglanti"] = {
            ad: baglanir(socket.AF_INET6 if adres.startswith("[") else socket.AF_INET,
                         "::1" if adres.startswith("[") else "127.0.0.1",
                         int(adres.rsplit(":", 1)[1]))
            for ad, adres in soketler
        }
        sonuc["http_200"] = http_200(kapi)
        gunluk = k.gunluk()
        esleme = re.search(r"Zee adlı dinleyici devri: (\d+) dinleyici adla eşlendi \((.*)\)\.", gunluk)
        sonuc["esleme_satiri"] = esleme.group(0) if esleme else None
        dur = sudo("systemctl", "stop", f"{ONEK}.service", kontrol=False)
        sonuc["stop_rc"] = dur.returncode
        sonuc["sonrasi"] = k.goster("Result", "ExecMainStatus", "ActiveState")
        sonuc["gunluk"] = k.gunluk()[-4000:]
    beklenen = [f"{ad}={adres}" for ad, adres in soketler]
    gecti = (
        sonuc["start_rc"] == 0
        and sonuc["durum"].get("ActiveState") == "active"
        and sonuc["durum"].get("Type") == "notify"
        and sonuc["uid"] not in ("0", "?")
        and sonuc["cap_eff"] == "0000000000000000"
        and all(sonuc["baglanti"].values())
        and sonuc["http_200"]
        and sonuc["esleme_satiri"] is not None
        and sonuc["esleme_satiri"].startswith("Zee adlı dinleyici devri: 4 ")
        and all(b in sonuc["esleme_satiri"] for b in beklenen)
        and sonuc["stop_rc"] == 0
        and sonuc["sonrasi"].get("Result") == "success"
        and sonuc["sonrasi"].get("ExecMainStatus") == "0"
    )
    sonuc["gecti"] = gecti
    rapor["olumlu"] = sonuc
    return gecti


def olumsuz(dil, kok, rapor, ad, soketler, bildirim, ipv6_kipi, ifade):
    kapi = bos_kapi()
    proje = Path(tempfile.mkdtemp(prefix="zee-k376-", dir=kok))
    proje.chmod(0o755)
    proje_yaz(proje, bildirim)
    sonuc = {}
    with Kurulum(dil, proje, kapi, soketler, ipv6_kipi) as k:
        r = k.baslat()
        sonuc["start_rc"] = r.returncode
        time.sleep(0.5)
        sonuc["sonrasi"] = k.goster("Result", "ExecMainStatus", "ActiveState")
        sonuc["gunluk"] = k.gunluk()[-3000:]
    sonuc["gecti"] = (
        sonuc["start_rc"] != 0
        and sonuc["sonrasi"].get("ExecMainStatus") == "2"
        and ifade in sonuc["gunluk"]
        and "Öneri:" in sonuc["gunluk"]
        and "Sunucu dinliyor" not in sonuc["gunluk"]
    )
    rapor[ad] = sonuc
    return sonuc["gecti"]


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--dil", type=Path, required=True)
    a.add_argument("--cikti", type=Path, required=True)
    args = a.parse_args()
    dil = args.dil.resolve(strict=True)
    args.cikti.mkdir(parents=True, exist_ok=True)
    kok = Path(tempfile.mkdtemp(prefix="zee-k376-kok-"))
    kok.chmod(0o755)
    surum = subprocess.run(["systemctl", "--version"], capture_output=True, text=True).stdout.splitlines()
    rapor = {"schema": "zee-k376-systemd-olcum-1", "systemd": surum[0] if surum else "?"}
    sonuclar = []
    try:
        sonuclar.append(olumlu(dil, kok, rapor))
        p4, p6 = bos_kapi(), bos_kapi(socket.AF_INET6, "::1")
        iki = [("web_v4", f"127.0.0.1:{p4}"), ("web_v6", f"[::1]:{p6}")]
        sonuclar.append(olumsuz(dil, kok, rapor, "fazla_bildirim", iki,
                                iki + [("smtp_v4", f"127.0.0.1:{bos_kapi()}")],
                                "ipv6-only", "devredilmeyen: smtp_v4"))
        # Linux belirli IPv6 adresine (::1) bağlanan sokette V6ONLY'yi kendisi
        # açar; BindIPv6Only=both yalnız joker adreste çift yığın üretir.
        p4, p6 = bos_kapi(), bos_kapi(socket.AF_INET6, "::")
        iki = [("web_v4", f"127.0.0.1:{p4}"), ("web_v6", f"[::]:{p6}")]
        sonuclar.append(olumsuz(dil, kok, rapor, "v6only_kapali", iki, iki,
                                "both", "IPV6_V6ONLY kapalı"))
        rapor["durum"] = "gecti" if all(sonuclar) else "kaldi"
    except BaseException as hata:
        rapor["hata"] = str(hata)
        rapor["durum"] = "kaldi"
        raise
    finally:
        (args.cikti / "systemd-adli-soket-olcumu.json").write_text(
            json.dumps(rapor, ensure_ascii=False, indent=2) + "\n")
        subprocess.run(["rm", "-rf", str(kok)], check=False)
    ozet = {k: (v.get("gecti") if isinstance(v, dict) else v) for k, v in rapor.items()
            if k in ("systemd", "olumlu", "fazla_bildirim", "v6only_kapali", "durum")}
    print(json.dumps(ozet, ensure_ascii=False))
    if rapor["durum"] != "gecti":
        for anahtar in ("olumlu", "fazla_bildirim", "v6only_kapali"):
            if isinstance(rapor.get(anahtar), dict) and not rapor[anahtar].get("gecti"):
                print(f"--- {anahtar}\n{json.dumps(rapor[anahtar], ensure_ascii=False, indent=2)}",
                      file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
