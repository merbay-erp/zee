# 07 — Birimler ve projeler

Normatif kaynak: RFC-0009 §2–4.2 (geçici kabul). Uzak yayın/registry güven ve
taşıma zinciri spec/18–19'dadır; K-136 exact proje bildirimi, kilit ve CLI
bağını bu kaynak yüzeyine eklemiştir.

## Model (TANIMLI)

- **Dosya = birim.** `hesap_araclari birimini kullan`, aynı klasördeki
  `hesap_araclari.dil` dosyasını bağlar; bulunamazsa A010.
- Birim adı tanımlayıcı kurallarına uyar (tire kullanılamaz — RFC-0009
  dosya adı notu).
- Birim adı dosya adıyla NFC'de birebir eşleşmek ZORUNDADIR (K-363). Büyük/
  küçük harf duyarsız dosya sistemi (macOS APFS, Windows NTFS) `çay` için
  `Çay.dil`'i açsa da bağlanmaz; yalnız harfi ya da Unicode biçimi farklı
  dosya her platformda aynı A010 tanısıdır. NFD saklanmış `çay.dil` aynı
  kimliktir ve her platformda bağlanır.
- Kullanan dosya, birimin **tanımlarını** (işlem, yapı) görür; birimin
  **üst düzey cümleleri kapsüllüdür** — kullananın çıktısına karışmaz.
- Birimin testleri `dil dene` kapsamına `birim: <ad>` önekiyle katılır.
- Dışa çıkan her işlem tam, monomorfik public sözleşme taşır: bütün parametre
  türleriyle birlikte `<Tür> döndürür` ya da `değer döndürmez` (T039, spec/10).
- Birimin kendi aldığı işlem, bir üst kaynağa örtük yeniden açılmaz. Birim
  gövdesi onu çağırabilir; kullanan yalnız birimin doğrudan tanımını görür.
- **Rota birimi (K-211, ADR-084):** `panel_guvenlik rotalarını kullan`, birimin
  üst düzey rota cümlelerini bu satırın yerine, birimdeki sırasıyla ana
  programa katar; tanımlar `birimini kullan` gibi alınır. Rotasız birim rota
  olarak alınamaz (A010). `birimini kullan` rotaları almaz.

## Çakışma ve döngü (TANIMLI)

- Aynı ad iki kaynaktan gelirse **A008** — sessiz gölgeleme yoktur;
  çözüm kullanıcıya bırakılır (adlardan birini değiştir / tek kaynağa topla).
- Aynı tanımın elmas yoldan iki kez gelmesi (`c → a` ve `c → b → a`)
  çakışma değildir: çakışma tanımın geldiği yola değil tanımlandığı kökene
  bakılarak sayılır; birimin testleri kökeni başına bir kez alınır
  (K-164/ADR-072). Görünürlük kuralı değişmez.
- Birimde doğan tanı (ayrıştırma, sözleşme, gövde, çalışma, test) o birimin
  kökenini taşır; satır birimin metnine göredir. CLI alıntıyı birim
  dosyasından alır, LSP tanıyı `kullan` satırına taşıyıp birim adını ve
  özgün satırı söyler (K-164/ADR-072).
- Birimlerin döngüsel kullanımı **YASAK** (A009); ortak tanımlar üçüncü
  birime taşınır.

## Yükleme anlamı (TANIMLI)

Birim çözümü de IO soyutlamasının arkasındadır. Eski `BirimYukleyici` API'si
korunur; proje araçları isteyen dosyanın ve yüklenen kaynağın kimliğini taşıyan
kökenli yükleyiciyi kullanır. Testlerde birimler sahte dünyadan gelir,
determinizm sözü birim yüklemede de geçerlidir.
Ayrıştırma "ön tarama + tohumlu ayrıştırma" ile yapılır: kullanan dosya,
birimin işlem adlarını çağrı çözümünde görür.

## Proje bildirimi (TANIMLI — K-076)

- Proje kökünde `proje.dil` bulunur; bildirim de geçerli zee kaynağıdır.
  Dış dünya `yetkinlikler`i ve exact `ağ_hedefleri` burada açılır; paket bu
  kümeyi aşamaz (spec/23, P015).
