# ADR-105 — Credential dosyasından mevcut CLI ortamına host köprüsü

- **Durum:** kabul, sınırlı POSIX pilot
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-245/P25, GB-068

## Bağlam ve karar

Exact `zee-cekirdek` talebi D3, mevcut SMTP/DB/anahtar ortam bağlarının
credential dosyasından sağlanmasını ister. Talep SHA'sı ve anonim dosya
kökenleri [ürün kaydındadır](../dogfood/zee-cekirdek/credential/README.md).
Mevcut env API'si yeterlidir; core freeze altında yeni grammar, intrinsic,
CLI komutu, `zee.ayar` alanı, facade export veya dependency eklenmez.

`scripts/credential-baslat.py` açık mutlak root, ENV=basename ve trusted
ikili seçimiyle mevcut `çalıştır` argv'sini başlatır. Bütün dosyalar çocuk
başlamadan doğrulanır: nofollow directory zinciri, sahip/mode, regular tek
hardlink, nonblocking FIFO reddi, pre/open/post identity ve son snapshot
kontrolü. En fazla 16 eşleme ve toplam 64 KiB, tek satır strict UTF-8;
ambient eşleme varsa ret, fallback veya trim yoktur. Ayrıntılı çağrı ve
izin sözleşmesi [host rehberindedir](../docs/credential-baslat.md).

Çocuk stdio'su DEVNULL'dur; dış çıktı sabit schema/source/exit metadata'dır.
Host eşlemesi native CLI proje allowlist'i veya ağ iznini açmaz. Launcher
ikiliyi authenticate etmez; mevcut imzalı seçici ve işletmeci güven sınırı
korunur. Restart-only rotation vardır; çoklu dosyanın atomik credential
snapshot'ı veya canlı reload sözü verilmez.

## Güvenlik ve açık kapsam

GB-068 kabul edilen sınırdır: dosya değeri çocuk env'sine dönüşür ve süreç
belleği/alt süreç inheritance'da bulunabilir. Bu, systemd file credential'ın
noninheritance avantajını koruyan file-native tasarım değildir. Aynı UID,
root, ACL/NFS, genel taint ve OS sandbox garantisi yoktur. Windows DACL,
gerçek systemd servisi, canlı ürün/provider ve tam P25 kabulü açıktır.

## Kanıt ve etki incelemesi

Normal POSIX Cargo wrapper 14 actual Python vakasını atlamasız yürütür:
gerçek dosya/izin/UID/symlink/FIFO/TOCTOU/boyut/UTF8/collision/rotation,
child secret echo bastırması, native env allowlist ret/kabul ve gerçek
loopback SMTP AUTH1/DATA1. Fixture credential'ları public sentineldir.
Temiz exact kaynak kampanyası çalışma ağacı denemesinden ayrıca kaydedilir.

RFC/spec/tanı/ABI/API, production katman graph'ı, benchmark iş yükü ve
compiler cache değişmez; ilgili kayıtların değişmemesi bilinçlidir.
Markdown rehberleri, günlük/backlog, ürünleşme/kapı durumu, güvenlik/kanıt
haritası, faz matrisi, README sayıları ve kanıt özeti aynı dalgada yenilenir.

Host metadata/readiness biçimleri giriş sürümüyle K-167 uyumluluk
fixture'ında aktiftir; bunlar compiler CLI/grammar/facade export'u değildir.
