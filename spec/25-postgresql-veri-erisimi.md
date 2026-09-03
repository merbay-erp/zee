# 25 — PostgreSQL veri erişimi

Normatif kaynak: RFC-0026, ADR-060. Durum: **TANIMLI — K-163 ilk yerel profil**.

## Proje bildirimi

PostgreSQL kullanan proje `veritabanı` yetkinliğini ve şu üç alanı birlikte
bildirmek ZORUNDADIR:

```zee
veritabanı_hedefi "postgresql://127.0.0.1:5432/uygulama" olsun
veritabanı_bağlantı_değişkeni "UYGULAMA_DATABASE_URL" olsun
veritabanı_göçleri "göçler" olsun
```

Hedef kullanıcı/parola/query taşıyamaz; yalnız `localhost`, `127.0.0.1` veya
`::1` kabul edilir. Ortam değişkenindeki URL tek TCP host, exact hedef ve
`sslmode=disable` taşımak ZORUNDADIR. Migration yolu relative ve proje içinde
olmalıdır.

## Sorgular

- `"SQL" sorgusunu parametreler ile okumayı dene`, iki argümanlı typed
  intrinsic'tir. Parametreler `Metin listesi`, sonuç
  `Sonuç<List<Metin sözlüğü>>`dür.
- `"SQL" sorgusunu parametreler ile değiştirmeyi dene`, aynı bind kuralıyla
  `Sonuç<TamSayı>` döndürür ve yalnız eylem içinde geçerlidir.
- Parametreyi SQL metnine birleştirmek adaptör davranışı OLAMAZ. Her parametre
  extended-query protokolünde ayrı `TEXT` değeri olarak bind edilmelidir.
- Okuma sütunları non-null metin olmak ZORUNDADIR. Farklı PostgreSQL tipi
  `::text`, NULL olasılığı `COALESCE` ile sorguda açıkça çözülür.
- Veritabanı reddi `C025` Hata değeridir. PostgreSQL sağlıyorsa `sqlstate`,
  `kısıt` ve `tablo` Hata verisinde korunur.

Değişiklik application/state write etkisidir; GET/HEAD rotasında ve eylem
dışında reddedilir. İç içe eylem savepoint'tir. Aynı eylemde kalıcı dosya ile
PostgreSQL yazısı YASAKTIR.

## Migration

`dil göçür [proje]`, yalnız `NNNN_aciklama.sql` düzenli dosyalarını sürüm
sırasıyla tek transaction içinde uygular. Sürüm pozitif ve tekil, açıklama
küçük ASCII/sayı/alt çizgi olmalıdır. Kendi `BEGIN`, `COMMIT`, `ROLLBACK` veya
`SAVEPOINT` komutunu taşıyan migration reddedilir.

Uygulayıcı `_zee_gocleri` tablosunda sürüm, ad ve içerik SHA-256'sını tutar ve
transaction-scoped advisory lock alır. Aynı ağaç ikinci koşuda atlanır.
Uygulanmış dosyanın kaybolması veya aynı sürümün ad/içerik değiştirmesi C026 ile
fail-closed durur.

## Kaynak ve iz sınırı

Sorgu 64 KiB; parametre sayısı 100; tek parametre 64 KiB; sonuç 10.000 satır,
100 sütun ve 16 MiB; migration tek dosya 1 MiB, toplam 8 MiB sınırını AŞAMAZ.
Bu değerlerin tek sahibi `KaynakSinirlari.veritabani()`dir.

Okuma, değişiklik, değerler ve structured hata `zee-io-izi\t1` içine kanonik
sırada yazılır. Replay aynı SQL/parametreleri exact eşleştirir ve dış
veritabanına bağlanamaz.

## Açık sınır

Uzak host, TLS doğrulama, connection pool, async driver, PostgreSQL dışı
veritabanı, genel ORM/schema DSL ve production migration işletimi AÇIKTIR.

