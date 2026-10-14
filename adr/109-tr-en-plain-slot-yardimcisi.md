# ADR-109 — Mevcut görünüm ile anonim TR–EN plain Metin slotları

- **Durum:** kabul, dar uygulama yardımcısı
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-249/P21/C4, GB-071

## Bağlam ve karar

Exact `zee-cekirdek` C4 talebi layout/slot ve TR–EN çeviri ihtiyacını belirtir.
[Anonim ürün kesiti](../dogfood/zee-cekirdek/tr-en/README.md) immutable ürün
commit'i, göreli dosya yolları ve SHA-256 ile kökenini korur. Özel kaynak
ve etiketler kopyalanmaz; gerçek ürün sayfası değiştirilmez.

Mevcut `görünüm`, `Metin sonucu`, birim ve sözlük yeterlidir. Tek bildirim
tercihleri fragmentinde başlık/içerik/eylem etiketi isimli plain Metin
parametreleridir. Sabit sözlük yalnız `tr`/`en` ve üç anahtar kabul eder;
bilinmeyen dil/anahtar başarısız sonuçtur. Sabit article/h1/p/button ve
attribute adları, varsayılan metin/attribute kaçışı kullanılır. `lang`
yalnız iki whitelist değerini alır. Ham HTML, HTML slotu, dinamik tag,
attribute adı veya URL/scheme yoktur. Çıktı tam document değildir.

## Güvenlik ve kabul sınırı

GB-071 kabul edilen sınırdır: bu dar yardımcının default escaping'i genel
HTML sanitizer, güvenilir raw slot veya bütün i18n/layout sistemi sayılmaz.
List/detail/router, çoğul/locale/date/currency/RTL, ürün adoption'ı ve tam
P08 korpusu açık kalır. Yeni compiler grammar/type/intrinsic/CLI/API veya
dependency/cache yoktur; core freeze genişletilmez.

## Kanıt

Tek proje korpus kaydı üç kaynak birimini ve dört Zee testini sahiplenir.
Native Cargo wrapper yedi fixed CLI senaryosunu atlamasız ister: denetle,
exact TR+EN, dene4/4, script/quote XSS snapshot'ı, unknown dil/anahtar ve
lang injection ret. Exact snapshot yanında bağımsız HTML parser tag,
attribute ve plain text yapısını doğrular. Test aracı explicit yollar,
private byte snapshot, bounded pipe/deadline, kill/wait/join ve başlangıç/
son input hash/BuildJSON eşliği ister; dirty bilgisini değiştirmez.

Bağımsız inceleme ilk adaptörün Python3.11 `file_digest` çağrısını gerçek
Python3.9'daki hata ile buldu. 64 KiB streaming SHA-256 kullanılır; actual
Python3.9.6 ve3.13.3 yedi/dört senaryoyu geçti. Eski başarısız aday ve
çalışma ağacı proof'ları tarihsel tutulur; temiz exact kabul ayrı alınır.

## Etki incelemesi

Spec/RFC/tanı/ABI/API ve compiler katman/SCC yüzeyi değişmez. K-167'de yeni
compiler veya production formatı açılmaz; JSON yalnız native test raporudur.
Performans iş yükü/araç zinciri ve uzun ömürlü compiler state değişmez;
ADR-045/049/046 ve soak/RC geçmişi bu prototiple güncellenmez. K-164 mevcut
görünüm/Metin/sözlük kapasitesiyle karşılaştırılır; yeni compiler sürtünmesi
veya F-kimliği icat edilmez. Dogfood/kanıt/faz/güvenlik/günlük/backlog,
README/sürüm/ürünleşme kayıtları ve üretilen kanıt özeti birlikte yenilenir.
