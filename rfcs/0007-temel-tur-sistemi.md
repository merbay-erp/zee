# RFC-0007 — Temel Tür Sistemi

- **Durum:** **geçici kabul** (31 Ağu 2026 — tür yüzeyi gerçeklendi; dönüş
  birleşimi, genişleme ve daraltma testli; onay kapısı: usability oturumları.)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-009 (dönüşüm), K-014 bulgu 3 (tür/değer adları)
- **İlgili golden programlar:** tümü; özellikle 08–11, 19–22
- **Gerçekleme:** `Tur`/`VeriTuru`/`SozlukDegerTuru` (`cozumleyici.rs`), T-kodları

## Özet

Statik tür güvenliği + yerel tür çıkarımı (manifesto 6). Başlangıçta kullanıcı
tür yazmaz; yapı alanı ve public/paket işlem parametresinde açık sözleşmeye
aşamalı geçer. Tehlikeli örtük dönüşüm yoktur.

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
| Ondalık | keyfî katsayılı onluk değer; yalnız sonsuz bölüm 34 anlamlı hane | ✅ K-092 |
| Küme\<T\>, Süre, Para | — | ❌ stdlib fazları |

## 2. Çıkarım ve açık tür

- Her ifadenin türü yapısından çıkar; değişken türü ilk bağlamada sabitlenir
  ve değişemez (T002 — golden korpusun en öğretici hatası).
- Açık tür yazımı yapı alanında `yaş TamSayı`; işlem sözleşmesinde K-083/K-086
  ile `sayıyı Ondalık olarak al` + `Ondalık döndürür`dür. Açık işlem çağrı
  beklemeden denetlenir; T037–T042 kısmi/bilinmeyen/uyuşmayan sözleşmeyi ve
  dönüşsüz kalan akış yolunu reddeder.
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

## 4. Değer kimliği ve kopyalama

Bütün türler değer semantiğindedir: atama, argüman geçişi, dönüş ve koleksiyona
ekleme derin değer kopyası üretir. v1'de kullanıcıya görünür referans kimliği
ve paylaşılan değiştirilebilir alias yoktur. Liste gezmesinin sınırlı
değer-sonuç imleci ile T053 kaynak sabitliği RFC-0019/spec-17'dedir.

## 5. Açık sorular

1. **GerçekSayı/Ondalık:** ÇÖZÜLDÜ — RFC-0013/K-092 bitişik virgül kuralı +
   keyfî hassasiyetli onluk aritmetikle gerçeklendi
   (`3,14`; `0,1+0,2=0,3`). Kullanıcıya ikinci, ikilik kayan noktalı sayı
   türü açılmaz; K-125/ADR-029 FFI'da da örtük binary32/binary64 eşlemesini
   yasaklar. Bilimsel gösterim ayrı bir sözdizimi kararıdır.
2. **Generics ve trait/arayüz:** başlangıç işlemleri çağrı-güdümlü kalır;
   public işlemler tam açık ve v1'de bilinçli monomorfiktir (K-083/K-086,
   spec/10). Gerçek çokbiçimlilik v2+ ADR/RFC işidir.
3. **Tür adlarının Türkçe çekimi:** hata mesajlarında "Liste<TamSayı>" teknik
   gösterimi kullanılıyor; çocuk modunda "tam sayı listesi" okunuşu düşünülebilir.
4. Sözlük anahtarının TamSayı olabilmesi; iç içe koleksiyonlar.

## Dört soru süzgeci

Doğal ✓ (tür adları Türkçe, çekimli kullanım hedefi not edildi) ·
Deterministik ✓ (örtük dönüşüm yasağı + T-kod bekçileri) · Öğrenilebilir ✓
(tür yazmadan başlanır; T002 en sık ilk ders) · Savunulabilir — Ondalık
gelene dek kısmi ✓.

## Korpus etkisi

K-083: golden 33 açık Ondalık imzasını ve runtime genişlemesini sabitler.
