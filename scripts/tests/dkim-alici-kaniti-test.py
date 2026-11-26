#!/usr/bin/env python3
"""K-416: dkim-alici-kaniti.py sentetik .eml testleri (stdlib, ağsız)."""

import hashlib
import importlib.util
import pathlib
import subprocess
import sys
import tempfile
import unittest

KOK = pathlib.Path(__file__).resolve().parents[2]
BETIK = KOK / "scripts" / "dkim-alici-kaniti.py"
SPEC = importlib.util.spec_from_file_location("kanit", BETIK)
K = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(K)
SHA = "a" * 40

# Sentetik ileti: alıcı adresi, IP ve Received zinciri çıktıya sızmamalı.
ILETI = (
    "Authentication-Results: mx.alici.example;\r\n"
    "       dkim=pass header.i=@posta.gonderen.example header.s=z2026a header.b=AbCd;\r\n"
    "       spf=pass (alici.example: 192.0.2.25 izinli) smtp.mailfrom=posta.gonderen.example;\r\n"
    "       dmarc=pass (p=REJECT sp=REJECT dis=NONE) header.from=gonderen.example\r\n"
    "Authentication-Results: sahte.example; dkim=fail\r\n"
    "Received: from mx1.gonderen.example (mx1.gonderen.example [192.0.2.25])\r\n"
    "        by mx.alici.example with ESMTPS id abc for <kisi@alici.example>\r\n"
    "DKIM-Signature: v=1; a=ed25519-sha256; d=gonderen.example; s=z2026a; b=ZZZ\r\n"
    "From: Gonderen <bilgi@gonderen.example>\r\n"
    "To: kisi@alici.example\r\n"
    "Subject: Deneme\r\n"
    "\r\n"
    "Gizli govde metni\r\n"
)
SIZMAMALI = ["gonderen.example", "alici.example", "192.0.2.25", "z2026a", "kisi@", "Gizli govde"]


def ileti(ar_degeri=None, from_="Gonderen <bilgi@gonderen.example>"):
    metin = ILETI.replace("From: Gonderen <bilgi@gonderen.example>", f"From: {from_}")
    if ar_degeri is not None:
        bas = metin.index("Authentication-Results: mx.alici.example;")
        son = metin.index("Authentication-Results: sahte.example")
        metin = metin[:bas] + f"Authentication-Results: {ar_degeri}\r\n" + metin[son:]
    return metin.encode()


def header_d(deger):
    return deger.replace("header.i=@posta.gonderen.example", "header.d=posta.gonderen.example")


