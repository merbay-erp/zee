# Production katman ve bağımlılık yönü

K-149/ADR-046, fiziksel satır bütçesinin yanına gerçek modül bağımlılığı
kapısını koyar. Amaç yalnız küçük dosya değildir: parser semantic/runtime
ayrıntısını, checker runtime/adaptör ayrıntısını, runtime parser/checker/LSP
ayrıntısını ve LSP compiler iç geçişlerini doğrudan bilmemelidir.

## Katman yönü

K-226 `surum` temel sahibi yalnız deterministik build sabitleri/sunumudur;
CLI ve LSP temele bağımlanır. Host Git sorgusu build betiğindedir, çalışan
dil/lexer/checker katmanına girmez. ADR-093 ve exact fixture bu eklenen sahibi
ve iki kenarı kaydeder; internal module desteklenen api::v1 export değildir.

| Katman | Doğrudan bağımlanabileceği katmanlar |
|---|---|
| `temel` | `temel` |
| `model` | `temel`, `model` |
| `altyapi` | `temel`, `model`, `altyapi` |
| `sozdizimi` | `temel`, `model`, `altyapi`, `sozdizimi` |
| `semantik` | önceki dört katman + `semantik` |
| `proje` | temel→semantik + `proje` |
| `web` | temel/model/altyapı + proje + `web` |
| `runtime` | bütün ürün alt katmanları + `runtime` |
| `adapter` | ürün katmanlarının tamamı ve adaptörler |
| `muhendislik` | bütün katmanlar; ürün kodu buna bağımlanamaz |

Bu yön tek tek Rust dosyalarının sırasını değil sorumluluk seviyesini anlatır.
Aynı `proje` kümesindeki paket/registry/yayın koreografisi bu kararla
çevrimsiz ilan edilmez; mevcut doğrudan kenarlar ayrıca exact tabanda
görünürdür. Katmanlar arası ters kenar ise tabana yazılsa bile reddedilir.

## Makine-okunur sahiplik

[`katman-mimarisi-v1.tsv`](../compiler/tests/fixtures/katman-mimarisi-v1.tsv)
37 production sahibinin katmanını, bütün doğrudan iç bağımlılıklarını ve tek
cümlelik sorumluluğunu taşır. Kapsam şunların tamamıdır:

- `lib.rs`, bütün kök kitaplık modülleri ve alt modülleri;
- `main.rs` ile `cli/` tek `dil_cli` adaptörü;
- `dillsp` ürün adaptörü;
- depo sayısı, faz matrisi, işlev eğilimi ve performans gözlemi mühendislik
  ikilileri.

Yeni `.rs` dosyası mevcut üst sahibine girer. Yeni kök modül veya ikili yeni
sahip satırı ister. Test-only `testler.rs` dosyaları ile `#[cfg(test)]`
öğeleri production kenarı sayılmaz; hedefe özgü production kodlarının birleşik
kenarları ise bütün Tier-1 platformlar için korunur.

`katman_mimarisi_testi` Rust yorumlarını, iç içe blok yorumlarını, normal/byte/
raw metinleri ve karakter sabitlerini token akışından çıkarır; yaşam sürelerini
kodla karıştırmaz. `crate::`, `dil::`, toplu `use` ve doğrudan bilinen modül
yollarını toplar. `crate`/`dil` kökünü takma adla gizlemek reddir. Kök public
derleme, proje-yükleme ve runtime sembolleri kavramsal sahiplerine eşlenir;
bilinmeyen kök sembolü `lib` cephesine gider ve aşağı katmanda ters kenar olur.

Kapı üç ayrı drift'i durdurur:

1. kaynak ağacındaki sahiplerle fixture birebir değilse yeni/kayıp modül;
2. bulunan doğrudan kenarlar exact, sıralı bağımlılık tabanından farklıysa
   eklenen veya kaldırılan bağımlılık;
3. exact taban güncellense bile kaynak→hedef katmanı izinli yönde değilse
   ters katman bağımlılığı.

## Çevrim kapısı

K-150/ADR-047 exact graph üzerinde SCC hesabını ayrıca zorunlu yapar. Katman
yönünün izin vermesi çevrimi kabul ettirmez. Yeni sahipler-arası SCC reddedilir;
artık graph'ta bulunmayan allowlist kaydı da bayat izin olarak kapıyı kırar.

İlk incelemedeki `tani ↔ kaynak_sinirlari` çevrimi bağımlılıksız
`tani_politikasi` bütçesiyle; `cozumleyici ↔ hir` çevrimi ortak private
`semantic_model` tür sahibiyle kırıldı. `cozumleyici::Tur` public yolu yeniden
dışa aktarımla uyumludur, HIR artık checker sahibini tüketmez.

