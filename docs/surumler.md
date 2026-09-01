# Sürüm notları

Her sürüm: ne eklendi, ne değişti, neyin sözü verildi. Kırıcı değişiklik
sessizce yapılamaz (master plan bölüm 23) — burada duyurulur.

## v0.7.0 — 1 Eylül 2026

**Tema: editör zekâsı — dil, düzenleyicide de Türkçe düşünüyor.**

(Üstteki K-069..K-072 kalemleri bu sürümündür: çıkış kodu, kitaplık
olgunlaşması, zengin doğrulamalar, morfolojili yeniden adlandırma.)

## Yolda (v0.8.0'a birikenler)

- **Normatif otorite ve v1 kapıları** (K-081, ADR-010): geçerli dilin kesin
  davranışını spec anlatır; RFC değişikliği yetkilendirir ama spec+conformance
  testi aynı değişiklikte güncellenmeden yürürlüğe girmez. Kaynak denetimli
  [v1.0 sürüm kapıları](v1-surum-kapilari.md) bağlayıcıdır. RFC-0006 ve
  RFC-0011 güncel gerçeklemeyle uzlaştırıldı; RFC-0015 production web/eylem
  sınırını taslağa aldı.
- **Deneysel web korkuluğu** (K-082): gerçek TCP sunucusu artık sıradan
  `dil çalıştır` ile açılmaz; yalnız açık `--deneysel-web` opt-in'i ve görünür
  production uyarısıyla localhost'ta çalışır. Panel örnekleri eğitim demosu
  olarak yeniden etiketlendi; giriş/kaydet/sil/çıkış durum değişiklikleri POST
  kontrolüne alındı. GET ile silmeme regression testidir.
- **Açık işlem imzası** (K-083): başlangıç için `sayıyı al` aynen kalır;
  public/paket API'si `sayıyı Ondalık olarak al` yazabilir. Açık gövde hiç
  çağrılmasa da denetlenir, imza çağrıyla değişmez. Liste/sözlük/Seçenek/Sonuç
  ve yapı tür yazımları desteklenir; TamSayı→Ondalık runtime değeri de
  genişler. T037/T038, golden 33 ve çağrı sırası permütasyonlarıyla 270 test.
- **Atomik kalıcı dosya** (K-084, RFC-0016): `dosyasına ... yaz/ekle`
  aynı klasörde geçici dosya + disk eşzamanlama + atomik replace kullanır.
  Unix `flock` / Windows `LockFileEx` süreç kilidi iki yazarın
  güncellemesini korur; ani süreç sonu kilidi bırakır. Biçimleyici, paket
  bildirimi ve `proje.kilit` tek-dosya yazmaları da aynı çekirdeğe taşındı.
  Hata enjeksiyonu, eşzamanlı okuyucu ve iki bağımsız CLI yazarıyla 276 test.
- **Gerçek son tarih iptali** (K-085, RFC-0011/spec-09): `içinde` artık
  gövdeyi bitirip geç kaldığını sonradan söylemez. Mutlak deadline
  blok/işlem/döngü sınırlarında denetlenir; `bekle` kalan süreye kırpılır,
  HTTP bağlantı/yazma/okuma tek bütçeyi kullanır. İç içe bloklarda yalnız
  deadline sahibi `yetişmezse` kolu çalışır; iptal sonrası yan etki yoktur.
  Ç001 iç nöbetçisiyle 123 katalog kodu ve toplam 279 test; playground'da
  ardışık sanal beklemeler de aynı son tarih anlamını taşır.
- **Tam public işlem sözleşmesi** (K-086, spec-10): yerel başlangıç
  işlemlerinde `sayıyı al` çıkarımı korunur; birim/paket işlemleri bütün
  parametrelerini `<ad> <Tür> olarak al` ve dönüşünü `<Tür> döndürür` ya da
  `değer döndürmez` ile açıkça bildirir. Dönüş türü gövdeyle ve bütün akış
  yollarıyla kanıtlanır (T039–T042). v1 public modeli bilinçli monomorfik
  kaynak ABI'sidir; kırıcı semver sınırı spec'te sabittir. Standart kitaplık
  bu sözleşmeye geçirildi; `liste_araclari` somut Ondalık ABI'si taşır,
  TamSayı listeleri çağrıda genişler ve uç sonuçları da `71,0` gibi Ondalık
  döner. Liste/özyineleme/çağrı sırası ve gerçek paket
  olumsuzuyla ve örtük yeniden-dışa-açma yasağıyla toplam 289 test, 127
  katalog kodu. V1-P0-01 kapandı.
