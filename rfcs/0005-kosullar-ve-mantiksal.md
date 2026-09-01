# RFC-0005 — Koşullar ve Mantıksal İfadeler

- **Durum:** **geçici kabul** (31 Ağu 2026 — ve/veya/değilse dahil yüzey
  gerçeklendi ve korpusla sabitlendi; onay kapısı: usability oturumları.)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-005 (ise/değilse), K-010 (karşılaştırmalar), K-027 (bağlaçlar), K-097 (ifade katmanları)
- **İlgili golden programlar:** 05, 06, 07, 09, 23; anti-örnek A03, A05
- **Normatif katman:** spec/20, RFC-0021
- **Gerçekleme:** `kosul_ifadesi`/`kosul_atomu` (`ayristirici.rs`); testler `a03_dogrusu_calisir` vd.

## Özet

Koşullar sembolsüzdür ve yüklem-sonludur: karşılaştırma kelimesi cümlenin
sonundadır (`-se/-sa` eki koşullaştırır). `ve`/`veya` zinciri ve `değilse`
olumsuzlaması vardır; karışım ve belirsizlik hatadır.

## 1. Koşul cümlesi ve zincir

```
puan 90 veya daha büyükse          # kol 1
    "Pekiyi" yaz
değilse puan 70 veya daha büyükse  # kol 2 (else-if)
    "İyi" yaz
değilse                            # varsayılan kol
    "Geçmedi" yaz
```

- Kollar sırayla sınanır; ilk doğru kolun gövdesi çalışır (golden 05).
- `değilse` yalnız bir `ise` bloğunun ardından, aynı hizada gelir; başıboş
  `değilse` → **S031**.

## 2. Karşılaştırma yüklemleri (K-010)

| Kalıp | Anlam | Not |
|---|---|---|
| `X Y veya daha büyükse` | X ≥ Y | "veya daha" ikilisi kalıba aittir, bağlaç değildir |
| `X Y veya daha küçükse` | X ≤ Y | |
| `X Y den büyükse` / `küçükse` | X > Y / X < Y | ek ayrık (sabit) ya da bitişik (`gizliden küçükse`) |
| `X Y e eşitse` | X = Y | eşitlik her türde; büyüklük yalnız sayılarda (T001) |
| `X Y` (olana kadar bağlamında) | X = Y | `bildi doğru olana kadar` |
| `X çiftse` / `X tekse` | 2 ile bölüm | TamSayı ister (T007) |
| `X varsa` / `yoksa` | Seçenek dolu mu | RFC-0008 |
| `S de A varsa` | sözlükte anahtar | RFC-0009 alanı |
| `X başarılıysa` / `başarısızsa` | Sonuç durumu | RFC-0008 |
| `M A içeriyorsa` | metin arama | |
| `X boşsa` / `doluysa` | koleksiyon/metin boş mu | |
| Çıplak yüklem: `büyük/küçük/eşit` | `olduğu sürece`, `olmalı`, `değilse` içinde | |

## 3. Olumsuzlama: `değilse` (K-027)

Yüklem sonuna gelir; içteki koşul olumlu biçimiyle okunur:

- `a b ye eşit değilse` → a ≠ b
- `c d den küçük değilse` → c ≥ d
- `bayrak değilse` → Mantıksal değerin tersi (tek adla)

## 4. Bağlaçlar: `ve` / `veya` (K-027)

- `A ve B ve C` — hepsi; `A veya B` — en az biri. Kısa devre: VE ilk yanlışta,
  VEYA ilk doğruda durur (yan etkisiz koşullarda gözlemlenemez; ileride yan
  etkili ifadeler gelirse sırayla-soldan garanti edilmiştir).
- **Karışım hatadır (S030).** Gerekçe: `A ve B veya C` parantezsiz iki anlama
  gelir; dil parantez öncelik mekanizması sunmadığı için sessiz bir öncelik
  kuralı (VE bağlar gibi) çocuk için görünmez tuzak olurdu (anti-örnek A05
  ilkesi). Kullanıcı ya tek tür bağlaç kullanır ya da koşulu `ise`
  basamaklarına böler — her iki çözüm de okunur Türkçedir.
- K-097/RFC-0021 sırası gereği her karşılaştırma önce tek AST olur, boolean
  zincir sonra kurulur. `veya daha` karşılaştırma bölgesinde tam tüketildiği
  için boolean ayırıcı değildir.

## 5. Açık sorular

1. Ternary benzeri kısa seçim (`... ise X değilse Y` ifade konumunda) —
   RFC-0002 §6'daki okunurluk ilkesiyle birlikte değerlendirilecek.
2. `değil`in yüklemsiz bağımsız kullanımı (`doğru değil` ifade konumunda) —
   şimdilik yalnız koşul bağlamında.
3. Karşılaştırma zinciri (`1 den küçük x 10 dan büyükse`? matematiksel
   `a < x < b`) — aday değil; `ve` zinciri bu işi görüyor.

## Dört soru süzgeci

Doğal ✓ (yüklem-sonlu, ekli koşul Türkçenin kendisi) · Deterministik ✓
(S030 karışım bekçisi; "veya daha" tek token ileri bakışla ayrışır) ·
Öğrenilebilir ✓ (sembol yok, kalıplar sesli okunur) · Savunulabilir ✓
(kısa devre + tür bekçileri profesyonel beklentiyi karşılar).

## Korpus etkisi

Golden 05/07/09 ve A03 değişmedi. Karşılaştırma→boolean katman sırası ile
S030 fail-closed sınırı ayrıca `ifade_grameri_testi.rs` conformance dosyasına
bağlandı.