K-160/ADR-059 son kalan paket çevrimini gerçek sahiplik ayrımıyla kaldırdı:
`paket_modeli` davranışsız veri sözleşmesi, `paket` çözüm kararı, `registry`
taşıma/protokol, `artefakt_dogrulama` güven ve kurulum, `yayin` üst seviye
orkestrasyon sahibidir. Model production davranışına bağımlı değildir;
verification paket/registry/yayın sahibini çağırmaz; registry artefaktı
verification'a devreder; resolver taşıma ve imza ayrıntısını bilmez. Eski
`tedarik` cephesi K-167/ADR-064 ile kaldırıldı (DEP-004); kurulum katmanı
`artefakt_dogrulama/kurulum.rs` altındadır.

Güncel production graph'ta SCC sayısı ve
[`izinli-katman-cevrimleri-v1.tsv`](../compiler/tests/fixtures/izinli-katman-cevrimleri-v1.tsv)
izin sayısı sıfırdır. Yeni çevrim veya yeni izin CI'ı durdurur; C001
silinmiştir. Gelecekte zorunlu bir istisna doğarsa tanı kodlarıyla karışmayan
`SCC-NNN` kimliği kullanılır; tarihsel C001 kayıtları yeniden adlandırılmaz.

## K-149'da temizlenen iki ters kenar

- FIPS 180-4 `sha256_hex`, paket modülünden temel `guvenlik` sahibine taşındı.
  Böylece morfoloji profil parmak izi artık `sozdizimi → proje/paket` ters
  bağımlılığı kurmaz; paket, registry, artefakt doğrulama, web ve IO izi aynı temel ilkeyi
  tüketir.
- Gregoryen gün↔tarih algoritması runtime'dan yeni temel `zaman` sahibine
  taşındı. Tedarik zinciri RFC 3339 zamanı için runtime'a bağımlanmaz;
  yorumlayıcı ve CLI aynı yardımcıyı kullanır. Eski
  `yorumlayici::gunlerden_tarih_utc` yolu geriye uyumlu yeniden dışa aktarılır.

Güncel graph ayrıca şu sınırları makinece gösterir: `ayristirici` semantic/HIR/
runtime'a; `cozumleyici` runtime/LSP'ye; `yorumlayici` lexer/parser/checker/LSP/
WASM'a; `lsp` lexer/parser/checker/runtime'a doğrudan bağımlı değildir.

## Değişiklik protokolü

```bash
cd compiler
cargo test --locked --test katman_mimarisi_testi
cargo test --locked --test bagimlilik_cevrimi_testi
cargo run --locked --bin faz_test_matrisi -- --denetle \
  --rapor target/faz-test-matrisi.md
```

Bir bağımlılık farkında fixture körlemesine güncellenmez. Önce sorumluluğun
mevcut sahibinde kalıp kalmadığı, daha aşağı bir ortak ilkeye taşınıp
taşınamayacağı ve katman yönü değerlendirilir. Kabul edilen yeni kenar aynı
committe ADR/rehber etkisi ve ilgili davranış testleriyle girer. Kaldırılan
kenar da tabandan silinir; artık kullanılmayan bağımlılık gelecekteki kod için
sessiz izin olarak bırakılmaz.

Bu kapı Rust tip denetiminin veya semantic testlerin yerine geçmez. Makro ile
üretilmiş dinamik adları tam bir Rust semantic graph'ı olarak iddia etmez;
production kaynakta açık modül yollarının değişim kapısıdır. Tam faz matrisi,
Clippy, rustdoc, WASM ve ilgili RFC/spec kanıtları ayrıca zorunludur.

K-206 sonuç dönüşümü `postgresql/sonuc.rs` içinde aynı `postgresql`
sahibindedir. Transport/havuz katmanı saf modele yükselmez; yeni
üst sahip/bağımlılık kenarı/SCC yoktur. Fixture değişikliği gerekmedi.

### ürün E özelliklerinin ana kaynak hattı — K-213

[ADR-086](../adr/086-site-hatlarinin-birlestirilmesi.md) mevcut ürün E
parola üretimi/TOTP/SMTP, rota birimi, görünüm, görsel ve gizli değer
yüzeylerini K-208/K-209 bcrypt/IP ile aynı hatta toplar. Ürün kabulü
dosya kontrolüyle değil bütün proje `dil dene .` ile alınır. Ham ürün E
şablonu normal katalog üretimini ister. Exact source/CI kapanışı beklenir;
canlı dağıtım ve otomatik parola göçü bu işte yapılmaz.

K-215 etki incelemesi: SBOM path yama doğrulaması aynı sürüm mühendisliği
sahibindedir; yeni production sahibi veya ters temel→adaptör kenarı eklenmez.
Exact katman ve çevrim fixture'ları kapıyla doğrulanır.

