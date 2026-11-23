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
import os
import random
import re
import socket
import subprocess
import sys
import shutil
import tempfile
import time
from pathlib import Path

KOK = Path(__file__).resolve().parents[1]
ONEK = "zee-k376-olcum"
DENEME = 3
# Bu koşuda verilen kapılar: iki çağrı aynı kapıyı asla almaz.
VERILEN = set()
SIRA = [0]


class KurulumHatasi(Exception):
    """Soket birimi başlatılamadı; teşhis metnini taşır, yeni kapılarla denenir."""

    def __init__(self, neden, teshis):
        super().__init__(neden)
        self.teshis = teshis
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


def gecici_aralik():
    try:
        alt, ust = map(int, Path("/proc/sys/net/ipv4/ip_local_port_range").read_text().split())
        return alt, ust
    except (OSError, ValueError):
        return 32768, 60999


def kapi_bos_mu(kapi):
    """Kapı her adres ve ailede boş mu: 0.0.0.0 ve çift yığın [::] bağlanabilmeli."""
    for aile, adres in ((socket.AF_INET, "0.0.0.0"), (socket.AF_INET6, "::")):
        with socket.socket(aile, socket.SOCK_STREAM) as s:
            if aile == socket.AF_INET6:
                s.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 0)
            try:
                s.bind((adres, kapi))
            except OSError:
                return False
    return True


def bos_kapi(*_):
    """Çekirdeğin geçici (ephemeral) aralığı DIŞINDA, her ailede boş ve bu
    koşuda daha önce verilmemiş bir kapı. Geçici aralıktan seçip bırakmak
    yarışlıdır: aynı kapı bir istemci bağlantısına ya da sonraki seçime
    verilebilir (K-376 CI'da aralıklı `systemctl start …socket` hatası)."""
    alt, _ = gecici_aralik()
    ust = min(alt, 32768) - 1
    taban = 20000 if ust > 20000 + 2000 else 10000
    for _ in range(500):
        kapi = random.randint(taban, ust)
        if kapi not in VERILEN and kapi_bos_mu(kapi):
            VERILEN.add(kapi)
            return kapi
    raise SystemExit(f"{taban}-{ust} aralığında boş kapı bulunamadı")


def teshis_topla(birimler):
    """Başarısızlıkta status, birim günlüğü ve dinleyen soketler."""
    cikti = []
    komutlar = [["systemctl", "status", "--no-pager", "-l", *birimler],
                ["journalctl", "--no-pager", "-o", "short-precise", "-n", "80",
                 *[x for b in birimler for x in ("-u", b)]]]
    komutlar.append(["ss", "-ltnp"] if shutil.which("ss") else
                    ["cat", "/proc/net/tcp", "/proc/net/tcp6"])
    for komut in komutlar:
        r = sudo(*komut, kontrol=False)
        cikti.append(f"$ {' '.join(komut)}\n{r.stdout}{r.stderr}")
    return "\n".join(cikti)


