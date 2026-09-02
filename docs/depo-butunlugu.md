# Kanıt haritası ve canlı depo sayıları

K-118, iki bakım borcunu tek yapısal kapıda kapatır: normatif belgelerin hangi
test dosyalarıyla kanıtlandığını görünür tutmak ve README'deki hareketli
sayıları elle saymamak.

## Spec ↔ code kanıt haritası

[Makine-okunur harita](kanit-haritasi-v1.tsv) her numaralı RFC, ADR ve spec
dosyasını tam bir kez listeler. Dört sekmeli alan şunlardır:

1. depo köküne göre belge yolu;
2. `kanitli`, `kismi` veya `taslak` durumu;
3. virgülle ayrılmış Rust test dosyaları; yalnız `taslak` için `-` olabilir;
4. kapsamı ya da açık kalan sınırı anlatan kısa not.

Tazelik testi; yeni belgenin haritasız kalmasını, yinelenen/bilinmeyen kaydı,
var olmayan test yolunu, test taşımayan `.rs` kanıtını ve testsiz `kanitli` ya
da `kismi` satırını reddeder. Harita “test dosyası var” kanıtıdır; testin
iddiasıyla normatif metnin doğruluğunu kod incelemesi yine değerlendirmelidir.

`kismi`, belgenin çalışan dilimi testli olduğu halde açık kapsamı bulunduğunu;
`taslak`, gerçekleme sözü verilmediğini gösterir. Bu ayrım özellikle uzak paket
taşıması, telemetri için gelecek ikili audit'i ve self-hosting aşamalarında
yanlış tamamlanmışlık izlenimini engeller.

## README sayı üreticisi

```bash
cd compiler
cargo run --bin depo_sayilari
cargo run --bin depo_sayilari -- --denetle
cargo run --bin depo_sayilari -- --yaz
```

Araç depo ağacından şunları hesaplar:

- iki rakamla numaralı golden programlar;
- `compiler/src` ve `compiler/tests` içindeki `#[test]` vakaları ile
  `compile_fail` doctest'leri;
- tanı kimliği fixture'ındaki etkin/ayrılmış kayıtlar;
- RFC dosyaları ve kendi `Durum` satırlarından kabul/geçici/taslak dağılımı;
- ADR dosyaları ve kabul sayısı;
- normatif spec bölüm sayısı.

`--yaz`, README'deki işaretli bloğu ortak atomik dosya çekirdeğiyle değiştirir.
`--denetle` tek byte farkında başarısız olur ve normal test paketi bu kipi
çalıştırır. Böylece yeni test, tanı veya karar belgesi sayıları elle arayıp
değiştirilmez; geliştiren kişi yalnız üreticiyi çalıştırır.
Bu yazma K-128/ADR-032 metadata sözleşmesini de tüketir; README'nin var olan
owner/group/ACL/xattr bilgisi sayı tazelenirken sessizce düşürülemez.

Tarihsel K-kayıtlarındaki “o gün toplam N test” cümleleri bilinçli snapshot'tır
ve yeniden yazılmaz. README'nin bugünkü canlı sayıları için tek otorite
işaretli otomatik bloktur.

## Kritik işlev eğilim raporu

K-144/ADR-041'in [işlev eğilim raporu](islev-egilimi.md), sabit Clippy
ölçümünü incelenmiş TSV tabanıyla karşılaştırır. Rapor elle düzenlenmez:

```bash
cd compiler
cargo run --locked --bin islev_egilimi -- --denetle
cargo run --locked --bin islev_egilimi -- --rapor-yaz
```

`--denetle`; yeni, kayıp veya büyüme payını aşan kritik işlev kadar raporun
tek byte bayatlamasını da reddeder. Taban yenilemek rapor yenilemekten ayrı ve
bilinçli bir mimari incelemedir; düşen değerler kendiliğinden yeni borç alanı
açmaz.

## Immutable conformance verisi

`scripts/conformance-korugu.sh`, CI taban Git revizyonunda bulunan bütün
`conformance/**/*.json` veri/şemalarıyla yayımlanmış morfoloji semantic
kayıtlarını korur. Var olan artefaktın değiştirilmesi, silinmesi veya yeniden
adlandırılması reddedilir; yeni anlam yeni profil/şema kimliğiyle ayrı dosya
olarak eklenir. Morfoloji ve `zee-esz-1` scheduler korpusları aynı genel kapıyı
paylaşır.
