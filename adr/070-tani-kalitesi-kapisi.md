# ADR-070 — Tanı kalitesi kapısı

- **Durum:** kabul
- **Tarih:** 6 Eylül 2026
- **İlgili kayıt:** K-166, RFC-0010, ADR-024 revizyonu, ADR-025, V1-P1-27

## Bağlam

RFC-0010 her tanının kod, Türkçe mesaj, kesin konum ve öneri taşımasını
ister; ADR-024 çoklu tanı hattını kurar. Fakat "acemi bir çocuğun en sık
yaptığı hatalarda tanı gerçekten hatalı satırı mı gösteriyor, öneri var mı,
tek hata kaç tanı üretiyor" sorusunun ölçüsü yoktu. Üçüncü dış inceleme K-166
ile en sık 50 hata için span/öneri/gürültü düzeltme başarısı istedi. Gerçek
kullanıcı frekansı (K-161/K-162) henüz yoktur; ölçülebilir bir vekil gerekir.

## Karar

1. **Mutasyon korpusu.** `tani_kalitesi` ikilisi golden korpusuna (birim
   kullananlar hariç) on bir deterministik acemi hatası uygular: girinti
   silme, sekme girintisi, kapanmayan tırnak, `=`, eksik `olsun`, ad yazım
   hatası, İngilizce `print`, büyük harf, nokta ondalık, parantez ve boş blok.
   Her mutasyon hatalı satırı bilir; ad yazım hatasında beklenen konum ilk
   kullanım satırıdır.
2. **Hata sınıfı ve “en sık”.** Sınıf = (operatör, ilk tanı kodu). En az üç
   vakalı sınıflar frekans sırasıyla en çok 50 taneye kadar kapıdadır; bugün
   korpus 14 sınıf üretir. Sıralama gerçek kullanıcı verisi gelene kadar bu
   korpusun frekansıdır ve raporda böyle yazılır.
3. **Ölçütler.** Öneri %100 (RFC-0010); ilk tanının işareti hatalı satırda
   ±1 satır ≥ %90; gürültü (tek hatadan doğan tanı sayısı) medyan ≤ 2 ve tepe
   ≤ 4. İhlal yalnız `docs/tani-kalitesi-istisnalari-v1.tsv` içinde ≥40
   karakter gerekçe ve K-işi ile geçebilir; rapor bayatsa CI kırılır.
4. **İlk düzeltmeler.** İşlem/yapı/`göre` başlıklarının S007'si öneri
   kazandı; gövde dışına düşen `sayıyı al` parametre satırı oturum kalıbı
   S043'ü yerine yol gösteren S004 üretir; çoklu tanı hattında aynı eksik
   tanımın A001/A003/A007 tekrarları (ekli biçimler dahil) bastırılır, düşen
   satırın baş sözü ve başarısız `olsun` tanımı kök neden sayılır, başlığı
   bozuk işlemin çağrıları T016 iç tutarlılık tanısı üretmez (ADR-024 §8).

## Reddedilen seçenekler

- **Gerçek kullanıcı verisini beklemek:** ölçü olmadan düzeltme kanıtı
  verilemez; korpus vekildir ve insan verisi gelince yeniden sıralanır.
- **Her gürültüyü parser'da bastırmak:** ardıl tanılar bazen gerçek ikinci
  hatadır; yalnız tek eksik tanımın tekrarları ayıklanır.
- **Sabit 50 vaka listesi elle yazmak:** golden büyüdükçe kendiliğinden
  büyüyen mutasyon daha dürüst ve bakımı ucuzdur.

## Sonuçlar

- İlk taban 244 vaka/14 sınıf: düzeltmelerden önce 6 ihlal (S007 öneri %60–62,
  gürültü tepe 5–9); sonra sıfır ihlal ve istisna.
- Yeni golden programı korpusu büyütür; yeni tanı kodu ya da kurtarma
  değişikliği raporu yeniler. Kapı dil semantiğine dokunmaz; ADR-025 kimlik
  kuralları geçerlidir.