- **Uygulama eylemi ve yöntemli web adaptörü** (K-087, RFC-0015/spec-11):
  açık imzalı `eylem` aynı çağrı sözdizimiyle web/CLI/görev/test bağlamında
  kullanılır; HTTP yanıtı/çerezi ve geri alınamayan ekran/girdi/donanım etkisi
  taşıyamaz. GET/HEAD'in doğrudan ve çağrı grafiğindeki dolaylı yazması T045,
  rota içi uygulama yazması T046'dır. Açık GET/HEAD/POST/PUT/PATCH/DELETE,
  404/405, 64 KiB+100 alan için 413 ve 30 saniye için 504 gerçeklendi. Her
  eylem çalışma hatası ya da başarısız Sonuçta geri alınan iç içe dosya
  savepoint'i taşır; IO desteği yoksa C021 ile fail-closed davranır. Gerçek TCP
  HEAD gövdesini bastırır. Geri alma, araya giren başka yazarın verisini
  ezmek yerine C021 verir. Hermetik ve gerçek CLI geri alma kanıtlarıyla 304
  test ve 134 katalog kodunda V1-P0-02
  kapandı. Çok-dosyalı süreç-çökmesi atomikliği bu sözün parçası değildir.
- **Production web güvenlik profili** (K-088, RFC-0017/spec-12): unsafe rota
  ilk satırda public/oturum/rol politikasını açıklar (T049/T050), zorunlu
  alanlar 400 ve otomatik synchronizer CSRF 403 kapısından geçer. Native
  runtime 256 bit OS CSPRNG belirteç üretir, oturum kimliğinin yalnız SHA-256
  özetini sunucuda tutar; girişte session+CSRF rotation, 30 dakika ömür,
  sunucu rolü ve logout revoke vardır. `dil parola-özeti` Argon2id PHC üretir.
  `--web-proxy https://host` loopback'teki HTTPS proxy için Host/proto/Origin
  doğrular; `__Host-` Secure+HttpOnly+SameSite=Lax çerez, HSTS/CSP ve CRLF/
  request-smuggling korkuluklarını uygular. 321 test ve 138 katalog koduyla
  V1-P0-03 kapandı.
- **Sürümlü morfoloji profili** (K-089, RFC-0018/spec-13): çözümleyici ve
  LSP'nin ayrı ek listeleri `zee-tr-1` tek kaynağında birleşti. İyelik ayrı
  soyut kimlikle iki katmanlı üretilir; düzenli kök×bütün tek/iki katman
  `üret→çöz` property'leri, ters ses değişimi ve A002 belirsizlik korpusu
  tablo snapshot'ını korur. Rename artık `fiyatıyla→elmasıyla` zincirini de
  giydirir. `proje.dil` profili sabitler (P011), `proje.kilit` v2 ana/paket
  profilini taşır; `dil morfoloji [kelime]` kararı görünür kılar. 333 test ve
  139 katalog koduyla V1-P1-02 kapandı.
- **Deterministik görev scheduler'ı** (K-090, RFC-0011/spec-14):
  `eşzamanlı olarak` görevleri artık kaynak sırasında bitiren sahte bir blok
  değildir; `hepsini bekle`, tek iş parçacıklı future scheduler'ında görevleri
  `bekle` noktalarında dönüşümlü ilerletir. Kaynak sırası bağlayıcıdır; 2 sn ve
  1 sn bekleyen görevler toplam 2 sn'de biter ve data race oluşmaz. İlk
  yönetilmemiş hata bekleyen kardeşleri sonraki yan etkileri başlamadan iptal
  eder; dış son tarih bütün görev ağacına yayılır. T051 her grubu aynı
  sözcüksel kapsamda tek join'e zorlar; T033 sonuç erişimini join sonrasına
  bırakır. Eylem transaction'ları savepoint sahipliği için atomik scheduler
  dilimidir. 343 test ve 140 katalog koduyla V1-P1-03 kapandı; çok çekirdekli
  paralellik v1 sözü değildir.
- **Yapılandırılmış Hata değeri** (K-091, RFC-0008/spec-15): `Sonuç<T>` hata
  tarafı artık kod, Türkçe mesaj, `Seçenek<Hata>` neden zinciri ve Metin
  sözlüğü verisi taşır. `hatanın kodu` `göre` ile eşlenir; neden `varsa` ile
  güvenle açılır; `hatanın json metni` bütün zinciri sabit anahtar sırasında
  verir. Eski `"..." hatasını döndür` `GENEL` koduyla aynı kullanıcı çıktısını
  korur; yerleşik denemeler kararlı kod üretir. S044/T052 olumsuzları,
  yeniden yayma ve geriye uyum regresyonlarıyla 350 test ve 142 katalog
  kodunda V1-P1-04 kapandı.
