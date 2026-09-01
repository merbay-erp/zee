# RFC-0010 — Hata ve Tanılama Standardı

- **Durum:** kabul (yaşayan standardı belgeler)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-026, K-113, K-114; hata kataloğu ve kimlik bekçileri
- **Gerçekleme:** `compiler/src/tani.rs`, `compiler/src/ayristirici/kurtarma.rs`,
  `docs/hata-katalogu.md`, `katalog_testi.rs`, `tani_kimligi_testi.rs`,
  `parser_kurtarma_testi.rs`

## Özet

Her tanı aynı sözleşmeye uyar: **kod + Türkçe açıklama + kaynak konumu +
işaret + öneri + katalog bağlantısı**. Tanı bir ceza değil, dersin kendisidir.

## 1. Tanı sözleşmesi (bağlayıcı)

```
HATA T001

Büyüklük karşılaştırması sayılar arasında yapılır; burada Metin var.

2 | toplam 10 dan büyükse
    ^

Öneri:
Karşılaştırılan iki değerin de sayı olduğundan emin ol.

Ayrıntı için: dil hata T001
```

Kurallar:

1. **Kod:** `Ö###` — S sözdizimi, A ad çözümleme, T tür, C çalışma zamanı,
   D doğrulama, Ç iç akış (kullanıcı görmez). Numara asla yeniden kullanılmaz.
2. **Açıklama:** tek cümle, Türkçe, teknik jargonsuz; neyin OLDUĞU söylenir.
3. **Konum:** 1 tabanlı satır/sütun + işaret uzunluğu; kaynak satırı basılır.
4. **Öneri:** eyleme dönük — kullanıcının bir SONRAKİ adımı. Değer içeren
   hatalarda somut değerler gösterilir (D001: "Beklenen: 5 — bulunan: 4";
   A001: tanımlı adların listesi; T028: yapının alan listesi).
5. **Katalog bağlantısı:** her rapor `dil hata <kod>` satırıyla biter; katalog
   ikiliye gömülüdür, çevrimdışı çalışır.
6. **İki tüketim modu:** normal derleme/çalıştırma ilk tanıda durur. `denetle`
   ve LSP, §2.1'deki sınırlı kurtarma hattıyla birden çok tanı verir.

## 2. Süreç bekçileri (bağlayıcı)

- **Katalog eşleme testi:** kaynaktaki her tanı kodu `docs/hata-katalogu.md`de
  belgeli olmalı ve tersi (`katalog_testi.rs` — CI'da). Belgelenmemiş hata
  gemiye binemez.
- **Makine çıktısı:** `dil denetle --json` aynı tanıyı RFC 8259 JSON'la verir;
  alan adları (`kod, mesaj, satir, sutun, uzunluk, oneri`) kararlıdır —
  editör entegrasyonları buna güvenebilir.
- Tanı metinleri regression testlerinde kod + içerik düzeyinde sabitlenir.

### 2.1 Çoklu tanı ve parser kurtarma (bağlayıcı)

`dil denetle`, JSON çıktısı ve LSP aynı çoklu-tanı görünümünü kullanır.
Lexer token üretemediği lexical hatada tek tanı döner. Lexer başarılıysa
parser şu güvenilir senkronizasyon noktalarında sürer:

1. Hatalı cümlenin `SatirSonu` sınırı tüketilir.
2. Cümle bir alt gövde açtıysa yalnız ona ait dengeli `Girinti`…`Cikinti`
   bölgesi atlanır.
3. Sonraki aynı-girintili kardeş kendi ebeveyn bloğunda ayrıştırılır.

Kurtarma fiziksel kapsam derinliğini sonraki cümleye sızdıramaz. Kısmi AST
normal parser'ın üretemeyeceği boş/imkânsız düğüm taşıyamaz ve yürütülebilir
program sayılmaz. Parser, birim ve checker tanıları `(satır, sütun, kod,
mesaj)` sırasıyla deterministiktir; belge başına en çok 20 tanı yayımlanır.
Bu sayı daha çok hata olmadığı anlamına gelmez, editör tanı seli bütçesidir.

### 2.2 Sürümler arası tanı kimliği (bağlayıcı)

Yayımlanmış `Ö###` kodu aynı semantik olayı anlatmaya devam eder. Bir tanının
ailesi veya hangi koşulda doğduğu değişiyorsa yeni kod ayrılır. Kaldırılan kod
silinmez ve başka anlamda yeniden kullanılmaz; katalog ile kimlik fixture'ında
`ayrılmış` mezar taşı olarak kalır.

`compiler/tests/fixtures/tani-kimlikleri-v1.tsv` her kodu aktif/ayrılmış
durumu, aile uyumlu tekil semantik anahtarı ve katalogdaki kanonik “Ne oldu”
özetiyle eşler. Katalog↔fixture testi kod kümesini, durumu ve özeti birebir
doğrular. Yalnız editoryal özet düzeltmesi fixture'ı aynı değişiklikte açıkça
günceller; tüketici davranışını değiştiren semantik fark yeni kod ister.

Dinamik mesaj ayrıntısı, öneri, konum ve işaret uzunluğu kimliğin parçası
değildir. Program içinde yönetilebilir değer olan `Hata.kod`, derleyici
tanısının `Tani.kod` alanından ayrı bir sözleşmedir.

## 3. Açık sorular

1. Uyarı (warning) kavramı: v0'da yalnız hata var. Aday ilk uyarı: yalnız
   büyük/küçük harfle ayrışan adlar (A07).
2. Çocuk modu üslubu: aynı kod için daha kısa/yumuşak metin varyantı
   (bölüm 16); tek kaynak-çift üslup mimarisi.

## Dört soru süzgeci

Doğal ✓ · Deterministik ✓ (kod sabit, biçim sabit) · Öğrenilebilir ✓ (hatanın
kendisi ders) · Savunulabilir ✓ (JSON sözleşmesi + katalog bekçisi).
