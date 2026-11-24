# RFC-0029 — Listede sıra ile öğe erişimi

- **Durum:** **kabul — uygulandı** (K-386, 24 Kasım 2026; ADR-166). Taslak
  10 Eylül 2026 (K-180).
- **Tarih:** 10 Eylül 2026 (taslak) · 24 Kasım 2026 (kabul ve uygulama)
- **İş:** K-180 (kaynak: K-164/F010, K-179, K-355/F002) → K-386
- **Karar:** Mustafa, 24 Kasım 2026 — dört bağımsız ürün kanıtıyla
  `dogfood-change` istisnası; K-180'in "syntax freeze içinde açılmaz" kaydı
  bilinçli olarak kaldırıldı.
- **Normatif yüzey:** [spec/01](../spec/01-sozcukleme.md) sıra eki noktası,
  [spec/17](../spec/17-deger-semantigi-ve-gezme.md) sıra ile okuma,
  [spec/20](../spec/20-ifade-grameri.md) sıra erişimi.

## Özet

Zee listelerinde N'inci öğeye ulaşmanın tek yolu sayaçlı gezmeydi. Bu RFC
1 tabanlı, yalnız okuyan, O(1) sıra erişimini kabul eder:

```dil
harfler "z", "e", "e" listesi olsun
üçüncü harflerin 3. öğesi olsun          # sabit sıra
orta sayıların sıra. öğesi olsun         # ad sırası (asıl ihtiyaç)
sonuç sayıların sıra. öğesini almayı dene olsun   # Sonuç<T>, SIRA_DISI
```

## Motivasyon (kanıt)

Dört bağımsız kaynak aynı sınıfı üretti (ADR-074 §4: iki üründe tekrar eden
sınıf tasarım maddesidir):

- **K-164 / kanıt özeti (F010):** `bilgiye çevir` başlık→alan eşlemesi ve
  `çiftleri birleştir` iki listeyi sırayla eşleme iç içe sayaçlı gezmeydi
  (O(n²)); korpus çifti `f010-sira-ile-oge-erisimi` (S015 reddi) ve
  `f010-sayacla-oge-secimi` (sayaç çözümü).
- **K-179 / kelime avı:** `sıradaki harfi al` (ipucu için iki harf listesini
  hizalama), `sıradaki kelimeyi al` (rastgele sıradaki kelime), skor
  satırının ikinci alanı — üçü de sayaç döngüsü.
- **K-355/F002 / ürün H (ZeeMail):** 21 Kasım 2026 üçüncü ürün aynı sayaç
  yardımcısını yeniden yazdı.
- **Bağımsız karşılaştırma raporu:** beş klasik algoritmada ikili arama,
  her yoklaması O(n) sayaç gezmesi olduğundan O(log n) yerine O(n log n)
  çıktı. Ölçüm aşağıdadır.

## Karar

### 1. Yüzey: aday C (`<liste>nin <sıra>. öğesi`)

Taslağın üç adayından **C** seçildi; A (sabit `3.`) onun özel hâlidir.

- Sıra tek token: ad ya da TamSayı sabiti. Asıl ürün ihtiyacı döngü sayacıdır
  (C); A'yı ayrı kalıp yapmak gereksizdi.
- **B (yazıyla sıra, `üçüncü öğesi`) reddedildi:** sonlu sözcük kümesi ya da
  `zee-tr-1`'de olmayan `-inci` eki ister; değişken sıra veremez. Yeni ek
  gerekmeden çözüm varken morfoloji profili değiştirilmez.
- Liste tamlayan ekli tek addır (`harflerin`, `hedef_listenin`); ek mevcut
  `zee-tr-1` çözümüyle ayıklanır (`harfler + tamlayan`). **Yeni ek yoktur.**
  `öğesi`/`öğesini` kalıp kelimesidir, morfolojiyle çözülmez.
- Türkçe matematik yazımıyla aynıdır ("dizinin n. terimi").

### 2. 1 tabanlılık

`1. öğe` `ilki`, `<adet>. öğe` `sonu`dur. Çocuk "üçüncü" der; `ilki` zaten
ilk öğedir. Ürünlerin elle yazdığı sayaçlar tabanda tutarsızdı: kanıt özeti
0 tabanlı sayaç kullandı, kelime avının iki yardımcısından biri 0, öteki 1
tabanlıydı ve testleri tabanı ayrıca adlandırmak zorunda kaldı
("sıfır tabanlıdır" / "bir tabanlıdır"). Tek, dilin kendi sayımıyla uyumlu
taban bu karışıklığı kaldırır. 0 ve negatif sıra yoktur: sabitse derleme hatası
**T059** ("Sıra 1'den başlar"), değişkense çalışma zamanında C031.

### 3. Hata modeli

