#!/usr/bin/env python3
"""K-416 (RFC-0044 B3 kabul 2): gerçek alıcıda DKIM/DMARC kanıt satırı.

Alıcıdan alınan ham iletiyi (`.eml`, "orijinali göster") okur. En üstteki,
yani alıcının eklediği `Authentication-Results` başlığından (RFC 8601) dkim,
spf ve dmarc sonuçlarını, dkim'in `header.d`/`header.s` değerlerini çıkarır.
`docs/dkim-alici-kanitlari-v1.tsv` için anonim bir satır üretir.

Açık depoya hiçbir gerçek ad girmez. Satırda alan yerine `<kurum-alanı>`,
seçici yerine `<seçici>` yazılır. Alıcı adresi, IP, Received zinciri ve
ileti gövdesi basılmaz. Gerçek `.eml` özel depoda kalır; açık satır ona
yalnız dosyanın SHA-256'sıyla bağlanır.

Kullanım:
  python3 -I scripts/dkim-alici-kaniti.py ileti.eml --saglayici Gmail \
      --zee-sha <40 hane> [--tarih YYYY-AA-GG]

Çıkış 0: dkim=pass, dmarc=pass ve dkim alanı From ile hizalı; satır
stdout'a yazılır. Çıkış 1: kanıt değil (neden stderr'de, adsız). Çıkış 2:
kullanım ya da okunamayan dosya.
"""

import argparse
import datetime
import email
import email.policy
import email.utils
import hashlib
import re
import sys

BASLIK = "tarih\tsaglayici\talan\tsecici\tdkim\tspf\tdmarc\thizalama\teml_sha256\tzee_sha"
SAGLAYICI = re.compile(r"^[A-Za-z][A-Za-z0-9 .-]{0,39}$")
SONUC = re.compile(r"^(dkim|spf|dmarc)=([A-Za-z]+)\b(.*)$", re.S)
OZELLIK = re.compile(r"\b(header\.[a-z]+)=([^\s;]+)")


def yorumlari_at(deger):
    """RFC 5322 yorumlarını (iç içe dahil) ve tırnaklı dizgeleri ayıklar."""
    cikti, derinlik, tirnak = [], 0, False
    for karakter in deger:
        if tirnak:
            tirnak = karakter != '"'
            continue
        if karakter == '"' and derinlik == 0:
            tirnak = True
        elif karakter == "(":
            derinlik += 1
        elif karakter == ")" and derinlik:
            derinlik -= 1
        elif derinlik == 0:
            cikti.append(karakter)
    return "".join(cikti)


def ar_ayristir(deger):
    """`authserv-id; dkim=pass header.d=…; spf=…` → (authserv, sonuç listesi)."""
    parcalar = [p.strip() for p in yorumlari_at(deger).split(";")]
    authserv = parcalar[0].split()[0] if parcalar and parcalar[0] else ""
    sonuclar = []
    for parca in parcalar[1:]:
        eslesme = SONUC.match(parca)
        if eslesme:
            yontem, sonuc, kalan = eslesme.groups()
            ozellikler = dict(OZELLIK.findall(kalan))
            sonuclar.append((yontem, sonuc.lower(), ozellikler))
    return authserv, sonuclar


def from_alani(ileti):
    adres = email.utils.parseaddr(str(ileti.get("From", "")))[1]
    return adres.rsplit("@", 1)[-1].lower().rstrip(".") if "@" in adres else ""


def hizali(a, b):
    """DMARC gevşek hizalamasının PSL'siz yaklaşığı: alanlar aynı ya da biri
    ötekinin alt alanı. Asıl hüküm alıcının kendi `dmarc=` sonucudur."""
    return bool(a) and bool(b) and (a == b or a.endswith("." + b) or b.endswith("." + a))


def kanit(baytlar, saglayici, zee_sha, tarih):
    """(satır, hata) — hata None değilse kanıt değildir."""
    ileti = email.message_from_bytes(baytlar, policy=email.policy.compat32)
    basliklar = ileti.get_all("Authentication-Results") or []
    if not basliklar:
        return None, "Authentication-Results başlığı yok"
    _, sonuclar = ar_ayristir(str(basliklar[0]))
    def ilk(yontem):
        return next((s for s in sonuclar if s[0] == yontem), None)
    dkim_tumu = [s for s in sonuclar if s[0] == "dkim"]
    dkim = next((s for s in dkim_tumu if s[1] == "pass"), dkim_tumu[0] if dkim_tumu else None)
    spf, dmarc = ilk("spf"), ilk("dmarc")
    alan = (dkim[2].get("header.d", "") if dkim else "").lower().rstrip(".")
    hiza = "evet" if hizali(from_alani(ileti), alan) else "hayır"
    dkim_s = dkim[1] if dkim else "yok"
    dmarc_s = dmarc[1] if dmarc else "yok"
    satir = "\t".join([
        tarih, saglayici, "<kurum-alanı>", "<seçici>", dkim_s,
        spf[1] if spf else "yok", dmarc_s, hiza,
        hashlib.sha256(baytlar).hexdigest(), zee_sha,
    ])
    if dkim_s != "pass":
        return satir, f"dkim={dkim_s}"
    if dmarc_s != "pass":
        return satir, f"dmarc={dmarc_s}"
    if hiza != "evet":
        return satir, "dkim header.d From alanıyla hizalı değil"
    return satir, None


def main(argv=None):
    ayr = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ayr.add_argument("eml")
    ayr.add_argument("--saglayici", required=True)
    ayr.add_argument("--zee-sha", required=True)
    ayr.add_argument("--tarih", default=datetime.date.today().isoformat())
    ayr.add_argument("--baslik", action="store_true", help="TSV başlık satırını da bas")
    args = ayr.parse_args(argv)
    if not SAGLAYICI.match(args.saglayici):
        print("--saglayici yalnız sağlayıcı adı olmalı (ör. Gmail)", file=sys.stderr)
        return 2
    if not re.fullmatch(r"[0-9a-f]{40}", args.zee_sha):
        print("--zee-sha 40 haneli küçük harf commit SHA'sı olmalı", file=sys.stderr)
        return 2
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", args.tarih):
        print("--tarih YYYY-AA-GG olmalı", file=sys.stderr)
        return 2
    try:
        with open(args.eml, "rb") as dosya:
            baytlar = dosya.read(32 * 1024 * 1024 + 1)
    except OSError as hata:
        print(f".eml okunamadı ({hata.strerror})", file=sys.stderr)
        return 2
    if len(baytlar) > 32 * 1024 * 1024:
        print(".eml 32 MiB sınırını aşıyor", file=sys.stderr)
        return 2
    satir, hata = kanit(baytlar, args.saglayici, args.zee_sha, args.tarih)
    if hata:
        print(f"Kanıt değil: {hata}", file=sys.stderr)
        return 1
    if args.baslik:
        print(BASLIK)
    print(satir)
    return 0


if __name__ == "__main__":
    sys.exit(main())
