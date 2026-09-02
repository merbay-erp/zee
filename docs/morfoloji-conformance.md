# Morfoloji conformance korpusu

Bu belge K-123/B-009'un bakım sözleşmesidir. Korpus Rust bootstrap
derleyicisinden bağımsız olarak depo kökündeki
`conformance/morfoloji/zee-tr-1.json` dosyasında yaşar; ikinci bir Zee
derleyicisi Rust kaynak koduna bakmadan aynı profil kararlarını sınayabilir.

## Şema

Dosya UTF-8 JSON'dur ve `conformance/morfoloji/sema-v1.schema.json` içindeki
JSON Schema 2020-12 sözleşmesini izler. Üst alanlar:

- `sema`: `zee-morfoloji-conformance-v1`;
- `normatif`: kararın RFC/spec otoritesi;
- `profil`: kimlik, sayısal sürüm, azami katman, K-122 uyumluluk SHA-256'sı,
  sıralı soyut ek+yüzey tablosu ve geçerli iki katmanlı zincirler;
- `cozum_vakalari`: yüzey, görünür kapsam kökleri, sıralı bütün yapısal
  kök+ek çözümleri ve beklenen `kok`/`hata` kararı;
- `uretim_vakalari`: kök+soyut ek zinciri ile beklenen kanonik yüzey; geçersiz
  zincirde `null`.

Alan adları ve değerler Rust enum/varyant adı taşımaz. `kok`, `hata`, A001 ve
A002 doğrudan dil sözleşmesidir; bütün metinler NFC UTF-8'dir. Dizi sırası
gözlenebilir profil çıktısının parçası olduğu için korunur.

## Kapsam

İlk `zee-tr-1` korpusu:

- yedi tek ek ve altı geçerli iyelik+dış ek üretimini;
- p/ç/t/k yumuşaması, `nk→ng`, tek hecede yumuşamama;
- ikizleşme, ardından sertleşme ve ünlü düşmesi geri çözümlerini;
- doğrudan ad eşleşmesinin ekli adaylardan önce kazanmasını;
- tek çözüm, A001 adayı-yok ve A002 tek/iki katman belirsizliğini;
- üç geçersiz zincirin üretilememesini

27 çözüm/karar ve 21 üretim vakasında taşır. Her vaka kararlı, benzersiz bir
`kimlik` alanına sahiptir.

## Tüketici protokolü

Bir gerçekleme şu sırayı izler:

1. Şema ve `profil.kimlik` desteğini doğrula; bilinmeyen profilde fail-closed
   dur.
2. Her `cozum_vakasi.yuzey` için kapsamdan bağımsız bütün yapısal çözümleri
   aynı sırada üret.
3. `kapsam` adlarıyla gerçek resolver kararını çalıştır ve `karar`ı eşle.
4. Her `uretim_vakasi` için kanonik yüzeyi veya geçersiz zincirde `null`u
   eşle.
5. Korpusun bağladığı `uyumluluk_sha256` kaydının desteklenen immutable profil
   kaydı olduğunu doğrula.

Rust bootstrap kanıtı `compiler/tests/morfoloji_conformance_testi.rs` içindeki
tek veri güdümlü testtir. Test şemayı parse eder; profil tablosu, zincir
kapsamı, bütün yapısal çözümler, resolver kararı ve üretimi ayrı ayrı eşler.

```bash
cd compiler
cargo test --locked --test morfoloji_conformance_testi
```

## Sürüm ve değişmezlik

`zee-tr-1.json` ile `sema-v1.schema.json` yayımlandıktan sonra K-122 Git-tarih
koruğunun parçasıdır; yerinde değiştirilemez veya silinemez. Yeni profil
`zee-tr-2.json`, kırıcı veri biçimi ise `sema-v2.schema.json` olarak eklenir.
Eski dosyalar kalır. Korpus genişletmesi eski beklenenleri değiştirmeden yeni
bir conformance şeması/profil kaydıyla sürümlenir.

Bu sonlu korpus bütün Unicode uzayının ispatı değildir. K-111'in geniş
property/fuzz katmanı ile K-122'nin 53.248 vektörlü parmak izi tamamlayıcı
kapılar olarak çalışmaya devam eder.
