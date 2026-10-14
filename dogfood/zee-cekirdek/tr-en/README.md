# K-249 — anonim TR–EN sayfa kesiti

Bu kesit, etkin `zee-cekirdek` ürününün C4 talebinden türetilen tek anonim
bildirim tercihleri sayfasıdır. Mevcut `görünüm`, `Metin sonucu`, birim ve
sözlük davranışı yeterlidir; compiler/grammar/API değişikliği gerekmez.

`ceviri.dil` yalnız `tr` ve `en` dilleri ile `baslik`, `icerik`, `eylem`
anahtarlarını kabul eden uygulama sözlüğüdür. Bilinmeyen dil veya anahtar
başarısız sonuç verir. `sayfa.dil` içindeki `sayfayiKur` başlık, içerik ve eylem
etiketini isimli `Metin` parametreleri olarak alır. Görünümün varsayılan text
ve attribute kaçışı kullanılır; `ham` içerik, HTML slotu, dinamik tag/attribute
adı veya URL/scheme girdisi yoktur. `ceviriSayfasiniKur` bu iki yardımcıyı
birleştirir. Çıktı tam HTML document'i değil, sabit bir `article` fragmentidir.

Depo kökünden:

```sh
compiler/target/debug/dil denetle dogfood/zee-cekirdek/tr-en
compiler/target/debug/dil dene dogfood/zee-cekirdek/tr-en
compiler/target/debug/dil çalıştır dogfood/zee-cekirdek/tr-en
compiler/target/debug/dil biçimle --denetle dogfood/zee-cekirdek/tr-en
```

Dört Zee testi deterministik sözlük, bilinmeyen dil/anahtar, text/attribute
kaçışı ve dil özniteliği whitelist'ini sınar. `ana.dil` TR ve EN çıktısını
sabit sırayla basar. Native kanıt ayrıca iki exact HTML snapshot'ını, script
ve quote payload'larının text olarak kalmasını ve bağımsız HTML parser'ıyla
tag/attribute yapısının korunmasını denetler. Bilinmeyen dil, kayıp anahtar
ve dil özniteliği injection probe'ları başarısız sonuç metnini açıkça basar;
fixture bu sonuçları işlediğinden CLI exit 0'dır.

## Talep ve anonimleştirme kökeni

Köken etkin `zee-cekirdek` ürün kaydıdır; bütün dosya yolları bu ürünün
harici köküne görelidir. Exact ürün commit'i `cd5d12f0008620819e6167af5146e83503fd2b38`.

| Gerçek dosya | SHA-256 | İlgisi |
| --- | --- | --- |
| `docs/dil-talepleri/2026-10-14-zee-urunlesme.md` | `694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7` | C4, satır 156–162: layout/slot/TR–EN çeviri ihtiyacı |
| `cekirdek/kaynak/gorunum.dil` | `66ee583ce610e35445a8ab2d43e1438e36aeb91a2373f1e0b2f17df65b9eed59` | Mevcut görünüm ve kabuk/slot bileşimi |
| `cekirdek/kaynak/eposta_ayarlari.dil` | `8116cd0de756e40840841377c9ca1ea212ad79aae564f0c894ca0e5a91342edf` | Mevcut tercih sayfasındaki iki görünüm çağrısı |

Üç dosyanın disk byte'ları exact commit ile karşılaştırıldı. C4 bölüm hash'i
`b29c7f7600fd161430b19ac56f1c783fccdce8370c1ec4023c51cc1d355a4f77`:
UTF-8 metnin 156–162 numaralı satırları LF ile birleştirilir, sonuna LF
eklenmez ve bu normalize byte dizisinin SHA-256'ı alınır. Bu bölüm değeri
ham dosya aralığı hash'i değildir; tabloda tam dosyanın exact byte hash'i
ayrı verilmiştir.
Özel ürün dosyaları değiştirilmedi. Anonim sayfanın etiketleri ve yardımcıları
bağımsız yazıldı; özel müşteri adı, alan/sunucu veya sır taşınmadı. Talebin
HTML bileşimi ve TR–EN ihtiyacı sınanır; özel ürünün sayfaları kabul edilmiş
sayılmaz.

## Kabul sınırı

Dış adayın yerel macOS ARM64 koşusunda 7 CLI senaryosu ve 4 Zee testi geçti;
4 kaynak kanonik biçimdedir. Kullanılan debug ikilisinin gerçek kimliği
`176a35f09926dafdecfc15e30f1e5776714f3361`, development `dirty=true` ve hash'i
`ec71ba6206e9374c895c9e03b28cf80bd14f1faa9549ab92fe7c8d1a966a1577` idi.
Bu çalışma ağaca alınmadan veya temiz exact kaynakta tekrar yürütülmeden
final kaynak/artefakt kabulü değildir. Ham CLI çıktıları ve kaynak hash'leri dış adayın `proof.json` dosyasında
korunur. Taşınabilir native kampanya `scripts/tests/tr-en-sayfa-test.py` ile
açık `--dil`, `--project`, `--snapshots` yolları ve Python `-I -B` kullanır;
`compiler/tests/tr_en_sayfa_testi.rs` tek Cargo vakası exact yedi senaryoyu,
dört kaynak testini, input hash/kimlik eşliğini ve owned cleanup sonucunu
denetler. `compiler/tests/fixtures/tr-en-sayfa/` üç exact HTML snapshot taşır.

Genel layout/slot sistemi, HTML taşıyan slot, sayfa/router entegrasyonu,
çoğul/locale/date/currency/RTL çevirisi, bütün sitelerin P08 korpusu,
yeni platform veya canlı ürün kabulü açıktır. Bu kesit P21/C4'ün dar
uygulama yardımcısı ve escaping kanıtıdır.