- Üç zorunlu Metin alanı tanımlar: `proje`, `sürüm`, `giriş`. `morfoloji`
  alanı kaynakların sürümlü ek profilini sabitler; yeni projeler
  `zee-tr-1` yazar, alanı olmayan eski proje aynı profile varsayılır.
  Desteklenmeyen açık profil P011'dir (spec/13). İsteğe bağlı
  `yerel_bağımlılıklar`, göreli klasör yollarından oluşan Metin listesidir.
  Exact uzak paket kullanılıyorsa `registry`, `registry_kök_sürümü`,
  `registry_kök_özeti` ve `uzak_bağımlılıklar` birlikte bulunur; ayrıntılı
  güven sözleşmesi spec/19'dadır (P017).
- Sürüm `X.Y.Z`; giriş proje içindeki göreli bir `.dil` yoludur. Mutlak yol,
  `..`, ters bölü ve platform sürücü öneki yasaktır (P003/P004).
- `dil çalıştır`, `dil denetle` ve `dil dene` bir klasör aldığında giriş
  kaynağını bu bildirimden bulur; doğrudan dosya kullanımı geriye uyumludur.
- `dil biçimle <klasör>` gizli/hedef klasörleri ve sembolik bağları izlemeden
  bütün `.dil` kaynaklarını sıralı toplar. Önce hepsi bellekte doğrulanır;
  herhangi biri hatalıysa hiçbir kaynak yazılmaz (K-077).
- Çalışan programın göreli dosya IO yolları giriş dosyasının klasöründen
  çözülür; çağıran kabuğun o anki klasörü programın anlamını değiştirmez.
- Bildirimde yan etki, işlem, yapı, test, bilinmeyen veya tekrarlı alan yoktur
  (P001/P002). Böylece proje keşfi kaynak çalıştırmadan deterministiktir.

### Göreli IO kökü ve dosya sınırı kökü (K-382)

TANIMLI: Çalışan programın göreli dosya IO kökü giriş kaynağının klasörüdür:
proje kipinde `giriş` alanının klasörü (`giriş "kaynak/ana.dil"` → `kaynak/`),
doğrudan dosya kipinde o dosyanın klasörü. `dil çalıştır <proje>`,
`dil çalıştır <proje>/<giriş>`, web worker'ı, işçi grubunun her işçisi ve
`--web-systemd-soketi` aynı köke çözer; çağıran kabuğun klasörü ve systemd
`WorkingDirectory=` anlamı değiştirmez. Dosya okuma/yazma/ekleme/taşıma/silme,
dosya yanıtı, imza okuması, e-posta eki ve binary/multipart yüklemenin
`.zee/yuklemeler` geçici dizini ile rotaya verilen `yukleme_gecici_yolu` bu
köke göredir. Proje dosya sınırı kökü (spec/23) ise `proje.dil` klasörüdür;
`.zee/web-durumu-v1.json` ve `.zee/birincil-*.kilit` gibi çalışma zamanı durum
dosyaları da orada durur. İki kök giriş proje kökündeyken çakışır.

TANIMLI: Proje kipinde `..` ve mutlak yol reddedildiğinden giriş klasörü
dışında kalan proje kökü yolları göreli yolla adlandırılamaz; bunlara giriş
klasörü içindeki, canonical hedefi proje kökü içinde kalan bir sembolik bağla
ulaşılır.

AÇIK: Göreli IO kökünü proje klasörüne taşıyan bir bildirim alanı yoktur.
Projeler arası paylaşılan veri için K-382 takip önerisi `veri_kökü`,
K-394 ile mutlak ve işletimce verilen bir kök olarak açıldı (aşağıda).

ZORUNLU: Proje dosya sınırında yol çözümü bir yokluğa çarparsa ileti bunu
güvenlik reddi gibi göstermez: yazma/ekleme/taşıma hedefinin klasörü yoksa
`"<yol>" için hedef klasör "<klasör>" yok; göreli yollar giriş dosyasının
klasöründen çözülür, klasörü önce oluşturun`, okunacak/taşınacak kaynak yoksa
`"<yol>" bulunamadı; göreli yollar giriş dosyasının klasöründen çözülür`
yazılır (ölümcül C013 ve `… dene` kalıbının başarısız Sonuç'u aynı metni
taşır). İleti programın
verdiği göreli yolu taşır, mutlak yol taşımaz. Yokluk dışındaki çözüm
hataları `dosya yolu güvenle çözülemedi` olarak kalır.

