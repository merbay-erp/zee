# K-207 — Güncel HTTP master’da LSP ön incelemesi

İnceleme talebi §3/5’in eski 0,16/4,6/20 sn iddiası güncel master’da yeniden ölçüldü. Compiler yeni optimize edilmedi; tam didChange/analiz mimarisi sürer ve artımlılık işi K-207’de açık kalır.

## Exact ortam ve örnekler

Temiz c7433123ac1755bb81c476e2698ffee661500a69, macOS 26.6.2 (25G83), Apple M4 Pro / 24 GiB, Rust 1.93.1 release. Sonraki 3d7ff93 yalnız CI/güvenlik aracı ve belgedir; compiler/src aynıdır. Kalıcı performans tarihçesi/tabanı değiştirilmedi, hard eşik uygulanmadı.

[Standart olcum raporu](standart.md) ve [JSON](standart.json): 3 bağımsız örnek, 2 ısınma; her örnekte yeni Sunucu, didOpen sonra aynı tam metin didChange. Önceki K-207 exact 25 örnekli kayıt yerinde kalır. Geçici K221-on-inceleme ölçüm etiketi yeni bir compiler özelliği veya kapanmış artımlılık işi değildir.

[Gerçek düzenleme ham sonuçları](gercek-duzenleme.json): her boyutta yeni release dillsp, bir didOpen, aynı süreçte 2 ısınma + 5 sıralı düzenleme; son bloktaki literal 1/2 arasında değişir. JSON/stdio roundtrip dahildir, kaynak hash’leri ve binary SHA-256 kayıtlıdır.

| Satır | p50 | p95 |
| --- | ---: | ---: |
| 2000 | 5,477 ms | 5,526 ms |
| 5000 | 17,508 ms | 18,076 ms |
| 10000 | 50,473 ms | 51,535 ms |
| 20000 | 159,590 ms | 165,731 ms |

Bunlar sentetik ölçek gözlemidir; gerçek tuş gecikmesinin tüm editör/işletim sistemi yollarını veya farklı proje grafikleri için sınır garantisini vermez. Anahtar değişikliği ölçümü ilk sandbox denemesinde donanım bilgisi erişimine takıldı; yalnız tamamlanan ikinci tur JSON’u kayıtlıdır.

## Exact ürün snapshot’ı

[Ürün JSON’u](urun-duzenleme.json): zee-cekirdek 35b56c79a48d159215ac662e53b5437ba1ead143 Git archive kopyası, kullanıcı çalışma ağacına dokunulmadı. Özel ürün kaynakları depoya taşınmadı; yalnız nötr göreli dosya adı/hash/sayı kayıtlıdır. Her dosyada ayrı dillsp, açılış sonra 2 ısınma + 5 baş yorum düzenlemesi; bütün metin iletilir.

1517 satırlık cekirdek/kaynak/urun_akisi.dil: p95 16,428 ms, başlangıç ve beş düzenlemede tanı 0. 145 satırlık ana.dil/proje grafiği: p95 194,978 ms, tanı 2→2; committed şablonda üretilen modul_katalogu.dil olmadığı için iki A010 vardır (CLI’da da aynıdır). İkinci sayı hatasız tam üretim projesi kabulü değildir.

Sonuç: eski çok saniyeli maliyet güncel kaynakta tekrarlanmadı; K-207 kapsam optimizasyonunun mevcut kanıtı korunur. Cache/artımlı analiz gerçekleştirilmiş gösterilmez.