- **Keyfî hassasiyetli Ondalık** (K-092, RFC-0013/spec-16): ilk bootstrap'ın
  dokuz kesir hanesi ve i64/i128 katsayı sınırı kaldırıldı. Tek Ondalık türü
  keyfî uzunlukta katsayı+ölçek taşır; toplama/çıkarma/çarpma ve sonlu onluk
  bölüm tamdır. Sonsuz bölüm platformdan bağımsız 34 anlamlı haneye,
  yarımlar sıfırdan uzağa yuvarlanır. Uzun sabit ve negatif metin dönüşümü,
  büyük/küçük değer, exact `1/8`, `10/3` bağlamı, karşılaştırma, JSON/para ve
  i64 daraltma taşmasıyla 356 test yeşildir. S032 emekliye ayrıldı; katalog
  141 etkin + 1 ayrılmış kod taşır. Release ölçümünde 20 bin exact Ondalık
  toplaması 6,8 ms'dir; `BigInt` kapasite maliyeti ölçüm arşivinde görünürdür.
  V1-P1-01 kapandı.
- **Değer semantiği ve gezme imleci** (K-093, RFC-0019/spec-17): bütün
  kullanıcı değerleri derin kopyadır; gizli paylaşılan alias yoktur. Liste
  gezmesi öğeyi kopyalayıp turun sonunda aynı sıraya geri yazan değer-sonuç
  imlecidir; alan yazma ve yeniden bağlama aynı kalıcı sonucu verir. Gezilen
  kaynağı ekleme/silme/yeniden bağlama, sözlüğe yazma ve aynı kaynağı iç içe
  gezme T053 ile derlemede durur; doğal `sayılara` yüzeyi artık A002 yerine
  doğrudan bu tanıyı alır. Derin kopya, beş olumsuz ve 1–24 uzunluk property
  korpusuyla 360 test ve 142 etkin + 1 ayrılmış kod yeşildir. V1-P1-05'in
  makine tarafı tamamdır; gerçek çocuk/profesyonel usability sonucu gelmeden
  kapı dürüstçe açık kalır.
- **Tekrar üretilebilir paket yayını** (K-094, ADR-006/RFC-0020/spec-18):
  `dil anahtar üret` var olanı ezmeyen Unix 0600 Ed25519 yayıncı anahtarı ve
  yeni proje iskeletlerinde `*.zee-anahtar` Git dışlama koruması;
  `dil paketle` platform metadata'sı taşımayan sıralı `.zep`, SPDX 3.0.1
  JSON-LD SBOM, in-toto/SLSA v1 provenance ve üçünü ad+boyut+SHA-256 ile
  imzalayan `zee-yayin-v1` üretir. `SOURCE_DATE_EPOCH` ile dört dosya byte-byte
  yinelenir. İmza/paket/SBOM/provenance oynama, path traversal/fazladan byte,
  sembolik bağ ve yerel yol bağımlılığı fail-closed testlidir. P012 eklendi.
  Bu yayın öz-imzası tek başına registry güveni değildir; TUF tarzı eşik kök,
  targets/snapshot/timestamp, doğrulanmış cache, yanked ve duyuru tamamlanana
  kadar V1-P1-07 açık kalır. Doküman tazelik testiyle toplam 370 test yeşildir.
- **Registry metadata güven zinciri** (K-095, ADR-006/RFC-0020/spec-19): ağ
  dışı SHA-256 ile sabitlenen root, rol başına Ed25519 eşik, hem eski hem yeni
  root eşiğini isteyen ardışık rotasyon ve kanonik kapalı JSON zarfı çalışır.
  Tek güncelleme saatiyle timestamp→snapshot→targets sürüm/boyut/SHA-256
  bağları; kalıcı sürüm+aynı-sürüm-özeti rollback/equivocation koruması ve
  zincir tamamlanmadan ya da daha yeni sonuçtan sonra bayat sonuçla durumu
  uygulamayan transaction sınırı kuruldu. Exact
  hedef yayıncı yetkisi dört yayın dosyasını bağlar; yanked ve etkin kritik
  duyuru varsayılan reddedilir. Eşik/rotasyon, rollback/expiry, aynı sürümlü
  farklı içerik, mix-and-match, fast-forward zehirleme, kanonik/limit,
  bozuk kalıcı durum, RFC3339 takvim, yanlış yayıncı ve politika olumsuzlarıyla
  toplam 378 test;
  P013/P014 ile 144 etkin + 1 ayrılmış tanı yeşildir. Taşıma, kalıcı durum
  dosyası, doğrulanmış cache/offline ve CLI bitmeden V1-P1-07 açık kalır.
