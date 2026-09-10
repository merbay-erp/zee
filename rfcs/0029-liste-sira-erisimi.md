# RFC-0029 — Listede sıra ile öğe erişimi (taslak)

- **Durum:** taslak — K-174 syntax freeze penceresi (6 Eylül–4 Ekim 2026)
  içinde uygulanmaz; freeze kapanışı ve K-161/K-162 insan verisinden sonra
  değerlendirilir.
- **Tarih:** 10 Eylül 2026
- **İş:** K-180 (kaynak: K-164/F010, K-179)
- **Karar:** yok (bu belge yalnız sorunu ve adayları sabitler)
- **Normatif yüzey:** yok — spec/20 katman kuralı ve RFC-0021 §6 kapısı
  uygulanana dek geçerli tek yol sayaç döngüsüdür.

## Özet

Zee listelerinde bugün yalnız `ilki`, `sonu`, `adedi` ve gezme vardır; N'inci
öğeye ulaşmanın tek yolu sayaçlı gezmedir. Bu boşluk iki bağımsız gerçek
üründe beş ayrı işlemde aynı kalıbı yeniden yazdırdı. Bu RFC sorunu kanıtla
sabitler, Türkçe yüzey adaylarını ve seçim ölçütlerini önden yazar; uygulama
freeze sonrasına bırakılır.

## Motivasyon (kanıt)

- **K-164 / kanıt özeti (F010):** `bilgiye çevir` başlık→alan eşlemesi,
  `çiftleri birleştir` iki listeyi sırayla eşleme — her ikisi de iç içe
  sayaçlı gezme (O(n²)); `f010-sira-ile-oge-erisimi` korpus çifti S015 ile
  reddi ve sayaçla çözümü taşır.
- **K-179 / kelime avı:** `sıradaki harfi al` (ipucu için iki harf listesini
  hizalama), `sıradaki kelimeyi al` (rastgele sıradaki kelime), skor
  satırının ikinci alanını çekme — üçü de sayaç döngüsü.
- ADR-074 §4 kuralı: iki bağımsız üründe tekrar eden sınıf tasarım
  maddesidir. Tek üründeki rahatsızlık anekdot sayılmıştı; ikinci ürün aynı
  sınıfı üç yerde daha üretti.
- Maliyet: her kullanım 6–8 satır ve bir yardımcı işlem; okunabilirlik
  düşüyor (Zee/Rust satır oranına katkı, docs/dogfood-karsilastirma.md).

## Tasarım adayları

Yüzey Türkçe sıra sayısıyla kurulmalı; sembol ve parantez yasağı (spec/02)
korunur. Sayım **1 tabanlıdır**: `ilki` zaten ilk öğedir, çocuk "üçüncü" der.

| Aday | Örnek | Katman (spec/20) | Not |
|---|---|---|---|
| A. Sıra sayısı + `öğesi` | `harflerin 3. öğesi` | erişim/postfix | Nokta, sayıdan sonra sıra ekidir; `3.` sözcüklemesi (spec/01) ondalık virgülle çakışmaz. |
| B. Yazıyla sıra | `harflerin üçüncü öğesi` | erişim/postfix | Okunaklı; sıra sözcüğü sonlu küme (birinci…onuncu) ya da morfoloji `-inci` eki ister. |
| C. Değişken sıra | `harflerin sıra. öğesi` / `harflerin sıradaki öğesi` | erişim/postfix | Sabit değil, ad taşır; asıl ihtiyaç budur (döngü sayacı). |

C olmadan A/B yalnız sabit sıra verir ve ürün ihtiyacını karşılamaz;
tasarımın çekirdeği "sıra bir ifadedir"dir. Aday yazım: `<liste>nin <sıra>.
öğesi` — sıra TamSayı ifadesi (ad ya da sabit), `.` sıra eki.

## Anlam

- `1 ≤ sıra ≤ adedi` ise öğenin **kopyası** (spec/17 değer semantiği).
- Aralık dışı: **çalışma hatası** (yeni C kodu), `ilki`'nin boş listedeki
  davranışıyla aynı sınıf. Beklenen arıza için RFC-0008 kalıbı:
  `<liste>nin <sıra>. öğesini almayı dene` → Seçenek ya da Sonuç (karar
  açık: boş = "yok" mu, hata mı?).
- Sıra TamSayı olmalı (T tanısı); Ondalık ve Metin reddedilir.
- Sözlük için yoktur (anahtar erişimi zaten var); Metin harfleri için liste
  üzerinden (`harflerin 3. öğesi`) yeterlidir.

## Seçim ölçütleri (önden bağlanır)

1. **İnsan verisi:** K-161/K-162 oturumlarına bir kart eklenir: çocuklar
   "3. öğesi" mi "üçüncü öğesi" mi yazıyor, 0/1 tabanı hangisini bekliyor.
   Kart olmadan yüzey seçilmez.
2. **Katman ve tam tüketim:** spec/20 sırasına oturmalı; `3.` tokeni sözcükleme
   düzeyinde belirsizlik yaratmamalı (fuzz + formatter parse-equivalence).
3. **Morfoloji:** `harflerin` tamlayanı mevcut çözümle aynı; `-inci` eki
   `zee-tr-1` profilinde yoksa yalnız A/C uygulanabilir.
4. **Conformance:** aralık içi/dışı, kopya semantiği, döngü sayacıyla
   kullanım, T tanısı; golden + regression + hata kataloğu aynı committe
   (RFC-0021 §6).
5. **Sürtünme geri sayımı:** kabulden sonra K-164/F010 ve K-179 sayaç
   döngüleri yeni yüzeye taşınır; korpus çiftleri "eski ret/yeni çözüm"
   olarak güncellenir.

## Kapsam dışı

Dilimleme, olumsuz sıra, sondan sayma, atama ile öğe değiştirme (spec/17
gezme yazması zaten var), sözlük sıra erişimi.

## Karar zamanı

4 Ekim 2026 freeze kapanışından ve K-161/K-162 verisinden sonra; RFC-0006
çağrı yüzeyi kararıyla aynı oturumda ele alınır, çünkü ikisi de "sayı +
ek" okunuşuna dayanır.
