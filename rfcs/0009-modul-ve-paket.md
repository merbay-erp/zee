# RFC-0009 — Modül ve Paket Modeli

- **Durum:** **geçici kabul — birim katmanı (§2) + proje bildirimi (§3)**
  (K-029/K-076; onay kapısı: usability). Yerel/uzak bağımlılık ve kilit
  katmanı (§4) TASLAK — Faz 3/5.
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
   §6.2 (çocuk basitliği ↔ kapsülleme).

## 3. Proje bildirimi (K-076 — gerçeklendi)

Her gerçek proje kökünde tek `proje.dil` taşır. Ayrı bir TOML/JSON biçimi
öğretilmez; bildirim geçerli, yan etkisiz zee kaynağıdır:

```
proje "uzay-oyunum" olsun
sürüm "0.1.0" olsun
giriş "program.dil" olsun
```

- Alanlar zorunlu ve tektir; bilinmeyen alan P001'dir.
- Sürüm üç sayılıdır (`X.Y.Z`); giriş proje içinde kalan göreli `.dil`
  yoludur (P003/P004).
- `dil çalıştır/denetle/dene <klasör>` bildirimin girişini kullanır.
- `dil biçimle <klasör>` proje ağacındaki bütün `.dil` kaynaklarını yol
  sırasıyla, önce tümünü doğrulayıp sonra yazar (K-077; kaynak hatası yüzünden
  yarım biçimleme yok).
- `dil yeni <ad>` çalışan program, test, BENIOKU ve bildirimi birlikte üretir.
- Bildirim yolunun proje dışına çıkamaması bütün platformlarda aynı denetlenir.

## 4. Paket modeli (projeler arası — Faz 3/5)

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

## 5. Çözüm sırası

`kullan` çözümü deterministiktir: (1) aynı klasör birimi → (2) proje
bağımlılığı (kilit dosyasındaki sürüm) → başka arama yolu YOK (gizli global
paket dizini yok; çevrimdışı okul kurulumunda sürpriz yok).

## 6. Açık sorular

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
