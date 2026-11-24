#!/usr/bin/env python3
"""K-386 / RFC-0029: ikili aramanın sıra erişimiyle O(log n) olduğunu ölçer.

Her n için iki Zee programı üretilir ve `dil çalıştır` ile koşulur:

* `sira`  — yoklama `sayıların orta. öğesi` (RFC-0029, O(1) erişim);
* `sayac` — yoklama K-386 öncesi sayaç döngüsü (her yoklama O(n)).

Liste kurulumunun maliyeti (aynı program, sıfır arama) çıkarılır; kalan süre
arama sayısına bölünür. Güvenli profilin 64 MiB yığın sınırı nedeniyle en büyük
n 400 000tir. Çıktı Markdown tablosudur ve gözlemseldir: izlenen
performans tarihçesine (docs/performans-gecmisi-v2.tsv) yazılmaz.

Kullanım: python3 scripts/liste-sira-olcumu.py <dil-ikilisi> [--tekrar 3]
"""

import argparse
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

KURULUM = """sayılar boş liste olsun
1 den {n} e kadar her i için
    değer i ile 3 ün çarpımı olsun
    sayılara değeri ekle
toplam_adım 0 olsun
bulunanlar 0 olsun
"""

# Her arama listede gerçekten olan bir değeri arar: (k*7919 mod n)+1. sıra.
ARAMA = """1 den {k} e kadar her j için
    çarpım j ile 7919 un çarpımı olsun
    kalan çarpımın {n} e bölümünden kalanı olsun
    hedef_sıra kalan ile 1 in toplamı olsun
    aranan hedef_sıra ile 3 ün çarpımı olsun
    alt 1 olsun
    üst {n} olsun
    bitti yanlış olsun
    bitti doğru olana kadar tekrarla
        alt üstten büyükse
            bitti doğru olsun
        değilse
            toplam_adımı 1 artır
            toplam alt ile üst ün toplamı olsun
            orta toplamın 2 ye bölümü olsun
{yoklama}
            değer aranana eşitse
                bulunanları 1 artır
                bitti doğru olsun
            değilse değer aranandan küçükse
                alt orta ile 1 in toplamı olsun
            değilse
                üst orta ile 1 in farkı olsun
"""

YOKLAMA = {
    "sira": "            değer sayıların orta. öğesi olsun",
    "sayac": """            sayaç 1 olsun
            değer 0 olsun
            her sayı için
                sayaç ortaya eşitse
                    değer sayı olsun
                sayacı 1 artır""",
}

SON = '"bulunan: " ile bulunanlar ile ", adım: " ile toplam_adım yaz\n'


def program(n: int, k: int, kip: str) -> str:
    metin = KURULUM.format(n=n)
    if k:
        metin += ARAMA.format(n=n, k=k, yoklama=YOKLAMA[kip])
    return metin + SON


def kos(dil: str, kaynak: str, klasor: Path) -> tuple[float, str]:
    yol = klasor / "olcum.dil"
    yol.write_text(kaynak, encoding="utf-8")
    bas = time.perf_counter()
    sonuc = subprocess.run([dil, "çalıştır", str(yol)], capture_output=True, text=True)
    sure = time.perf_counter() - bas
    if sonuc.returncode != 0:
        raise SystemExit(f"dil çalıştır başarısız:\n{sonuc.stdout}{sonuc.stderr}")
    return sure, sonuc.stdout.strip()


def main() -> int:
    ayr = argparse.ArgumentParser()
    ayr.add_argument("dil")
    ayr.add_argument("--tekrar", type=int, default=3)
    arg = ayr.parse_args()
    satirlar = []
    with tempfile.TemporaryDirectory() as gecici:
        klasor = Path(gecici)
        for kip, olcekler, k in (
            ("sira", (1_000, 10_000, 100_000, 400_000), 2_000),
            ("sayac", (1_000, 2_000, 4_000, 8_000), 20),
        ):
            for n in olcekler:
                taban = statistics.median(
                    kos(arg.dil, program(n, 0, kip), klasor)[0] for _ in range(arg.tekrar)
                )
                sureler, cikti = [], ""
                for _ in range(arg.tekrar):
                    sure, cikti = kos(arg.dil, program(n, k, kip), klasor)
                    sureler.append(sure)
                bulunan, adim = (int(p.split(": ")[1]) for p in cikti.split(", "))
                if bulunan != k:
                    print(f"HATA: {kip} n={n} {bulunan}/{k} bulundu", file=sys.stderr)
                    return 1
                arama_us = (statistics.median(sureler) - taban) / k * 1e6
                satirlar.append((kip, n, k, adim / k, arama_us))
    print("| yoklama | n | arama | ort. adım | arama başına µs |")
    print("|---|---:|---:|---:|---:|")
    for kip, n, k, adim, us in satirlar:
        print(f"| {kip} | {n} | {k} | {adim:.2f} | {us:.1f} |")
    return 0


if __name__ == "__main__":
    sys.exit(main())
