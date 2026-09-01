# Tanı kimliği bakım rehberi

Bu rehber K-114/ADR-025'in uygulama yüzeyidir. Amaç, bir çocuğun bugün gördüğü
`S004` ile yıllar sonra editörün gördüğü `S004`ün aynı olayı anlatmasını
sağlamaktır.

## İki bağımsız kapı

1. `compiler/tests/katalog_testi.rs`, üretim kaynaklarında doğan kodlarla
   `docs/hata-katalogu.md` kayıtlarını birebir karşılaştırır.
2. `compiler/tests/tani_kimligi_testi.rs`, katalogdaki her kodun durumunu ve
   kanonik anlamını `compiler/tests/fixtures/tani-kimlikleri-v1.tsv` ile
   karşılaştırır.

İlk kapı “kod belgeli mi?”, ikinci kapı “bu kod hâlâ aynı şey mi?” sorusunu
yanıtlar. Fixture ayrıca semantik anahtarların tekil, küçük ASCII ve doğru
ailede olmasını denetler.

## Yeni tanı ekleme

1. Olayın mevcut bir kodla gerçekten aynı semantik olay olup olmadığını
   kontrol et. Tüketicinin farklı davranması gerekiyorsa yeni koddur.
2. İlgili ailede sıradaki boş kodu seç; ayrılmış kodu geri alma.
3. Üretim noktasını, katalog satırını, fixture kaydını ve olumlu/olumsuz
   davranış testini aynı commit'te ekle.
4. Semantik anahtarı aile önekiyle yaz: `soz.`, `ad.`, `tur.`, `calisma.`,
   `dogrulama.`, `proje.` veya `ic.`. Anahtar sonradan yeniden adlandırılmaz.
5. Katalog özetini olay kimliği olacak kadar kesin, dinamik ayrıntıdan bağımsız
   tut. Somut ad/değer gibi bağlam `Tani.mesaj` ve öneride kalır.

## Kodu emekliye ayırma

Kaynak artık kodu üretmese bile katalog ve fixture satırı silinmez. Katalog
özeti `**ayrılmış** — ...`, fixture durumu `ayrilmis` olur. Eski semantik
anahtar korunur. A004, C014 ve S032 bu mezar taşı davranışının ilk tabanıdır.

## Anlam ile metin ayrımı

Yazım, noktalama veya daha doğal Türkçe gibi editoryal bir katalog düzeltmesi
fixture özetini de değiştirir; böylece değişiklik kod incelemesinde görünür.
Tanının hangi koşulda doğduğu, ailesi veya tüketicinin vereceği karar
değişiyorsa bu editoryal değildir ve yeni kod gerekir.

Dinamik mesaj, öneri, satır/sütun ve işaret uzunluğu kimlik fixture'ına
konmaz. JSON alan biçimi ve çoklu tanı sırası RFC-0010'un ayrı
sözleşmeleridir. Program içinde taşınan yapılandırılmış `Hata.kod` değerleri de
bu derleyici tanı kataloğunun parçası değildir.

## Yerel doğrulama

```sh
cd compiler
cargo test --test katalog_testi --test tani_kimligi_testi
```

Fixture farkını özellikle incele: var olan satırın silinmesi, durumunun
`ayrilmis`ten `aktif`e dönmesi veya aynı kodun anlam değiştirmesi sürüm
uyumluluğu ihlalidir.
