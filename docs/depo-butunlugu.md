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

## Faza özgü test envanteri

K-146/ADR-043'ün [faz matrisi](faz-test-matrisi.md), kaynakta görünen test
özniteliklerini yaklaşık saymaz. Cargo JSON çıktısındaki gerçek lib/bin/test
çalıştırılabilirlerini ve libtest'in vaka listesini her Tier-1 platformunda
yeniden çıkarır:

```bash
cd compiler
cargo run --locked --bin faz_test_matrisi -- --denetle \
  --rapor target/faz-test-matrisi.md
```

Her vaka tam bir birincil faza aittir. Yeni/sahipsiz/yinelenen test, boşalan
seçici, kayıp regression/fuzz/conformance kaynağı veya bayat kanonik belge
kapıyı kapatır. Dinamik rapor faz başına count, pass/fail/ignored ve duvar
süresini verir. `cfg` nedeniyle aktif toplam platforma göre değişebilir;
README'deki kaynak vakası toplamı ile karıştırılmaz. Süre performans eşiği
değildir; K-148'in kullanıcı iş yükü gözlemleri aşağıdaki ayrı kanaldadır.

## Performans gözlemi ve tarihçe

K-148/ADR-045, K-152/ADR-049 ve K-153/ADR-050'nin
[ölçüm rehberi](olcumler.md), sabit iş yüklerinde parse, checker, typed-HIR,
runtime, yürütme, in-process LSP engine initialize, gerçek dillsp process
cold-start, open/change ve Unix tepe RSS
yüzeylerini ayırır. Varsayılan release koşusu iki ısınma ve 25 örnekten ham
dağılım, p50/p95 ile min/max üretir. İzlenen
[`performans-gecmisi-v2.tsv`](performans-gecmisi-v2.tsv) yalnız incelenmiş
tabanları taşır; her satır exact Git SHA, ayrı milestone, temiz çalışma ağacı,
OS/CPU/RAM/Rust/release ve gerçek örnekleme sayılarıyla provenance sahibidir.
RSS 25 tur değil, koşu sonundaki tek süreç-tepe görüntüsüdür. Araç yeni sonucu
ayrı TSV'ye yazar.

```bash
cd compiler
cargo build --locked --release --bin dillsp
cargo run --locked --release --bin olcum -- \
  --tur 25 --gecmis ../docs/performans-gecmisi-v2.tsv \
  --json target/performans.json --rapor target/performans.md \
  --gecmis-cikti target/performans-gecmisi.tsv \
  --kayit K-NNN-makine --git-sha GIT_SHA --milestone K-NNN \
  --dillsp target/release/dillsp
```

`--gecmis-cikti` kirli çalışma ağacını ve HEAD'den farklı SHA'yı reddeder.
Shared CI bu üç dosyayı job summary ve indirilebilir artefakt yapar; runner
gürültüsü nedeniyle hard gate uygulamaz. `--esik-yuzde` yalnız sabitlenmiş
adanmış benchmark makinesinde açıkça verilebilir. Geçmiş şema, metadata ve
p50/p95 tutarlılığı `olcum` birim testleriyle; CI kablolaması ve eşiksizlik
politikası mimari sınır testiyle korunur.

## Production katman graph'ı

K-149/ADR-046'nın [katman rehberi](katman-mimarisi.md) ve
[`katman-mimarisi-v1.tsv`](../compiler/tests/fixtures/katman-mimarisi-v1.tsv)
bütün production Rust dosyalarını 37 üst sahibe bağlar. Exact doğrudan
bağımlılık tabanı eklenen kadar artık kullanılmayan kenarı da görünür inceleme
ister; katman yönü ters kenarı tabana yazmakla geçilebilir olmaz.

```bash
cd compiler
cargo test --locked --test katman_mimarisi_testi
cargo test --locked --test bagimlilik_cevrimi_testi
```

Tarayıcı test-only kodu, yorum ve metinleri graph'a katmaz; hedefe özgü
production yollarını birleşik korur. Yeni kök modül/ikili sahiplenilmezse,
mevcut modüle yeni kenar eklenirse veya sozdizimi/semantik/runtime/adaptör yönü
tersine çevrilirse `Engineering gates` fazı kapanır.

K-150/ADR-047 bunun üstünde SCC hesabı yapar. Açıklamasız yeni SCC,
graph'tan kaybolduğu halde duran izin ve son tarihi geçen geçici izin
`bagimlilik_cevrimi_testi` ile reddedilir. İzin fixture'ı exact üyeler,
gerekçe, son tarih ve kaldırma işi taşır.

## Semantic regresyon korpusu

K-147/ADR-044'ün [semantic regresyon korpusu](semantic-regresyon-korpusu.md),
geçmiş compiler bug'larını `regression/v1.tsv` ile küçük `.dil` kaynaklarına
bağlar. Vaka kimliği, K-kaydı, faz, kip, kesin tanı spanı, exit ve çıktı tek
satırdır. Ağaç ile manifest birebir değilse ya da fixture minimality sınırını
aşarsa test başarısız olur. Faz matrisi bu testi ayrı `Semantic regression`
fazında sayar; böylece genel yeşil toplam geçmiş arıza kimliğini gizlemez.
`semantic-regresyon-korugu.sh` ayrıca CI tabanındaki vaka+bug+yol üçlüsünün
silinmesini veya yeniden kullanılmasını reddeder; yeni bug yalnız yeni satır
ve fixture ekler.

## Immutable conformance verisi

`scripts/conformance-korugu.sh`, CI taban Git revizyonunda bulunan bütün
`conformance/**/*.json` veri/şemalarıyla yayımlanmış morfoloji semantic
kayıtlarını korur. Var olan artefaktın değiştirilmesi, silinmesi veya yeniden
adlandırılması reddedilir; yeni anlam yeni profil/şema kimliğiyle ayrı dosya
olarak eklenir. Morfoloji ve `zee-esz-1` scheduler korpusları aynı genel kapıyı
paylaşır.
