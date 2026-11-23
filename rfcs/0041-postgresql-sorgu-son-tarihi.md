# RFC-0041 — PostgreSQL sorgusunun son tarihe bağlanması

- **Durum:** taslak — 23 Kasım 2026. Core freeze (K-160A/ADR-059) altında
  uygulanmaz; gerçek ürün kanıtı ya da correctness kararıyla ayrı iş açılır.
- **Tarih:** 23 Kasım 2026
- **İş:** K-371
- **İlgili belgeler:** [spec/09](../spec/09-son-tarih-ve-iptal.md),
  [spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md),
  [spec/25](../spec/25-postgresql-veri-erisimi.md),
  [ADR-060](../adr/060-postgresql-adaptor-siniri.md),
  [ADR-077](../adr/077-request-bound-postgresql-tenant-baglami.md),
  [K-371 denetimi](../docs/k371-dis-etki-hata-eslemesi.md)
- **Freeze sınıfı adayı:** `correctness`; grammar, tanı kodu, CLI ve
  `dil::api::v1` yüzeyi açmaz.

## Sorun

spec/09 son tarihi blok/cümle sınırlarında ve bekleme noktalarında denetler;
HTTP ve SMTP adaptörleri kalan süreyi socket bütçesine taşır. PostgreSQL
adaptörü ise yalnız bağlantı kurmada 5 sn, havuz checkout'unda 2 sn sınır
taşır; sorgu yürütmesi süresizdir. K-371 ölçümü (loopback PostgreSQL 16):

- `1 saniye içinde` bloğunda `SELECT pg_sleep(4)` → blok 4,1 sn sürer,
  `yetişmezse` 4. saniyede çalışır;
- web rotasında `pg_sleep(33)` → istek 30 sn son tarihine rağmen 504'ü
  33,0 sn'de verir; işçi o süre boyunca başka isteğe yanıt vermez.

Kilit bekleyen ya da planı bozulmuş tek sorgu bu yüzden tek işçili web
dağıtımını son tarihin ötesinde kilitler. Davranış bugün spec/25'te AÇIK
madde olarak kayıtlıdır; yanlış HTTP kodu üretmez (504 doğru), yalnız geç
üretir.

## Seçenekler

1. **Sorgu başına `statement_timeout`.** Okuma/değişiklikten önce kalan
   süre (ms) ile `SET LOCAL statement_timeout` (transaction içinde) ya da
   transaction dışı okumada tek ifadelik oturum ayarı. Sunucu `57014` ile
   iptal eder. Artı: sürücü değişmez. Eksi: ek round trip; transaction
   dışında `SET`'in havuzdaki bağlantıya sızmaması için `RESET` disiplini;
   iptal açık transaction'ı abort eder.
2. **İstemci tarafı süre + iptal isteği.** Socket okuma süresi kalan bütçe;
   dolunca `CancelRequest` gönderilir ve bağlantı geçersiz sayılır. Artı:
   sunucu ayarına dokunmaz. Eksi: iptal ile sonuç yarışı; bağlantı kaybı
   sınıfı C027 ile karışabilir.
3. **Yalnız belge.** spec/09 sınırına PostgreSQL'i ekleyip ürünlere
   `lock_timeout`/`statement_timeout`'u göç ya da rol ayarıyla vermelerini
   söylemek.

## Taslak karar ve açık sorular

Seçenek 1, `lock_timeout` ile birlikte öne çıkıyor. Kabul öncesi yanıtlanması
gerekenler:

- İptal edilen sorgu `C025` (`sqlstate=57014`) Hata değeri mi, yoksa
  dönüşteki `son_tarihi_denetle` ile doğrudan `Ç001` mi? HTTP/SMTP modeli
  ikincisidir: süre dolmuş sonuç değere dönüşmez.
- Eylem içinde iptal savepoint'i abort eder; yerel geri alma `C021`'e mi,
  `Ç001`'e mi öncelik verir? COMMIT süresince iptal C027 belirsizlik sınıfına
  girer mi (spec/25 COMMIT kuralı otomatik tekrar YASAK)?
- Transaction dışı okuma yeniden bağlanma kuralı (spec/25) iptal sonrası
  tekrar DENEMEZ olmalı.
- IO izi (`zee-io-izi`) süre bütçesini kaydetmemeli; replay deterministik kalır.

## Kabul kanıtı

Gerçek PostgreSQL ile: `1 saniye içinde` + `pg_sleep(4)` ≤ 1,5 sn ve
`yetişmezse`; web `pg_sleep(33)` → 504 ≤ 31 sn ve sonraki istek 200; eylem
içinde iptal sonrası satır yazılmamış ve havuz bağlantısı geçersiz; transaction
dışı okumada otomatik tekrar yok; `postgresql-recovery-v1.tsv` karar matrisi
yeni satırla; spec/25 AÇIK maddesinin ZORUNLU'ya çevrilmesi.
