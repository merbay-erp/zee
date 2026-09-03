#!/usr/bin/env python3
"""Tek bağlantılık PostgreSQL COMMIT belirsizliği hata-enjeksiyon proxy'si.

`before`, frontend COMMIT çerçevesini PostgreSQL'e iletmeden bağlantıyı keser.
`after`, COMMIT'i iletir; backend ReadyForQuery yanıtını alınca bütün COMMIT
yanıtını yutar ve istemci bağlantısını keser. Böylece istemci iki durumda da
aynı EOF'yi görürken veritabanı durumu sırasıyla rollback ve commit olur.
"""

from __future__ import annotations

import argparse
import json
import socket
import struct
import threading


def tam_oku(baglanti: socket.socket, uzunluk: int) -> bytes:
    parcalar: list[bytes] = []
    kalan = uzunluk
    while kalan:
        parca = baglanti.recv(kalan)
        if not parca:
            raise EOFError("bağlantı kapandı")
        parcalar.append(parca)
        kalan -= len(parca)
    return b"".join(parcalar)


def cerceve_oku(baglanti: socket.socket) -> tuple[bytes, bytes]:
    tur = tam_oku(baglanti, 1)
    uzunluk_baytlari = tam_oku(baglanti, 4)
    uzunluk = struct.unpack("!I", uzunluk_baytlari)[0]
    if uzunluk < 4 or uzunluk > 64 * 1024 * 1024:
        raise ValueError(f"geçersiz PostgreSQL çerçeve uzunluğu: {uzunluk}")
    govde = tam_oku(baglanti, uzunluk - 4)
    return tur, uzunluk_baytlari + govde


def kapat(*baglantilar: socket.socket) -> None:
    for baglanti in baglantilar:
        try:
            baglanti.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        try:
            baglanti.close()
        except OSError:
            pass


def saydam_baglanti(
    istemci: socket.socket, backend_host: str, backend_port: int
) -> None:
    backend = socket.create_connection((backend_host, backend_port), timeout=10)

    def aktar(kaynak: socket.socket, hedef: socket.socket) -> None:
        try:
            while parca := kaynak.recv(64 * 1024):
                hedef.sendall(parca)
        except OSError:
            pass
        finally:
            kapat(istemci, backend)

    on = threading.Thread(target=aktar, args=(istemci, backend), daemon=True)
    arka = threading.Thread(target=aktar, args=(backend, istemci), daemon=True)
    on.start()
    arka.start()
    on.join()
    arka.join()


def ana() -> int:
    ayr = argparse.ArgumentParser()
    ayr.add_argument("--mode", choices=("before", "after"), required=True)
    ayr.add_argument("--listen-host", default="127.0.0.1")
    ayr.add_argument("--listen-port", type=int, default=55432)
    ayr.add_argument("--backend-host", default="127.0.0.1")
    ayr.add_argument("--backend-port", type=int, default=5432)
    ayr.add_argument("--skip-connections", type=int, default=0)
    ayr.add_argument("--skip-commits", type=int, default=0)
    args = ayr.parse_args()

    durum = {
        "mode": args.mode,
        "commit_seen": False,
        "commit_forwarded": False,
        "commit_ready": False,
        "commits_seen": 0,
    }
    commit_goruldu = threading.Event()
    bitti = threading.Event()

    dinleyici = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    dinleyici.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    dinleyici.bind((args.listen_host, args.listen_port))
    dinleyici.listen(4)
    print(
        json.dumps(
            {"ready": True, "host": args.listen_host, "port": args.listen_port},
            sort_keys=True,
        ),
        flush=True,
    )
    for _ in range(args.skip_connections):
        istemci, _ = dinleyici.accept()
        saydam_baglanti(istemci, args.backend_host, args.backend_port)
    istemci, _ = dinleyici.accept()
    dinleyici.close()

    backend = socket.create_connection((args.backend_host, args.backend_port), timeout=10)
    istemci.settimeout(30)
    backend.settimeout(30)

    # PostgreSQL startup paketi tür baytı taşımaz.
    startup_uzunlugu_baytlari = tam_oku(istemci, 4)
    startup_uzunlugu = struct.unpack("!I", startup_uzunlugu_baytlari)[0]
    startup = startup_uzunlugu_baytlari + tam_oku(istemci, startup_uzunlugu - 4)
    backend.sendall(startup)

    def frontend_aktar() -> None:
        try:
            while not bitti.is_set():
                tur, kalan = cerceve_oku(istemci)
                sorgu = kalan[4:].rstrip(b"\x00").strip().upper()
                commit = tur == b"Q" and sorgu == b"COMMIT"
                if commit:
                    durum["commits_seen"] += 1
                    if durum["commits_seen"] > args.skip_commits:
                        durum["commit_seen"] = True
                        commit_goruldu.set()
                        if args.mode == "before":
                            bitti.set()
                            kapat(istemci, backend)
                            return
                        durum["commit_forwarded"] = True
                backend.sendall(tur + kalan)
        except (EOFError, OSError, ValueError):
            bitti.set()
            kapat(istemci, backend)

    def backend_aktar() -> None:
        try:
            while not bitti.is_set():
                tur, kalan = cerceve_oku(backend)
                if args.mode == "after" and commit_goruldu.is_set():
                    if tur == b"Z":
                        durum["commit_ready"] = True
                        bitti.set()
                        kapat(istemci, backend)
                        return
                    continue
                istemci.sendall(tur + kalan)
        except (EOFError, OSError, ValueError):
            bitti.set()
            kapat(istemci, backend)

    on = threading.Thread(target=frontend_aktar, daemon=True)
    arka = threading.Thread(target=backend_aktar, daemon=True)
    on.start()
    arka.start()
    bitti.wait(35)
    kapat(istemci, backend)
    on.join(timeout=1)
    arka.join(timeout=1)
    print(json.dumps(durum, sort_keys=True), flush=True)
    return 0 if durum["commit_seen"] else 2


if __name__ == "__main__":
    raise SystemExit(ana())
