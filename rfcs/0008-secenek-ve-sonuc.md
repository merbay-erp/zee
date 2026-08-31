# RFC-0008 — Seçenek ve Sonuç

- **Durum:** **geçici kabul** (31 Ağu 2026 — tam yüzey gerçeklendi:
  `hatasını döndür`, Sonuç<değer>, otomatik sarmalama VE akış-duyarlı
  daraltma T036/K-037; onay kapısı: usability oturumları)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-017 (var/yok), K-018 (dene)
- **İlgili golden programlar:** 15, 16, 17
- **Gerçekleme:** YokSabiti/SecenekVar/IcDeger/SonucBasarili/SonucHatasi/
  DosyaOkumayiDene (`agac.rs` + üç katman); C008/C009/C012, T018/T023/T024

## Özet

Dilde null yoktur. "Değer olmayabilir" Seçenek'tir ve Türkçenin en doğal
ikilisiyle konuşur: **var/yok**. "İşlem başarısız olabilir" Sonuç'tur:
**dene → başarılıysa → değeri/hatası**.

## 1. Seçenek (K-017)

```
işlem ilk çift sayıyı bul
    sayıları al
    her sayı için
        sayı çiftse
            sayıyı döndür
    yok döndür

bulunan sayılar için ilk çift sayıyı bul olsun
bulunan varsa
    "Bulundu: " ile bulunanın değeri yaz
değilse
    "Çift sayı yok" yaz
```

- **Doğuş:** bir işlem hem gerçek değer hem `yok` döndürüyorsa dönüş türü
  Seçenek\<T\> olur (dönüş birleşimi, T018). Yalnız `yok` döndüren işlem
  hatadır ("içi belirlenemiyor").
- **Sorgu:** `varsa` / `yoksa` (T023: yalnız Seçenek üzerinde).
- **Erişim:** `<adın> değeri` — yalnız `varsa` (ya da `yoksa`nın `değilse`
  dalı) içinde statik güvenli; dal dışı korumasız erişim **T036** derleme
  hatasıdır (v0.2, K-037). C008 artık yalnız iç savunmadır.
- **Kelime seçimi:** `boş` bilinçli olarak koleksiyonlara ayrıldı
  (`boş liste`, `argümanlar boşsa`); yokluk her zaman `yok`. İki kavramın
  karışmaması çocuk öğretiminde önemli.

## 2. Sonuç (K-018)

```
sonuç "veriler.txt" dosyasını okumayı dene olsun

sonuç başarılıysa
    sonucun değerini yaz
değilse
    "Okunamadı: " ile sonucun hatası yaz
```

- **Doğuş:** yerleşik dene-ifadeleri VE kullanıcı işlemleri: bir işlemde
  `"sıfıra bölünmez" hatasını döndür` ile değer dönüşü karışırsa işlemin türü
  Sonuç<değer> olur; başarı dalları çözümleyicide işaretlenip çalışma
  zamanında otomatik sarılır. Sonuç'u olduğu gibi geçiren işlem (tek dönüş
  türü zaten Sonuç) ÇİFT SARILMAZ — testli.
- **Sorgu:** `başarılıysa` / `başarısızsa`. **Erişim:** `değeri` (başarısızken
  C009), `hatası` (başarılıyken C009) — yanlış tarafa erişim daima yakalanır.
- **Düz biçim sözleşmesi:** dene'siz okuma (`dosyasının satırları`) hata
  anında C012 çalışma hatası verir ve tanısı dene'yi önerir. Kural: **hata
  YÖNETİLECEKSE dene, yönetilmeyecekse düz biçim** — programın kısa hali
  kirlenmeden, hata yönetimi isteyene açık.

## 3. Panik ayrımı (master plan bölüm 9)

Beklenen hatalar Sonuç ile taşınır; C-kodlu çalışma hataları (sıfıra bölme,
boş listenin ilki...) invariant ihlalidir ve programı durdurur. v0'da ikisi de
Türkçe tanıyla biter; fark, Sonuç'un programa DEVAM şansı vermesidir.

## 4. Açık sorular

1. **Kullanıcı işlemlerinden Sonuç:** GERÇEKLENDİ — `hatasını döndür`.
   Kalan: hata türünün Metin'den zengin türe evrimi (kod + mesaj + veri).
2. ~~Akış-duyarlı daraltma~~ — GERÇEKLENDİ (v0.2, K-037/T036): varsa /
   başarılıysa / başarısızsa dalları ve `değilse` tersinmeleri daraltır;
   tam veri-akışı analizi bilinçli olarak yok (anlaşılabilirlik).
3. **`dene`nin genelleşmesi:** `sayısını almayı dene` (C004'ü Sonuç'a çevirir),
   ileride `adresten getirmeyi dene`. Kalıp: mastar + dene.
4. **Zorunlu ele alma:** kullanılmayan Sonuç değeri uyarı vermeli mi
   (Rust `must_use` benzeri)? Eğitimde faydalı; gürültü riski usability'de
   sınanmalı.

## Dört soru süzgeci

Doğal ✓ (var/yok ve dene Türkçede tam karşılık) · Deterministik ✓ (yanlış
taraf erişimi daima tanılı) · Öğrenilebilir ✓ (null kavramı hiç öğretilmiyor —
"yok" zaten bilinen kelime) · Savunulabilir — §4.1 kapanınca tam ✓.

## Korpus etkisi

§4.1 kabulünde golden 15/16'ya kullanıcı-tanımlı Sonuç örneği eklenir;
§4.3 kabulünde golden 04 (sayıya çevirme) dene'li varyant kazanır.
