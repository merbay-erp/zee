# ADR-063 — Sabit HTTP durumlu yanıt sınırı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-163/F014/F032, RFC-0015, spec/11

## Bağlam

Çatlı dogfood'u önce bulunamayan/taslak içeriği güvenli açıklama gövdesiyle
HTTP 200 vermek zorunda kaldı. İkinci bağımsız ihtiyaçta PostgreSQL readiness
başarısızlığının liveness'tan ayrılıp 503 olması gerekti. Yalnız otomatik 5xx
eşlemesi bu ürün kontrollü durumları anlatamaz; genel response builder ise V1
HTTP yüzeyini ihtiyaçtan geniş açar.

## Karar

Rota gövdesinde `<ifade> yanıtını <durum> durumuyla gönder` kabul edilir.
`durum` kaynakta görünen bir `TamSayı` literalidir ve 100..599 aralığındadır.
Değişken, hesaplanmış ifade ve aralık dışı değer S037 üretir. Eski
`<ifade> yanıtını gönder` 200 davranışını aynen korur.

Bu cümle yalnız web adaptörü etkisidir; işlem/eylem içine taşınamaz. İlk yanıt,
cookie/session taslağı, deadline ve rollback davranışı RFC-0015/spec-11'in
mevcut tek-response transaction sınırını değiştirmez. Ham header yazımı,
dinamik status policy veya response nesnesi bu kararla açılmaz.

## Kanıt ve açık sınır

Parser/yorumlayıcı web regresyonu 503 readiness ile sonraki 200 liveness'ı
aynı worker'da doğrular; 99, 600 ve değişken durum olumsuzları S037'dir.
K-163 ürününde gerçek TLS PostgreSQL kapatıldığında readiness 503 olurken
liveness 200 kalmış, backend dönüşünde aynı process readiness 200'e dönmüştür.
F014'ün bulunamayan/taslak sayfa 200 workaround'u da 404 ile emekli edilmiştir.

Uygulama tarafından seçilen header'lar, streaming response API'si ve dinamik
durum üretimi yeni gerçek ürün kanıtı olmadan CORE FREEZE altında kapalıdır.
