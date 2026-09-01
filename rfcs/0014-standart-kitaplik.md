# RFC-0014 — Standart Kitaplık: sınır, adlandırma, dağıtım

- **Durum:** taslak — **çalışan prototip ekli** (1 Eyl 2026): `kitaplik/`
  altında zee'yle yazılmış DÖRT birim, ikiliye gömülü ve testli.
  Kesinleşme kapısı: usability oturumları (ADR-008 Stage 1 kapısıyla aynı).
- **Tarih:** 1 Eylül 2026
- **Bağlam:** master plan bölüm 13; ADR-008 (Stage 1: "stdlib'in parçaları
  yeni dilde"); RFC-0009 (birim modeli); beslendiği kayıtlar K-046 (kalan),
  K-048.

## 1. Sorun

Dil yüzeyi bugüne dek çekirdeğe gömülü kalıplarla büyüdü (`dosyanın
satırları`, `adresinden gelen yanıt`, `bugünün tarihi`...). Bu, başlangıç
için doğruydu: kalıp = sözdizimi = tanı kalitesi. Ama sürdürülemez —
her yeni yetenek ayrıştırıcıyı büyütemez. Nerede duracağını bilen bir
sınır ve zee'yle yazılmış bir kitaplık katmanı gerekiyor.

## 2. Sınır ilkesi: çekirdek nedir, kitaplık nedir

**Çekirdekte kalır** (ayrıştırıcı kalıbı olmayı hak eder):

1. Yeni **söz dizimi** ya da **bağlama biçimi** gerektirenler (olsun, ise,
   döngüler, işlem/yapı/test, dene, eşzamanlı, geldiğinde).
2. **IO sınırını** geçenler — determinizm sözü IO soyutlamasından geçer
   (yaz/sor, dosya, ağ, saat, rastgele, sensör).
3. **Tür sisteminin parçası** olanlar (değeri/hatası, varsa, dönüşümler).

**Kitaplığa gider** (zee'yle yazılır, birim olarak gelir):

- Saf hesap ve dönüşümler: matematik, liste/metin yardımcıları, sıralama,
  arama, biriktirme kalıpları.
- Alan bilgisi: birim çevirileri, istatistik, oyun yardımcıları.
- İleride (Faz kapılarıyla): JSON/CSV'nin zengin biçimleri, DB sürücü
  sözleşmesi, kripto (bölüm 13 "ayrı modül" der), GPIO paketleri.

**Turnusol:** "Bu özellik IO'ya ya da yeni sözdizimine muhtaç mı?" Hayırsa
kitaplıktır. (K-046 kalan işlemi çekirdeğe girdi çünkü sözdizimi kalıbıydı;
obeb/okek kitaplıktadır çünkü kalanla YAZILIR.)

## 3. Adlandırma düzeni

- **Birim adları:** tek kelime ya da alt çizgili, küçük harf, Türkçe:
  `matematik`, `liste_araclari`. (RFC-0009 dosya adı kuralları geçerli.)
- **İşlem adları:** yüklem-sonlu ve iyelikli — dilin geri kalanıyla aynı
  ritim: `mutlak değerini hesapla`, `en büyüğünü bul`, `obebini hesapla`.
  İngilizce ad, kısaltma, sembol YASAK (A10 anti-örneği kitaplık için de
  bağlayıcı).
- **Parametre adları** anlam taşır (`sayıyı al`, `bölüneni al`) — imza
  belgelenirken bu adlar sözleşmenin parçasıdır.

## 4. Dağıtım: gömülü kitaplık

Standart birimler **derleyici ikilisine gömülür** (include_str). Sonuç:

- Kurulumsuz ve internetsiz çalışır (bölüm 16 "internetsiz okul kurulumu");
  playground dahil — tarayıcıdaki derleyici de aynı kitaplığı taşır.
- Sürüm = derleyici sürümü: kitaplık ile dil asla ayrı düşmez (v0 için
  doğru sadeleştirme; paket katmanı Faz 5'te bunun ÜSTÜNE gelir).

**Çözüm sırası:** `X birimini kullan` önce dosyanın klasörüne bakar
(RFC-0009 aynen), bulamazsa gömülü kitaplığa düşer. Yerel dosya gömülüyle
aynı adı taşıyorsa yerel kazanır — bu sessiz gölgeleme DEĞİLDİR: kullanıcı
kendi yazdığı dosyayı bilerek adlandırmıştır; A010 tanısı bulunamayan ad
için gömülü birimleri de listeler.

## 5. Kararlılık sözleşmesi

- Gömülü bir birimdeki işlem imzası yayınlandıktan sonra ancak sürüm
  notuyla değişir (docs/surumler.md; bölüm 23 "sessiz kırılma yok").
- Her gömülü birim kendi `test` bloklarını taşır ve CI'da koşar: kitaplık,
  golden korpusla aynı hakemliğe tabidir.
- Bu RFC kesinleşene dek gömülü kitaplık **deneysel** etiketlidir —
  ADR-008 Stage 1 resmî olarak usability kapısından sonra açılır; buradaki
  prototip o aşamanın yürütülebilir taslağıdır.

## 6. İlk katman haritası (bölüm 13 ↔ durum)

| Bölüm 13 katmanı | Durum |
|---|---|
| Temel (Metin, Sayı, Liste, Sözlük, Seçenek, Sonuç) | çekirdekte ✅; yardımcılar → `matematik`, `liste_araclari` (bu RFC) |
| Küme | AÇIK — dilde yok; ihtiyaç korpusla kanıtlanınca tür RFC'si |
| Sistem (Dosya/Dizin/Yol/Süreç/Ortam) | dosya çekirdekte; dizin/süreç/ortam Faz sonrası (çocuk modu etkisi düşünülerek) |
| Zaman | çekirdekte ✅ (Tarih/Saat/Süre) |
| Ağ | HTTP çekirdekte; TCP/UDP/DNS Faz 5 |
| Veri (JSON/CSV/DB) | okuma çekirdekte; zengin biçim + DB sözleşmesi Faz 3+ |
| Güvenlik (hash/HMAC) | Faz sonrası; "düşük seviyeli kripto ayrı modülde" |
| Test | çekirdekte ✅ (test/olmalı); fixture/snapshot/property AÇIK |
| IoT | simülatör çekirdekte; GPIO/seri paketleri Faz 6 |
| Grafik | Faz sonrası; "çekirdeğe gömülmez" (bölüm 13) aynen |

## 7. Prototipin içeriği (bu depoda, çalışır)

- **`matematik`**: `mutlak değerini hesapla` · `üssünü hesapla` (üs ≥ 0) ·
  `tam karekökünü hesapla` (Newton, taban) · `obebini hesapla` (Öklit,
  K-046 kalanıyla) · `okekini hesapla`.
- **`liste_araclari`**: `toplamını hesapla` · `en büyüğünü/küçüğünü bul` ·
  `ortalamasını hesapla` · `medyanını hesapla` (toplam/ortalama/medyan
  DAİMA Ondalık döner — K-070).
- **`metin_araclari`** (K-053 harf erişimi doğunca): `tersini hesapla` ·
  `ünlülerini say` · `baş harfini bul`.
- **`sozluk_araclari`**: `en çok geçeni bul` (eşitlikte ilk eklenen).

## 8. Açık sorular

1. Küme türü ve sözdizimi.
2. ~~Belge üretimi~~ — ASGARİ biçim gerçeklendi (K-049): `dil belge matematik`
   işlem başlıklarını ve test sayısını basar. Zengin biçim (açıklama satırları,
   örnekler) AÇIK.
3. Paket katmanı geldiğinde (Faz 5) gömülü kitaplığın paketlerle ilişkisi
   (gömülü = "sıfırıncı paket deposu" olarak mı kalır?).
4. Ondalık matematik (karekök vb. Ondalık sürümleri) — hassasiyet
   sözleşmesi RFC-0013'ün üstüne nasıl biner?

## Dört soru süzgeci

Doğal ✓ (işlem adları dilin ritminde) · Deterministik ✓ (saf hesap;
IO'suz) · Öğrenilebilir ✓ (obeb dersteki Öklit'in kendisi) ·
Savunulabilir — usability + bu RFC'nin kabulüyle tam ✓.
