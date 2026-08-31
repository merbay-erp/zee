# RFC-0011 — Structured Concurrency

- **Durum:** taslak (tasarım — gerçekleme Faz 5; K-023'ün sorusunu ÇÖZER)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-023 ("hepsini bekle sonrası sonucu döndür neyi döndürür?")
- **İlgili golden programlar:** 26 (paralel görevler), 27 (zaman aşımı)
- **Gerçekleme:** yok

## Özet

Eşzamanlılık blok-kapsamlıdır: `eşzamanlı olarak` bloğunda başlayan her iş,
o blok bitmeden sahiplenilir — sahipsiz (dangling) görev diye bir şey yoktur.

## 1. Temel model ve K-023'ün cevabı

```
eşzamanlı olarak
    profil müşterinin profilini getir
    faturalar müşterinin faturalarını getir
    cihazlar müşterinin cihazlarını getir

hepsini bekle
"Profil: " ile profil yaz
```

**Kural (K-023 çözümü):** `eşzamanlı olarak` bloğundaki her satır
`<ad> <ifade>` biçiminde bir GÖREV BAĞLAMASIDIR. `hepsini bekle`den sonra bu
adlar SIRADAN DEĞERLERDİR — görevlerin sonuçları adlara oturur. Master
plandaki örnekteki `sonucu döndür` belirsizdi; düzeltme: birleşik bir sonuç
isteniyorsa AÇIKÇA kurulur (bir yapıya konur ya da ayrı ayrı döndürülür).
Golden 26 bu RFC kabulünde buna göre revize edilir.

Determinizm: bekle'den ÖNCE görev adlarına erişim derleme hatasıdır (yeni
T-kodu); yani programın gözlemleyebildiği hiçbir şey zamanlamaya bağlı değildir.

## 2. Hata ve iptal

1. Görevlerden biri hata verirse: kalanlar İPTAL edilir, blok o hatayla biter.
   Hata yönetimi istenirse görev ifadesi `dene`li yazılır → ad Sonuç tutar
   (RFC-0008): `profil müşterinin profilini getirmeyi dene`.
2. Blok gövdesinden erken çıkış (`döndür`) tüm görevleri iptal eder —
   structured concurrency'nin özü: kapsam biter, işler biter.
3. İptal edilebilirlik: v1'de görevler yalnız BEKLEME noktalarında iptal
   edilir (ağ/dosya/süre beklemeleri); saf hesap döngüsü iptali §5.3.

## 3. Zaman aşımı (golden 27 yüzeyi)

```
5 saniye içinde
    veri "https://ornek.dev/rapor" adresinden gelen yanıt olsun
    verinin gövdesini yaz
yetişmezse
    "Zaman aşımı, sonra tekrar dene" yaz
```

- `N saniye içinde` bloğu kendi kapsamındaki işlere son tarih koyar; süre
  dolarsa içerideki işler iptal edilir, `yetişmezse` kolu çalışır.
- Süre sabitleri (`5 saniye`, `yarım saniye`, `2 dakika`) Süre türünü ister —
  RFC-0013 ailesine bağlı; v1 alt kümesi tam sayı + `saniye/dakika`.

## 4. Yürütme modeli (gerçekleme yönlendirmesi)

v1 hedefi **tek iş parçacıklı, işbirlikli** çalıştırıcıdır (async değil
"sıralı-görünümlü eşzamanlılık"): görevler yalnız bekleme noktalarında
dönüşümlü ilerler. Gerekçe: veri yarışı SINIFI yok (bölüm 10 "data race
varsayılan zorlaştırılır" hedefinin en güçlü hali: imkânsızlaştırılır);
determinizm testlerde korunur (IO soyutlaması sıralamayı da sahteleyebilir).
Gerçek paralellik (çok çekirdek) v2+ ve ayrı ADR.

## 5. Açık sorular

1. `hepsini bekle` dışında `ilkini bekle` (yarış) gerekli mi?
2. Görev sonucuna beklemeden kısmi erişim (akış/stream) — kapsam dışı, v2.
3. Saf hesap döngülerinin iptali (önleyici kesme yok — işbirlikli noktalar).
4. `eşzamanlı olarak` içinde döngüyle N görev başlatma (dinamik sayıda görev)
   ve sonuçların listeye toplanması sözdizimi.

## Dört soru süzgeci

Doğal ✓ ("hepsini bekle" günlük dil) · Deterministik ✓ (bekle-öncesi erişim
hatası + tek iş parçacıklı model) · Öğrenilebilir ✓ (kapsam = yaşam süresi,
başka kavram yok) · Savunulabilir ✓ (structured concurrency endüstride kanıtlı;
iptal/timeout birinci sınıf).

## Korpus etkisi

Golden 26 revize edilecek (`sonucu döndür` → açık kurulum); golden 27 aynen
kalır. Revizyon bu RFC kabul edildiğinde yapılır ve günlüğe işlenir.
