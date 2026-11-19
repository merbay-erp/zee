# K-340 — Çok işçi grubunun `zee.ayar` alanları

Bu anonim kesit aktif `zee-cekirdek` ürününün (bkz. [KOKEN](../KOKEN.md))
üretim ayarlarına bağlıdır. Ürün deposu özeldir; müşteri, alan adı, sunucu
yolu ve gerçek veri taşınmaz (ADR-087).

Exact harici ürün commit'i: `abaca3aba0e90f57d26435c4befffa903346c425`
(18 Kasım 2026).

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `cekirdek/deploy/` altındaki site worker unit dosyası (bütün web ayarları `ExecStart` bayrağı) | `9f1a3a1080eb5aec2862e0c5212c365a7061dc18` | `591d9fcb7ac140db8316dd521a433ff4ef8f872bd89bf71a820bc022881148f1` |
| `docs/dil-talepleri/2026-10-14-zee-urunlesme.md` (K-234 talebi: üretim ayarları sürümlü yapılandırmada, CLI ezer) | `6b2fa83bca9e15a1778b0fb6ad3f15b5d7388b73` | `694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7` |
| `docs/olcek-ve-onbellek.md` (çok worker systemd şablonu, ayrı portlu süreçler) | `a7f50cdf1bf7b0bcc466e09fe36d6b8c7292b56c` | `71e016648031384ab19c806ca85480252002af285cf80bcf28a321f9442eee70` |

## Sürtünme

Ürün K-234'te "üretim ayarları proje içinde sürümlü bir yapılandırmada; CLI
bayrakları bu dosyayı ezer" istedi; `zee.ayar` web modu, worker portu ve oran
ayarlarını taşıyor. K-336 tek komutluk çok işçi grubunu açtı ama işçi sayısı
ve sağlık portu yalnız CLI bayrağıydı: gruba geçen ürün bu iki ayarı yeniden
systemd `ExecStart` satırına yazmak zorunda, üretim ayarı iki yere bölünüyordu.
K-164 kataloğunda karşılığı "ayar yüzeyi eksik" (K-234 ile aynı sınıf).

## Minimal değişiklik

[zee.ayar](zee.ayar) `guvenlik` dalına `isci-sayisi` ve `saglik-port`
yazar; [ana.dil](ana.dil) değişmeden `dil çalıştır .` ile dört işçili grup
açılır. Karar [ADR-161 eki](../../../adr/161-cok-isci-web-profili-ve-acilis-birincilligi.md),
[spec/12 §7](../../../spec/12-web-guvenlik-profili.md),
[zee.ayar rehberi](../../../docs/zee-ayar.md).

`dil denetle ana.dil` bu klasörde hermetik geçer; gerçek grup davranışı
`compiler/tests/web_cok_surec_testi.rs::isci_grubu_zee_ayar_ile_baslar_ve_cli_ezer`.

## Açık

Ürün unit dosyasının `zee.ayar`'a geçmesi ve `ExecStart`'ın yalnız
`dil çalıştır <proje-kökü>` biçimine inmesi ürün tarafındadır.
