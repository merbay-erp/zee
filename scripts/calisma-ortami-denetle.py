#!/usr/bin/env python3
"""K-192: kurulu release ikililerinin sistem bağımlılığı ve Rust'sız PATH provası.

Yalnız Python standart kütüphanesi. Rapor, temiz makine/minimum OS kanıtı değildir.
Windows araç bulma: https://github.com/microsoft/vswhere/wiki/Find-VC
DLL listesi: https://learn.microsoft.com/en-us/cpp/build/reference/dependents
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile


def calistir(komut, *, ortam=None, girdi=None):
    sonuc = subprocess.run(komut, input=girdi, env=ortam, capture_output=True, timeout=30)
    if sonuc.returncode != 0:
        raise ValueError(f"Komut başarısız ({sonuc.returncode}): {komut[0]}: "
                         + sonuc.stderr.decode("utf-8", errors="replace")[:2000])
    return sonuc.stdout


def windows_dumpbin():
    bulunan = shutil.which("dumpbin")
    if bulunan:
        return Path(bulunan)
    programlar = os.environ.get("ProgramFiles(x86)")
    if not programlar:
        raise ValueError("Windows VC araçlarını bulmak için ProgramFiles(x86) yok")
    vswhere = Path(programlar) / "Microsoft Visual Studio/Installer/vswhere.exe"
    kurulum = calistir([str(vswhere), "-latest", "-products", "*", "-requires",
                       "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                       "-property", "installationPath"]).decode("utf-8-sig").strip()
    if not kurulum:
        raise ValueError("MSVC kurulumu bulunamadı")
    kok = Path(kurulum)
    surum = (kok / "VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt").read_text(encoding="utf-8-sig").strip()
    yol = kok / "VC/Tools/MSVC" / surum / "bin/Hostx64/x64/dumpbin.exe"
    if not yol.is_file():
        raise ValueError(f"dumpbin bulunamadı: {yol}")
    return yol


def bagimliliklar(ikili):
    ortam = dict(os.environ, LC_ALL="C", LANG="C")
    if sys.platform == "darwin":
        komut = ["/usr/bin/otool", "-L", str(ikili)]
        metin = calistir(komut, ortam=ortam).decode("utf-8")
        adlar = [satir.strip().split(" (compatibility version", 1)[0]
                 for satir in metin.splitlines()[1:] if satir.strip()]
        baslik = calistir(["/usr/bin/otool", "-l", str(ikili)], ortam=ortam).decode("utf-8")
        abi = [satir.strip() for satir in baslik.splitlines()
               if re.match(r"\s*(cmd LC_(BUILD_VERSION|VERSION_MIN_MACOSX)|minos |sdk |version )", satir)]
    elif sys.platform.startswith("linux"):
        arac = shutil.which("readelf")
        if not arac:
            raise ValueError("ELF bağımlılık denetimi için readelf gerekli")
        komut = [arac, "-d", str(ikili)]
        metin = calistir(komut, ortam=ortam).decode("utf-8")
        adlar = re.findall(r"\(NEEDED\).*\[([^\]]+)\]", metin)
        baslik = calistir([arac, "--version-info", str(ikili)], ortam=ortam).decode("utf-8")
        abi = sorted(set(re.findall(r"GLIBC(?:XX)?_[0-9.]+", baslik)))
    elif sys.platform == "win32":
        arac = str(windows_dumpbin())
        komut = [arac, "/DEPENDENTS", str(ikili)]
        metin = calistir(komut, ortam=ortam).decode("utf-8", errors="replace")
        adlar = [satir.strip() for satir in metin.splitlines()
                 if re.fullmatch(r"\s*[^\s]+\.dll\s*", satir, flags=re.IGNORECASE)]
        baslik = calistir([arac, "/HEADERS", str(ikili)], ortam=ortam).decode("utf-8", errors="replace")
        abi = [satir.strip() for satir in baslik.splitlines()
               if "version" in satir.lower() or "machine" in satir.lower()]
    else:
        raise ValueError(f"Desteklenmeyen platform: {sys.platform}")
    if not adlar:
        raise ValueError(f"Doğrudan sistem bağımlılığı okunamadı: {ikili.name}")
    return {"arac": komut[0], "dogrudan": sorted(set(adlar)), "abi_basliklari": abi,
            "ham_arac_ciktisi": metin}


def rustsiz_ortam():
    ortam = dict(os.environ)
    if sys.platform == "win32":
        kok = Path(os.environ["SystemRoot"])
        yollar = [str(kok / "System32"), str(kok)]
    else:
        yollar = ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
    ortam["PATH"] = os.pathsep.join(yollar)
    for arac in ("rustc", "cargo", "rustup"):
        if shutil.which(arac, path=ortam["PATH"]):
            raise ValueError(f"Yalıtılmış PATH hâlâ {arac} içeriyor")
    return ortam


def lsp_cerceve(mesaj):
    govde = json.dumps(mesaj, ensure_ascii=False).encode("utf-8")
    return f"Content-Length: {len(govde)}\r\n\r\n".encode("ascii") + govde


def lsp_cevaplari(cikti):
    cevaplar = []
    while cikti:
        baslik, ayrac, kalan = cikti.partition(b"\r\n\r\n")
        if not ayrac:
            raise ValueError("LSP yanıt başlığı eksik")
        uzunluklar = [satir.split(b":", 1)[1].strip() for satir in baslik.split(b"\r\n")
                      if satir.lower().startswith(b"content-length:")]
        if len(uzunluklar) != 1 or not uzunluklar[0].isdigit():
            raise ValueError("LSP yanıt uzunluğu geçersiz")
        uzunluk = int(uzunluklar[0])
        if uzunluk > 8 * 1024 * 1024 or uzunluk > len(kalan):
            raise ValueError("LSP yanıt gövdesi eksik veya aşırı büyük")
        cevap = json.loads(kalan[:uzunluk])
        if not isinstance(cevap, dict) or cevap.get("jsonrpc") != "2.0":
            raise ValueError("LSP yanıtı JSON-RPC nesnesi olmalı")
        if "id" in cevap and type(cevap["id"]) is not int:
            raise ValueError("LSP prova yanıt kimliği gönderilen tam sayı türünde olmalı")
        cevaplar.append(cevap)
        cikti = kalan[uzunluk:]
    return cevaplar


def prova(dil, dillsp):
    ortam = rustsiz_ortam()
    surum = calistir([str(dil), "sürüm"], ortam=ortam).decode("utf-8").strip()
    if not surum.startswith("dil "):
        raise ValueError("dil sürüm çıktısı geçersiz")
    with tempfile.TemporaryDirectory(prefix="zee-rustsiz-") as gecici:
        kaynak = Path(gecici) / "merhaba.dil"
        kaynak.write_text('"Dünyaya merhaba" yaz\n', encoding="utf-8")
        cikti = calistir([str(dil), "çalıştır", str(kaynak)], ortam=ortam)
        if cikti != "Dünyaya merhaba\n".encode("utf-8"):
            raise ValueError("Rust'sız PATH program çıktısı farklı")
    istekler = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize",
         "params": {"processId": None, "rootUri": None, "capabilities": {}}},
        {"jsonrpc": "2.0", "id": 2, "method": "shutdown"},
        {"jsonrpc": "2.0", "method": "exit"},
    ]
    cevaplar = lsp_cevaplari(calistir([str(dillsp)], ortam=ortam,
                                    girdi=b"".join(map(lsp_cerceve, istekler))))
    kimlikli = {c.get("id"): c for c in cevaplar if "id" in c}
    if (len([c for c in cevaplar if "id" in c]) != 2
            or set(kimlikli) != {1, 2}
            or not isinstance(kimlikli[1].get("result"), dict)
            or not isinstance(kimlikli[1]["result"].get("capabilities"), dict)
            or "error" in kimlikli[1] or "error" in kimlikli[2]
            or "result" not in kimlikli[2] or kimlikli[2]["result"] is not None):
        raise ValueError("Rust'sız PATH LSP initialize/shutdown yanıtı geçersiz")
    return {"path": ortam["PATH"], "rust_araclari": "yok", "surum": surum,
            "program_ciktisi": cikti.decode("utf-8"), "program_exit": 0,
            "lsp_initialize_shutdown_exit": "gecti"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artefakt", type=Path, required=True)
    parser.add_argument("--kurulu", type=Path, required=True)
    parser.add_argument("--rapor", type=Path, required=True)
    parser.add_argument("--beklenen-sha", required=True)
    args = parser.parse_args()
    rapor = {"sema": "zee-calisma-ortami-1", "sonuc": "kaldi",
             "tarih_utc": datetime.now(timezone.utc).isoformat(),
             "os": platform.platform(), "mimari": platform.machine(),
             "python": platform.python_version(), "ikililer": {},
             "sinir": "Rust araçları PATH dışında; temiz makine/minimum OS/SDK provenance kanıtı değildir."}
    try:
        sha = (args.artefakt / "GIT_SHA").read_text(encoding="ascii").strip()
        if not re.fullmatch(r"[0-9a-f]{40}", sha):
            raise ValueError("GIT_SHA exact 40 küçük hex olmalı")
        if sha != args.beklenen_sha:
            raise ValueError("Artefakt GIT_SHA beklenen kaynak commit'iyle uyuşmuyor")
        rapor["git_sha"] = sha
        ozetler = {}
        for satir in (args.artefakt / "SHA256SUMS").read_text(encoding="ascii").splitlines():
            ozet, ad = satir.split("  ", 1)
            if ad in ozetler or not re.fullmatch(r"[0-9a-f]{64}", ozet):
                raise ValueError("SHA256SUMS özeti veya tekilliği geçersiz")
            ozetler[ad] = ozet
        uzanti = ".exe" if sys.platform == "win32" else ""
        ikililer = [args.kurulu.resolve() / (ad + uzanti) for ad in ("dil", "dillsp")]
        for ikili in ikililer:
            ozet = hashlib.sha256(ikili.read_bytes()).hexdigest()
            if ozetler.get(ikili.name) != ozet:
                raise ValueError(f"Kurulu ikili artefakt özetiyle uyuşmuyor: {ikili.name}")
            rapor["ikililer"][ikili.name] = {"sha256": ozet, **bagimliliklar(ikili)}
        rapor["prova"] = prova(*ikililer)
        rapor["sonuc"] = "gecti"
    except (OSError, ValueError, subprocess.SubprocessError) as hata:
        rapor["hata"] = str(hata)[:4000]
    args.rapor.parent.mkdir(parents=True, exist_ok=True)
    args.rapor.write_text(json.dumps(rapor, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Çalışma ortamı: {rapor['sonuc']} → {args.rapor}")
    return 0 if rapor["sonuc"] == "gecti" else 1


if __name__ == "__main__":
    sys.exit(main())
