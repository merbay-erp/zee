#!/usr/bin/env python3
"""K-376: adlı çoklu dinleyici devrini ve sd_notify'ı gerçek systemd ile ölçer.

Birim şablonları docs/adli-dinleyici-devri.md içindeki üç ```ini bloğundan
(soket, web hizmeti, K-381 bağlantı hizmeti) okunur (belgelenen şablon =
ölçülen şablon). K-381 bağlantı koşusu dogfood/urun-h/baglanti-yankisi
programını iki adlı soketle çalıştırır ve gerçek SMTP benzeri oturum ile
SIGTERM boşaltmasını ölçer. Dört adlı soket (iki IPv4, iki
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
    if len(bloklar) != 3:
        raise SystemExit("belgede tam üç ini bloğu (socket, web service, bağlantı service) bekleniyor")
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

    def __init__(self, dil, proje, kapi, soketler, ipv6_kipi="ipv6-only", baglanti=False):
        self.soketler = soketler  # [(ad, ListenStream)]
        # Koşu başına benzersiz birim adı: durdurulmakta olan önceki birimle
        # ya da daemon-reload sırasında eski tanımla karışmaz.
        SIRA[0] += 1
        self.onek = onek = f"{ONEK}-{os.getpid()}-{SIRA[0]}"
        soket_sablonu, web_sablonu, baglanti_sablonu = sablonlar()
        hizmet_sablonu = baglanti_sablonu if baglanti else web_sablonu
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


def smtp_oturumu(aile, adres, kapi):
    """Karşılama, NOOP, çıplak LF reddi ve QUIT; dönen satırlar."""
    satirlar = []
    with socket.socket(aile, socket.SOCK_STREAM) as s:
        s.settimeout(20)
        s.connect((adres, kapi))
        dosya = s.makefile("rb")
        oku = lambda: dosya.readline().decode("utf-8", "replace").rstrip("\r\n")
        satirlar.append(oku())
        s.sendall(b"NOOP\r\n")
        satirlar.append(oku())
        s.sendall(b"MAIL FROM:<a@b>\n")
        satirlar.append(oku())
        s.sendall(b"QUIT\r\n")
        satirlar.append(oku())
        satirlar.append("EOF" if dosya.read(1) == b"" else "acik")
    return satirlar


def tls_kimligi(kok):
    """K-381 A2b: bu koşuya özel test CA + localhost yaprak (openssl)."""
    d = Path(tempfile.mkdtemp(prefix="zee-k381-tls-", dir=kok))
    ec = ["-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes"]
    def ossl(*a):
        subprocess.run(["openssl", *a], cwd=d, check=True, capture_output=True)
    ossl("req", "-x509", *ec, "-keyout", "ca.key", "-out", "ca.pem", "-days", "2",
         "-subj", "/CN=Zee K381 systemd CA", "-addext", "basicConstraints=critical,CA:TRUE",
         "-addext", "keyUsage=critical,keyCertSign,cRLSign",
         "-addext", "subjectKeyIdentifier=hash")
    ossl("req", *ec, "-keyout", "anahtar.pem", "-out", "y.csr", "-subj", "/CN=localhost")
    # Python 3.13 VERIFY_X509_STRICT: CA'da kritik keyUsage, yaprakta EKU/SAN/AKI.
    (d / "ek.cnf").write_text(
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\n"
        "extendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n"
        "subjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n")
    ossl("x509", "-req", "-in", "y.csr", "-CA", "ca.pem", "-CAkey", "ca.key", "-CAcreateserial",
         "-out", "yaprak.pem", "-days", "2", "-extfile", "ek.cnf")
    (d / "zincir.pem").write_bytes((d / "yaprak.pem").read_bytes() + (d / "ca.pem").read_bytes())
    return d


def credential_destegi():
    """Bu systemd credential dizinini hizmete bağlayabiliyor mu?"""
    yol = Path(tempfile.mkdtemp(prefix="zee-k381-cred-")) / "x"
    yol.write_text("x")
    r = sudo("systemd-run", "--wait", "--pipe", "-p", f"LoadCredential=X:{yol}",
             "sh", "-c", 'test -r "$CREDENTIALS_DIRECTORY/X"', kontrol=False)
    return r.returncode == 0


def tls_istemcisi(ca):
    import ssl
    baglam = ssl.create_default_context(cafile=str(ca))
    baglam.minimum_version = ssl.TLSVersion.TLSv1_2
    return baglam


def tls_oturumu(baglam, smtp, smtps):
    """Implicit TLS karşılama + STARTTLS + STARTTLS enjeksiyonu."""
    sonuc = {}
    with socket.create_connection(("127.0.0.1", smtps), timeout=20) as ham:
        with baglam.wrap_socket(ham, server_hostname="localhost") as s:
            f = s.makefile("rb")
            sonuc["implicit"] = [f.readline().decode().rstrip("\r\n"), s.version()]
            s.sendall(b"QUIT\r\n")
            sonuc["implicit"].append(f.readline().decode().rstrip("\r\n"))
    with socket.create_connection(("127.0.0.1", smtp), timeout=20) as ham:
        f = ham.makefile("rb")
        f.readline()
        ham.sendall(b"STARTTLS\r\n")
        hazir = f.readline().decode().rstrip("\r\n")
        with baglam.wrap_socket(ham, server_hostname="localhost") as s:
            g = s.makefile("rb")
            s.sendall(b"NOOP\r\n")
            sonuc["starttls"] = [hazir, s.version(), g.readline().decode().rstrip("\r\n")]
    with socket.create_connection(("127.0.0.1", smtp), timeout=20) as ham:
        f = ham.makefile("rb")
        f.readline()
        ham.sendall(b"STARTTLS\r\nNOOP\r\n")
        sonuc["enjeksiyon"] = [f.readline().decode().rstrip("\r\n"), f.read().decode()]
    return sonuc


def baglanti_kosusu(dil, kok, rapor):
    """K-381: dogfood bağlantı programı gerçek systemd altında (A2a + A2b TLS)."""
    smtp = bos_kapi()
    smtps = bos_kapi()
    soketler = [("smtp_v4", f"127.0.0.1:{smtp}"), ("smtp_v6", f"[::1]:{smtp}"),
                ("smtps_v4", f"127.0.0.1:{smtps}"), ("smtps_v6", f"[::1]:{smtps}")]
    kimlik = tls_kimligi(kok)
    sonuc = {"soketler": soketler}
    proje = Path(tempfile.mkdtemp(prefix="zee-k381-", dir=kok))
    proje.chmod(0o755)
    kaynak = KOK / "dogfood/urun-h/baglanti-yankisi"
    pd = (kaynak / "proje.dil").read_text(encoding="utf-8")
    liste = ", ".join(f'"{ad} {adres}"' for ad, adres in soketler)
    pd = re.sub(r"^dinleyiciler .*$", f"dinleyiciler {liste} listesi olsun", pd, flags=re.M)
    (proje / "proje.dil").write_text(pd, encoding="utf-8")
    shutil.copy(kaynak / "ana.dil", proje / "ana.dil")
    kurulum = Kurulum(dil, proje, 0, soketler, baglanti=True)
    hizmet = BIRIM_KOKU / f"{kurulum.onek}.service"
    birim = kurulum.birimler[hizmet]
    zincir, anahtar = kimlik / "zincir.pem", kimlik / "anahtar.pem"
    if credential_destegi():
        sonuc["kimlik_kaynagi"] = "LoadCredential"
        birim = birim.replace(
            "[Service]\n",
            "[Service]\n"
            f"LoadCredential=POSTA_TLS_ZINCIRI:{zincir}\n"
            f"LoadCredential=POSTA_TLS_ANAHTARI:{anahtar}\n", 1)
    else:
        # Konteyner systemd'si credential bağlaması kuramıyor: aynı PID'le
        # (exec) ortam değişkeni kaynağına düşülür (K-339 ikinci kaynak).
        sonuc["kimlik_kaynagi"] = "ortam (credential desteklenmiyor)"
        satir = next(x for x in birim.splitlines() if x.startswith("ExecStart="))
        komut = satir.split("=", 1)[1]
        dolar = "$" + "$"
        sarmal = ("ExecStart=/bin/sh -c 'POSTA_TLS_ZINCIRI=\"" + dolar + f"(cat {zincir})\" "
                  "POSTA_TLS_ANAHTARI=\"" + dolar + f"(cat {anahtar})\" exec {komut}'")
        birim = birim.replace(satir, sarmal, 1)
    kurulum.birimler[hizmet] = birim
    with kurulum as k:
        r = k.baslat()
        sonuc["start_rc"] = r.returncode
        durum = k.goster("ActiveState", "MainPID", "Type")
        sonuc["durum"] = durum
        pid = durum.get("MainPID", "0")
        status = sudo("cat", f"/proc/{pid}/status", kontrol=False).stdout
        alan = {x.split(":", 1)[0]: x.split(":", 1)[1].split() for x in status.splitlines() if ":" in x}
        sonuc["uid"] = alan.get("Uid", ["?"])[0]
        sonuc["cap_eff"] = (alan.get("CapEff") or ["?"])[0]
        try:
            sonuc["v4"] = smtp_oturumu(socket.AF_INET, "127.0.0.1", smtp)
            sonuc["v6"] = smtp_oturumu(socket.AF_INET6, "::1", smtp)
            sonuc["tls"] = tls_oturumu(tls_istemcisi(kimlik / "ca.pem"), smtp, smtps)
        except OSError as hata:
            sonuc["oturum_hatasi"] = str(hata)
        # Açık bağlantı varken durdurma: bekleyen okuma `kapanıyor` alır.
        try:
            with socket.create_connection(("127.0.0.1", smtp), timeout=20) as s:
                dosya = s.makefile("rb")
                sonuc["bosaltma_karsilama"] = dosya.readline().decode().rstrip("\r\n")
                dur = subprocess.Popen(["sudo", "systemctl", "stop", f"{k.onek}.service"])
                sonuc["bosaltma_satiri"] = dosya.readline().decode().rstrip("\r\n")
                dur.wait(timeout=60)
                sonuc["stop_rc"] = dur.returncode
        except (OSError, subprocess.TimeoutExpired) as hata:
            sonuc["bosaltma_hatasi"] = str(hata)
        sonuc["sonrasi"] = k.goster("Result", "ExecMainStatus", "ActiveState")
        sonuc["gunluk"] = k.gunluk()[-4000:]
    beklenen = ["220 mx.ornek.test ESMTP", "250 2.0.0 tamam", "500 5.5.2 sözdizimi hatası",
                "221 2.0.0 hoşça kal", "EOF"]
    sonuc["gecti"] = (
        sonuc["start_rc"] == 0
        and sonuc["durum"].get("ActiveState") == "active"
        and sonuc["durum"].get("Type") == "notify"
        and sonuc["uid"] not in ("0", "?")
        and sonuc["cap_eff"] == "0000000000000000"
        and sonuc.get("v4") == beklenen
        and sonuc.get("v6") == beklenen
        and sonuc.get("bosaltma_satiri") == "421 4.3.2 hizmet kapanıyor"
        and sonuc.get("stop_rc") == 0
        and sonuc["sonrasi"].get("Result") == "success"
        and sonuc["sonrasi"].get("ExecMainStatus") == "0"
        and "bağlantı kabul ediliyor" in sonuc["gunluk"]
        and sonuc.get("tls", {}).get("implicit", [None])[0] == "220 mx.ornek.test ESMTP"
        and sonuc["tls"]["implicit"][1] in ("TLSv1.2", "TLSv1.3")
        and sonuc["tls"]["implicit"][2] == "221 2.0.0 hoşça kal"
        and sonuc["tls"]["starttls"] == ["220 2.0.0 TLS için hazır", "TLSv1.3", "250 2.0.0 tamam"]
        and sonuc["tls"]["enjeksiyon"] == ["220 2.0.0 TLS için hazır", ""]
    )
    rapor["baglanti"] = sonuc
    return sonuc["gecti"]


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
        sonuclar.append(denemeli(rapor, "baglanti", lambda: baglanti_kosusu(dil, kok, rapor)))
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
            if k in ("systemd", "olumlu", "fazla_bildirim", "v6only_kapali", "baglanti", "durum")}
    print(json.dumps(ozet, ensure_ascii=False))
    if rapor["durum"] != "gecti":
        for anahtar in ("olumlu", "fazla_bildirim", "v6only_kapali", "baglanti"):
            if isinstance(rapor.get(anahtar), dict) and not rapor[anahtar].get("gecti"):
                print(f"--- {anahtar}\n{json.dumps(rapor[anahtar], ensure_ascii=False, indent=2)}",
                      file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