K-218 / ADR-089: explicit `--web-oran-deposu bellek` seçimi yalnız oran
durumunu süreç başına sınırlı Arc/Mutex deposuna taşır. Varsayılan dosya
backend'i ve kalıcı oturum/revoke paylaşımı sürer; ayrı worker/restart için
ortak proxy kotası gerekir. Soak sürücüsü her dönemde 32768 anahtar
doldurup süre dolumuyla temizler; yeni tampon derleyici RSS'ine dahildir.
Sabit benchmark korpusu ve eski performans tarihçesi değiştirilmedi.
Katman sahibi web_guvenligi'dir; bin_soak bu sahibe tek ileri kenar ekler.

K-219 / ADR-090 etki incelemesi: bellek oran ayarı ve tahliye indeksi mevcut web_guvenligi sahibindedir; bin_dil ve bin_soak aynı ileri kenarları kullanır. Yeni kök modül, sahipler-arası kenar veya SCC eklenmez; fixture değişmez.

K-220 / ADR-091: yeni alt dosyalar parser, yorumlayici ve ag_istemcisi sahiplerinde kalır; proje metadata kendi sahibindedir. Mevcut ileri kenarlar korunur, yeni kök sahip veya ters katman geçişi yoktur. Fixture/SCC/public API aynı kaynak kapısında incelenir.

K-232 MIME ve readonly safe-open alt modülleri mevcut `eposta` altyapı
sahibindedir. Merkezi e-posta limitleri temel `kaynak_sinirlari` sahibindedir;
bilinçli tek yeni exact kenar `eposta → kaynak_sinirlari`, adaptör aşağı
yöndedir. Yeni üst sahip/ters kenar/owner SCC veya facade export yoktur.
Temel bütçe alt modülü `posta` adını taşır; `eposta` üst adaptör sahibiyle
module adı çakışması oluşturulmaz. Kapı/fixture ters temel kenarına izin
vermek için değiştirilmedi; public EpostaSinirlari ve bütçe değerleri aynıdır.

K-240: GET C018/deadline gövdesi mevcut HTTP handler’dadır; replay trait
uygulaması `yorumlayici/io_izi/iz_oynatici.rs` alt dosyasındadır. Aynı
yorumlayıcı owner, aynı değer/state ve temel→adaptör yönü korunur.

K-240 SMTP testleri ayrı native test dosyasına, DB/posta profil erişicileri
mevcut domain profil dosyalarına taşınır. Test isimleri/faz sahipliği, profil
değerleri, public yüzey ve üretim gövdeleri korunur; spec kanıtı yeni exact
test dosyasına taşınmıştır.

K-253: yeni private CLI `web_kapanis` adapter sahibi processwide SIGTERM
disposition ve listener poll yaşam döngüsünü taşır. `dil_cli -> web_kapanis`
exact kenarı bilinçli eklenir; helper yalnız std/libc kullanır, temel veya
semantic modelden adapter'a ters geçiş ve yeni SCC yoktur. Domain handler,
transaction ve desteklenen v1 facade sahipliği değişmez (ADR-113).

## K-257–K-260 production sahipliği incelemesi

Yeni private CLI `web_metrikleri` ve `web_listener` adaptör sahipleri eklendi.
Exact ileri kenarlar `dil_cli -> web_metrikleri`, `dil_cli -> web_listener` ve
`web_listener -> surum` olarak fixture'da bilinçli tanımlıdır. Metrik sahibi
std/libc ve sınırlı numeric state taşır; PostgreSQL havuzunun callback/observer
arayüzü sahiplik sınırıdır, `postgresql` temel/altyapıdan CLI adaptöre import
etmez. Snapshot/state() callback dışında havuz sahibinde örneklenir; CLI sınırlı
veri snapshotını tüketir. IO
havuz callback'ine girmez. Yeni `postgresql/havuz/metrik.rs` mevcut PostgreSQL
sahibinin alt dosyasıdır. `web_kapanis` K253 sahibinde kalır.

K258/K260 Python host/credential kompozisyonu Rust graph'a hayali kenar eklemez;
K246 exact Popen/group/WNOWAIT ve tek designated descriptor ownership korunur.
Desteklenen `dil::api::v1` export/grammar aynı kalır. Katman/SCC fixture'ı yalnız
snapshot susturmak için yenilenmez; kaynak graph, ters kenar ve public allowlist
aynı root dalgasında kapılardan geçer. Clean final gate olmadan yeni source
SCC/API kabulü tamamlanmış sayılmaz. K262 aynı base'deki correctness dokümanı
bu append ile birleştirilir; eski full-file içerikle overwrite yapılmaz.
