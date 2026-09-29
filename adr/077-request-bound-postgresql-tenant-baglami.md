# ADR-077 — Request-bound PostgreSQL tenant bağlamı

- **Durum:** kabul
- **Tarih:** 29 Eylül 2026
- **İlgili kayıt:** K-186, RFC-0026, spec/25, GB-027

## Bağlam

Zee ERP PostgreSQL RLS politikalarını her korumalı istekte doğru tenant ile
çalıştırmalıdır. Tenant kimliğini form, URL veya uygulama sözlüğünden almak
yetki atlamasıdır. Bağlantı havuzunda session-level ayar bırakmak ise bir
tenant'ın bağlamını sonraki isteğe sızdırabilir.

## Karar

1. Proje, `veritabanı_istek_bağlamı` alanında yalnız doğrulanmış
   `şema.işlev` adını bildirebilir.
2. Korumalı rota yetkilendirildikten sonra runtime doğrulanmış sunucu oturumu
   kullanıcısını bu işleve bind eder; istemci tenant değeri veremez.
3. Resolver tam bir satır ve `tenant_id` UUID döndürür. Sonuç aynı leased
   bağlantıda `set_config('app.tenant_id', ..., true)` ile kurulur.
4. Rota okumaları request transaction'ını paylaşır. Zee `eylem`leri bu tabanın
   içindeki savepoint'tir; yazma yine açık eylem dışında yasaktır.
5. Başarı transaction'ı yanıttan önce tamamlar. Hata, timeout ve yanıtsız bitiş
   rollback eder. Public rotalar tenant transaction'ı açmaz.

## Reddedilen seçenekler

- **Tenant'ı `istek`/formdan almak:** istemci başka tenant seçebilir.
- **Uygulamaya genel `tenant ayarla` intrinsic'i vermek:** güvenilir kökeni
  compiler/runtime sınırından çıkarır.
- **Session-level `SET`:** havuz lease'i yeniden kullanıldığında bağlam sızar.
- **Her sorguya tenant parametresi ekletmek:** unutulabilir ve RLS'in merkezi
  güvenlik sınırını zayıflatır.

## Sonuçlar

Tenant seçimi Zee kodunun serbest girdisi değildir; doğrulanmış kullanıcıdan
veritabanı içindeki dar resolver ile türetilir. Transaction-local ayar havuza
sızmaz. Resolver'ın security-definer yetkisi, sabit `search_path`i ve en az
yetki grant'leri ürün migration'ının sorumluluğundadır.
