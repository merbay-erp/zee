# RFC-0004 — Değer Tanımı ve Kapsam

- **Durum:** **geçici kabul** (31 Ağu 2026 — kapsam kuralı K-034 ile karara
  bağlandı ve gerçeklendi; onay kapısı: usability oturumları.)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-003 (olsun), K-011 (ek yazımı), K-020 (alan atama önceliği)
- **İlgili golden programlar:** 02, 07, 22
- **Gerçekleme:** `olsun` kolları `ayristirici.rs`; ortamlar `cozumleyici.rs` / `yorumlayici.rs`

## Özet

Değer bağlama tek kalıptır: `<ad> <ifade> olsun`. Aynı kalıp yeniden atamadır;
tür değişimi yasaktır (T002).

## Kurallar

1. **Tanım = atama.** `yaş 10 olsun` adı yoksa tanımlar, varsa günceller.
   Güncellemede tür korunmalıdır: `x 5 olsun` sonrası `x "a" olsun` → **T002**
   ("türü sonradan değişemez; yeni ad kullan").
2. **Ad yalın yazılır.** Tanımda ek yoktur (`toplam 0 olsun`); KULLANIMDA ekler
   bitişiktir ve ad çözümleme aday-kök eşlemesiyle yalına döner (RFC-0002 morfoloji;
   `toplamı`, `sayacı`→`sayaç`, `şekle`→`şekil`).
3. **`olsun` satırının yorumlanma önceliği** (belirsizlik çözümü, deterministik):
   1. `<sözlük-in> <anahtar> değeri <değer> olsun` → sözlüğe yazma (K-015)
   2. üç tokenlik `<ad-in> <alan> <tek değer> olsun` ve kuyruk yapılı kalıp
      DEĞİLSE → yapı alanına yazma (K-020 bulgusu: kalıplar alan-atamadan önce)
   3. aksi halde → değer tanımı; ifade bölgesi RFC'lerdeki kalıplarla okunur
4. **Örtük adlar:** `diye sor` cevabı `yanıt` adını Metin türüyle bağlar (K-007).
   Döngü değişkenleri (`her sayı için`, aralık döngüsü) gövde için bağlanır.

## Kapsam modeli (v0.2'de KARARA BAĞLANDI — K-034)

- **Blok kapsamı:** gövdede doğan ad gövdeyle ölür (döngü değişkeni dahil);
  dıştaki ada atama kalıcıdır. Gölgeleme yapısal olarak yoktur. Çözümleyici ve
  yorumlayıcı aynı kuralı uygular; kapsam_testi.rs sabitler.
- **İşlem gövdesi taze ortamda çalışır:** yalnız parametrelerini görür; üst
  düzey adlara erişemez (kapanış/global yok). Testler de taze ortam alır.

## Açık sorular (v1 kararları)

1. ~~Blok kapsamı~~ — KARARA BAĞLANDI (K-034, v0.2'de gerçeklendi).
2. **Sabitler:** `değişmez pi 3 olsun` benzeri bir kalıp gerekli mi? (Ondalık
   türü RFC-0007'ye bağlı.)
3. **İlk tanım / yeniden atama ayrımı:** ayrı kalıp (örn. tanım `olsun`,
   güncelleme `yapılsın`?) — mevcut tek-kalıp K-003'ün sadeliği kazanıyor;
   T002 tür bekçisi en tehlikeli hatayı zaten yakalıyor.

## Dört soru süzgeci

Doğal ✓ (dilek kipi "olsun" Türkçede tam bu iş için var) · Deterministik ✓
(öncelik sırası yazılı, T002/A002 bekçili) · Öğrenilebilir ✓ (tek kalıp) ·
Savunulabilir — blok kapsamı sorusu çözülünce tam ✓.

## Korpus etkisi

Yok. Kapsam kararı (açık soru 1) kabul edilirse korpus etkilenmez — korpus
zaten sızıntıya dayanan program içermiyor.
