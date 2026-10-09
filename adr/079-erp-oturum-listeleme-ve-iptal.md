# ADR-079 — ERP için güvenli oturum listeleme ve tekil iptal

- **Durum:** kabul
- **Tarih:** 30 Eylül 2026
- **İlgili kayıt:** K-188, RFC-0017, spec/12, GB-028

## Bağlam

Zee ERP parola, MFA, rol, kullanıcı ve tenant güvenlik değişikliklerinde mevcut
oturumları sunucu tarafında düşürmek; ayrıca kullanıcıya etkin cihazlarını
gösterip tek bir oturumu kapatmak zorundadır. Runtime ortak kalıcı depoda
worker/restart güvenli revoke tutuyordu fakat uygulamaya yalnız mevcut oturumu
kapatma cümlesini veriyordu. ERP'nin çerez belirtecini saklaması veya ikinci
bir oturum otoritesi kurması güvenlik durumunu ikiye bölerdi.

## Karar

1. Runtime, kullanıcının geçerli kimlikli oturumlarını `web-oturumu`
   yetkinliği altında listeler. Sonuç yalnız SHA-256 özet kimlik, kullanıcı,
   rol ve oluşturma/son erişim/son geçerlilik anlarını taşır.
2. Ham çerez belirteci ile CSRF belirteci uygulamaya hiçbir zaman açılmaz.
3. Tekil iptal yalnız 64 haneli küçük harfli SHA-256 kimliğini kabul eder ve
   istek transaction'ında ortak depoya uygulanır. Bulunmayan kimlik idempotent
   sıfır, silinen kimlik bir döndürür.
4. Mevcut oturum iptal edilirse yanıtla tarayıcı çerezi de silinir. Hata veya
   socket yazma arızasında mevcut transaction rollback sözü korunur.
5. ERP kullanıcı-geneli iptali listeyi dolaşıp her güvenli kimliği iptal ederek
   kurar; runtime ikinci bir kullanıcı/tenant yetki modeli eklemez.
6. Gerçek ürün ihtiyacı iki yeni ifade kalıbını zorunlu kıldığı için K-174
   syntax freeze penceresi 30 Eylül–28 Ekim 2026 olarak yeniden başlatılır.

## Reddedilen seçenekler

- **Ham cookie tokenını ERP'ye vermek:** bearer sırrını uygulama/veritabanı ve
  log yüzeyine taşır.
- **ERP'de ayrı session tablosu:** runtime deposu ile yarışan iki revoke
  otoritesi ve fail-open sapma üretir.
- **Yalnız kullanıcı-geneli revoke:** etkin cihaz görünümü ve tek cihaz çıkışı
  ihtiyacını karşılamaz.

## Sonuç

Zee, ortak depo sahipliğini koruyarak gerçek ERP güvenlik yaşam döngüsünü
destekler. Değişiklik dil yüzeyini büyüttüğü için uyumluluk fixture'ı, yeniden
başlatılmış syntax freeze, dogfood reproducer'ı ve güvenlik bulgusu aynı
değişiklik setindedir.
