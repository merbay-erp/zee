# RFC-0030 — Legacy parola doğrulama ve istemci adresi

- **Durum:** **geçici kabul** (K-208/K-209 gerçek ürün dilimi)
- **Tarih:** 13 Ekim 2026
- **İlgili kayıtlar:** ADR-082, spec/12, GB-043/044

Zee çekirdek talebi legacy panel kullanıcılarının mevcut bcrypt parolasını korumayı
ve Site F audit kaydına güvenilir istemci IP'sini taşımayı gerektirir.
Exact ürün kökeni [KOKEN](../dogfood/zee-cekirdek/KOKEN.md) ve talep dosyasındadır.
Genel hash algoritması seçimi, bcrypt üretimi veya başlık erişim API'si açılmaz.

Mevcut `parola ozet ile doğrulanıyorsa` Argon2id yanında `$2a/$2b/$2y`
bcrypt doğrular. Biçim 60 ASCII bayt, maliyet 4–14 ve parola en fazla
72 UTF-8 baytı olmalıdır. Aşım/bozuk biçim/yanlış parola `yanlış`tır.
Digest karşılaştırması bcrypt kitaplığının sabit zamanlı karşılaştırmasıdır;
bütün doğrulamanın sürelerinin aynı olması veya aynı hash maliyeti iddia edilmez.
Yeni özet yalnız Argon2id'dir. Ürün başarılı girişte eski biçimi SQL
`parola_ozeti LIKE '$2%'` veya mevcut `ozet "$2" ile başlıyorsa` ile ayırır.

`istemci adresi` sıfır argümanlı Metin intrinsic'ine indirilir. Native host
oran sınırına verdiği aynı kimliği handler'a verir: proxy doğrulayıcısı
veya soket IP'si. X-Forwarded-For/form/sorgu alanı kaynak değildir.
IPv6 kanonik ve parantezsizdir. HTTP isteği dışında varsayılan boş metindir;
HTTP'den bağımsız eylem içinde kullanımı mevcut T044 sınırında reddedilir.
Handler IP'yi eyleme Metin parametresi olarak aktarır. Yetkinlik ağ-sunucusudur.

IO trace yeni okuma olayını kaydeder; replay yalnız boş veya kanonik IP kabul
eder. İz özel veridir. KVKK bağlamında IP kişisel veri olarak ele alınır;
uygulama saklama süresini kendi politikasıyla belirler. Otomatik audit
saklama veya sunucu/ürün dağıtımı bu dil değişikliğine ait değildir.

Testler: Node bcryptjs vektörleri; üç önek/maliyet/ASCII ve UTF-8 byte sınırı;
real worker proxy/socket/IPv6/spoofed XFF ve ardışık istek; CLI boş değer,
eylem etki sınırı, IO replay. Mevcut Argon2 akışı ve handler'lar korunur.
