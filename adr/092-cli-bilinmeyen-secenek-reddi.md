# ADR-092 — Bilinmeyen CLI seçeneği kaynak yolu değildir

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-221, ürünleşme A6(1)

## Problem ve karar

`çalıştır --bilinmeyen-secenek` kaynak dosyası açmaya çalışıp yanıltıcı bir
dosya hatası veriyordu. Exact ürün talebi
`cd5d12f0008620819e6167af5146e83503fd2b38` A6(1), yeni runtime ayarlı bir
ürünün eski ikiliye geri dönüşünde bu farkı gösterir. Komutun seçenek
bölgesindeki bilinmeyen `--` argümanı kaynak keşfinden önce exit 2, seçenek
adı ve etkin compiler sürümüyle reddedilir. Bilinmeyen güvenlik seçeneği
uyarıyla geçmez. Bir gelecek seçeneğinin hangi sürümde ekleneceği tahmin
edilmez; yalnız etkin sürüm kesin olarak bilinir.

`çalıştır` kaynak yolundan sonraki argümanlar ve `iz kaydet` program
argümanları kullanıcı programına aittir. Bunlar compiler seçeneği diye
reddedilmez. Mevcut web/çocuk bayraklarının ayıklanması ve komuta özgü değer,
tekrar ve çakışma denetimleri korunur.

Bu dilim sürümlü runtime config, compiler selector veya yeni dil grameri
eklemez. A6(2), kayıtsız güvenlik anahtarının uyarıyla kabulü için izin
değildir; ayrı tehdit modeli ve ürün ayarı ister. K-174 kalıp/koşul/komut
fingerprint'i değişmez.

## Kanıt ve etki incelemesi

`cli_secenek_testi` bilinmeyen seçeneğin yol hatası olmadan reddini,
program argümanı korunmasını ve mevcut JSON denetimini doğrular. Minimal
`regression/cli/bilinmeyen-secenek.dil`, CLI kipinde stderr ve exit'i exact
korur; kaynak düzeltme SHA'sı izleyen provenance commit'inde kaydedilir.

Spec/27 ve mevcut RFC-0028 uyumluluk çerçevesi geçerlidir. Tanı kataloğuna
yeni parser/checker tanı kodu eklenmez; yeni Rust modülü/owner/kenar/SCC,
API/ABI, dependency, cache veya uzun ömürlü durum yoktur. ADR-046 fixture,
ADR-045/049 performans tarihçesi ve ADR-069 soak tabanı bu dilimde değişmez.
CLI stderr davranışı bilinçli bugfix/correctness beyanıyla kaynaklanır.
Backlog, sürümler, master plan ve V1 kapıları aynı toplu değişiklikte gerçek
kabul sınırını taşır. Yeni golden veya RFC gerekmemesi bu etki incelemesinin
sonucudur; mevcut semantic regression ve CLI testleri kararı yürütür.
