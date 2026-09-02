# RFC-0009 — Modül ve Paket Modeli

- **Durum:** **geçici kabul — birim (§2), proje (§3), yerel paket + kilit
  (§4.1), tam public kaynak ABI'si** (K-029/K-076/K-078/K-086; onay kapısı:
  usability). K-094 yayın çekirdeği RFC-0020/spec-18'de; K-095 registry
  metadata güveni spec/19'da çalışır. Taşıma/cache/CLI §4.2/ADR-006/RFC-0020
  §6 kapsamında sürmektedir.
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** master plan bölüm 7 ("modül mü birim mi; kullanıcı testiyle karar"), bölüm 14
- **Gerçekleme:** `lib.rs` (kökenli yükleyici + T039 public sınırı),
  `paket.rs` (grafik, SHA-256,
  kilit), `proje.rs` (bildirim), `ayristirici.rs` (`birimini/paketini`);
  testler `birim_testi.rs` + `proje_testi.rs`.

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
   Her görünür işlem bütün parametre ve dönüş türlerini açıkça bildirir
   (K-086/spec-10); eksik sözleşme T039'dur. Birimin aldığı başka işlemler
   örtük re-export edilmez.
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
yetkinlikler boş liste olsun
ağ_hedefleri boş liste olsun
yerel_bağımlılıklar boş liste olsun
```

- İlk üç alan zorunlu ve tektir; `yetkinlikler`, `ağ_hedefleri` ve
  `yerel_bağımlılıklar` isteğe bağlıdır. İlk ikisi K-127/RFC-0024/spec-23
  dış dünya politikasını, sonuncusu göreli proje klasörlerini tanımlar
  (P001/P002/P005/P015).
- Sürüm üç sayılıdır (`X.Y.Z`); giriş proje içinde kalan göreli `.dil`
  yoludur (P003/P004).
- `dil çalıştır/denetle/dene <klasör>` bildirimin girişini kullanır.
- `dil biçimle <klasör>` proje ağacındaki bütün `.dil` kaynaklarını yol
  sırasıyla, önce tümünü doğrulayıp sonra yazar (K-077; kaynak hatası yüzünden
  yarım biçimleme yok).
- `dil yeni <ad>` çalışan program, test, BENIOKU, bildirim ve kilidi birlikte
  üretir.
- Bildirim yolunun proje dışına çıkamaması bütün platformlarda aynı denetlenir.

## 4. Paket modeli

### 4.1 Yerel paketler ve kilit (K-078 — gerçeklendi)

Paket, kendi `proje.dil` bildirimi olan başka bir zee projesidir. Bağımlılık
adı/yolu iki ayrı yerde yinelenmez: kullanan proje yalnız klasörü listeler;
paket adı ve sürümü bağımlı bildirimin tek gerçek kaynağıdır:

```text
proje "uzay_oyunum" olsun
sürüm "0.1.0" olsun
giriş "program.dil" olsun
yerel_bağımlılıklar "../grafik", "../ses" listesi olsun
```

Uygulama kaynağında paket açıkça birimden ayrılır:

```text
grafik paketini kullan
```

Kurallar:

1. Paket adı küçük harfli, tek Türkçe/Latin tanımlayıcıdır; aynı grafikte iki
   ayrı kök aynı adı taşıyamaz (P007).
2. Yalnız **doğrudan** bildirilen paket kullanılabilir. Geçişli bağımlılık
   grafikte bulunsa bile kullananın API'si değildir (A011).
3. Paket girişinin işlem, yapı ve testleri görünür; üst düzey cümleleri
   kapsüllüdür. Görünür işlemler tam monomorfik kaynak ABI'si taşır. Paket
   içindeki `X birimini kullan`, X'i paketin kendi kaynak klasöründe çözer;
   kaynak kökeni özyineleme boyunca korunur.
4. Bildirim bağımlılık döngüsü, kaynak kullanım döngüsü, ad çakışması ve proje
   dışına çıkan giriş/birim sembolik bağı sessizce kabul edilmez (P007/P009,
   A008/A009).
5. Ağ, gizli global paket klasörü, post-install betiği ve ortam değişkenli
   arama yolu yoktur. Yerel grafik tamamen çevrimdışı ve belirgindir.
6. Paket kendi yetkinlik ve outbound origin ihtiyacını bildirir. Bu küme ana
   uygulamanın bildirimini aşarsa grafik P015 ile derlemeden önce reddedilir;
   nihai yetki sahibi uygulamadır (RFC-0024/spec-23).

`dil kilitle <proje>` deterministik `proje.kilit` üretir. Dosya şunları içerir:

- bütün doğrudan/geçişli paketler, sürümleri ve ana projeye göre göreli yolları;
- her paketin gizli/hedef klasörleri ve sembolik bağları dışarıda bırakılmış
  `.dil` kaynaklarının SHA-256 özeti;
- doğrudan bağımlılık kenarları.

Kayıtlar ada/yola göre sıralıdır; mutlak makine yolu yazılmaz. Var olan kilit
beklenen metinle byte-byte aynı değilse `çalıştır/denetle/dene` P008 verir.
Bağımlılık değişikliği ancak kullanıcı `dil kilitle` diyerek sabitlenir.
Derleme boyunca kilit denetlenen bellek görüntüsü kullanılır; denetimden sonra
kaynak yeniden okunmaz.

`dil ekle <yerel-yol> [proje]` (K-079), elle manifest düzenlemenin güvenli
karşılığıdır. Yol kabuğun çalışma klasöründen çözülür, manifestte proje köküne
göre göreli `/` ayraçlı biçimde saklanır. Yorumlar/alanlar korunur, yollar
sıralanıp tekilleştirilir. Aday manifest ve bütün geçişli grafik **yazmadan
önce** doğrulanır; başarısız çözüm mevcut manifest/kilide dokunmaz. Kilit yazma
hatasında eski iki dosya geri yüklenir. Aynı kanonik kök farklı yol yazımıyla
ikinci kez eklenmez.

`dil paketler [proje]` (K-080), çözülmüş ve kilidi doğrulanmış grafiği
doğrudan/geçişli ayrımıyla; ad, sürüm, ana projeye göre yol ve SHA-256 özetle
gösterir. `dil çıkar <paket> [proje]` yalnız doğrudan bağımlılığı hedefler.
Ana projenin herhangi bir `.dil` kaynağında paketi alan bir
`X paketini kullan` bildirimi kalmışsa P010 ile, bildirim ve kilide dokunmadan
durur. Başarılı kaldırma K-079'un aday-grafik-doğrulama ve iki dosyalı geri alma
sözleşmesini kullanır. Kaldırılan doğrudan paket başka bir paketin bağımlılığı
ise çözülmüş grafikte geçişli olarak kalabilir.

### 4.2 Uzak paketler ve yayın (Faz 5 — K-094/K-095 güven çekirdekleri)

- `dil anahtar üret` ve `dil paketle`, deterministik `.zep`, SPDX 3.0.1 SBOM,
  SLSA v1 provenance ve Ed25519 `zee-yayin-v1` üretir. Kesin biçim ve saldırı
  sınırları RFC-0020 §1–5 ile spec/18'dedir.
- K-095 ağ dışı sabit/eşik root, çift eşikli rotasyon,
  timestamp→snapshot→targets, rollback/expiry ve exact yayıncı/yanked/duyuru
  byte doğrulamasını spec/19'da çalıştırır.
- Registry adına göre uzak `dil ekle <ad@X.Y.Z>`, limitli taşıma, kalıcı
  metadata/cache ve offline kullanım henüz çalışan yüzey değildir. Var olan
  `dil ekle` yalnız açık yerel yolu kabul eder.
- Sürüm aralığı bilinçli olarak kararlaştırılmamıştır; ilk uzak istemci exact
  `X.Y.Z` dışında seçim yapmayacaktır.
- Paket adları küçük harf Türkçe tanımlayıcıdır; typosquatting/confusable
  denetimi RFC-0002'nin S028 altyapısını registry tarafında yeniden kullanır.

## 5. Çözüm sırası

`kullan` çözümü türüyle deterministiktir: `birimini` yalnız kullanan kaynağın
klasöründeki `.dil` dosyasına, sonra gömülü standart birime bakar; `paketini`
yalnız kaynak sahibinin doğrudan proje bağımlılığına bakar. Başka arama yolu
YOKTUR.

## 6. Açık sorular

1. birim/modül kelime kararı (usability).
2. `özel` görünürlük işareti v1'de mi v2'de mi.
3. Nitelikli erişim sözdizimi (çakışma çözümü için).
4. Paket içi dışa-açıklık: her tanım yerine açık bir `dışa aç` yüzeyi gerekir mi?

## Dört soru süzgeci

Doğal ✓ ("hesaplar birimini kullan" sesli okunur) · Deterministik ✓ (tek
arama sırası, çakışma=hata, döngü=hata) · Öğrenilebilir ✓ (dosya=birim,
bildirim yok) · Savunulabilir ✓ (kapsülleme + kilit dosyası + registry yolu).

## Korpus etkisi

Kabulde golden korpusa iki dosyalı bir "birim" örneği eklenir (31. program
adayı) ve golden 15'in işlemleri bir birime taşınmış varyant kazanır.
