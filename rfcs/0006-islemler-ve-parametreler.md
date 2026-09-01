# RFC-0006 — İşlemler ve Parametreler

- **Durum:** **geçici kabul** (31 Ağu 2026, kurucunun devrettiği yetkiyle):
  Seçenek A (için/ile yüzeyi) resmî yüzeydir; usability oturumları ONAY
  KAPISI olarak kalır — oturum bulguları aksini gösterirse bu karar B lehine
  revize edilir (K-032). Karar gerekçesi §3'te
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-016 (çağrı sözdizimi), K-014 (kelime çakışmaları), K-007 (yanıt), K-067 (sayısal genişleme)
- **İlgili golden programlar:** 12, 13, 14, 15, 30
- **Gerçekleme:** `islem_ayristir`/`cagri_kalibi` (`ayristirici.rs`), `cagri_denetle`
  (`cozumleyici.rs`), `islem_cagir` (`yorumlayici.rs`)

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
- K-083 progressive disclosure: public/paket API'sinde
  `sayıları Ondalık listesi olarak al` açık sözleşmesi kullanılabilir. Bir
  işlemin bütün parametreleri açık ya da bütünü çıkarımlı olmak zorundadır.
  Açık gövde çağrı beklemeden denetlenir; dönüş türü gövdeden çıkarılıp
  sabitlenir ve çağrılar imzayı terfi ettiremez.
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
   public API için v1 sözleşmesi değildir. K-083 açık parametre sözleşmesini
   ekledi; generic ve paket/public zorunluluğu V1-P0-01'de kalır.
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

**Öneri (usability'de sınanacak):** A kalır; B'nin cümle biçimi, dönüş değeri
olan işlemler İÇİN EK olarak değerlendirilir (ikisi aynı anda yaşayabilir:
A ifade konumunda, B öğretici/adım-adım stilde). C yalnız A/B yetersiz kalırsa.

## 4. Bilinen gerilimler (K-014 devamı)

- `al` parametre kelimesi işlem adlarında geçemez sayılmalı mı? (30'da
  "karesini al" bu yüzden "karesini hesapla" oldu.) Öneri: RFC kesinleşirken
  `al` işlem adının SON kelimesi olamaz kuralı.
- `yanıt` (K-007) ve `sonuç` bağlamsal adlarının işlem gövdesindeki durumu
  belgelendi; ayrılmış kelime listesi RFC-0002 §5'e bağlı.

## 5. Açık sorular

1. ~~Parametrelerde açık tür~~ — K-083 ile
   `sayıları TamSayı listesi olarak al` gerçeklendi. Kalan: paket/public
   sınırında zorunluluk ve açık ABI uyumluluk politikası.
2. Generic işlem ile açık türün birlikte progressive disclosure modeli;
   generic sözdizimi hâlâ AÇIK.
3. Çok değerli dönüş (K-023'ün "hepsini bekle" sorusuyla birleşik).
4. ~~Özyineleme ve tanım-sonrası çağrı~~ — GERÇEKLENDİ (v0.2, T035/C019).
5. ~~Argümanların çok-tokenli ifade olabilmesi~~ — GERÇEKLENDİ (v0.2, K-038).

## Dört soru süzgeci (mevcut A yüzeyi için)

Doğal — kısmen ✓ (usability verisi şart) · Deterministik ✓ (en-uzun-ad +
başlık ön-tarama kuralları) · Öğrenilebilir ✓ (tanım tarafı çok güçlü: "işlem
ortalamayı hesapla / sayıları al" sesli okunuşta kendini açıklıyor) ·
Savunulabilir — çağrı yüzeyi usability kapısını, public imza modeli ise
V1-P0-01'i bekliyor.

## Korpus etkisi

Karar A'da kalırsa yok. B eklenirse golden 12/14'e alternatif biçim örneği
eklenir; C gelirse korpus ve anti-örnek A06 gerekçesi güncellenmek zorunda —
bu da C'nin maliyetinin bir parçası.
