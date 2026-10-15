# K-272 — sabit tam TR/EN liste–detay HTTP golden

Bu proje mevcut görünüm/Metin/birim/sözlük ve GET handler yetenekleriyle hazırlanmış **sabit golden örnektir**. Gerçek anonim ürün sayfası kaydı veya P08 adoption kabulü değildir. K-249’un `article` fragment projesi, ana giriş byte’ları ve 7 native + 4 Zee çıktısı korunur. Burada ayrı proje ve ayrı ana giriş vardır.

`ceviri.dil` mevcut K249 dosyasının byte-eş kopyasıdır. `yerlesim.dil` mevcut çeviri helperını ve plain Metin named-slot yaklaşımını kullanır; `tamSayfayiKur` tek typed görünümden `html/head/meta/title/body/header/nav/main/section/article/footer` ağacını üretir. Yalnız sabit `<!doctype html>` başlığı bu yerel görünüm sonucuna eklenir. Kullanıcı verisiyle HTML metni birleştirilmez, HTML Metin slotu veya `ham` yoktur. Dinamik alanlar text/title/aria-label bağlamında varsayılan kaçıştan geçer. Tag/attribute adları sabittir; URL alanı çağırandan alınmaz. Dil whitelist’i yalnız tr/en’dir.

Liste ve detay bölümleri aynı sabit iki-kayıt şablonundadır; `hidden` boolean attribute route seçimini gösterir. Bu görsel seçim özel veri erişim kontrolü değildir; golden verisi tamamen anonim ve sabittir. Genel layout/body slot sistemi veya arbitrary HTML parça bileşimi gerçeklenmez. Tarih/sayı/çoğul/RTL/locale pazarlığı veya yerleşik çeviri kataloğu yoktur.

`rota.dil` altı exact route’u mevcut helpera bağlar. `ana.dil` bunları gerçek GET handler olarak sunar:

| Route | Dil | Sayfa |
|---|---|---|
| /tr/bildirimler | tr | liste |
| /tr/bildirimler/eposta | tr | birinci detay |
| /tr/bildirimler/ozet | tr | ikinci detay |
| /en/notifications | en | liste |
| /en/notifications/email | en | birinci detay |
| /en/notifications/digest | en | ikinci detay |

Bilinmeyen dil/yol pure helperda başarısız sonuçtur; kayıtlı olmayan gerçek HTTP yolu mevcut yürütücünün 404 yanıtıdır. Yeni HTTP parser/router/grammar/API yoktur. İşlem kaydetme, kuyruk/SMTP gönderme, DB, kimlik veya notification preference yazma yapılmaz.

## Koşu

```sh
compiler/target/debug/dil denetle golden/tr-en-list-detail
compiler/target/debug/dil dene golden/tr-en-list-detail
compiler/target/debug/dil biçimle --denetle golden/tr-en-list-detail
python3 -I -B scripts/tests/tr-en-list-detail-test.py \
  --framework-root "$PWD" --dil compiler/target/debug/dil \
  --output /owned/fresh-tr-en-proof
```

HTTP ana girişi yalnız test host’unun private receipt argümanı ve explicit owned inherited listener profiliyle yürütülür. Test, mevcut `native-listener-host.py`/`worker-supervisor.py`/`native-capabilities.py` byte snapshot’ını kullanır; yeni owner veya PID/port discovery yazmaz. Loopback listener host tarafından oluşturulur, exact nonce/PID/port/BuildJSON ready ve activation doğrulanır, WNOWAIT grup temizliği aynı owner’dan gelir. Ayrı owned foreign sentinel host cleanup’ından sağ çıkar ve sonra yalnız kendi owner’ı temizler. Servis/global install/PATH değişikliği yoktur.

## Kanıt ve sınırlar

Native kampanya 9 actual HTTP isteği yapar: altı 200 HTML snapshot’ı, bilinmeyen locale404, bilinmeyen route404, TR detay typed-slot XSS200. Yedi checked-in exact HTML snapshot’ı ayrıca bağımsız HTMLParser ile doctype/lang/charset/title/nav/list/detail/footer, fixed href, hidden state ve text/attribute yapısı açısından sınanır. XSS profili yalnız owned proje kopyasındaki iki plain veri literalini değiştirir; ana fixture veya özel ürün değişmez. Script/img/quote payload’ı karakter verisi ve title attribute değeri olarak kalır; yeni tag veya event attribute oluşmaz.

Beş kaynak testi çeviri, whitelist (injection dahil), fixed seçim sınırı ve route retlerini yürütür. Son input byte/hash ve BuildJSON kimliği, private binary, source/snapshot/helper eşliği, owned temp cleanup kontrol edilir. Aynı kampanya mevcut K249 runner’ını değişmeden çalıştırarak 7 native + 4 kaynak testini yeniden doğrular. Yeni Rust wrapper tek test önerisidir; dış stage’de Cargo/root gate çalıştırılmamıştır. Native Linux/Windows/minOS kabulü veya temiz release/adoption bu koşudan çıkmaz; actual given build dirty bilgisi raporda aynen kalır.

## Talep kökeni ve kayıt sınıfı

Mevcut etkin zee-cekirdek C4 talebi commit `cd5d12f0008620819e6167af5146e83503fd2b38` içindedir. Bu çalışma aynı C4 sürtünmesini mevcut K164/K249 kapasitesiyle karşılaştırır; yeni compiler sürtünmesi veya F-kimliği icat etmez. Exact provenance metadata:

- `docs/dil-talepleri/2026-10-14-zee-urunlesme.md` SHA256 `694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7`;
- `cekirdek/kaynak/gorunum.dil` SHA256 `66ee583ce610e35445a8ab2d43e1438e36aeb91a2373f1e0b2f17df65b9eed59`;
- `cekirdek/kaynak/eposta_ayarlari.dil` SHA256 `8116cd0de756e40840841377c9ca1ea212ad79aae564f0c894ca0e5a91342edf`.

C4 satır156–162 LF ile birleştirilip son LF olmadan hash: `b29c7f7600fd161430b19ac56f1c783fccdce8370c1ec4023c51cc1d355a4f77`. Bu değerler exact git-show byte’larıyla tekrar doğrulandı. Özel ürünün kaynak gövdesi/etiketleri kopyalanmadı ve hiçbir özel repo değiştirilmedi. Yeni somut anonim ürün sayfası sağlanmadığından bu bir **golden tasarım örneği** olarak kalır; dogfood/korpus-v1.tsv veya P08 kapsamı yeni gerçek ürün kanıtı varmış gibi büyütülmez. P21/C4 genel ürün kabulü, gerçek sayfa/list-detail migration ve P08 ayrı açıktır.

Runner sınırları: iki CLI stream’i okunurken ayrı64KiB sınır, tek15sn komut bütçesi (legacy paket60sn), HTTP connect/send/recv/EOF/parse için tek3sn, tüm kampanya180sn. Signal yalnız stop flag kurar; registered owned process-group cleanup finally ile yürür. Sabit failed proof tür/reason dışında argüman/body/stderr taşımaz. Yeni/legacy runner ve eski4kaynak/3snapshot dahil orijinaller tüm iki kampanya bittikten sonra tekrar hashlenir; eski kampanya da aynı private binary snapshot’ını kullanır. Başarılı Cargo raw proof target altında korunur; ek ast fazı gerçek formatter çağrısından gelir.
