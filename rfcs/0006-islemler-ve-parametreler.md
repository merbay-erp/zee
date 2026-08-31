# RFC-0006 — İşlemler ve Parametreler

- **Durum:** **geçici kabul** (31 Ağu 2026, kurucunun devrettiği yetkiyle):
  Seçenek A (için/ile yüzeyi) resmî yüzeydir; usability oturumları ONAY
  KAPISI olarak kalır — oturum bulguları aksini gösterirse bu karar B lehine
  revize edilir (K-032). Karar gerekçesi §3'te
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-016 (çağrı sözdizimi), K-014 (kelime çakışmaları), K-007 (yanıt)
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
  yalnız gövdenin başında ve tam iki kelimelik satır olarak tanınır.
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

1. **Tanım çağrıdan önce gelir.** Çağrı tanıma, "satır/bölge tanımlı bir işlem
   adıyla bitiyor mu?" sorusuyla yapılır; bilinmeyen ada çağrı diye bir durum
   sözdiziminde yoktur (S004'e düşer, öneri açıklar).
2. **En uzun ad önce eşlenir** (determinizm; iç içe ad çakışmalarında).
3. Argümanlar addan önce gelir ve `için` ya da `ile` ayracıyla biter (S019);
   birden çok argüman `ve` ile ayrılır; her dilim tam bir ifade bölgesidir
   (v0.2, K-038): `tabanın tam kısmı için yuvarla` geçerli.
4. **v0 monomorfizmi:** gövde İLK çağrının argüman türleriyle denetlenir; imza
   sabitlenir (T017), parametre sayısı uymazsa T015. Özyineleme v0'da yok (T016).

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

1. Özyineleme: v1'de gelmeli (T016 kalkar); denetim sırası için işlem
   imzalarının ön-bildirimi ya da iki-geçişli denetim gerekir.
2. Parametrelerde açık tür: public API ilkesi (manifesto 6) gereği
   `sayıları al (Liste<TamSayı>)` benzeri isteğe bağlı tür eki — sözdizimi
   tasarlanmadı.
3. Çok değerli dönüş (K-023'ün "hepsini bekle" sorusuyla birleşik).
4. ~~Argümanların çok-tokenli ifade olabilmesi~~ — GERÇEKLENDİ (v0.2, K-038).

## Dört soru süzgeci (mevcut A yüzeyi için)

Doğal — kısmen ✓ (usability verisi şart) · Deterministik ✓ (en-uzun-ad +
tanım-önce kuralları) · Öğrenilebilir ✓ (tanım tarafı çok güçlü: "işlem
ortalamayı hesapla / sayıları al" sesli okunuşta kendini açıklıyor) ·
Savunulabilir — monomorfizm ve özyineleme kısıtları v1'de kalkmalı.

## Korpus etkisi

Karar A'da kalırsa yok. B eklenirse golden 12/14'e alternatif biçim örneği
eklenir; C gelirse korpus ve anti-örnek A06 gerekçesi güncellenmek zorunda —
bu da C'nin maliyetinin bir parçası.