- **K-016 çağrı karar deneyi** (K-096, RFC-0006/spec-02): çalışan A yüzeyi
  nihai V1 kararı sayılmadan önce serbest üretim, kör A/B/C kartları, üç sıra
  grubuna dengeli dağıtım, çocuk/profesyonel alt grup eşikleri ve yedi teknik
  bağlam önden bağlandı. B güçlü çıkarsa ikinci çağrı sözdizimi eklenmeyecek;
  B-003/RFC-0021 expression grammar turuna dönülecek. Anonim katılımcı/özet
  şablonları ve kişisel veri koruması hazırlandı. Parser'ın eski tanım-sırası
  yorumu güncel ön-tarama/karşılıklı özyineleme gerçeğine düzeltildi. Dil
  yüzeyi değişmedi; 378 test tabanı korunuyor ve V1-P0-07 gerçek 10 çocuk +
  5 profesyonel sonucu gelene kadar açık kalıyor.
- **Katmanlı ifade grameri** (K-097, RFC-0021/spec-20/ADR-002): mevcut yüzey
  primary → erişim/postfix → çağrı → aritmetik → birleştirme → karşılaştırma
  → boolean güç sırasına ve tam bölge tüketimine bağlandı. `ile`/`ve` gibi
  bağlama göre ayrılan kelimelerin sahipliği, en uzun işlem adı, karışık
  `ve/veya` için S030, işlem-adı kuyruğunun postfix'i gölgelememesi, tam
  sıfır-argüman çağrısının korunması ve tanımsız katman birleşimi için
  fail-closed davranış normatifleşti. Böylece görünür `sayısı` işleminin
  `metnin sayısı` ifadesini yanlış S019'a çevirdiği çakışma, geçerli tam
  sıfır-argüman çağrılarını bozmadan kapandı. Yeni ifade özelliği katman,
  çakışma matrisi, AST/lowering,
  formatter ve olumlu/olumsuz conformance kanıtı olmadan parser'a dal
  ekleyemez. Sekiz bağımsız testle toplam 386 test yeşil; kullanıcı yüzeyi
  değişmeden V1-P0-08 kapandı. Fiziksel parser parçalama B-005/K-099'da
  tamamlandı.
- **Core AST intrinsic/yetkinlik sınırı** (K-098, ADR-011): HTTP, sensör,
  CSRF ve parola için dört alan-özel AST varyantı kaldırıldı. Mevcut Türkçe
  cümleler parser'da ad alanlı kararlı kimlik ve sıralı argüman taşıyan tek
  `Intrinsic` düğümüne indirilir. Tür imzası, gereken ağ/donanım/web oturumu/
  kriptografi yetkinliği ve statik etki merkezi kayıttadır; checker, etki
  çözümleyici ve runtime aynı kaydı tüketir. `kapı kapalıysa` ikinci intrinsic
  yerine genel olumsuzlamayı kullanır. Kimlik tekilliği, dört lowering ve iki
  tür olumsuzuyla yedi yeni test; mevcut HTTP/sensör/web/parola regresyonları
  dahil toplam 393 test yeşildir. Kaynak semantiği değişmeden B-004 ve
  V1-P0-09 kapandı; fiziksel handler ayrımı B-005/K-099'da tamamlandı, izin
  politikası B-023'tür.
- **Derleyici fiziksel faz sınırları** (K-099, ADR-012): 2709 satırlık parser
  cümle/ifade, 3067 satırlık checker cümle/ifade/çağrı ve 3181 satırlık
  runtime cümle/ifade handler modüllerine ayrıldı. Kökler 1160/963/1965 satıra
  indi; alt modüller yalnız `pub(super)` görünür ve public Rust API değişmedi.
  Üç kaynak-mimari testi büyük handler'ların köke dönmesini ve ilan edilmiş
  faz bütçelerinin aşılmasını engeller. Hata kataloğu taraması yeni alt
  modülleri özyinelemeli kapsar. 393 davranış testi aynen korunup toplam 396
  test yeşil kaldı; kaynak semantiği değişmeden B-005/V1-P0-10 kapandı.
- **Checker semantik katmanları** (K-100, ADR-013): 963 satırlık checker kökü
  143 satırlık geçiş orkestrasyonuna indirildi. Türler, bağlam, sembol, akış,
  çağrı, public sözleşme, etki/yetkinlik ve dönüş/control-flow ayrı tek-sahipli
  modüllere taşındı; eski `eylem.rs` aynı davranışla `cozumleyici/etki.rs`
  oldu. Public `Tur`, `VeriTuru`, `SozlukDegerTuru` ve `ad_cozumle` API'si
  yeniden dışa aktarımla korundu. İki yeni sahiplik/API testiyle mimari test
  sayısı beşe, toplam test sayısı 398'e çıktı. Kaynak semantiği değişmeden
  B-006/V1-P0-11 kapandı; semantic ID ardılı B-010/K-101 ile tamamlandı.
