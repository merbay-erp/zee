# RFC-0008 — Seçenek ve Sonuç

- **Durum:** **geçici kabul** (31 Ağu 2026; K-091 revizyonu 1 Eyl 2026 —
  `hatasını döndür`, Sonuç<değer, Hata>, otomatik sarmalama,
  akış-duyarlı daraltma ve yapılandırılmış hata yüzeyi gerçeklendi;
  onay kapısı: usability oturumları)
- **Tarih:** 31 Ağustos 2026; K-091 revizyonu 1 Eylül 2026
- **İlgili günlük kayıtları:** K-017 (var/yok), K-018 (dene), K-091 (Hata)
- **İlgili golden programlar:** 15, 16, 17
- **Gerçekleme:** YokSabiti/SecenekVar/IcDeger/SonucBasarili/SonucHatasi/
  DosyaOkumayiDene ve `HataDegeri` (`agac.rs` + üç katman);
  S044, C008/C009/C012, T018/T023/T024/T052

## Özet

Dilde null yoktur. "Değer olmayabilir" Seçenek'tir ve Türkçenin en doğal
ikilisiyle konuşur: **var/yok**. "İşlem başarısız olabilir" Sonuç'tur:
**dene → başarılıysa → değeri/hatası**. Hata tarafı düz metin değil;
çocuğa anlaşılır mesajı, profesyonele kararlı kodu ve bağlamı birlikte veren
bir `Hata` değeridir.

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
  Sonuç<değer, Hata> olur; başarı dalları çözümleyicide işaretlenip çalışma
  zamanında otomatik sarılır. Sonuç'u olduğu gibi geçiren işlem (tek dönüş
  türü zaten Sonuç) ÇİFT SARILMAZ — testli.
- **Sorgu:** `başarılıysa` / `başarısızsa`. **Erişim:** `değeri` (başarısızken
  C009), `hatası` (başarılıyken C009) — yanlış tarafa erişim daima yakalanır.
- **Düz biçim sözleşmesi:** dene'siz okuma (`dosyasının satırları`) hata
  anında C012 çalışma hatası verir ve tanısı dene'yi önerir. Kural: **hata
  YÖNETİLECEKSE dene, yönetilmeyecekse düz biçim** — programın kısa hali
  kirlenmeden, hata yönetimi isteyene açık.

## 3. Yapılandırılmış Hata (K-091)

`Hata` değişmez, birinci sınıf ve etiketli bir değerdir:

| Alan | Tür | Anlam |
|---|---|---|
| `kodu` | Metin | Makinece eşlenen kararlı etiket |
| `mesajı` | Metin | İnsana dönük Türkçe açıklama |
| `nedeni` | Seçenek\<Hata\> | İsteğe bağlı alt neden; zincir kurar |
| `verisi` | Sözlük\<Metin, Metin\> | Sıra korumalı tanı bağlamı |

Üretim biçimleri:

```zee
"olmadı" hatasını döndür
"DOSYA_YOK" kodlu "Dosya bulunamadı" hatasını döndür
"AYAR_OKUNAMADI" kodlu "Ayarlar yüklenemedi" hatasını alt_hata nedeniyle döndür
"KAYIT_GECERSIZ" kodlu "Kayıt doğrulanamadı" hatasını bilgi verisiyle döndür
```

İlk satır geriye uyum içindir ve `GENEL` kodlu Hata üretir. Yapılandırılmış
kod metin sabitidir; `[A-Z][A-Z0-9_]*` biçimindedir (S044). Neden `Hata`, veri
Metin sözlüğü olmalıdır (T052). İkisi birlikteyse önce `nedeniyle`, sonra
`verisiyle` gelir. Var olan Hata `hata hatasını döndür` ile yapısını kaybetmeden
yeniden yayılır; zenginleştirme gerekiyorsa yeni kodlu hata eskiyi neden olarak
sarır. Böylece neden grafiği üretim anında yönlü ve döngüsüzdür.

```zee
sonuç başarısızsa
    hata sonucun hatası olsun
    kod hatanın kodu olsun
    koda göre
        "DOSYA_YOK" ise
            "Dosyayı seçer misin?" yaz
        değilse
            hatanın mesajı yaz

    neden hatanın nedeni olsun
    neden varsa
        alt nedenin değeri olsun
        altın kodu yaz
```

`sonucun hatası` artık `Hata` döndürür. Bununla birlikte `hata yaz`, metin
birleştirme ve `hatanın metni` yalnız mesajı basar; eski program çıktıları
değişmez. Tam yapı `hatanın json metni` ile deterministik olarak serileşir.
Yerleşik denemeler `DOSYA_OKUMA`, `SAYI_BICIMI` ve `ONDALIK_BICIMI` kodlarını
üretir. Kodlar public sözleşmenin parçasıdır; değişmeleri semver incelemesi
ister.

## 4. Panik ayrımı (master plan bölüm 9)

Beklenen hatalar Sonuç ile taşınır; C-kodlu çalışma hataları (sıfıra bölme,
boş listenin ilki...) invariant ihlalidir ve programı durdurur. v0'da ikisi de
Türkçe tanıyla biter; fark, Sonuç'un programa DEVAM şansı vermesidir.

## 5. Açık sorular

1. ~~Kullanıcı işlemlerinden Sonuç ve yapılandırılmış hata~~ — GERÇEKLENDİ:
   `hatasını döndür`, K-091 `Hata` kod/mesaj/neden/veri sözleşmesi.
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
"yok" zaten bilinen kelime) · Savunulabilir ✓ (kod ve zincir açık;
eski metin gösterimi korunuyor).

## Korpus etkisi

Golden 15/16'nın eski insan çıktısı korunur. K-091 conformance korpusu;
kod/eşleme, neden+veri, deterministik JSON, yeniden yayma, üç yerleşik kod ve
S044/T052 olumsuzlarını ayrıca kilitler. `dene`nin yeni işlemlere genelleşmesi
gelecekte kendi olumlu/olumsuz korpusunu ister.