def eski_birimleri_temizle():
    """Önceki (kesilmiş) koşudan kalan ölçüm birimleri koşuyu bozmasın."""
    eskiler = sorted(BIRIM_KOKU.glob(f"{ONEK}*"))
    adlar = [y.name for y in eskiler]
    r = sudo("systemctl", "list-units", "--all", "--plain", "--no-legend",
             f"{ONEK}*", kontrol=False)
    adlar += [satir.split()[0] for satir in r.stdout.splitlines() if satir.strip()]
    adlar = sorted(set(adlar))
    if adlar:
        sudo("systemctl", "stop", *adlar, kontrol=False)
        sudo("systemctl", "reset-failed", *adlar, kontrol=False)
    for yol in eskiler:
        sudo("rm", "-f", str(yol), kontrol=False)
    sudo("systemctl", "daemon-reload", kontrol=False)
    return adlar


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
        # Koşu başına benzersiz birim adı: durdurulmakta olan önceki birimle
        # ya da daemon-reload sırasında eski tanımla karışmaz.
        SIRA[0] += 1
        self.onek = onek = f"{ONEK}-{os.getpid()}-{SIRA[0]}"
        soket_sablonu, hizmet_sablonu = sablonlar()
        birim_adlari = " ".join(f"{onek}-{ad}.socket" for ad, _ in soketler)
        ortak = {"@KAPI@": str(kapi), "@DIL@": str(dil), "@PROJE_KOKU@": str(proje),
                 "@KULLANICI@": getpass.getuser(), "@ORIGIN@": ORIGIN,
                 "@SOKETLER@": birim_adlari}

        def doldur(metin, ek):
            for anahtar, deger in {**ortak, **ek}.items():
                metin = metin.replace(anahtar, deger)
            return metin.replace("zee-uygulama", onek)

        self.birimler = {}
        for ad, adres in soketler:
            icerik = doldur(soket_sablonu, {"@AD@": ad, "@ADRES@": adres})
            if ipv6_kipi != "ipv6-only":
                icerik = icerik.replace("BindIPv6Only=ipv6-only", f"BindIPv6Only={ipv6_kipi}")
            self.birimler[BIRIM_KOKU / f"{onek}-{ad}.socket"] = icerik
        self.birimler[BIRIM_KOKU / f"{onek}.service"] = doldur(hizmet_sablonu, {})

    def birim_adlari(self):
        return [f"{self.onek}.service"] + [f"{self.onek}-{ad}.socket" for ad, _ in self.soketler]

    def __enter__(self):
        for yol, icerik in self.birimler.items():
            subprocess.run(["sudo", "tee", str(yol)], input=icerik.encode(),
                           stdout=subprocess.DEVNULL, check=True)
        sudo("systemctl", "daemon-reload")
        for ad, adres in self.soketler:
            r = sudo("systemctl", "start", f"{self.onek}-{ad}.socket", kontrol=False)
            if r.returncode != 0:
                teshis = teshis_topla(self.birim_adlari())
                self.__exit__()
                raise KurulumHatasi(
                    f"{self.onek}-{ad}.socket ({adres}) başlatılamadı: rc={r.returncode} {r.stderr.strip()}",
                    teshis)
        return self

    def baslat(self):
        # Type=notify: start yalnız READY=1 gelince (ya da hata/zaman aşımı) döner.
        return sudo("systemctl", "start", f"{self.onek}.service", kontrol=False)

    def goster(self, *alanlar):
        r = sudo("systemctl", "show", f"{self.onek}.service", *(f"-p{a}" for a in alanlar))
        return dict(satir.split("=", 1) for satir in r.stdout.splitlines() if "=" in satir)

    def gunluk(self):
        # Yalnız bu koşunun çağrısı: önceki koşunun satırları karışmaz.
        kimlik = self.goster("InvocationID").get("InvocationID", "")
        if not kimlik:
            return ""
        return sudo("journalctl", "--no-pager", "-o", "cat",
                    f"_SYSTEMD_INVOCATION_ID={kimlik}", kontrol=False).stdout

    def __exit__(self, *_):
        birimler = self.birim_adlari()
        sudo("systemctl", "stop", *birimler, kontrol=False)
        sudo("systemctl", "reset-failed", *birimler, kontrol=False)
        for yol in self.birimler:
            sudo("rm", "-f", str(yol), kontrol=False)
        sudo("systemctl", "daemon-reload", kontrol=False)
        time.sleep(0.5)


def olumlu(dil, kok, rapor):
    kapi = bos_kapi()
    smtp = bos_kapi()
    imap = bos_kapi()
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
        dur = sudo("systemctl", "stop", f"{k.onek}.service", kontrol=False)
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


def denemeli(rapor, ad, kos):
    """Soket birimi başlatılamazsa teşhisi kaydedip yeni kapılarla en çok
    DENEME kez dener; ölçümün kendi başarısızlığı yeniden denenmez."""
    for deneme in range(1, DENEME + 1):
        try:
            return kos()
        except KurulumHatasi as hata:
            kayit = {"kosu": ad, "deneme": deneme, "neden": str(hata), "teshis": hata.teshis[-6000:]}
            rapor.setdefault("kurulum_hatalari", []).append(kayit)
            print(f"--- {ad} deneme {deneme}: {hata}\n{hata.teshis}", file=sys.stderr)
    raise SystemExit(f"{ad}: soket birimleri {DENEME} denemede başlatılamadı")


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
        kalan = eski_birimleri_temizle()
        if kalan:
            rapor["temizlenen_eski_birimler"] = kalan

        def fazla():
            iki = [("web_v4", f"127.0.0.1:{bos_kapi()}"), ("web_v6", f"[::1]:{bos_kapi()}")]
            return olumsuz(dil, kok, rapor, "fazla_bildirim", iki,
                           iki + [("smtp_v4", f"127.0.0.1:{bos_kapi()}")],
                           "ipv6-only", "devredilmeyen: smtp_v4")

        def v6only():
            # Linux belirli IPv6 adresine (::1) bağlanan sokette V6ONLY'yi kendisi
            # açar; BindIPv6Only=both yalnız joker adreste çift yığın üretir.
            iki = [("web_v4", f"127.0.0.1:{bos_kapi()}"), ("web_v6", f"[::]:{bos_kapi()}")]
            return olumsuz(dil, kok, rapor, "v6only_kapali", iki, iki,
                           "both", "IPV6_V6ONLY kapalı")

        sonuclar.append(denemeli(rapor, "olumlu", lambda: olumlu(dil, kok, rapor)))
        sonuclar.append(denemeli(rapor, "fazla_bildirim", fazla))
        sonuclar.append(denemeli(rapor, "v6only_kapali", v6only))
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