TANIMLI: `dil dene` hermetik adaptörü dosya tablosunu programın verdiği yol
metniyle anahtarlar; disk kökü kavramı yoktur.

## Paylaşılan veri kökü (K-394, RFC-0045, ADR-170)

TANIMLI: `proje.dil` isteğe bağlı `veri_kökü "AD" olsun` alanı, kökün mutlak
yolunu taşıyan ortam değişkeninin ya da aynı adlı systemd credential'ın adını
bildirir (`A-Z`, `0-9`, `_`; harf ya da `_` ile başlar; en çok 128 bayt).
Değer spec/23 K-339 credential kaynak kuralıyla (credential dizini, yoksa ortam; ikisi
birden ret) her `veri:` çağrısında okunur; tek satır, en çok 4096 bayttır.
Ayrı projeler aynı adı/yolu bildirerek aynı kökü paylaşabilir. Kök yalnız
ana projeden gelir: yerel ya da uzak paket `veri-*` ihtiyacını kökü olmadan
bildirir (ana kümenin alt kümesi, P015) ve paketin `veri:` yolları ana
projenin köküne çözülür; paket `veri_kökü` bildiremez, ana proje `veri-*`
açıp kökü bildirmezse grafik P023'tür. Bu yüzden `veri-*` kullanan bir
kütüphane paketi tek başına denetlenirken (kendi `proje.dil`'i ana proje
olur) P023 beklenir; tanı önerisi paketi kullanan kök projeden ya da paketi
yerel bağımlılık olarak alıp `veri_kökü` bildiren bir test kök projesinden
denetlemeyi gösterir.

TANIMLI: `veri:` ile başlayan dosya yolu bu köke gider; önek her kipte
ayrılmıştır ve hiçbir kipte proje klasöründe yerel dosya adı sayılmaz.
Önek sonrası `/` ile ayrılmış, boş olmayan bileşenlerdir; boş bileşen,
`.`, `..`, baştaki `/`, ters bölü, iki nokta, NUL ve denetim karakteri
YASAKTIR; bileşen en çok 255, toplam en çok 1024 bayttır.

ZORUNLU: Kök mutlak olmalı, `/` olamaz ve `.`/`..` taşıyamaz; `/`'den her
bileşen sembolik bağ izlenmeden açılır (yolda bağ YASAKTIR). Kökün ve üst
klasörlerinin sahibi root ya da etkin kullanıcıdır; üst klasörler grup/diğer
kullanıcılarca yazılamaz (yapışkan bitli klasör hariç). Linux'ta
`fs.protected_hardlinks` kapalıysa kök reddedilir. Kök okunamaz ya da
kurala uymazsa çağrı başarısız olur; proje klasörüne geri düşülmez.

ZORUNLU: Kök altındaki her klasör sembolik bağ izlenmeden açılır. Yaprağın
doğrudan üst klasörü dışındaki her klasör (kök dahil) root'a ya da etkin
kullanıcıya ait olmalı ve grup/diğer kullanıcılarca yazılamamalıdır;
paylaşılan yazma yalnız son klasörde olur. Yaprak sembolik bağ olamaz;
var olan yaprak normal dosyadır. Okuma, sembolik bağ izlenmeden ve
bloklamadan açılan tanıtıcıdan yapılır; tanıtıcı normal dosya ve tek
hard-link olmalıdır. Veri kökünde klasör oluşturma işlemi yoktur; hedef
klasör yoksa ileti klasörü işletmenin oluşturacağını söyler.