- **Semantic kimlik modeli** (K-101, ADR-014): `YapiId`, `IslemId` ve
  `SymbolId` newtype'ları eklendi. `Tur::Yapi` çıplak `usize` yerine kimlik
  taşır; fiziksel yapı konumu ayrı dizinden çözülür. İşlem imzaları ve
  özyineleme kaydı `IslemId`, yerel sembol tablosu ad→(`SymbolId`, tür)
  kullanır. Checker çözülmüş değişken, yapı oluşturma ve işlem çağrısı AST
  düğümlerini ID ile bağlar; kaynak adını tanı/runtime geçişi için korur. Üç
  davranış ve bir mimari testle toplam 402 test yeşildir; kaynak semantiği
  değişmeden B-010/V1-P0-12 kapandı; faz modeli B-018/K-102 ile tamamlandı.
- **Tür güvenli derleyici fazları** (K-102, ADR-015): `KaynakMetni`,
  `TokenAkisi`, `AyristirilmisAst`, crate-içi `BaglanmamisProgram` ve yalnız
  başarılı checker'ın üretebildiği `BaglanmisProgram` eklendi. Standart
  denetle/çalıştır/dene hatları fazlı API ve `calistir_baglanmis[_io]`
  girişlerini kullanır; eski raw `Program` API'si yalnız uyumluluk adaptörüdür.
  Parsed AST'nin doğrudan yürütülemeyeceği compile-fail dahil dört yeni testle
  toplam 406 test yeşildir. Kaynak semantiği değişmeden B-018/V1-P0-13
  kapandı; ayrı typed HIR B-019 olarak açık kaldı.
- **Typed HIR çekirdeği** (K-103, ADR-016): checker'ın her denetlenmiş ifade
  için ürettiği tür ve `SymbolId`/`IslemId`/`YapiId` bağı ayrı
  `HirIfadeBilgisi` kaydına taşındı; `HirDugumId` program içi semantic düğüm
  kimliğidir. `BaglanmisProgram` artık zorunlu `HirProgram` sahibidir; AST
  tanı ve v0 uyumluluğu için salt-okunur kalır. İki davranış ve bir mimari
  testle toplam 409 test yeşildir. K-104 ardılı runtime geçişini tamamladı.
- **HIR-bağlı standart runtime** (K-104, ADR-016): `calistir_baglanmis[_io]`
  ve kaynak test hattı `CalistirmaProgrami::Hir` kullanır. Değişken erişimi/
  güncellemesi, işlem çağrısı ve yapı oluşturma yalnız `HirBagi` ile çözülür;
  HIR kolu kaynak adına geri düşmez. Eşzamanlı görev özgün ifade düğümünü
  ödünç alır. Kaynak adlarını bilerek bozan iki regresyon ve standard-hat
  mimari testi ve dönüşsüz çağrı HIR kanıtıyla toplam 413 test yeşildir;
  B-019/V1-P0-14 kapandı.

- **Proje modeli** (K-076): geçerli zee sözdizimli `proje.dil` (`proje`,
  `sürüm`, `giriş`); `dil çalıştır/denetle/dene <klasör>`; `dil yeni`
  bildirimi hazır üretir. P001–P004 Türkçe proje tanıları. Doğrudan dosya
  kullanımı geriye uyumludur. Göreli dosya IO'su giriş klasörüne sabitlendi;
  `--güvenli` bayrağının program argümanına sızması da kapandı.
- **Yerel paketler ve kilit** (K-078): `yerel_bağımlılıklar` başka zee
  projelerini bağlar; `X paketini kullan` yalnız doğrudan bağımlılığı alır.
  Kaynak kökeni paket içi birimlerde korunur. `dil kilitle` geçişli grafiği
  göreli yol, sürüm, kenar ve SHA-256 `.dil` özetiyle deterministik
  `proje.kilit`e sabitler; eksik/bayat kilit P008'dir. Döngü, yinelenen ad,
  proje dışına çıkan sembolik bağ ve geçişli bağımlılığa gizli erişim testli.
- **Güvenli paket ekleme** (K-079): `dil ekle <yerel-yol> [proje]` yolu
  proje köküne göre taşınabilir biçime çevirir; yorumları koruyup listeyi
  sıralar. Yeni bağımlılık grafiği yazmadan önce bütünüyle doğrulanır. Başarılı
  işlem bildirimi ve kilidi birlikte günceller; yinelenen gerçek kök ikinci kez
  eklenmez, öz-bağımlılık P007'dir.