| Durum | Davranış |
|---|---|
| Sıra TamSayı değil, nesne liste değil, öğe türü belirsiz boş liste | Derleme **T058** (kesin konum + öneri) |
| Sabit sıra < 1 | Derleme **T059** |
| `1 ≤ sıra ≤ adet` dışı, `dene`siz | Çalışma hatası **C031**: "Listenin 7. öğesi yok: liste 5 öğeli." |
| `… öğesini almayı dene` | `Sonuç<T>`; başarısızsa Hata kodu `SIRA_DISI`, mesaj `listenin 7. öğesi yok (5 öğe)` |
| Sıra eki kalıba oturmuyor | Sözdizimi **S049** (hesaplanan sıra → ara ad önerisi; yazma girişimi → gezme yazması önerisi) |

Taslağın açık sorusu ("boş = yok mu, hata mı?") **Sonuç** ile kapandı:
RFC-0008 ailesinde `sayısını almayı dene`, `dosyasını okumayı dene` gibi
beklenen arıza Sonuç'tur; Seçenek "neden yok"u taşıyamaz. `ilki`'nin boş
listedeki C007 sınıfı ile `dene`siz C031 aynı sınıftır (sessiz devam YASAK).

### 4. Değer semantiği ve gezme (spec/17, T053)

- Sonuç öğenin bağımsız **kopyasıdır**; kopyanın alanını yazmak ya da iç
  listesine eklemek kaynak listeyi değiştirmez.
- Okuma kaynağı değiştirmez; gezilen listeyi gezme sırasında sıra ile okumak
  T053 değildir. Gezilen listeye ekleme hâlâ T053'tür.
- **O(1):** yorumlayıcı liste adını ortamdan ödünç alır, yalnız seçilen öğeyi
  kopyalar. Bu, sayan ayırıcıyla deterministik olarak sınanır
  (`liste_sira_bellek_testi`). Sınır: işlem argümanı yine spec/17 değer
  kopyasıdır; listeyi argüman alan bir `ikili ara` işlemi çağrı başına O(n)
  kopya öder. Kopya-üzerine-yazma (paylaşımlı arka depo) ayrı bir
  performans işidir, bu RFC'nin kapsamı değildir.

### 5. Yazma kapsamı: **açılmadı** (yalnız okuma)

İndeksle yazma (`listenin 3. öğesi "x" olsun`) değerlendirildi ve **bu
sürümde açılmadı**:

- Dört ürün kanıtının hiçbiri yazma istemiyor; sayaç döngülerinin tamamı
  okumaydı. Freeze altında en küçük güvenli kapsam yalnız okumadır.
- Yazma, gezme sırasında kaynak sabitliği (T053) kuralına yeni bir değiştirme
  yolu ekler ve `olsun` cümlesinin ad konumunu (bugün tek ad ya da sözlük
  değeri) genişletir — grammar etkisi okumadan büyüktür.
- Mevcut yollar yeter: gezme yazması (spec/17 §4 değer-sonuç imleci) ya da
  yeni liste kurma.

Yazma girişimi sessizce başka anlama düşmez: S049 "sıra ile öğe YAZMA bu
sürümde yoktur" der ve gezme yazmasını önerir. Gerçek ürün ihtiyacı çıkarsa
yeni RFC ve K-işiyle açılır.

### 6. Grammar etkisi

- **Sözcükleyici:** tek dar token, **sıra eki noktası** (`TokenTur::SiraEki`).
  Nokta yalnız önceki TamSayı/Kelime/Metin tokenine bitişik, aynı satırda ve
  ardından boşluk + `öğe…` kelimesi gelirse üretilir. Önceden bu nokta her
  konumda S001'di; bu yüzden K-386 öncesinde geçerli her programın token
  dizisi aynıdır. `3.14`, `x.y`, `3 .`, `3.öğesi` S001 kalır; `3.14` için
  ondalık virgül önerisi korunur, köşeli parantez (`harfler[0]`) S001'i
  artık sıra erişimini önerir.
- **Ayrıştırıcı:** erişim/postfix katmanında (spec/20) iki tam-bölge kalıbı:
  4 token (`L S . öğesi`) ve 6 token (`L S . öğesini almayı dene`). Bölgenin
  tamamı eşleşmezse S049.
- **AST/HIR:** `Ifade::ListeOgesi { liste, sira, dene }` (`dene`: Sonuç biçimi); çözümleyici,
  etki/yetkinlik, invariant ve test kapsamı gezicileri iki çocuğu yürütür.
  Yeni değer türü yoktur.
- **Kapsam dışı (bu sürüm):** hesaplanan sıra ifadesi (`sıra ile 1 in
  toplamı. öğesi` → ara ad), iyelik zinciri (`tablonun 2. öğesinin 3. öğesi`
  → ara ad; `zee-tr-2` morfoloji adayı F001/F013), dilimleme, sondan sayma,
  sözlük sıra erişimi, Metin üzerinde doğrudan sıra (önce `metnin harfleri`).