TANIMLI: Desteklenen işlemler: dosya okuma, aralıklı okuma, SHA-256, ilk
bayt imzası, dosya yanıtı, e-posta MIME eki ve bağlantıya dosya aktarımı
(`veri-okuma`); satır yazma/ekleme, çözerek yazma/ekleme, atomik taşıma
(veri↔veri ve proje↔veri) ve silme (`veri-yazma`). Eylem savepoint'i aynı
yolla çalışır.

AÇIK: Veri kökünde dizin listeleme, görsel küçültme, bağlantı blok
dosyası ve `.zee/yuklemeler` geçici dizini bu sürümde yoktur (açık ret
iletisi); Windows ve wasm hedeflerinde veri kökü desteklenmez.

ZORUNLU: Bildirimsiz tek dosya kipi (`dil çalıştır dosya.dil`,
`--deneysel-web dosya.dil`, `--izin ağ`) ve `--güvenli` kipi `veri:` yolunu
reddeder ve yerel dosya oluşturmaz. Bildirimli projede `veri_kökü` yoksa
`veri:` yolu `veri-*` yetkinliği reddiyle başarısız olur.

TANIMLI: `dil dene` hermetik adaptörü `veri:` yolunu da metin anahtarı
sayar; disk ve ortam okunmaz. Ret iletileri kökün mutlak yolunu taşımaz;
programın `veri:` yolunu ve ortam adını taşır.

## Yerel paket ve kilit (TANIMLI — K-078)

`dil biçimle --denetle <dosya|proje>` kaynağı değiştirmeden kanonik biçimle
karşılaştırmak ZORUNDADIR. Uygun biçim exit 0, biçim farkı exit 1'dir;
içerik, mtime ve izin yazımı YASAKTIR. Varsayılan `biçimle` yazma kipi sürer.

- `X paketini kullan`, kullanan kaynağın sahibi olan projenin **doğrudan**
  `yerel_bağımlılıklar` üyesi X'in giriş kaynağını bağlar; yoksa A011.
- Paket adı bağımlı bildirimin `proje` alanıdır ve küçük harfli tek
  Türkçe/Latin tanımlayıcı olmalıdır. Grafikte adlar benzersizdir (P007).
  Alfabe lexer'ın tanımlayıcı kümesidir (ASCII küçük harf/rakam, `_`,
  `ç ğ ı ö ş ü â î û`; rakamla başlamaz). Başka yazıdan homoglif (Kiril `р`,
  Yunan `ρ`), birleştirici im, bağ harfi ve görünmez biçim karakteri
  yerel bağımlılıkta P007, uzak bağımlılıkta P017'dir (K-363). `dil paketle`
  ana projenin adına aynı kuralı uygular (K-366, spec/18).
- Paket de birim gibi yalnız işlem/yapı/test tanımlarını dışarı verir; üst
  düzey cümleleri çalışmaz. Paket içi birim, onu kullanan gerçek dosyanın
  klasöründen çözülür.
- Paket işlemlerinde de aynı tam public imza zorunludur; yalnız paket içinde
  çağrılması ya da hiç çağrılmaması bu zorunluluğu değiştirmez.
- Geçişli bağımlılık çözülür ve kilitlenir ama doğrudan bildirilmedikçe
  kaynakta kullanılamaz. Bildirim döngüsü ve aynı adlı ayrı paket hatadır.
- `dil kilitle <proje>` bütün grafiği ada göre sıralı `proje.kilit` dosyasına
  yazar: kilit biçimi sürüm 3'te proje/paket sürümü, morfoloji profili, ana
  projeye göre göreli yol, bağımlılık kenarı ve bütün `.dil` kaynaklarının
  SHA-256 özeti. Mutlak yol yazılmaz.
- Var olan kilit güncel grafikle byte-byte aynı değilse proje komutları P008
  verir. Kilit doğrulandıktan sonra aynı komut kaynakları yeniden okumaz.
- Kaynak toplama gizli/hedef klasörleri ve sembolik bağları izlemez; paket
  girişi ve birimleri kendi proje köklerinin dışına çıkamaz (P009).
- `dil ekle <yerel-yol> [proje]` yolu kanonik çözer ama manifestte ana proje
  köküne göre göreli ve `/` ayraçlı saklar. Aday manifest/grafik yazmadan önce
  doğrulanır; yorumlar korunur, yollar sıralanır ve yinelenen gerçek kök
  yeniden eklenmez (K-079).
