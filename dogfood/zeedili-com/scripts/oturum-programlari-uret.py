#!/usr/bin/env python3
"""K-320: oturum yürütücüsünün sesli okuma programlarını golden/ kaynaklarından üretir.
Kullanım: oturum-programlari-uret.py [--denetle]  (denetle: üretilmiş dosya güncel değilse çıkış 1)."""
import json, sys
from pathlib import Path

SITE = Path(__file__).resolve().parents[1]
DEPO = SITE.parents[1]
HEDEF = SITE / "static/oturum/programlar.js"
# Kitteki sıra (docs/usability-kiti.md "Okutulacak programlar"); 26 yalnız profesyoneller.
SIRA = ["01", "02", "05", "06", "08", "12", "14", "23", "32", "07", "26"]

def uret():
    kayit = []
    for no in SIRA:
        yol = sorted((DEPO / "golden").glob(f"{no}-*.dil"))[0]
        kayit.append({"no": no, "dosya": f"golden/{yol.name}", "kod": yol.read_text(encoding="utf-8"),
                      "yalniz_profesyonel": no == "26"})
    return ("// Üretilmiş dosya — scripts/oturum-programlari-uret.py; elle düzenleme.\n"
            "window.OTURUM_PROGRAMLARI = " + json.dumps(kayit, ensure_ascii=False, indent=1) + ";\n")

metin = uret()
if "--denetle" in sys.argv:
    if not HEDEF.exists() or HEDEF.read_text(encoding="utf-8") != metin:
        print("oturum programları bayat: scripts/oturum-programlari-uret.py çalıştır", file=sys.stderr); sys.exit(1)
    print("oturum programları golden/ ile güncel")
else:
    HEDEF.write_text(metin, encoding="utf-8"); print(f"yazıldı: {HEDEF.relative_to(DEPO)}")