- **WASM/native:** ayrı kod üretimi yoktur; playground ve native yolları aynı
  yorumlayıcıyı kullanır, ABI değişmez.

### 7. Uyumluluk (K-167/ADR-064)

- Yeni kalıp kelimeleri `öğesi`, `öğesini` (`dil-yuzeyi-v1.tsv` aktif,
  giriş `1.0.0-dev`); yeni tanılar S049, T058, T059, C031 (`tani-kimlikleri-v1`).
- Kaldırma ya da yeniden adlandırma yoktur; deprecation kaydı gerekmez.
- Syntax freeze kaydı (`docs/v1-syntax-freeze-v1.tsv`) lexer genişlemesini
  açıkça yazar; özet yenilendi.

## Seçim ölçütleri — durum

1. **İnsan verisi (K-161/K-162 kartı):** **karşılanmadı.** Oturum verisi
  yok; seçim 24 Kasım kararıyla dört ürün kanıtına dayandı. Kart ileride
  koşulursa 1 tabanlılık ve yüzey için sonuç bu RFC'ye eklenir; aykırı
  sonuç ayrı RFC ister.
2. **Katman ve tam tüketim:** karşılandı (spec/20; biçimleyici
   parse-equivalence ve golden 34).
3. **Morfoloji:** karşılandı; `-inci` gerekmedi, yeni ek yok.
4. **Conformance:** `compiler/tests/liste_sira_erisimi_testi.rs` (ilk/son/orta,
   0 ve n+1, boş liste, negatif, iç içe liste, yapı kopyası, T053 ile gezme,
   S049/S001, biçimleyici, test bloğu) ve `liste_sira_bellek_testi.rs`;
   golden 34; hata kataloğu aynı committe.
5. **Sürtünme geri sayımı:** kanıt özeti `bilgiye çevir`/`çiftleri
   birleştir`, kelime avı `sıradaki harfi al`/`sıradaki kelimeyi al` (iki
   yardımcı kaldırıldı) ve skor alanı sıra erişimine taşındı; ürün çıktıları
   bayt bayt aynı (ürün kaynağı değişikliği kanıt özeti sayfasını değiştirmedi, Zee/Rust/Go eş; kelime avı uçtan uca
   testleri aynı). Korpusa `f010-sira-erisimi-cozumu` eklendi; eski ret ve
   sayaç çözümü tarihsel vaka olarak kalır. Ürün H'nin kendi kaynağı harici
   depodadır; değişiklik ürün sahibine bırakıldı.

## Ölçüm: ikili arama O(log n)

`scripts/liste-sira-olcumu.py` (24 Kasım 2026, Apple M4 Pro, release `dil`,
`dil çalıştır`, 3 tekrarın medyanı, liste kurulum süresi çıkarılmış). Her
arama listede olan bir değeri arar. Gözlemseldir; izlenen performans
tarihçesine yazılmaz.

| yoklama | n | arama | ort. adım | arama başına µs |
|---|---:|---:|---:|---:|
| sıra erişimi | 1 000 | 2 000 | 8,99 | 38,1 |
| sıra erişimi | 10 000 | 2 000 | 12,36 | 50,1 |
| sıra erişimi | 100 000 | 2 000 | 15,71 | 62,7 |
| sıra erişimi | 400 000 | 2 000 | 17,71 | 66,0 |
| sayaç (K-386 öncesi) | 1 000 | 20 | 9,25 | 5 268,8 |
| sayaç (K-386 öncesi) | 2 000 | 20 | 10,15 | 11 485,4 |
| sayaç (K-386 öncesi) | 4 000 | 20 | 11,15 | 25 012,3 |
| sayaç (K-386 öncesi) | 8 000 | 20 | 12,15 | 54 087,1 |

Sıra erişiminde n 400 kat büyürken adım sayısı log₂ n ile (9 → 17,7) ve
arama süresi yalnız 1,7 kat artar: O(log n). Sayaçla n her iki katına
çıktığında süre ~2,2 kat artar: O(n log n). n = 1 000'de fark ~140 kattır.
Güvenli profilin 64 MiB yığın tahmini sayaç değişkenini n ≈ 16 000'de C024
ile durdurur (her yoklamada gezme kopyası); sıra erişimi 400 000'de rahattır.

## Kapsam dışı

Dilimleme, olumsuz sıra/sondan sayma, sıra ile yazma (§5), sözlük sıra
erişimi, hesaplanan sıra ifadesi ve iyelik zinciri (§6), listenin argüman
kopyasını kaldıran kopya-üzerine-yazma.
