# RFC-0002 — Lexical ve Unicode Kuralları

- **Durum:** **geçici kabul** (31 Ağu 2026 — yüzey gerçeklendi ve korpusla
  sabitlendi; onay kapısı: usability oturumları. Kalan açık sorular §6'da.)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-001 (yorum), K-011 (ek yazımı), K-014 (kelime çakışmaları)
- **İlgili golden programlar:** tümü; anti-örnekler A07, A08
- **Gerçekleme:** `compiler/src/sozcukleyici.rs` (testler: `golden_testi.rs` S-kodları)

## Özet

Kaynak metnin karakter kuralları ve token dizisi. Girinti kuralları RFC-0003'te.

## 1. Kaynak biçimi

- Kodlama **UTF-8**; geçersiz UTF-8 kaynak okuma aşamasında reddedilir.
- Satır sonu `\n`; `\r\n` kabul edilir, `\r` satır içeriğine dahil edilmez.
- Dosya uzantısı `.dil` (geçici; isim kararıyla birlikte kesinleşir).

## 2. Unicode ve tanımlayıcı alfabesi

- **Birleşik biçim zorunluluğu (v0):** birleştirici imler (U+0300–U+036F)
  reddedilir → **S029**. Kaynak, önceden birleştirilmiş karakterlerle yazılır
  (g + ˘ değil, ğ). Tam NFC normalizasyonu ileride ayrı ADR ile
  değerlendirilecek; v0 kuralı daha katıdır ve determinizmi şimdiden sağlar.
- **Tanımlayıcı karakterleri:** ASCII harf/rakam, `_`, Türkçe harfler
  (ç ğ ı İ ö ş ü ve büyükleri) ve Türkçe yazımdaki şapkalı ünlüler
  (â î û ve büyükleri — "kâr" geçerli bir tanımlayıcıdır).
- **Homoglyph koruması:** başka alfabelerden harfler (örn. Kiril "а", U+0430)
  tanımlayıcıda **S028** hatası verir; tanı kod noktasını gösterir.
  Gerekçe: anti-örnek A08 + master plan bölüm 18 (supply-chain).
- Tanımlayıcı rakamla başlayamaz (rakamla başlayan dizi sayı okunur).
- **Büyük/küçük harf duyarlıdır.** Uzlaşım: tür adları büyük harfle başlar
  (`Öğrenci`, `TamSayı`), değer adları küçük. Yalnız büyük/küçük farkıyla
  ayrışan tanımlayıcılar için uyarı **açık soru** (A07; İ/i–I/ı tuzağı).

## 3. Tokenlar

| Token | Biçim | Notlar |
|---|---|---|
| Metin | `"..."` | Kaçışlar (v0.2, K-036): `\"` `\\` `\n`; bilinmeyen kaçış → S040. Kapanmayan tırnak → S002 |
| TamSayı | ASCII rakamlar | i64; sınır aşımı → S006. Negatif sabit `-3` geçerli (v0.2, K-036): işaret rakama bitişik |
| Kelime | tanımlayıcı alfabesi | Anahtar kelime DEĞİLDİR — bkz. §5 |
| Virgül | `,` | Liste sabiti ve ileride argüman ayracı |
| SatirSonu / Girinti / Cikinti | — | RFC-0003 |

Diğer her karakter → **S001** "beklenmeyen karakter" (öneri: kalıpları
kelimelerle yaz — anti-örnek A03).

## 4. Yorumlar (K-001)

`#` satır sonuna kadar yorumdur; token üretmez. Yorum işareti, "noktalama
minimum" ilkesinin kabul edilmiş üç istisnasından biridir (tırnak, virgül, #).
Blok yorum yoktur; biçimleyici `#yorum` → `# yorum` normalleştirir.

## 5. Anahtar kelimesizlik ve bağlamsal kelimeler

Sözcükleyicide ayrılmış kelime YOKTUR: `yaz`, `olsun`, `ise` sıradan
Kelime tokenlarıdır; anlamı ayrıştırıcı konuma göre verir (yüklem-sonlu
dağıtım). Sonuçları:

- `not`, `sayaç` gibi kelimeler serbestçe değişken adı olur (K-014).
- Bağlamsal kelimelerin listesi ayrıştırıcı RFC'lerinde tutulur; bilinen
  çakışma bulguları K-014'te: `al` (parametre/işlem adı), `sonuç`
  (kalıp/tür/değişken), `yanıt` (girdi/HTTP).
- Sabit değeri olan üç kelime tekil-ifade konumunda özeldir:
  `doğru`, `yanlış`, `yok` (+ koleksiyon boşluğu için `boş liste`/`boş sözlük`
  kalıpları). Bu adlarla değişken tanımlamak **önerilmez**; v1'de hata
  yapılması açık soru.

## 6. Açık sorular

1. ~~Metin kaçış dizileri~~ — GERÇEKLENDİ (v0.2, K-036): `\"` `\\` `\n`, S040.
2. ~~Negatif sayı sabiti~~ — GERÇEKLENDİ (v0.2, K-036): işaret rakama bitişikse
   sabittir; ondalıkta işaret gövdeye bir kez uygulanır.
3. Ondalık/GerçekSayı sözdizimi (Türkçe ondalık virgülü mü, nokta mı? —
   virgül liste ayracıyla çakışır; ciddi tasarım işi).
4. Büyük/küçük yalnız-fark uyarısı (A07) ve confusable denetiminin tanımlayıcı
   ötesine (metin sabitlerine) genişletilmesi.
5. Tam NFC: v0 "birleşik biçim zorunlu" kuralı yeterli mi, tablolu
   normalizasyon mu gelmeli? (ADR adayı.)

## Dört soru süzgeci

1. Doğal mı? — Evet: yazım kuralları Türkçe imlaya dayanır (şapkalı ünlüler dahil).
2. Deterministik mi? — Evet: her kural token düzeyinde hatayla sınanır (S001–S029).
3. Öğrenilebilir mi? — Evet: üç noktalama istisnası dışında ezber yok.
4. Savunulabilir mi? — Evet: homoglyph/birleştirici im koruması güvenlik
   modelinin (bölüm 18) lexical ayağıdır.

## Korpus etkisi

Yok — korpus zaten bu kurallarla yazılmıştır; A08 artık S028 ile makine
tarafından da doğrulanır (`a08_homoglyph_reddedilir` testi).
