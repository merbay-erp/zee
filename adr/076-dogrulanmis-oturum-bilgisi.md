# ADR-076 — Doğrulanmış oturum bilgisi rota ortamında okunur

- **Durum:** kabul
- **Tarih:** 29 Eylül 2026
- **İlgili kayıt:** K-185, RFC-0017, spec/12, GB-026

## Bağlam

Zee ERP her veritabanı isteğinde tenant bağlamını güvenilir kullanıcı
kimliğinden kurmak zorundadır. Zee oturum deposu kullanıcı ve rolü sunucuda
tutuyor, yetkiyi bunlarla denetliyor fakat rota gövdesi bu iki doğrulanmış
değeri okuyamıyordu. Uygulamanın tenant veya kullanıcıyı URL, form ya da ham
çerezden yeniden kurması yetki atlama sınırı doğururdu.

## Karar

1. Her rota ortamında `Metin` değerli `oturum` sözlüğü bulunur.
2. Geçerli kimlikli sunucu oturumunda sözlük `kullanıcı` ve `rol` anahtarlarını
   taşır; anonim istekte boştur.
3. Değerler yalnız doğrulanmış sunucu oturum kaydından gelir. İstek başlığı,
   form alanı ve tarayıcı çerezi bu sözlüğü dolduramaz; iç taşıma alanları
   adaptör sınırında silinip güvenilir değerlerle yeniden kurulur.
4. Oturum kimliği/özeti, CSRF belirteci ve başka sırlar uygulamaya açılmaz.
5. `oturum`, `istek` ve `çerezler` gibi yalnız rota kapsamındadır; rota dışında
   kullanımı A001'dir. Yeni anahtar sözcük veya grammar eklenmez.

## Reddedilen seçenekler

- **Tenant kimliğini URL/formdan almak:** istemci başka tenant seçebilir.
- **Ham oturum çerezini uygulamaya vermek:** sırrı gereksiz yere açar ve
  uygulamayı oturum deposu ayrıntılarına bağlar.
- **İmzalı özel claim söz dizimi eklemek:** ERP'nin ilk ihtiyacı kullanıcı ve
  rolle sınırlıyken grammar ve anahtar yaşam döngüsünü büyütür.

## Sonuçlar

Zee ERP oturum kullanıcısını dar bir PostgreSQL işlevine verip tenant
bağlamını sunucuda kurabilir. Regresyon, tarayıcının iç alanları taklit etse
bile sunucu oturumundaki değerlerin görüldüğünü ve adın rota dışına
sızmadığını doğrular. K-185 ve GB-026 kapanır.
