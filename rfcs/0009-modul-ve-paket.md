# RFC-0009 — Modül ve Paket Modeli

- **Durum:** taslak — **birim katmanı (§2) gerçeklendi** (31 Ağu 2026):
  kullan/tohumlu ayrıştırma/A008-A010 tanıları/birim testleri dene kapsamında;
  9 test yeşil (K-029). Paket katmanı (§3) Faz 3/5
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** master plan bölüm 7 ("modül mü birim mi; kullanıcı testiyle karar"), bölüm 14
- **Gerçekleme:** `lib.rs` (dosyayi_coz + BirimYukleyici), `ayristirici.rs`
  (kullanilan_birimler + tohumlu ayrıştırma); testler `birim_testi.rs`

## Özet

Kod paylaşımının iki katmanı: **birim** (aynı projedeki dosyalar arası) ve
**paket** (projeler arası, registry üzerinden). İkisi de `kullan` ile alınır.

## 1. Terim kararı önerisi: "birim"

"Modül" yabancı kökenli ve soyut; "birim" hem Türkçe hem okulda bilinen kelime
("ders birimi"). Öneri: **dosya = birim**; ayrı `birim` bildirimi gerekmez
(dosya adı birim adıdır). Usability oturumunda "modül" karşısında sınanacak.

## 2. Birim modeli (proje içi)

```
# hesaplar.dil dosyasındaki işlemleri kullan:
hesaplar birimini kullan

ortalama notlar için ortalamayı hesapla olsun
```

Kurallar (öneri):

1. `X birimini kullan` — aynı klasördeki `X.dil` dosyasını alır; dosya adı
   tanımlayıcı kurallarına uyar (RFC-0002).
2. Alınan birimin **işlem, yapı ve test** tanımları görünür olur; üst düzey
   değişkenleri görünmez (kapsülleme; RFC-0004 taze-ortam ilkesiyle uyumlu).
3. Ad çakışması (iki birim aynı işlem adını verirse) **hatadır** — sessiz
   gölgeleme yok; tanı iki kaynağı da gösterir. Nitelikli erişim
   (`hesapların ortalamayı hesapla`sı?) v2 sorusu.
4. Döngüsel `kullan` hatadır (deterministik yükleme sırası).
5. Görünürlük: v1'de her tanım dışa açıktır; `özel` işaretleyicisi açık soru
   §5.2 (çocuk basitliği ↔ kapsülleme).

## 3. Paket modeli (projeler arası — Faz 3/5)

- Tek manifest: `proje.dil` benzeri TEK dosya (ad, sürüm, bağımlılıklar) +
  deterministik `kilit` dosyası. Manifestin kendisi de dilin sözdizimiyle
  yazılır (ayrı biçim öğretilmez) — örnek taslak:

```
proje uzay-oyunum
sürümü 0 nokta 1 olsun        # sürüm sözdizimi RFC-0013'e bağlı
grafik paketini kullan
```

- `dil ekle grafik` manifesti düzenler; `dil paketle` / `dil yayınla` Faz 5
  (imza, provenance, SBOM — bölüm 14/18 gereksinimleri o RFC'lerde).
- Paket adları küçük harf Türkçe tanımlayıcıdır; typosquatting/confusable
  denetimi RFC-0002'nin S028 altyapısını registry tarafında yeniden kullanır.

## 4. Çözüm sırası

`kullan` çözümü deterministiktir: (1) aynı klasör birimi → (2) proje
bağımlılığı (kilit dosyasındaki sürüm) → başka arama yolu YOK (gizli global
paket dizini yok; çevrimdışı okul kurulumunda sürpriz yok).

## 5. Açık sorular

1. birim/modül kelime kararı (usability).
2. `özel` görünürlük işareti v1'de mi v2'de mi.
3. Nitelikli erişim sözdizimi (çakışma çözümü için).
4. Alt klasörler: `oyun/araçlar.dil` → `araçlar birimini kullan` yolu nasıl
   yazılır? (Yol ayracı noktalama sorunu.)

## Dört soru süzgeci

Doğal ✓ ("hesaplar birimini kullan" sesli okunur) · Deterministik ✓ (tek
arama sırası, çakışma=hata, döngü=hata) · Öğrenilebilir ✓ (dosya=birim,
bildirim yok) · Savunulabilir ✓ (kapsülleme + kilit dosyası + registry yolu).

## Korpus etkisi

Kabulde golden korpusa iki dosyalı bir "birim" örneği eklenir (31. program
adayı) ve golden 15'in işlemleri bir birime taşınmış varyant kazanır.
