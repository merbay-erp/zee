# ADR-147 — Uygulamaya ait kapalı bileşen ve TR/EN katalog bileşimi

- **Durum:** uygulandı; son temiz kaynak doğrulaması ve ürün aktarımı açık
- İş: K-289; güvenlik sınırı GB-105

## İhtiyaç ve mevcut fark

cd5d12f0008620819e6167af5146e83503fd2b38 ürün talebi C4, metin
birleştirmesiz TR/EN liste ve detay sayfası, bileşen/parça/yerleşim/slot ve
çeviri kataloğu ister. Talep belgesinin SHA256'sı
694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7'dir.
K272'nin mevcut golden'ı tek tam ağacı üretir, liste ve detayı birlikte
saklar, çok sayıda etiketi dil dallarında tekrarlar; bileşen bileşimi değildir.
Özel ürün kaynağı kopyalanmaz veya değiştirilmez.

## Karar

Mevcut görünüm bloğu ve normal işlem/modül/sözlük mekanizması kullanılır.
Uygulamaya ait bağlantı, gezinme, liste, detay, altbilgi işlemleri ayrı
view parçaları üretir. Layout üç named slotu yalnız bu işlemlerden oluşturur;
dışarıdan raw HTML almaz. Katalog bütün içerik etiketlerini taşır, dil ve eksik
anahtar retleri açık kalır. Yalnız seçilen içerik render edilir. HTML birleştirme
yoktur; sabit DOCTYPE mevcut `ham` satırında yer alır.

`ham` spec/28 uyarınca güvenilir parça içindir. Güven zinciri uygulama
kaynağına dayanır; Metin'e yeni tür güvencesi eklenmez. Her veri leaf'i metin
veya öznitelik kaçışından geçer. Üçüncü taraf raw HTML, builtin çeviri,
grammar/facade/compiler değişikliği ve genel ürün geçişi bu kararın dışındadır.

## Kanıt ve kalan sınır

`scripts/tests/composed-tr-en-test.py` dört Zee testi, altı gerçek HTTP200,
iki404, iki düşman katalog HTTP200 ile bağımsız exact byte/ağaç oracle'ını
çalıştırır. XSS title ve body karakterleri metin kalır, çift kaçırılmaz.
`compiler/tests/composed_tr_en_testi.rs` bir Unix wrapper sahibidir; gerçek
CLI/proje/frontend/runtime/web/IO yürütür. Eski K272/K249 değiştirilmez.
Son temiz SHA ikilisiyle platform kabulü ayrıca kaydedilmelidir; dirty prova
ve anonim uygulama özel ürün benimsemesi veya genel güvenlik audit'i değildir.
