# 07 — Birimler ve projeler

Normatif kaynak: RFC-0009 §2–4.2 (geçici kabul). Uzak yayın/registry güven ve
taşıma zinciri spec/18–19'dadır; K-136 exact proje bildirimi, kilit ve CLI
bağını bu kaynak yüzeyine eklemiştir.

## Model (TANIMLI)

- **Dosya = birim.** `hesap_araclari birimini kullan`, aynı klasördeki
  `hesap_araclari.dil` dosyasını bağlar; bulunamazsa A010.
- Birim adı tanımlayıcı kurallarına uyar (tire kullanılamaz — RFC-0009
  dosya adı notu).
- Kullanan dosya, birimin **tanımlarını** (işlem, yapı) görür; birimin
  **üst düzey cümleleri kapsüllüdür** — kullananın çıktısına karışmaz.
- Birimin testleri `dil dene` kapsamına `birim: <ad>` önekiyle katılır.
- Dışa çıkan her işlem tam, monomorfik public sözleşme taşır: bütün parametre
  türleriyle birlikte `<Tür> döndürür` ya da `değer döndürmez` (T039, spec/10).
- Birimin kendi aldığı işlem, bir üst kaynağa örtük yeniden açılmaz. Birim
  gövdesi onu çağırabilir; kullanan yalnız birimin doğrudan tanımını görür.

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

## Yerel paket ve kilit (TANIMLI — K-078)

- `X paketini kullan`, kullanan kaynağın sahibi olan projenin **doğrudan**
  `yerel_bağımlılıklar` üyesi X'in giriş kaynağını bağlar; yoksa A011.
- Paket adı bağımlı bildirimin `proje` alanıdır ve küçük harfli tek
  Türkçe/Latin tanımlayıcı olmalıdır. Grafikte adlar benzersizdir (P007).
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