- **Paket grafiği ve güvenli kaldırma** (K-080): `dil paketler [proje]`
  doğrudan/geçişli paketleri sürüm, göreli yol ve kilit özetiyle listeler.
  `dil çıkar <paket> [proje]` yalnız doğrudan paketi kaldırır; kaynakta kalan
  `X paketini kullan` P010 ile işlemi değişiklik yapmadan durdurur. Başarıda
  bildirim/kilit birlikte güncellenir ve yazma hatasında geri alınır.
- **Proje çapında biçimleme** (K-077): `dil biçimle <klasör>` bütün `.dil`
  kaynaklarını deterministik yol sırasında biçimler; önce tamamını doğrular,
  tek hata varsa hiçbir dosyaya dokunmaz.

- **Gezmede yazma yansır** (K-074): `her kutu için / kutunun adedi ...`
  listeye geri yazılır — kopya tuzağı kapandı (TANIMLI).
- **Türk para yazımı** (K-075): `binlikli kuruşlusu` → "1.234.567,89".
- **Çerez silme** (K-073): `"oturum" çerezini sil` — Max-Age=0; panel
  çıkışı gerçek silmede.

## v0.6.0 — 1 Eylül 2026

**Tema: tip sistemi olgunlaştı, dil ayrıntıda medenileşti.**

- **Aralık iki yönde** (K-068): `5 ten 1 e kadar` geri sayar (sessiz
  boş dönüş tuzağı kapandı). roket.dil projesi.
- **Çıkış kodu** (K-069, K-024 kapanışı): `programı 1 ile bitir` —
  süreç kodu kabuğa gider (0–255, C020); CLI otomasyon kapısı.
- **Yeniden adlandırma** (K-072): dillsp + VS Code F2 — morfoloji
  farkındalıklı: ekler yeni köke Türkçe uyumla giydirilir (sayacı→puanı,
  renk→rengi). Metin/yorum dokunulmaz; tek katman ek kapsamı.
- **Doğrulamalar** (K-071): `içermeli`, genel `olmamalı`, çıplak `boş`
  atomu — test kültürü zenginleşti. Ölçüm: döngü izlemesi kapandı
  (v0.6.0 satırı; v0.2 tabanının altında).
- **Kitaplık** (K-070): `toplamını/ortalamasını hesapla` artık DAİMA
  Ondalık döner (deneysel-kırıcı; TamSayı listeleri genişlemeyle girer).
  Yeni gömülü birim: `sozluk_araclari` (`en çok geçeni bul`).

- **Para biçimi** (K-065): `tutarın kuruşlusu` — daima iki hane.
- **Sayısal genişleme çağrıda + imza terfisi** (K-067): Liste<TamSayı> →
  Liste<Ondalık> parametre; dar imza geniş argümanla terfi eder. Sözlük
  değerleri Ondalık olabilir. Kitaplığa medyan girdi.
- **Evrensel metin hali** (K-066): `değerin metni`; `dil belge` artık
  işlem açıklamalarını basar; playground'da Envanter vitrini.
- JSON okuma hoşgörüsü (K-063), alan-özellik gölgelemesi (K-064),
  envanter projesi; işlemden liste dönüşü testle sabitlendi; biçim
  hijyeni projeler+kitaplığa genişledi.

## v0.5.0 — 1 Eylül 2026

**Tema: kayıtlar ve koleksiyonlar — dil, veri işlerinin dili oldu.**

- **JSON okuma hoşgörüsü** (K-063): sayı/bool/null değerler Metin gelir
  (nokta → virgül); yalnız iç içe yapı C016.
- **Alan, özelliği gölgeler** (K-064): `ürünün adedi` alan okur —
  denetleyici yeniden yazımı. Envanter projesi eklendi (stok defteri).

- **Sıralama** (K-056): `sıralanmışı` — Metinler Türk alfabesi sırasıyla
  (ç, ğ, ı, i, ö, ş, ü yerli yerinde; TANIMLI); `tersi`.
- **Tarih farkı** (K-057): `X ile Y arasındaki günler` (işaretli).
- **Liste üyeliği + CSV yazma** (K-058): `sayılarda 5 varsa`;
  `tablonun csv metni`.
- **Morfoloji: iki katmanlı ek** (K-061): `kitabın fiyatıyla artır`
  (iyelik+araç zinciri) çözülür.
- **CSV hücreleri Metin** (K-062): isimli sütunlar birinci sınıf;
  sayı `değerin sayısı` ile bilinçli çevrilir. Golden 19 revize.