class Testler(unittest.TestCase):
    def calistir(self, baytlar, *ek):
        with tempfile.TemporaryDirectory() as dizin:
            yol = pathlib.Path(dizin) / "ileti.eml"
            yol.write_bytes(baytlar)
            return subprocess.run(
                [sys.executable, "-I", str(BETIK), str(yol), "--saglayici", "Gmail",
                 "--zee-sha", SHA, "--tarih", "2026-11-26", *ek],
                capture_output=True, text=True, check=False,
            )

    def test_gecerli_kanit_anonim_satir_uretir(self):
        baytlar = header_d(ILETI).encode()
        sonuc = self.calistir(baytlar, "--baslik")
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)
        baslik, satir = sonuc.stdout.splitlines()
        self.assertEqual(baslik, K.BASLIK)
        self.assertEqual(satir.split("\t"), [
            "2026-11-26", "Gmail", "<kurum-alanı>", "<seçici>", "pass", "pass", "pass",
            "evet", hashlib.sha256(baytlar).hexdigest(), SHA,
        ])
        for parca in SIZMAMALI:
            self.assertNotIn(parca, sonuc.stdout + sonuc.stderr, parca)

    def test_yalniz_en_ustteki_alici_basligi_sayilir(self):
        # İletiye gönderen tarafından konmuş sahte "pass" başlığı alttadır.
        ust = "mx.alici.example; dkim=fail header.d=gonderen.example; dmarc=fail"
        metin = ILETI.replace("Authentication-Results: sahte.example; dkim=fail",
                              "Authentication-Results: sahte.example; dkim=pass header.d=gonderen.example; dmarc=pass")
        bas = metin.index("Authentication-Results: mx.alici.example;")
        son = metin.index("Authentication-Results: sahte.example")
        metin = metin[:bas] + f"Authentication-Results: {ust}\r\n" + metin[son:]
        sonuc = self.calistir(metin.encode())
        self.assertEqual(sonuc.returncode, 1)
        self.assertIn("dkim=fail", sonuc.stderr)

    def test_kanit_olmayan_durumlar_cikis_1(self):
        for ar, neden in [
            ("mx.alici.example; dkim=pass header.d=gonderen.example; dmarc=fail", "dmarc=fail"),
            ("mx.alici.example; dkim=neutral header.d=gonderen.example; dmarc=pass", "dkim=neutral"),
            ("mx.alici.example; dkim=pass header.d=baska.example; dmarc=pass", "hizalı değil"),
            ("mx.alici.example; spf=pass", "dkim=yok"),
        ]:
            sonuc = self.calistir(ileti(ar))
            self.assertEqual(sonuc.returncode, 1, ar)
            self.assertIn(neden, sonuc.stderr, ar)
            for parca in SIZMAMALI:
                self.assertNotIn(parca, sonuc.stderr, parca)
        yok = ILETI.replace("Authentication-Results: mx.alici.example;", "X-Eski:", 1)
        yok = yok.replace("Authentication-Results: sahte.example; dkim=fail\r\n", "")
        self.assertEqual(self.calistir(yok.encode()).returncode, 1)

    def test_birden_cok_dkim_sonucunda_pass_secilir_yorumlar_atlanir(self):
        ar = ("mx.alici.example; dkim=fail (eski imza; header.d=x) header.d=eski.example;"
              " dkim=pass header.d=gonderen.example header.s=s1; spf=softfail; dmarc=pass")
        sonuc = self.calistir(ileti(ar))
        self.assertEqual(sonuc.returncode, 0, sonuc.stderr)
        self.assertEqual(sonuc.stdout.split("\t")[4:8], ["pass", "softfail", "pass", "evet"])
        alt = ileti("mx.alici.example; dkim=pass header.d=gonderen.example; dmarc=pass",
                    from_="<bilgi@posta.gonderen.example>")
        self.assertEqual(self.calistir(alt).returncode, 0)

    def test_kullanim_hatalari_cikis_2(self):
        with tempfile.TemporaryDirectory() as dizin:
            yol = pathlib.Path(dizin) / "i.eml"
            yol.write_bytes(header_d(ILETI).encode())
            for ek in (["--saglayici", "Gmail; rm", "--zee-sha", SHA],
                       ["--saglayici", "Gmail", "--zee-sha", "kisa"],
                       ["--saglayici", "Gmail", "--zee-sha", SHA, "--tarih", "26.11.2026"]):
                sonuc = subprocess.run([sys.executable, "-I", str(BETIK), str(yol), *ek],
                                       capture_output=True, text=True, check=False)
                self.assertEqual(sonuc.returncode, 2, ek)
            sonuc = subprocess.run([sys.executable, "-I", str(BETIK), str(pathlib.Path(dizin) / "yok.eml"),
                                    "--saglayici", "Gmail", "--zee-sha", SHA],
                                   capture_output=True, text=True, check=False)
            self.assertEqual(sonuc.returncode, 2)
            self.assertNotIn(dizin, sonuc.stderr)

    def test_kayit_dosyasi_basligi_betikle_ayni(self):
        tsv = (KOK / "docs" / "dkim-alici-kanitlari-v1.tsv").read_text(encoding="utf-8")
        satirlar = [s for s in tsv.splitlines() if s and not s.startswith("#")]
        self.assertEqual(satirlar[0], K.BASLIK)
        for satir in satirlar[1:]:
            alanlar = satir.split("\t")
            self.assertEqual(len(alanlar), 10, satir)
            self.assertEqual(alanlar[2:4], ["<kurum-alanı>", "<seçici>"], satir)
            self.assertEqual(alanlar[4], "pass")
            self.assertEqual(alanlar[6], "pass")


if __name__ == "__main__":
    unittest.main()
