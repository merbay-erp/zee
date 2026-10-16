# dene native web test köprüsü

Unix'te açık operator profili kullanılır. Önce olağan Zee testleri başarılı
olmalıdır. Altı rolün tümünü verin; output henüz var olmamalı ve kanonik
sahipli parent altında olmalıdır:

```sh
dil dene --web-suite /abs/fixtures/suite.json \
  --web-suite-sha256 SUITE_SHA256 --web-fixtures /abs/fixtures \
  --test-python /abs/trusted/python3 \
  --test-postgres-bin /abs/trusted/pg16.15/bin \
  --test-output /abs/owned/new-output /abs/project
```

Suite sözleşmesi [owned web kit](owned-web-test-kiti.md) ile aynıdır.
Python ve PostgreSQL16.15 binary'leri operator tarafından güvenilmiş roller;
suite bunları değiştiremez. Kit kaynakları compiler içine gömülüdür; proje
klasöründe Python aramaz. Credential/provider hesabı veya canlı hizmet
kullanmayın. Fixture SQL güvenilmiş test verisidir, SQL sandbox değildir.

`unit.json`, `runtime/scripts/` ve `results/proof.json` çıktı kökünde korunur.
Başarısız olağan test `native_started:false` bırakır, native suite çalışmaz.
Native rapor dört vaka/19HTTP isteği gibi gerçek sayıları kaydeder; güncel
suite'in sayıları dışında genelleme yapılmaz. Worker başına kaynak raporu
`results/coverage-<case>.json` altındadır. Unit ve native kaynak kapsamı ayrı
sonuçlardır; bağımsız worker sayaçları tek çalıştırmaymış gibi birleştirilmez.

İki PG vakasının cluster kimliği farklı olmalıdır; readonly observer commit1,
request rollback0 ve sonraki taze vaka0 durumunu doğrular. Bu, tüm HTTP
isteklerine ortak transaction rollback desteği değildir.

[ADR-144](../adr/144-dene-native-web-test-koprusu.md),
[kaynak kapsamı](kaynak-kapsami.md) ve
[ADR-142](../adr/142-zee-kaynak-dal-kapsami.md) güven sınırlarını açıklar.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-304 etki sınırı

K-304 bu köprüye gömülen kitin son iptal/deadline sonucunu korur; Python
bayt değişikliği yeni derlemede yer alır. [Direct kit kabulü](owned-web-test-kiti.md)
GB-113'ü dar kapsamda kapatır. `abdc5cf36b4d55021df5edad08cee3a7ad953ebd` native4 sonucu ayrı
`dene`/source-coverage sahibinin çalıştığı anlamına gelmez. S3 dört vaka/19HTTP
ve admission retleri kendi exact kaynağında kalır; yeni köprü kabulü veya
compiler grammar/API/coverage kapasitesi ileri sürülmez.
