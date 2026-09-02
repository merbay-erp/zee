# Morfoloji profil uyumluluk koruğu

Bu belge K-122/B-008'in bakım sözleşmesidir. Amaç `zee-tr-1` davranışının
aynı kimlik altında sessizce değişmesini iki bağımsız kapıyla engellemektir.

## Dondurulan kayıt

`compiler/src/morfoloji/uyumluluk.rs`, şema
`zee-morfoloji-uyumluluk-v1` altında kanonik bir semantic akış üretir. Akış:

- profil kimliği, sayısal sürüm, azami katman, ek tablosu ve zincir kuralını;
- ses geri çevrimi/belirsizlik sınırındaki 11 sabit ham yüzeyin sıralı bütün
  kök+ek çözümlerini;
- 4.096 deterministik kökün yedi tek ve altı iyelikli zincirindeki 53.248
  kanonik üretimini ve her üretimin sıralı bütün çözüm kümesini taşır.

Bu akışın SHA-256 özeti aşağıdaki yayımlanmış kayıttır:

```text
profil=zee-tr-1
şema=zee-morfoloji-uyumluluk-v1
sha256=e6034e7359e5d5d1bf5f3f06b6a7767616b2b6daf8ab916226b48d522f99f220
```

Kayıt `compiler/tests/fixtures/morfoloji-zee-tr-1.sha256` dosyasındadır.
`dil morfoloji --uyumluluk` çalışan derleyicinin ürettiği kaydı gösterir.

## İki kapı

1. `morfoloji_testi.rs`, çalışan üretim+çözüm davranışının kaydını fixture ile
   byte düzeyinde eşitler. Tablo veya kapsanan semantic davranış değişirse
   normal `cargo test` kırılır.
2. `scripts/morfoloji-profili-korugu.sh`, CI taban revizyonunda zaten bulunan
   bütün `morfoloji-zee-tr-N.sha256` kayıtlarını, bağımsız `zee-tr-N.json`
   conformance verilerini ve `sema-vN.schema.json` şemalarını immutable sayar.
   Fixture'ı yeni özete göre güncellemek de CI'ı kırar; silmek veya yeniden
   adlandırmak aynı biçimde yasaktır. CI tam Git geçmişiyle pull request
   tabanını ya da push öncesi commit'i denetler.

K-122'nin fixture'ı ilk kez eklediği tabanda önceki kayıt bulunmadığı için
yalnız bu bootstrap eklemesi tarih koruğundan muaftır. Dosya bir kez tabana
girdikten sonra istisna yoktur.

## Kırıcı değişiklik yolu

`zee-tr-1` kaydı **güncellenmez**. Yeni bir ek, çözüm, üretim, zincir veya
öncelik gerekiyorsa:

1. `zee-tr-2` kimliği ve ayrı gerçekleme açılır;
2. `morfoloji-zee-tr-2.sha256` yeni dosya olarak eklenir;
3. eski `zee-tr-1` fixture'ı ve uyumluluk desteği yerinde kalır;
4. proje/kilit geçişi ile ana dil sürümü ya da edition kararı aynı RFC'de
   yazılır;
5. iki profil için ayrı conformance kanıtı çalışır.

Yerel denetim:

```bash
cd compiler
cargo test --locked --test morfoloji_testi
cargo run --quiet -- morfoloji --uyumluluk
cd ..
scripts/morfoloji-profili-korugu.sh <taban-revizyonu>
```

## Bilinçli sınır

Parmak izi geniş ve deterministik bir davranış korpusudur; bütün olası Unicode
dizilerinin biçimsel ispatı değildir. K-111 property/fuzz katmanı bu nedenle
ayrıca kalır. K-123/B-009, ikinci bir compiler'ın Rust koduna bakmadan
tüketebileceği sürümlü `yüzey → kök → ek → karar` korpusunu ayrı
[conformance sözleşmesine](morfoloji-conformance.md) bağladı.
