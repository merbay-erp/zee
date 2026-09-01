# RFC-0006 — İşlemler ve Parametreler

- **Durum:** **geçici kabul**: Seçenek A (`için`/`ile`) çalışan ve normatif
  geçici yüzeydir; V1 için seçilmiş nihai yüzey değildir. K-096 ile gerçek
  kullanıcı verisi öncesinde A/B/C kör karşılaştırması, tek-genel-sözdizimi
  koşulu ve karar eşikleri donduruldu. Karar gerekçesi §3'te
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-016/K-096 (çağrı sözdizimi ve karar deneyi), K-014 (kelime çakışmaları), K-007 (yanıt), K-067 (sayısal genişleme), K-083/K-086 (açık imza)
- **İlgili golden programlar:** 12, 13, 14, 15, 30
- **Gerçekleme:** `islem_ayristir`/`cagri_kalibi` (`ayristirici.rs`),
  `cagri_denetle` + açık dönüş/akış kanıtı (`cozumleyici.rs`), public sınır
  (`lib.rs`), `islem_cagir` (`yorumlayici.rs`)

## Özet

İşlem adları eylem cümlesidir (`ortalamayı hesapla`); tanım `işlem` başlığıyla,
parametreler gövde başındaki `<ad-i> al` satırlarıyla yapılır. Çağrı yüzeyi
(bu RFC'nin ana sorusu) v0'da "argümanlar + için/ile + işlem adı" biçimindedir.

## 1. Tanım (kararlı sayılabilir)

```
işlem ortalamayı hesapla
    sayıları al

    toplam 0 olsun
    her sayı için
        toplamı sayıyla artır

    sonucu toplamı sayıların adedine böl
    sonucu döndür
```

- Ad, bir ya da çok kelimedir; yalnız kelimelerden oluşur (S022) ve
  benzersizdir (A005).
- Parametreler belirtme ekiyle bildirilir (`sayıları al` → parametre `sayılar`);
  yalnız gövdenin başında tanınır. Başlangıç biçimi tam iki kelimedir.
- K-083/K-086 progressive disclosure: public/paket API'sinde
  `sayıları Ondalık listesi olarak al` + `Ondalık döndürür` tam sözleşmesi
  zorunludur. Bir
  işlemin bütün parametreleri açık ya da bütünü çıkarımlı olmak zorundadır.
  Açık gövde çağrı beklemeden denetlenir; bildirilen dönüş gövde birleşimi ve
  bütün olağan akış yollarıyla doğrulanır, çağrılar imzayı terfi ettiremez.
- `döndür` yalnız işlem içinde (T020); dönüş türleri tek olmalı, `yok` ile
  karışım Seçenek üretir (T018 → RFC-0008).
- İşlem gövdesi taze ortamda çalışır: dış değişken görmez (RFC-0004).

## 2. Çağrı yüzeyi — v0 gerçeklemesi

```
ortalama notlar için ortalamayı hesapla olsun     # tek argüman
"Ayşe" ve 10 ile selamla                          # çok argüman, cümle çağrısı
bulunan sayılar için ilk çift sayıyı bul olsun    # çok kelimeli ad
```

Kurallar (hepsi gerçeklenmiş ve testli):

1. İşlem başlıkları dosya gövdesinden önce **ön-taranır**. Tanım çağrıdan sonra
   gelebilir; karşılıklı özyineleme geçerlidir. Çağrı tanıma, “satır/bölge
   ön-taranmış bir işlem adıyla bitiyor mu?” sorusuyla yapılır; bilinmeyen ada
   çağrı sözdiziminde S004'e düşer.
2. **En uzun ad önce eşlenir** (determinizm; iç içe ad çakışmalarında).
3. Argümanlar addan önce gelir ve `için` ya da `ile` ayracıyla biter (S019);
   birden çok argüman `ve` ile ayrılır; her dilim tam bir ifade bölgesidir
   (v0.2, K-038): `tabanın tam kısmı için yuvarla` geçerli.
4. **v0 çağrı-güdümlü imza:** gövde ilk çağrının argüman türleriyle denetlenir;
   sayı/listelerde TamSayı→Ondalık genişlemesi kabul edilir. Dar imza sonra
   geniş argüman görürse K-067 ile kaldırılır ve gövde geniş türle yeniden
   denetlenir. Diğer tür farkları T017, parametre sayısı T015'tir. Bu model
   public API için v1 sözleşmesi değildir. K-083 açık parametreyi, K-086 açık
   dönüşü ve paket/public zorunluluğunu ekledi. v1 public model bilinçli olarak
   monomorfiktir; generic sözdizimi v2+ sorusudur.
5. Özyineleme ve karşılıklı özyineleme geçerlidir. Özyinelemeli çağrıdan önce
   en az bir dönüşlü temel durum görülmelidir (T035); çağrı derinliği 500'dür
   (C019).

## 3. Ana soru: çağrı yüzeyi hangisi olmalı?

Uygulama deneyimiyle güncellenmiş karşılaştırma:

**Seçenek A — mevcut: `notlar için ortalamayı hesapla`**
- ✓ Gerçeklendi; 5 golden programda sorunsuz; yüklem-sonlu dağıtımla uyumlu
  (satır işlem adıyla, yani eylemle bitiyor — dilin geri kalanıyla aynı ritim).
- ✗ `için` döngü kelimesiyle, `ile` birleştirme/aritmetikle yükleniyor
  (K-004 üç-rol sorunu); "notlar için ortalamayı hesapla olsun" cümlesindeki
  `olsun` kuyruğu ilk okuyuşta doğal değil.

**Seçenek B — sonuç-bağlama: `notlar için ortalamayı hesapla, sonucu ortalama olsun`**
- ✓ Çağrı ile bağlama ayrışır; sesli okunuşu en doğal.
- ✗ Virgüllü iki-cümle yapısı yeni bir cümle türü ister; iç içe ifadelerde
  (çağrı sonucu doğrudan koşulda) çözüm sunmaz; daha uzun.

**Seçenek C — parantezli izin: `ortalama (notlar için ortalamayı hesapla) olsun`**
- ✓ İfade konumu sorununu genel çözer.
- ✗ "Noktalama minimum" ilkesinden ilk büyük taviz; çocuğun ilk karşılaştığı
  parantez olur (manifesto 5 gerilimi).

**Önden bağlanmış karar yöntemi (K-096):** A bugünkü gerçekleme olduğu için
hipotezdir, sonuç değildir. V1 bağlama göre ikinci bir çağrı biçimi taşımayacak;
tek genel yüzey değere bağlama, cümle çağrısı, iç içe ifade, dönüş,
özyineleme, tanım sırası ve çok-tokenli argüman bağlamlarının tamamını
karşılayacaktır. B kullanıcıların güçlü doğal tercihi olsa bile bugün tek
başına iç içe ifade vermediğinden doğrudan ek sözdizimi olmaz; B-003 expression
grammar tasarım turunu tetikler. C de aynı mimari tur ve migration kanıtı
olmadan seçilemez. Kör kartlar, sıra dengelemesi ve sayısal eşikler
[K-016 karar paketinde](../docs/k016-cagri-karar-paketi.md) bağlayıcıdır.

## 4. Bilinen gerilimler (K-014 devamı)

- `al` parametre kelimesi işlem adlarında geçemez sayılmalı mı? (30'da
  "karesini al" bu yüzden "karesini hesapla" oldu.) Öneri: RFC kesinleşirken
  `al` işlem adının SON kelimesi olamaz kuralı.
- `yanıt` (K-007) ve `sonuç` bağlamsal adlarının işlem gövdesindeki durumu
  belgelendi; ayrılmış kelime listesi RFC-0002 §5'e bağlı.

## 5. Açık sorular

1. ~~Parametrelerde ve dönüşte açık tür~~ — K-083/K-086 ile
   `sayıları Ondalık listesi olarak al` + `Ondalık döndürür` gerçeklendi;
   paket/public sınırında zorunlu, kaynak ABI/semver politikası spec/10'dadır.
2. Generic işlem sözdizimi v1 kapsamı dışında, v2+ için AÇIK.
3. Çok değerli dönüş (K-023'ün "hepsini bekle" sorusuyla birleşik).
4. ~~Özyineleme ve tanım-sonrası çağrı~~ — GERÇEKLENDİ (v0.2, T035/C019).
5. ~~Argümanların çok-tokenli ifade olabilmesi~~ — GERÇEKLENDİ (v0.2, K-038).

## Dört soru süzgeci (mevcut A yüzeyi için)

Doğal — kısmen ✓ (usability verisi şart) · Deterministik ✓ (en-uzun-ad +
başlık ön-tarama kuralları) · Öğrenilebilir ✓ (tanım tarafı çok güçlü: "işlem
ortalamayı hesapla / sayıları al" sesli okunuşta kendini açıklıyor) ·
Savunulabilir — çağrı yüzeyi usability kapısını bekliyor; public imza modeli
K-086/spec-10 ile çağrı sırasından bağımsızdır.

## Korpus etkisi

Gerçek katılımcı eşiği sağlanmadan korpus veya parser göçü yapılmaz. Karar
geldiğinde ham anonim formlar ve özet; bu RFC, spec/02, parser/formatter/LSP,
golden ve anti-example korpusu, README, sürüm notu, v1 kapısı ve backlog tek
atomik değişiklikte güncellenir. A kalırsa da karar kanıtı korpusa bağlanır;
B/C yönü seçilirse eski yüzey aynı ana sürümde sessizce kaldırılmaz.