- **Yapı listeleri** (K-060): kayıt tabloları — `boş liste`ye yapı ekle,
  gez, alan oku; `json metni` nesne listesi üretir. (T027 belgesi düzeltildi:
  Ondalık alan zaten vardı.)
- **Silme** (K-059): `sayılardan 5 i sil` / `defterden "elma" yı sil` —
  yoksa sessiz (idempotent). Girişli panel demosuna silme akışı eklendi
  (dosya-satırı silme saf zee: süz + birleştir + yaz).

## v0.4.0 — 1 Eylül 2026

**Tema: dil, kurucunun gerçek projelerine hazırlanıyor — web ve metin.**

- **Metin dalgası** (K-053): `parçaları`, `birleşmişi`, `yerine ...
  değişmişi`, `kırpılmışı`, `harfleri`, `ile başlıyorsa/bitiyorsa` —
  harf düzeyine ilk iniş. Gömülü `metin_araclari` birimi (saf zee).
- **JSON yazma** (K-054): `değerin json metni` — sıra-korumalı,
  deterministik serileştirme.
- **Deneysel web uygulaması katmanı** (K-050..K-052, K-055): HTML servis, örtük
  `istek` sözlüğü (sorgu + POST form), `adresine yönlendir` (303),
  `html güvenlisi` (XSS), örtük `çerezler` + `çerezine yaz` (oturum),
  önekli rotalar. Kanıtlar: mini-site, panel-not-defteri, girisli-panel
  (parola+oturum mantığı SAF ZEE) — üçü de localhost'ta canlı + hermetik.
- **Sınır (değişmedi):** parola düz metin, HTTPS yok — internete açık
  üretim Faz 5 güvenlik dalgasını bekler.

### v0.3.0 sonrası küçükler

- **Ünsüz ikizleşmesi morfolojisi** (K-049): `üssü`, `affı`, `zammı`,
  `reddi` (sertleşmeyle) çözülür; matematik biriminin doğal `üssü al`
  parametresi geri geldi.
- **`dil belge <birim>`**: işlem başlıkları + test sayısı (RFC-0014 §8.2).
- Playground'a "Kitaplık (obeb)" örneği eklendi.
- **Deneysel zee web sitesi** (K-050): sunucu HTML'i text/html olarak servis
  eder; projeler/mini-site.dil — rotalar + stil + gömülü kitaplık hesabı.
- **Web uygulaması dalgası** (K-051): örtük `istek` sözlüğü (sorgu + POST
  form, UTF-8 yüzde çözümü), `adresine yönlendir` (303, S041),
  `html güvenlisi` (XSS kaçışlaması). Kanıt: panel-not-defteri projesi —
  formlu, dosyada saklayan, gizli yollu eğitim paneli (localhost'ta canlı +
  hermetik tam-döngü testi). Sınır: HTTPS/hash Faz 5'te.
- **Oturum kapısı** (K-052): örtük `çerezler` sözlüğü + `çerezine yaz`
  (Set-Cookie, HttpOnly). Oturum mantığı saf zee'de: girisli-panel projesi
  (parola → dosyada oturum kimliği → korumalı rotalar; sahte çerez reddi
  testli). Parola düz metin — hash Faz 5.

## v0.3.0 — 1 Eylül 2026

**Tema: dil günlük Türkçeye yaklaştı; standart kitaplığın tohumu atıldı.**

- **Gömülü standart kitaplık** (RFC-0014 taslak + çalışan prototip, K-048):
  `matematik` (mutlak, üs, tam karekök, obeb-Öklit, okek) ve
  `liste_araclari` (toplam, uçlar, Ondalık ortalama) — zee'yle yazıldı,
  ikiliye gömülü, playground dahil her yerde kurulumsuz; kendi test
  blokları CI'da. Çözüm: yerel klasör → gömülü.
- **Öğretmen rehberi** (docs/ogretmen-rehberi.md): internetsiz sınıf
  kurulumu, 10 oturumluk ders sırası, hata kültürü.
- Proje kitaplığı 10 projeye çıktı (kelime sayacı, gün sayar).

- **Mantıksal ad tek başına koşul** (K-044): `hazır ise` / `hazır değilse`.
- **Boş koleksiyon tür çıkarımı** (K-045): `boş liste`/`boş sözlük` ilk
  eklemeyle türlenir; metin listeleri ve metin sözlükleri artık kurulabilir.
  KIRICI: hiç eklenmemiş boş listenin `ilki` artık derleme hatası (T014;
  eskiden çalışma anında C007).
- **Çocuk modu** (K-047): `dil çalıştır --güvenli` — ağ/sunucu kapalı,
  dosyalar çalışma klasörüyle sınırlı; hata `dene` ile yönetilebilir.