- `dil paketler [proje]` kilidi doğrulanmış grafiği doğrudan/geçişli ayrımı,
  sürüm, morfoloji profili, göreli yol ve SHA-256 özetle gösterir.
  `dil çıkar <paket> [proje]`
  yalnız doğrudan paketi kaldırır. Ana projenin bir `.dil` kaynağında ilgili
  `X paketini kullan` bildirimi kalmışsa P010 verir ve bildirim/kilit byte-byte
  değişmez. Başarıda aday grafik önce çözülür; bildirim ile kilit birlikte
  güncellenir ve kilit yazımı başarısızsa ikisi geri alınır (K-080).
- ZORUNLU (K-347): `dil ekle` ve `dil çıkar` (yerel ve registry) `proje.dil`i
  yalnız hâlâ komutun okuduğu byte'lardaysa yazar; karşılaştırma, bildirim ve
  `proje.kilit` yayını tek klasör yazma kilidi penceresindedir. Araya giren
  komut bildirimi değiştirdiyse hiçbir dosya yazılmaz ve komut "başka bir
  süreç tarafından değiştirildi … yeniden çalıştır" iletisiyle exit 2 verir;
  eşzamanlı iki komut birbirinin bağımlılığını ezemez. Kilit yazılamazsa iki
  dosya aynı kilit altında önceki byte'larına döner. Proje kaynak toplayıcısı
  `proje.dil` dahil kaynakları yalnız değiştirme penceresini bekleyen
  okuyucuyla açar; eşzamanlı yazım okuyan komutu düşürmez.

## Exact registry paketi ve kilit (TANIMLI — K-136)

- Uzak bağımlılık yalnız exact `ad@X.Y.Z` biçimindedir; aralık, etiket,
  ön-sürüm seçicisi veya registry öncelik araması yoktur. Bütün uzak
  bağımlılıklar aynı bildirimin HTTPS originini ve ağ dışı sabitlenen pozitif
  root sürümü + `sha256:` özetini kullanır (P017).
- `dil ekle ad@X.Y.Z [proje]` ilk kullanımda `--registry` ve `--kök
  <sürüm>@sha256:<özet>` ister. Aday yayın tam metadata/yayıncı zincirinden
  geçmeden bildirim veya kilit yazılmaz. `--çevrimdışı` yalnız önceden
  doğrulanmış cache'i kullanır.
- `dil kilitle` uzak bağımlılık varsa açıkça çevrimiçi yeniler;
  `--çevrimdışı` ağsızdır. `dil paketler` varsayılan ağsızdır; yalnız
  `--yenile` ağı açar. `çalıştır`, `denetle`, `dene` ve LSP sessiz ağ açmaz.
- Doğrulanmış nesneler proje kökündeki `.zee/registry/<root-özeti>` altında,
  açılmış kaynaklar `.zee/paketler/sha256/<arşiv-özeti>` altında tutulur.
  `.zep` görünmez kardeş geçicide açılır, tam ağaç yeniden doğrulanır, atomik
  adlandırılır ve kaynaklar salt-okunur yapılır. Sembolik bağ/fazladan dosya
  ya da dizin/özet uyuşmazlığı P016'dır.
- `proje.kilit` v3; ilk root sürüm+özeti, etkin root/timestamp/snapshot/targets
  sürüm+özetleri, yayıncı kimliği, `.zep`/SBOM/provenance/yayın özetleri,
  yanked durumu ve kritik duyuru kimliklerini taşır. `--yanked-kabul` ve
  `--kritik-kabul` ancak boş olmayan insan gerekçesiyle çalışır; gerekçe kilide
  yazılır ve sonraki ağsız çözümde korunur. Kritik kabul anahtarı sıralı etkin
  duyuru kümesini de taşır; yeni bir kritik duyuru eski gerekçeyi devralamaz.
- Yerel ve uzak paketler tek ad/köken grafiğine girer. Aynı P007/P008/P009/
  P010/P015 kapsülleme, kilit ve yetkinlik kuralları ikisine de uygulanır.
