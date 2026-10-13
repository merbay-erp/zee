# Anonim ürün talebi — K-208/K-209 (13 Ekim 2026)

Özgün talep özel ürün deposunda, exact ürün commit'i
`ebca9a6ddcc9c7535ee3204cfe2af3bbb7fc95d7` içinde korunur.
Bu dosya anonim gereksinim özetidir; müşteri veya dağıtım envanteri değildir.

## Legacy parola doğrulama

Legacy Next.js yönetim paneli müşterisi kullanıcı adını ve parolasını
koruyarak Zee ürününe geçmelidir. Mevcut `parola ozet ile doğrulanıyorsa`
yüklemi bcrypt `$2a$/$2b$/$2y$` özetlerini de doğrular. Üretim yoktur;
yeni özet Argon2id kalır. Maliyet 4–14, parola en fazla 72 UTF-8 byte;
bozuk/aşan özet veya uzun parola yanlış döner, sessiz kırpma yapılmaz.
İlk girişte parola göçünü ürün tarafı ayrıca transaction ile uygular ve sınar.

## Kanonik istemci adresi

Site F denetim izi, oran sınırının kullandığı doğrulanmış kimliği handler'da
ister: `istemci adresi`. Proxy doğrulaması altında Forwarded, doğrudan
bağlantıda soket eşidir. X-Forwarded-For, form veya sorgu kaynak olmaz.
İstek dışında boş metin döner; IPv6 kanonik metindir. `203.0.113.7`
yalnız sentetik örnektir. IP saklama/erişim/süre politikası ürün tarafındadır.

Reproducer ve davranış kararı RFC-0030, ADR-082 ve spec-12 ile bağlıdır.
Sunucu yükseltmesi ve ürün CI pini dil geliştirme işinin kapsamı dışındadır.