- **Kalan işlemi** (K-046): `17 nin 5 e bölümünden kalanı` — okul kuralı,
  kalan hiç negatif olmaz.
- **Zamir n'si morfolojisi** (K-041): `bilgisayarın_zarından` çözülür.
- **VS Code**: elle yazılmış LSP istemcisi (npm'siz) — canlı tanılar,
  hover, tanıma git, tamamlama editörde. ADR-008 (self-hosting aşamaları) kabul.
- dillsp: hover (Türkçe açıklama) + tanıma git. Performans arşivi
  (docs/olcumler.md) ve `olcum` koşucusu. Proje kitaplığı 8 projeye çıktı.

## v0.2.0 — 31 Ağustos 2026

**Tema: dil derinleşti, derleyici tarayıcıya taşındı.**

### Yeni dil yüzeyi

- **Özyineleme** (K-035): işlem kendini ve karşılıklı olarak birbirini
  çağırabilir; tanım sırası serbest (adlar ön-taranır). Kural: temel durum
  özyinelemeli çağrıdan ÖNCE gelir (T035) — tür çıkarımı "o ana dek görülen
  dönüşler" üzerinden yapılır, dil iyi alışkanlığı kendisi öğretir.
  Çalışma zamanı derinlik sınırı 500 (C019, K-040: her platformda aynı).
- **Blok kapsamı** (K-034, RFC-0004 kapanışı): gövdede doğan ad gövdeyle
  ölür (döngü değişkeni dahil); dıştaki ada atama kalıcıdır; gölgeleme
  yapısal olarak yoktur.
- **Akış-duyarlı daraltma** (K-037, T036): `varsa` / `başarılıysa` /
  `başarısızsa` dalları ve `değilse` tersinmeleri içinde `değeri` / `hatası`
  erişimi statik güvenli; dal dışında korumasız erişim artık DERLEME
  hatasıdır. "Boş değeri açmak" hatası çalışma zamanından derleme zamanına
  taşındı; C008/C009 iç savunmaya dönüştü.
- **Metin kaçışları** (K-036): `\"`, `\\`, `\n`; bilinmeyen kaçış S040.
- **Negatif sayı sabitleri** (K-036): `-3`, `-3,14` — işaret rakama bitişik.
- **Çok-tokenli çağrı argümanları** (K-038): `tabanın tam kısmı için yuvarla`
  — her `ve`/`ile` dilimi tam bir ifade bölgesi.

### Playground (K-039)

- `playground/zee-playground.html`: derleyicinin tamamı WebAssembly olarak
  tek HTML dosyasında. Çift tıkla açılır; kurulum ve internet gerekmez.
- 7 hazır örnek, girdiler alanı, görünür tohum — **aynı tohum + aynı girdi
  = her zaman aynı çıktı** (determinizm tarayıcıda da geçerli).
- ADR-001 korunur: wasm-bindgen yok; elle yazılmış C-ABI köprüsü
  (`compiler/src/wasm_api.rs`), çekirdek doğal derlemede de testli.
- Yeniden üretim: `playground/olustur.sh`.

### Tanılar

- Yeni: T035 (temel durum önce), T036 (korumasız değer/hata erişimi),
  S040 (bilinmeyen kaçış), C019 (çağrı derinliği).
- Yeniden konumlanan: C008/C009 artık iç savunma (derlemede T036 yakalar).

### Mühendislik

- CI üç platformda yeşil; wasm hedefi de derlenir (ubuntu).
- CLI, işi 32 MB yığınlı iş parçacığında koşar (Windows ana iş parçacığı
  1 MB'dır); sınıra dokunan test kendi yığınını getirir (K-040).
- Toplam 154 hermetik test (`cargo test --no-fail-fast`), clippy temiz,
  uyarısız derleme.

### Kırıcı değişiklikler

- Korumasız `değeri`/`hatası` erişimi olan programlar artık derlenmez (T036).
  Düzeltme tanının önerisindedir: erişimi `varsa`/`başarılıysa`/`başarısızsa`
  dalına al. 32 golden programın hiçbiri etkilenmedi.
- Gövde içinde tanımlanan ada gövde dışından erişim artık A001 (K-034).
  Korpusta buna dayanan program yoktu.

## v0.1.0 — 31 Ağustos 2026

İlk sürüm — proje doğdu. 32/32 golden korpus; v0.1 kabul kriterlerinin
8/8'i (master plan bölüm 33). Tam yüzey listesi: [README](../README.md).
Etiketler: `baslangic` (ilk taahhüt), `dogum` (merhaba.dil çalıştı),
`v0.1.0` (kabul kriterleri kapandı).
