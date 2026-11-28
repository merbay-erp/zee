# RFC-0045 — Paylaşılan veri kökü (`veri_kökü`, `veri:` öneki)

- **Durum:** **kabul — en küçük dilim uygulandı** (K-394,
  [ADR-170](../adr/170-paylasilan-veri-koku.md), 24 Kasım 2026). Dizin
  listeleme, görsel dönüştürme, Windows ve bağlantı blok dosyasının veri
  köküne alınması **açık**; her biri ayrı `dogfood-change` ister.
- **Tarih:** 24 Kasım 2026
- **İş:** K-394 (K-382 takip önerisi `veri_kökü`)
- **İlgili belgeler:** [spec/07](../spec/07-birimler.md) §Paylaşılan veri
  kökü, [spec/08](../spec/08-kalici-dosya.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [spec/26](../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [ADR-031](../adr/031-merkezi-yetkinlik-ve-outbound-istemci.md),
  [ADR-105](../adr/105-credential-host-koprusu.md) (K-339 credential
  kaynağı), [ADR-163](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md),
  [ADR-167](../adr/167-dosya-kodlanmis-icerik-ve-bellek-ozeti.md),
  [RFC-0039](0039-ag-sunucusu-yetkinligi.md) §G ("veri kökü yazımı"),
  [RFC-0041](0041-ikili-yanit-ve-ek-akisi.md),
  [RFC-0043](0043-bayt-kodlama-ve-ozet-yuzeyi.md)
- **Freeze sınıfı:** `dogfood-change` (`urun-h`, ZM-F038). Yeni kalıp,
  grammar kuralı ya da tür yok; bir proje alanı, iki yetkinlik adı, bir yol
  öneki ve P023.

## Özet

Aynı uygulamanın ayrı projeleri (ayrı veritabanı rolleri, en az ayrıcalık)
ortak bir veri alt ağacını paylaşabilsin: `proje.dil` kökün mutlak yolunu
taşıyan **ortam değişkeninin / systemd credential'ının adını** bildirir
(`veri_kökü "UYGULAMA_DEPO" olsun`), program o kökteki dosyaları `veri:`
önekiyle adlandırır (`veri:ekler/k1-1f…`). Okuma `veri-okuma`, yazma
`veri-yazma` yetkinliği ister; böylece sunan web hizmeti salt okur. Kökten
yaprağa kadar her bileşen sembolik bağ izlenmeden açılır, yaprak bağ ya da
çok hard-link'li olamaz ve paylaşılan yazma yalnız son klasörde olabilir;
yani bir hizmet ele geçirilse bile başka bir hizmete kök dışı dosya
okutamaz ya da yazdıramaz.

## Motivasyon

`urun-h` (posta ürünü) SMTP alıcısı, gönderim kapısı ve IMAP hizmetini ayrı
Zee projeleri olarak, ayrı PostgreSQL rolleri ve en az ayrıcalıkla koşar.
K-389 (RFC-0043 Dilim 1) ile gelen eki `base64 çözerek … dosyasına yazmayı
dene` ile bayt-kesin çıkarabiliyorlar; ayrı web projesi aynı eki
`… adıyla ek olarak göndermeyi dene` ile sunmalı (ZM-F038). Proje dosya
sınırı (spec/07 K-382, spec/23) bunu imkânsız kılıyor:

- Her proje yalnız kendi `proje.dil` klasörünün altını görür; `..` ve
  mutlak yol yasaktır.
- Giriş klasörü içindeki sembolik bağ ancak canonical hedefi **okuyan
  projenin** kökünde kalırsa izlenir; başka projenin klasörüne giden bağ
  reddedilir.

Ürünün bugünkü durumu: ekli gelen ileti görünür uyarıyla kesilir, ekli
gönderim `554` ile düşer, IMAP `APPEND` 64 KB'ta durur. İki geçici çözüm
güvenliği ya da işlevi bozar:

1. **Hizmetleri web kökünün altına taşımak:** alıcı/gönderici web projesinin
   kodunu ve durum dosyalarını (`.zee/web-durumu-v1.json`) görür; en az
   ayrıcalık ve ayrı rol modeli çöker.
2. **Ekleri PostgreSQL `bytea` olarak saklamak:** parametre başına 64 KiB
   sınırı (K-391 SQL parametre tavanı) 16 MiB eki parçalara böldürür;
   atomiklik, akış ve dosya yanıtı yolu kaybolur.

K-382, `veri_kökü` fikrini proje köküne göre göreli bir klasör olarak
önermiş ve kabul için ikinci gerçek tüketici kanıtı istemişti. ZM-F038 o
kanıttır; ama göreli klasör ayrı projeleri birleştiremediği için tasarım
mutlak, işletim tarafından verilen bir köke dönüşür.

## Tasarım

### 1. Bildirim

```zee
proje "posta_alici" olsun
sürüm "0.1.0" olsun
giriş "kaynak/ana.dil" olsun
yetkinlikler "ağ-dinleyici", "veri-okuma", "veri-yazma" listesi olsun
ağ_hedefleri boş liste olsun
veri_kökü "POSTA_DEPO" olsun
```

- Değer bir **ad**dır: `A-Z`, `0-9`, `_`; harf ya da `_` ile başlar, en
  çok 128 bayt. Yol kaynağa yazılmaz (dağıtıma özgüdür, açık depoya
  sunucu yolu girmez).
- Ad, mevcut gizli değer kaynak kuralıyla (K-339) çözülür: önce
  `$CREDENTIALS_DIRECTORY/<ad>` (systemd `LoadCredential=`/`SetCredential=`),
  yoksa ortam değişkeni; ikisi birden varsa ret. Değer tek satırdır,
  en çok 4096 bayt.
- `veri_kökü` en az bir `veri-*` yetkinliğiyle bulunur; ana projede `veri-*`
  de yalnız `veri_kökü` ile açılır; aksi P023.
- Kök yalnız ana projeden gelir. Paket `veri-*` ihtiyacını **kökü olmadan**
  bildirir (ana kümenin alt kümesi, P015); paket kaynağındaki `veri:` yolu
  ana projenin köküne ve yetkinliğine tabidir. Paket `veri_kökü`
  bildiremez; ana proje `veri-*` açıp kök bildirmezse grafik P023'tür.
- Kütüphane paketi tek başına `dil denetle` edilince kendi bildirimi ana
  proje olur ve kök yoktur; P023 beklenir. Tanı önerisi paketi kullanan kök
  projeden ya da paketi yerel bağımlılık olarak alan, `veri_kökü` bildiren
  küçük bir test kök projesinden denetlemeyi gösterir. Paketin `veri:`
  kullanımı her zaman kök projenin `veri_kökü`üne bağlıdır.

### 2. Kök değerinin kuralları

Kök her `veri:` çağrısında yeniden okunur ve doğrulanır (önbellek yok;
dönen credential/ortam değişikliği bir sonraki çağrıda geçerlidir):

1. Mutlak yol olmalı; `/` (dosya sisteminin kökü) olamaz; `.`/`..`
   bileşeni taşıyamaz.
2. `/`'den her bileşen `openat(O_DIRECTORY|O_NOFOLLOW)` ile açılır; yolda
   **hiçbir** sembolik bağ olamaz (işletme gerçek/kanonik yolu yazar;
   macOS'ta `/var` yerine `/private/var`).
3. Kökün üst klasörleri: sahibi root ya da hizmetin etkin kullanıcısı;
   grup/diğer yazamaz — yalnız yapışkan bitli klasör (`/tmp`) istisnadır.
   Kök klasörünün sahibi root ya da etkin kullanıcıdır.
4. Linux'ta `fs.protected_hardlinks` kapalıysa (`0`) ya da `/proc` bağlı
   değilse kök reddedilir (§Tehdit T4). K-422 (ADR-170 eki): hizmet değeri
   okuma izni olmadığından okuyamıyorsa (EACCES/EPERM; Ubuntu 22.04) koruma
   bilinmiyor kipidir ve kök kabul edilir; güvence veri kökü yazımlarının
   tek-bağ kipinden gelir (§Tehdit T11).
5. İletiler kökün mutlak yolunu taşımaz; ortam/credential adını ve
   programın `veri:` yolunu taşır.

### 3. `veri:` yolu

`veri:` sonrası `/` ile ayrılmış, boş olmayan bileşenlerdir. Boş bileşen
(`//`, sondaki `/`), `.`, `..`, baştaki `/`, ters bölü, iki nokta (sürücü
öneki/NTFS ADS), NUL ve denetim karakteri yasaktır; bileşen en çok 255,
toplam en çok 1024 bayttır. Önek her kipte ayrılmıştır: `veri:` ile
başlayan yol hiçbir kipte proje klasöründe yerel dosya adı sayılmaz.

Çözüm (`openat` yürüyüşü):

- Kökten yaprağın üst klasörüne kadar her klasör `O_NOFOLLOW` ile açılır;
  bağ ya da klasör olmayan bileşen reddedilir.
- **Ata değişmezi:** yaprağın doğrudan üst klasörü **dışındaki** her
  klasör (kök dahil) root'a ya da etkin kullanıcıya aittir ve grup/diğer
  yazamaz. Paylaşılan yazma (birden çok hizmetin ortak grubu) yalnız son
  klasörde olabilir. Böylece başka bir kullanıcı hiçbir ara klasör
  girdisini doğrulama ile kullanım arasında bağla değiştiremez.
- Yaprak `fstatat(AT_SYMLINK_NOFOLLOW)` ile denetlenir: bağ reddedilir;
  var olan yaprak normal dosya olmalıdır; okumada tek hard-link şarttır.
  Yazmada yaprak yok olabilir; üst klasör yoksa ileti klasörü işletmenin
  oluşturacağını söyler (veri kökünde klasör oluşturma işlemi yoktur).

### 4. Yetkinlikler

| Yetkinlik | Açtığı | Açmadığı |
|---|---|---|
| `veri-okuma` | `veri:` okuma, aralıklı okuma, SHA-256, ilk baytlar, dosya yanıtı, e-posta eki, bağlantıya dosya aktarımı | proje dosyası okuma; her türlü yazma |
| `veri-yazma` | `veri:` yazma, ekleme, çözerek yazma/ekleme, taşıma, silme | proje dosyası yazma; okuma (ayrıca `veri-okuma` gerekir) |

`dosya-okuma`/`dosya-yazma` veri kökünü, `veri-*` proje dosyalarını açmaz.
Taşımada kaynak ve hedef ayrı ayrı denetlenir (proje↔veri taşıması iki
tarafın yazma yetkinliğini ister; farklı dosya sisteminde K-332 EXDEV
kopyala-yayımla-sil yolu kullanılır). Statik T054 denetimi yol sınıfını
çoğu zaman bilemez (yol çalışma anında kurulur); dosya gereksinimini aynı
yönlü `veri-*` yetkinliğiyle de karşılanmış sayar, kesin sınıf ve
yetkinlik her çağrıda çalışma zamanında denetlenir. Bu, RFC-0039 §G'de
adı geçen "veri kökü yazımı" yetkinliğinin somut biçimidir.

### 5. İşlem kapsamı

Desteklenen (`veri:` ile):

- Okuma: `dosyasını okumayı dene`, `… konum baytından n baytını … okumayı
  dene`, `sha256 özetini almayı dene`, ilk bayt imzası, `… adıyla ek olarak
  göndermeyi dene` (RFC-0041 dosya yanıtı), e-posta MIME eki, bağlantıya
  `dosyasını aktarmayı dene`.
- Yazma: `dosyasına … yaz` (spec/08 atomik), ekleme, `base64|quoted_printable
  çözerek … yazmayı|eklemeyi dene` (no-clobber/ekleme, RFC-0043), atomik
  taşıma (veri↔veri, proje↔veri), silme. Eylem savepoint'leri (spec/08
  K-346) aynı yolla çalışır.

Bu dilimde **desteklenmeyen** (açık ret iletisiyle): dizin listeleme
(içerik adresli depoda kayıt veritabanındadır), görsel küçültme, bağlantı
blok dosyasının (`nokta bloğunu dosyaya almayı dene`) ya da
`.zee/yuklemeler` geçici dizininin veri kökünde açılması (ürün blok
dosyasını kendi kökünde alıp `veri:`'ye çözerek yazar ya da taşır).

### 6. Aynı kökü paylaşan projeler

Bir uygulamanın birden çok projesi aynı adı ya da aynı yolu bildirebilir.
Aynı köke **yazan** hizmetler o alt ağaç için tek bütünlük alanıdır:
birbirinin dosyasını ezebilir/silebilir (içerik adresli ad ve no-clobber
yazım bunu ürün düzeyinde yönetir). Okuyucu hizmetler ise yazarlara karşı
korunur: hiçbir yazar okuyucuya kök dışı içerik okutamaz (§Tehdit). Aynı
etkin kullanıcıyla koşan hizmetler tek ilkedir (aynı uid'nin klasörleri
birbirini değiştirebilir); ayrılık istenen hizmetler ayrı kullanıcıyla
koşar. Önerilen yerleşim:

```text
/var/lib/uygulama/depo        root:root     0755   (veri kökü)
/var/lib/uygulama/depo/ekler  root:posta    2770   (paylaşılan son klasör)
```

systemd: yazan birimlere `ReadWritePaths=/var/lib/uygulama/depo/ekler`,
okuyan web birimine `ReadOnlyPaths=/var/lib/uygulama/depo`; kök yolu
`SetCredential=POSTA_DEPO:/var/lib/uygulama/depo` ya da `Environment=`.

### 7. Kip davranışı

- **Proje kipi** (`dil çalıştır <proje>`, web worker, işçi grubu,
  `--web-systemd-soketi`, `--deneysel-web` ile proje): yukarıdaki gibi.
- **Tek dosya kipi** (`dil çalıştır dosya.dil`, `--deneysel-web dosya.dil`,
  `--izin ağ`): bildirimsiz profilde veri kökü yoktur; `veri:` yolu
  `yalnız proje kipinde, proje.dil veri_kökü bildirimiyle kullanılabilir`
  iletisiyle reddedilir ve hiçbir yerel dosya oluşturulmaz.
- **`--güvenli` (çocuk) kipi:** aynı ret.
- **`dil dene`:** hermetik adaptör dosya tablosunu yol metniyle anahtarlar
  (K-382); `veri:ekler/a` bir anahtardır, disk/ortam okunmaz. Yetkinlik
  denetimi projenin bildirimiyle aynıdır.
- **IO izi:** yol metni olduğu gibi kaydedilir; kök yolu ize girmez.
- **Windows / wasm:** veri kökü bu dilimde desteklenmez (açık ileti).

## Tehdit modeli ve güvenlik analizi

**Varlıklar:** her hizmetin kök dışı dosyaları (kod, `.zee` durumu,
credential'lar, `/etc`), paylaşılan kökteki ekler, ekin bütünlüğü.

**Aktörler:** (A1) uzak saldırgan — `veri:` yolunu etkileyen girdi
(ek adı, istek parametresi); (A2) ele geçirilmiş **eş hizmet** — paylaşılan
son klasöre yazabilen başka uid; (A3) aynı makinedeki başka yerel
kullanıcı; (A4) işletme hatası (yanlış izin/yol).

| # | Tehdit | Karşı önlem | Kanıt |
|---|---|---|---|
| T1 | `..`, mutlak yol, ters bölü, NUL, `//` ile kökten kaçış (A1) | Sözdizimi reddi IO'dan önce; bileşen bazlı | `veri_koku_goreli_yol_ve_bilesen_sozdizimini_reddeder`, `kacis_denemeleri_…` |
| T2 | Ara klasörde sembolik bağ (A2/A4) | Her bileşen `O_NOFOLLOW`; bağ ret | `veri_koku_ara_ve_yaprak_sembolik_bagi_izlemez` |
| T3 | Doğrulama ile kullanım arasında ara klasörü bağla değiştirme (TOCTOU, A2) | Ata değişmezi: üst klasör dışındaki klasörleri başkası değiştiremez; aksi ret | `veri_koku_ata_klasoru_…`, `ata_klasoru_paylasilan_…` |
| T4 | Yaprağı yarışla bağa/hard-link'e çevirip **okuyucuya** kök dışı dosya okutma (confused deputy, A2) | Okuma `O_NOFOLLOW|O_NONBLOCK` tanıtıcıdan; `S_ISREG` ve `st_nlink == 1`; Linux `protected_hardlinks` zorunlu | `veri_koku_yaprak_yarisinda_…` (3000 tur), `veri_koku_hard_link_…`, web 404 testi |
| T5 | Yaprağı yarışla bağa çevirip **yazara** kök dışı içeriği paylaşılan dosyaya kopyalatma (çift değiştirme, A2) | spec/08 iç okumaları (ekleme, eylem yedeği, taşıma/silme izi, geri alma) `O_NOFOLLOW`; kilit dosyası `O_NOFOLLOW`; yayın `rename` (bağı izlemez) | `veri_koku_cift_degistirme_…` (deterministik enjeksiyon; düzeltmesiz kodda sır kopyalanıyordu), `…yazar_yaprak_yarisinda…` |
| T6 | FIFO/aygıt yaprakla okuyucuyu bloklama (A2) | `O_NONBLOCK` açılış + `S_ISREG` | birim |
| T7 | Kökün kendisini ya da üst klasörünü değiştirme (A3/A4) | Kök yolunda bağ yok; üst klasör sahibi root/etkin kullanıcı, grup/diğer yazamaz (yapışkan hariç) | `veri_koku_mutlak_sembolik_bagsiz_ve_korunan_olmali` |
| T8 | Salt okuyucu hizmetin yazması (A1) | Ayrı `veri-yazma`; statik T054 + çalışma zamanı ret | `yazma_yetkisi_olmayan_proje_…` |
| T9 | Ortam eksik/yanlışken sessizce proje klasörüne düşme (A4) | Fail-closed: kök okunamazsa çağrı başarısız; `veri:` hiçbir kipte yerel ad değil | `ortam_degiskeni_yoksa_…`, `tek_dosya_kipinde_…` |
| T10 | Hata iletisinde mutlak yol/sunucu düzeni sızması | İletiler yalnız `veri:` yolunu ve ortam adını taşır | web ve kaçış testleri |
| T11 | Paylaşılan klasöre bağlanan sert bağla **yazara** dış dosyayı okutup tek bağlı yeni dosyaya aklatma — ekleme, eylem yedeği, taşıma/silme izi, EXDEV kopyası (A2; `protected_hardlinks` bilinmiyor/macOS) | K-422 tek-bağ kipi: kilit, mevcut içerik, metadata, iz ve kopya okuyucusu tanıtıcıda `st_nlink == 1` | `veri_koku_tek_bag_kipinde_…`, `sert_bagli_veri_yapragi_…`, `regression/security/veri-koku-sert-bag-ekleme.dil` |

**Kalan riskler (bilinçli sınır):**

- Aynı köke yazan hizmetler birbirinin dosyasını ezebilir/silebilir
  (§6); disk/inode tüketimi işletim kotası işidir (dosya başına 16 MiB
  tavanı sürer).
- ~~macOS'ta çekirdek hard-link koruması yoktur: yazarın iç okuması
  (ekleme) bir eş hizmetin bağladığı hard-link üzerinden yazara okunabilir
  dosyayı kopyalayabilir.~~ K-422 ile kapandı (T11): yazımlar tek-bağ
  kipindedir. Kalan: değer `0` iken okunamıyorsa kilit adına bağlanmış ve
  hizmetin açabildiği aygıt düğümü bir kez açılıp reddedilir (ADR-170 eki).
- POSIX/macOS ACL'leri ayrıca incelenmez (Linux'ta ACL maskesi mod grup
  bitlerine yansır).
- Ağ dosya sistemleri kapsam dışıdır (spec/08 ile aynı).

**Neden `openat2(RESOLVE_BENEATH)` değil:** macOS geliştirme hedefinde yok
ve spec/08 kalıcı dosya katmanı yol tabanlıdır (kilit, geçici dosya,
`ReplaceFileW` kurtarması). Ata değişmezi aynı güvenceyi (ara bileşenler
sabit) taşınabilir biçimde verir; tanıtıcı tabanlı kalıcı katman ileride
değişmezi gevşetebilir.

## Reddedilen seçenekler

- **Proje köküne göreli `veri_kökü "depo"` (K-382 taslağı):** ayrı proje
  köklerini birleştirmez; ZM-F038'i çözmez.
- **Kök dışı canonical hedefli bağa izin:** her projenin sınırını
  genişletir, yarışa açıktır ve hangi dışa izin verildiği bildirimde
  görünmez.
- **Genel "mutlak yol" yetkinliği:** en az ayrıcalığı bozar; kök
  sınırlanmaz.
- **Yolun kaynağa yazılması:** dağıtıma özgü değer kaynağa girer; açık
  depoda sunucu düzeni görünür; credential döndürülemez.
- **Hizmetleri web köküne taşımak / PostgreSQL `bytea`:** §Motivasyon.

## Uyumluluk ve göç

- `veri:` öneki ayrılmıştır. Yerel envanterde `veri:` ile başlayan dosya
  yolu kullanan kaynak yoktur; böyle bir dosya adı artık proje klasöründe
  aranmaz.
- spec/08 iç okumaları yaprak bağını izlemez: atomik replace zaten normal
  dosya hedefini şart koşuyordu; tek fark, eylem içinde **bağ olan**
  yaprağı taşıma/silme izinin artık C013 vermesidir.
- Alan yokluğu göç gerektirmez; mevcut projeler birebir aynı davranır.

## Kabul kanıtı

`compiler/tests/veri_koku_testi.rs` (iki proje aynı kök, salt okur web ek
sunar; yazma yetkisiz; eksik/güvensiz kök; kaçış ve bağlar; ata klasörü;
tek dosya; bildirimsiz proje; P023; `dene`), `compiler/src/guvenlik/veri_koku/testler.rs`
(sözdizimi, kök kuralları, bağ, yokluk, hard-link, ata değişmezi, okuyucu
yarışı, yazar yarışı, çift değiştirme), dogfood
`dogfood/urun-h/paylasilan-depo/`, ADR-170, spec/07 §Paylaşılan veri kökü,
GB-168.

## Açık sorular / sonraki dilimler

1. Dizin listeleme (ürün ihtiyacı doğarsa; yaprak adları için `openat` +
   `fdopendir`).
2. Bağlantı blok dosyasının doğrudan veri kökünde açılması (çift kopyayı
   önler; ölçülmüş ihtiyaç bekler).
3. Windows (`FILE_FLAG_OPEN_REPARSE_POINT` yürüyüşü + sahip/ACL eşdeğeri).
4. Tanıtıcı tabanlı spec/08 katmanı (`openat`/`renameat`) ile ata
   değişmezinin gevşetilmesi.
