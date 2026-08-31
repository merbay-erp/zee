# RFC-0007 — Temel Tür Sistemi

- **Durum:** taslak
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-009 (dönüşüm), K-014 bulgu 3 (tür/değer adları)
- **İlgili golden programlar:** tümü; özellikle 08–11, 19–22
- **Gerçekleme:** `Tur`/`VeriTuru`/`SozlukDegerTuru` (`cozumleyici.rs`), T-kodları

## Özet

Statik tür güvenliği + yerel tür çıkarımı (manifesto 6). Kullanıcı tür yazmaz;
tür adları yalnız yapı alanlarında görünür. Tehlikeli örtük dönüşüm yoktur.

## 1. Türler (v0 durumu)

| Tür | Gerçekleme | Master plan (bölüm 8) hedefine göre |
|---|---|---|
| TamSayı | i64; taşma denetimli (C002), tam bölme (C003) | ✅ |
| Metin | UTF-8; uzunluk karakter sayısıdır | ✅ |
| Mantıksal | `doğru`/`yanlış` | ✅ |
| Liste\<T\> | T ∈ {TamSayı, Metin, Sözlük-satırı}; homojen (T011) | ✅ kısmi |
| Sözlük\<Metin, T\> | T ∈ {TamSayı, Metin}; ekleme sırası korunur | ✅ kısmi (anahtar yalnız Metin) |
| Seçenek\<T\> | `yok` + değer; RFC-0008 | ✅ |
| Sonuç | değer/hata ikisi de Metin (v0) | ✅ kısmi |
| Tarih, Saat | IO soyutlamalı saat; Türkçe basım | ✅ |
| Yapılar | kullanıcı tanımlı; alanlar TamSayı/Metin/Mantıksal | ✅ |
| GerçekSayı/Ondalık | — | ❌ açık soru §4.1 |
| Küme\<T\>, Süre, Para | — | ❌ stdlib fazları |

## 2. Çıkarım ve açık tür

- Her ifadenin türü yapısından çıkar; değişken türü ilk bağlamada sabitlenir
  ve değişemez (T002 — golden korpusun en öğretici hatası).
- Açık tür yazımı yalnız yapı alanlarında: `yaş TamSayı` (T027 ile doğrulanır).
- Boş koleksiyonların öğe türü v0'da TamSayı varsayılır (BosListe/BosSozluk);
  tam çıkarım (kullanıma bakarak) v1 adayı.

## 3. Dönüşümler

- **Örtük dönüşüm yok.** Metin ↔ sayı yalnız açık kalıpla: `<metnin> sayısı`
  (başarısızlık C004 çalışma hatası — RFC-0008'in "dene" sorusuna bağlı).
- `yaz` her değeri GÖSTERİR (metne çevirme değil, basım biçimi): sayılar onluk,
  Mantıksal `doğru/yanlış`, listeler virgüllü, Tarih Türkçe ay adıyla.
  Gösterim biçimleri spesifikasyonda sabitlenmiştir; yereller arası değişmez
  (determinizm).
- Karşılaştırma: büyüklük yalnız sayılar; eşitlik aynı tür (T001).

## 4. Açık sorular

1. **GerçekSayı/Ondalık:** en büyük eksik. Türkçe ondalık ayracı virgüldür ama
   virgül liste ayracıdır (RFC-0002 §6.3). Adaylar: (a) nokta kabul et
   (yazım-dışı ama tanıdık), (b) `3 tam 14` sözel kalıbı, (c) bağlamla virgül.
   Karar ayrı RFC ister; Para/Süre türleri buna bağlı.
2. **Generics ve trait/arayüz:** kullanıcı işlemleri v0'da ilk-çağrı
   monomorfizmiyle idare ediyor (RFC-0006 §2.4); gerçek çokbiçimlilik ADR işi.
3. **Tür adlarının Türkçe çekimi:** hata mesajlarında "Liste<TamSayı>" teknik
   gösterimi kullanılıyor; çocuk modunda "tam sayı listesi" okunuşu düşünülebilir.
4. Sözlük anahtarının TamSayı olabilmesi; iç içe koleksiyonlar.

## Dört soru süzgeci

Doğal ✓ (tür adları Türkçe, çekimli kullanım hedefi not edildi) ·
Deterministik ✓ (örtük dönüşüm yasağı + T-kod bekçileri) · Öğrenilebilir ✓
(tür yazmadan başlanır; T002 en sık ilk ders) · Savunulabilir — Ondalık
gelene dek kısmi ✓.

## Korpus etkisi

Yok. Ondalık kararı geldiğinde golden 04'e (bölme) ve yeni golden programlara
etkisi olacak; o RFC'de ele alınır.
