# K-336/K-337 — Çok işçi profili ve açılış uzlaştırmasının tekliği

Bu anonim kesit aktif `zee-cekirdek` ürününün (bkz. [KOKEN](../KOKEN.md))
W2/W4 talebine bağlıdır. Ürün deposu özeldir; müşteri, alan adı, sunucu yolu
ve gerçek veri taşınmaz (ADR-087).

Exact harici ürün commit'i: `ca3870d854e4567c47bb6748fe8573223c885798`
(21 Ekim 2026; talep belgesi ve ölçüm sayfasının son değiştiği commit).

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `cekirdek/kaynak/ana.dil` (etkilenen açılış uzlaştırması, `sunucu başlat` öncesi) | `c636711dfd045a2a6b87824f13e0d880c9764d9a` | `bbb3ccb092cdc2b487eb1cbd2af29620b6407ccc5d08e66f16f905f73076c5b7` |
| `docs/dil-talepleri/2026-10-20-web-eszamanlilik-ve-kopma.md` (talep W2/W4) | `7dc54f0d49c763e7ade3a2990e08b05871b1e7cb` | `f500dd0452366a7174f58661104317dcac992f16c338fd5b62f91ec614af8dfa` |
| `docs/olcek-ve-onbellek.md` (ölçüm: 1 işçi 4,0; 3 işçi 12,0 istek/sn) | `a7f50cdf1bf7b0bcc466e09fe36d6b8c7292b56c` | `71e016648031384ab19c806ca85480252002af285cf80bcf28a321f9442eee70` |

## Sürtünme

Web işçisi istekleri sıralı işler; ürün doğrusal ölçeği elle açılan ayrı portlu
süreçler ve önlerindeki proxy `upstream` listesiyle alıyordu. Her süreç
açılışta yarım yüklemeleri ve sahipsiz `.zee/yuklemeler` dosyalarını
uzlaştırır; tek işçi yeniden başlarken ötekinin o an yüklediği (henüz
veritabanı satırı açılmamış) geçici dosyayı sahipsiz sayıp silebilir. Ürün bu
yüzden dağıtımda bütün işçileri birlikte yeniden başlatmak zorundaydı. K-164
kataloğunda karşılığı yok: yeni dar sınıf "süreç grubundaki rolünü bilemiyor".

## Minimal değişiklik

[Reproducer](ana.dil) `birincil süreç` değerini değişkene bağlar; yalnız
birincil işçi uzlaştırma işlemini çağırır, ikincil işçi bir satır yazar.
[proje.dil](proje.dil) `ağ-sunucusu` ve dosya yetkinliklerini açar. Çok işçi
grubu `dil çalıştır --web-proxy … --web-worker-port 8096 --web-isci-sayisi 4 .`
ile tek komutla başlar. Karar [ADR-161](../../../adr/161-cok-isci-web-profili-ve-acilis-birincilligi.md),
[RFC-0038](../../../rfcs/0038-cok-isci-web-profili.md),
[spec/12 §7](../../../spec/12-web-guvenlik-profili.md).

`dil denetle .` bu klasörde hermetik geçer; gerçek grup davranışı
`compiler/tests/web_cok_surec_testi.rs::isci_grubu_acilisi_yalniz_birincilde_kosturur_ve_yeniden_baslatmada_tekrarlamaz`.

## Açık

Ürün açılışının `birincil süreç` ile sarılması, systemd şablonunun tek
komutluk profile geçmesi ve proxy `upstream` listesinin tek hedefe inmesi ürün
tarafındadır.
