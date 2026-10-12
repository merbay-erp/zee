# Derleme ve bağımlılık envanteri

K-191, 12 Ekim 2026. İnceleme tabanı `f3276b9bb4a286ca897bc45fd0aca7c622f1e5c6`;
manifest `compiler/Cargo.toml`, çözüm `compiler/Cargo.lock`, araç zinciri
`compiler/rust-toolchain.toml`. Bu sayfa mevcut kapasiteyi ve sonraki işleri
ayırır; native backend veya self-hosting tamamlanmış değildir.

## Üç ayrı bağımlılık sınırı

| İş | Mevcut gereksinim | Açık sınır |
|---|---|---|
| Bootstrap derleyicisini kaynaklardan üretmek | Rust 1.93.1, Cargo, platform linker/SDK, lock'taki Rust kitaplıkları | Rust araç zinciri gereklidir; vendor yalnız crate kaynaklarını sağlar |
| Zee programını hazır `dil` ile çalıştırmak | Platforma uygun `dil`, işletim sistemi kitaplıkları; kullanılan ürüne göre dosya/ağ/veritabanı | Rust/Cargo kurulumu gerekmez; ikilinin bütün sistemlerde statik olduğu iddia edilmez |
| Zee kaynağından bağımsız native/WASM çıktı üretmek | Henüz gerçeklenmedi | Şimdiki `kaynagi_derle` bağlı program üretir; ağaç yürüyen yorumlayıcı çalıştırır |

Playground'daki `dil.wasm`, bootstrap derleyicisinin Rust'tan WASM'e derlenmiş
halidir. Zee programının doğrudan WASM modülüne çevrilmesiyle aynı kapasite
değildir. Zee'nin kendi paket bağımlılıkları (`proje.dil`, exact uzak sürümler,
`proje.kilit` v3) Cargo bağımlılıkları değildir; sözleşmeleri
[spec/07](../spec/07-birimler.md) ve [spec/19](../spec/19-registry-metadata-guveni.md).

## Doğrudan Rust bağımlılıklarının sorumlulukları

| Hedef | Crate | Sorumluluk |
|---|---|---|
| Native ve WASM | `num-bigint`, `num-traits` | Kesin sayısal altyapı |
| Native | `argon2`, `getrandom`, `rpassword` | Parola özeti, OS rastgeleliği, terminalde gizli giriş |
| Native | `ed25519-dalek`, `sha2` | Yayın/metadata imzası ve özet |
| Native | `serde`, `serde_json` | Kapalı metadata şemaları ve bakım çıktıları |
| Native | `unicode-normalization` | NFC normalizasyonu |
| Native | `ureq` (rustls) | HTTPS outbound/registry taşıması |
| Native | `postgres`, `postgres-native-tls`, `native-tls` | PostgreSQL ve TLS adaptörü |
| Native | `r2d2`, `r2d2_postgres` | Sınırlı bağlantı havuzu |
| Unix | `libc` | İşletim sistemi/metadata adaptörleri |

Exact geçişli sürümler ve checksum'lar Cargo.lock'tan alınır; manifestteki
SemVer aralığı `--locked` çalışmada bağımlılık yükseltmez. Kriptografi, TLS
ve genel sayı altyapısını yeniden yazmak bağımsızlık hedefinin önkoşulu
değildir ([ADR-001](../adr/001-bootstrap-dili.md)).

`native-tls` nedeniyle native ikililerin platform kitaplıkları ayrıca
incelenmelidir. Crate SBOM'u linker/SDK veya sistemde yüklenen bütün kitaplıkları
listelediği anlamına gelmez. K-192 bu gerçek ikili sınırını ölçmeden
kurulumsuz/statik dağıtım sözü genişletilmez.

## Araç zinciri ve provenance düzeltmesi

K-191 öncesinde kökten başlatılan sürüm betiği host/rustc/sysroot bilgisini
varsayılan Rust'tan okurken klonları `compiler/` altındaki pinle derliyordu.
İnceleme makinesinde varsayılan 1.98.1, proje pini 1.93.1 idi.

Ortak `scripts/rust-arac-zinciri.sh` tek exact `X.Y.Z` pinini doğrular,
`RUSTUP_TOOLCHAIN` değerini o pine bağlar ve çalışan `rustc` sürümünü kontrol
eder. Sürüm üretimi pinini üretilen exact HEAD'den alır; `--kirli-kabul` yerel
pin değişikliğini provenance'a taşımaz. Vendor denetimi incelenen çalışma
ağacının pinini kullanır. Derleme, sysroot remap ve metadata aynı araç
zincirini kullanır. Eksik araç zinciri ve uyuşmayan rustc hata verir;
betik otomatik araç zinciri kurmaz.

`bash scripts/tests/rust-arac-zinciri-test.sh` farklı varsayılan, stable/nightly,
eksik/yinelenen pin, uyuşmayan rustc, kirli yerel pin, iki klon ve boş Cargo
home girişlerini sahte araçlarla sınar. Bu, betiklerin yönlendirme regresyonudur;
gerçek derleme ve çift-klon hash kanıtının yerini tutmaz.

## Kanıtı yeniden üretmek

```bash
cd compiler
rustc --version
cargo metadata --locked --offline --format-version 1
cargo tree --locked --offline --target wasm32-unknown-unknown --depth 1
cargo check --locked --target wasm32-unknown-unknown --lib
cd ..
bash scripts/tests/rust-arac-zinciri-test.sh
bash scripts/offline-vendor-denetle.sh
# Temiz exact commit'te gerçek iki-klon artefakt kanıtı:
bash scripts/surum-artefakti.sh
```

İlk kaynak edinimi ve Rust hedef bileşenlerinin kurulması ağ gerektirebilir;
boş Cargo home vendor kapısı önceden kurulu Rust/linker/SDK'yi içermez.

## Bağımlılık sırasıyla sonraki işler

1. **K-191 kapalı kapsam:** derleme/bağımlılık envanteri ve araç zinciri
   provenance tutarlılığı. Dil semantiği, crate sürümleri ve SBOM şeması değişmez.
2. **K-192 kısmen açık:** Linux/macOS/Windows release ikililerinde sistem kitaplığı
   envanteri, linker/SDK provenance sınırı ve Rust bulunmayan makinede kurulum
   provası. Mevcut kurulum tatbikatının compiler dev ortamından bağımsızlığı
   ayrıca değerlendirilir; saha kanıtı olmadan kapanmaz.
   [Yerel kanıt](derleme-bagimlilik-kaniti-2026-10-12.md) macOS release
   ikilisinin sistem kitaplıklarını ve Rust olmayan PATH ile çalışmasını
   ölçtü; kaynak kimliği kanonik depoya bağlandı. Üç platform tatbikatı, temiz Linux container runtime ve uzak exact kaynak
   erişimi kanıtlandı; temiz Windows/macOS ve minimum OS sınırları açıktır.
3. **K-193 açık:** native/WASM kod üretimi için gerçek ürün ihtiyacı, mevcut
   semantik korpusuyla eşdeğerlik ölçütleri ve backend karar verisi.
   [Backlog](oncelikli-backlog.md), CORE FREEZE ve insan kanıtı kapıları korunur;
   henüz backend seçilmez veya yeni CLI/grammar açılmaz.
4. **Self-hosting açık:** [ADR-008](../adr/008-self-hosting-asamalari.md)
   aşamalarına göre önce Zee kitaplığı, ardından derleyici parçaları;
   Stage 0 bootstrap güven kökü olarak korunur.

## K-192 otomatik çalışma ortamı kapısı

`python3 scripts/calisma-ortami-denetle.py --artefakt <surum> --kurulu <bin>
--rapor <rapor.json> --beklenen-sha <tam-sha>` (`Windows: python`) kurulu
`dil`/`dillsp` hash'lerini artefakt manifestiyle ve kaynak SHA'yı beklenen
commit ile karşılaştırır. Fark varsa ikili çalıştırmadan kapı kırılır.

Linux `readelf`, macOS `otool -L` ve Windows `dumpbin /DEPENDENTS` doğrudan
sistem bağımlılıklarını raporlar. Windows araç bulma
[Microsoft vswhere tarifine](https://github.com/microsoft/vswhere/wiki/Find-VC),
DLL listesi [resmî dumpbin sözleşmesine](https://learn.microsoft.com/en-us/cpp/build/reference/dependents)
dayanır. Python ve inceleme araçları kanıt üreticisidir; Zee programının
runtime bağımlılığı olarak sunulmaz.

Ayrı çocuk süreçlerinin PATH'i yalnız sistem dizinleridir; `rustc`, `cargo`,
`rustup` bulunursa prova reddedilir. `dil sürüm`, exact merhaba-dünya çıktısı
ve gerçek `dillsp` initialize/shutdown/exit yanıtları kontrol edilir.
`zee-calisma-ortami-1` raporu başarılı/başarısız durum, kaynak SHA, ikili
hash'leri, OS/mimari, doğrudan kitaplıklar, ABI header gözlemleri ve prova
sınırını taşır. Hata raporu başarı gibi sunulmaz.

`kurulum-tatbikati` bu kapıyı üç platformda kurulu ikili üzerinde çalıştırır;
rapor exact commit+OS adıyla 90 günlük ayrı artefakta gider. Sürüm artefakt
kümesi/SBOM şeması değişmez. Saf ret regresyonları
`scripts/tests/calisma-ortami-test.py` ile farklı kaynak, değiştirilmiş ikili
ve yinelenen checksum'ı çalıştırmadan reddetmeyi kanıtlar.

İlk macOS prova K-191'in exact release ikililerinde geçti. Güncel üç platform sonuçları aşağıdaki kalıcı deftere kaydedildi. Builder'da Rust'ın kurulu
olmaya devam etmesi, minimum OS ve gerçek linker/SDK üretim provenance'ı
ayrı açık sınırlardır; PATH yalıtımı bunların yerine geçmez.

İlk uzak tatbikat `3bf59d6dd4641ba91fa983048f051cbbda6b70d3` kaynağında
Linux/macOS workflow'u geçti; Windows raporundaki program/LSP sonucu geçti,
ama log satırının CP1252 kodlanması workflow'u kırdı. K-192 log özeti ASCII,
JSON raporu UTF-8 olarak ayrıldı; ret regresyonları `PYTHONIOENCODING=ascii:strict`
altında da çalışır. Güncel `30dcfae` kaynağında üç platform koşusu geçti.

Linux `dil` doğrudan `libssl.so.3`/`libcrypto.so.3` ister; Windows ikilileri
`VCRUNTIME140.dll` ve UCRT/API-set kitaplıklarını ister. Bu gerçek bağımlılıklar
"yalnız tek statik ikili" sözü değildir. DLL/ELF listesi runtime yükleme
başarısıyla birlikte değerlendirilir. Ayrı tatbikat artefaktı JSON ile o
koşunun mevcut `surum/` kümesini birlikte saklar; böylece indirilen byte'lar
rapor hash'leriyle doğrulanıp ayrı runtime ortamında tekrar sınanabilir.

## Üç platform kanıtı ve temiz Linux runtime

`30dcfaeec34f2bb992719f7a210bdf3138a226b0` kaynak commit'inin
[38238657801 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38238657801)
Linux/macOS/Windows'ta başarılıdır. Üç rapor ve byte özeti
[platform kanıt defterinde](derleme-platform-kaniti-v1.tsv) kalıcıdır;
`kurulum_testi` kaynak/hash, schema, ikili listesi, Rust'sız PATH program ve
LSP gözlemlerini denetler. Archive zip digest'leri GitHub artifact metadata'sıdır;
indirilen ikili byte'ları rapor/SHA256SUMS ile ayrıca eşleştirildi.

Aynı Linux artefaktı Ubuntu amd64
`sha256:f610ab94648195aa356059f5b41d6085c9d4d903c072430cdd1af7bdb646106b`
image'ının yeni container'ında `--read-only --network none` ile çalıştı.
Yalnız artefakt ve merhaba kaynağı read-only bağlandı; `rustc`/`cargo` paketleri
ve PATH'te `rustc`/`cargo`/`rustup` yoktur. Program exact çıktı, stdio LSP
initialize/shutdown/exit geçti. Runtime Ubuntu 24.04.5 x86_64; libc6
`2.39-0ubuntu8.9`, libgcc-s1 `14.2.0-4ubuntu2~24.04.1`, libssl3t64
`3.0.13-0ubuntu3.15`. macOS arm64 Docker hostunda amd64 emülasyonu kullanıldı;
bu performans ya da bare-metal minimum OS ölçümü değildir. Windows/macOS
temiz fiziksel makine ve minimum OS/linker-SDK provenance sınırları açıktır.

Kalıcı JSON raporları `.gitattributes` içinde binary olarak işaretlidir;
Windows CRLF dahil indirilen ham byte'lar Git satır sonu dönüşümünden korunur.
Defterdeki SHA-256 normalize edilmiş JSON metninin değil ham raporun özetidir.

## K-195 Windows dağıtım profili — kapalı

Önceki `30dcfae` Windows artefaktı VCRUNTIME140/UCRT ister; tarihsel kanıt
korunur. Yeni MSVC dağıtım tarifi statik CRT, explicit `--target <exact-host>`
ve `-C link-arg=/Brepro` kullanır. Yalnız `surum-artefakti.sh` release
üretiminde etkindir; geliştirme/Cargo varsayılanı aynı kalır. Üç platform
kurulum tatbikatı aynı iki-klon scriptini çağırır. Profil provenance
`cargoArgs`/`crtLinkage`/`targetRustflags` alanlarıyla izlenir.

Windows kapısı `static` metadata kaydının dinamik CRT DLL importu
taşımadığını kontrol eder; tatbikat raporun `statik-crt-import-yok`
sonucunu ayrıca ister. Metadata olmadan yapılan eski genel runtime provası
K-195 kapanma kanıtı değildir. Yeni dumpbin header gözlemi PE
time-date-stamp/reproducible alanlarını da saklar. Sistem DLL bağımlılığı ve
minimum OS sınırı sürer. `/Brepro` MSVC'de dokümante olmayan bir seçenektir;
[cc-rs sahipleri](https://github.com/rust-lang/cc-rs/issues/373) ve kullanılan
cc 1.4.4 kaynak yorumu bu sınırı belirtir. Bayrak varlığı başarı kanıtı değildir.

Git Bash klon/Cargo home/sysroot yolları `cygpath -m`, çıktı yolu
`cygpath -am` ile native rustc yol biçimine getirilir. Göreli `--cikti`
depo köküne sabitlenir; compiler çalışma dizininde yeniden çözülmez.
[Cargo encoded rustflags](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)
boşluklu remap yollarını tek argüman tutar; caller encoded flags CRT/remap
tarifini ezemez. Giriş regresyonları göreli/boşluklu yol, caller bayrağı,
Windows hedef dizini ve eş olmayan mock ikilileri denetler.

İlk `ddaec060ac2d8b9a30461d64241c53bbcc19558c` kaynağının
[38242263231 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38242263231)
Linux/macOS çift-klon ve kur/program/LSP/kaldır job'larını geçti. Ham
runtime/provenance JSON'ları fixture'da, artifact/report özetleri
[platform defterinde](derleme-platform-kaniti-v1.tsv) kalıcıdır. Kurulum
testi metadata subject'lerini ve iki klon manifest özetini runtime binary
hash'leriyle karşılaştırır. Defter sonucu exact platform job'ına aittir;
tüm run Windows reddi nedeniyle başarısızdır.

İlk Windows çift-klon hash'leri dil için
`1debd0c76d98f057832b238338d98af7d3d1cf451a49b2fe64485dbf3c96330f` ve
`775bddf93399858a00ee63fc4eba7f6f420d66d0b742d4d6d84279c898b593fd` oldu.
Kur/runtime provasına geçilmedi, release artefaktı üretilmedi. PE timestamp
farkı çıkarımdır; ilk koşu ikilileri saklamadığı için byte nedeni henüz
kanıtlı değildir. Yeni koşuda eş olmayan a/b byte'ları ayrı
`yayinlanamaz-surum-farki` tanı artefaktına gider. `DURUM` dosyası
yayınlanamayacağını belirtir; release/SBOM/imza üretilmez.

Göreli yol düzeltmesi `5a4e81f9c3d14eb12395e70752f3031768648bb7` yerel
macOS arm64/Rust 1.93.1 release tam provasını geçti: iki klonda dil
`c73ee537c628def5151ea97206e714e9d3dd01f90dbced2745f14047fdcb603c`,
dillsp `064f2ab948f5b32a707962a0e8045cd7794136a18d97e40d78e72acd7372abad`.
Kur/program/LSP/kaldır başarılıdır; ayrıntı ve runtime rapor özeti
[çalışma günlüğündedir](../kararlar/gunluk.md). Unsigned artefakt
`compiler/target/k195-surmeler/goreli-prova/` altındadır.

`c7e3d04adf27905ade8b622d24bd76762123d2dc` yeni linker tarifinin
[38244549913 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38244549913)
üç platformda başarılıdır. Windows iki klonda dil
`48a1d3a4b63834bf21342a0b6af33d52464473bcb8ce0962f894b58c1572008d`,
dillsp `d873c81763efb51cfc5a624cbfb2b281775a413a7d1cb1f56a62eb2e7791da75`
hash'lerini üretti. İndirilen ikili byte'ları runtime/provenance/SHA256SUMS
ile eşleştirildi; statik CRT import kontrolü ve kur/program/LSP/kaldır geçti.
Ham rapor/provenance ve kaynak/job/artifact özetleri kalıcı defterdedir.
K-195 bu sınırlı dağıtım kapsamında kapandı. Temiz Windows/macOS, minimum OS ve
linker/SDK üretim provenance sınırları K-192 içinde ayrıca açıktır.

Rust bakım üreticisi dil semantiği maintenance kapsamındadır; yeni grammar,
komut, tanı ya da gömme API exportu yoktur. ADR-046/katman fixture ve
bağımlılık çevrim grafı yeni modül/kenar almaz. RFC/spec/hata kataloğu
normatif dil davranışı açısından incelendi ve değişmedi. Release profilinin
performans etkisi henüz ölçülmedi; ADR-045/049 uyarınca eski benchmark
satırları taşınmaz veya yeni baseline uydurulmaz.

K-195 tedarik CI ret nedeni test fixture'ının yanlış yol varsayımıdır:
release klonu kendi TMPDIR'ında açılırken test ilk remap argümanında
spaced fixture kökünü varsayıyordu. Giriş testi release çocuk süreçlerine
özel spaced TMPDIR verir; üretim tarifi değiştirilmez. Aynı test
macOS Bash 3 ve ağsız Linux Bash 5 container'ında doğrulanır.

K-192 ek temiz Linux kanıtı: `c7e3d04` CI ikilisi Debian 12 userland'ında
program exact çıktı/exit 0 ve LSP initialize/shutdown/exit provasını geçti.
Base amd64 `debian@sha256:a4672c0cb26fbdde88e38fa2dfb6c681942306680e41e4378b28770b6e79ee91`
image'ına yalnız libssl3/libgcc-s1 eklendi; libc6 `2.36-9+deb12u14`,
libgcc-s1 `12.2.0-14+deb12u1`, libssl3 `3.0.22-1~deb12u1`.
Rust araçları PATH'te yoktur; rustc/cargo paketleri dpkg'de yoktur. Çalışma
`--read-only --network none`; host macOS arm64, amd64 emülasyonudur.
Kernel Docker hostunundur; minimum OS tam destek veya performans garantisi
değildir. [Ham rapor](temiz-linux-runtime-kaniti-2026-10-12.json)
SHA-256 `5d9df5d2fb4a6c4273c8efee1ed1824016271153b02fb3461a85f92c09b56d5e`; Dockerfile, base/derived image kimliği,
package/kernel/ikili hash ve LSP yanıtları raporda korunur. K-192
minimum OS/linker-SDK ve temiz Windows/macOS sınırları ayrı açıktır.

## K-196 gerçek linker ve SDK izleme — kanıtı açık

Release derlemesi sabit Rust 1.93.1 `RUSTC_LOG=rustc_codegen_ssa::back::link=info`
izini alır. [Rust compiler tracing](https://rustc-dev-guide.rust-lang.org/tracing.html)
ve exact [1.93.1 link yürütme kaynağı](https://github.com/rust-lang/rust/blob/1.93.1/compiler/rustc_codegen_ssa/src/back/link.rs)
gerçek komutun ne zaman kaydedildiğini tanımlar. Unix `-C link-arg=-v`,
MSVC `-C link-arg=/VERBOSE:LIB` gerçek SDK/library aramasını görünür kılar.
Provenance `targetRustflags` aynı tarifi taşır; release dosya kümesi değişmez.

`scripts/derleme-ortami-kaydet.py` yalnız Python 3 standart kitaplığıyla
iki klonda iki binary komutunu seçer. Log shell olarak çalıştırılmaz;
quoted Debug String değerleri çözülür. İzdeki PATH/programdan gerçek araç
çözülür; executable byte SHA-256 ve o programın `--version`/`/?` çıktısı
kaydedilir. SDK referansları ham komut/build log gözlemidir; Apple
SDKSettings Version ve Windows Kits Lib sürümü varsa ayrıca çıkarılır.
Bu kapsam linker driver'ını kanıtlar; tüm C build-script compiler/assembler
araçları veya driver'ın bütün alt süreçlerini eksiksiz envanterlediğini
iddia etmez. Linux ayrı Windows/macOS SDK semantiği taşımaz.

Üretici `--build-environment` kaydını kaynak SHA/schema, a/b ve dil/dillsp,
araç hash/path/version/command varlığıyla denetler. Eski bakım CLI çağrısında
alan isteğe bağlıdır; resmi release scripti alanı daima üretir ve eksik
kanıtta durur. Windows hata log özeti ASCII-safe; rapor UTF-8'tir.
Python 3 artık release kanıtının build-time aracıdır; hazır Zee runtime
bağımlılığı değildir. Ret testleri ve gerçek iki-klon giriş mock'u birlikte
koşar; üç platform tatbikatı ret testini de yürütür.

Dil spec/RFC/tanı/API yüzeyi değişmedi. ADR-046/katman ve SCC fixture
incelendi, yeni production import/kenar yok; kapılar geçti. Verbose link
bayrakları nedeniyle ADR-045/049/ölçüm etkisi incelendi; profil karşılaştırması
olmadan benchmark tarihçesine baseline/performans satırı eklenmedi.
Exact clean kaynak ve uzak üç platform sonuçları gelmeden K-196 kapalı değildir.

GB-033 güven sınırı: kayıt aracı yalnız üreticinin kendi temiz-klon
loglarıyla kullanılır; harici indirilen log güvenilir girdi değildir.
Sürüm sorgusu logda çözülen programa gider; kanıt yayın imzasının yerine
geçmez. Bu sınır güvenlik defterinde ve ADR-068'de kayıtlıdır.

### K-196 yerel exact üretim doğrulaması

`e3adc22895840fc25069153b3457f17b74a2836d` temiz kaynağı yerel macOS
arm64 üzerinde iki temiz klonda aynı dil/dillsp SHA-256 değerlerini üretti.
Dört link kaydında `/usr/bin/cc`, Apple Clang 21.0.0
(`clang-2100.3.34.2`) ve macOS SDKSettings 27.0 doğrulandı.
Linker byte SHA-256: `b8763cf250e607a778bb4603cecb5b90338814d0a3dfcba0d57b1de242f610e9`.
Bu ikililer hash doğrulamalı kurulum, Rust araçları olmayan PATH ile program
ve LSP initialize/shutdown/exit, ardından manifestli kaldırma provasını geçti.
Uzak macOS job 114815113244 geçti; run 38250463153 için Linux/Windows
sonuçları ve kalıcı ham kayıtlar incelenmeden K-196 kapanmaz.
Kalıcı kanıt testi yeni `buildEnvironment` kayıtlarında dört araç kaydı ve
Apple/Windows SDK sürümü ister. Eski kayıtlar bu yeni alanı üretmemiştir.

Bu dilim spec/RFC/tanı/API, production Rust modül kenarı ve benchmark
tarihçesini değiştirmez; ilgili belgeler bu sınırla yeniden incelendi.

K-196 Linux tatbikat işi kuruluşun `itwise-gitsrv` koşucusunda başlamadan
kuyrukta kaldı. `kurulum-tatbikati` manuel girişindeki `linux_runner`
seçimi `configured` (mevcut kuruluş ayarı, varsayılan) veya `ubuntu-latest`
olabilir. Bu koşuya özel seçim kuruluş/repo değişkenlerini değiştirmez;
tag koşuları mevcut kuruluş ayarını izler. Aynı exact kaynağın üç platform
kanıtı için `gh workflow run kurulum-tatbikati.yml --ref codex/zee-derleme-bagimlilik -f linux_runner=ubuntu-latest`
ile GitHub Linux koşucusunda yeni tatbikat başlatılır.

K-196 ilk Windows job 114815113651 iki klonda dil/dillsp eş hash
üretti, fakat kayıt aracı iki link komutunu seçemedi; release metadata
ve runtime provasına geçilmedi. Temp build logları bu koşuda saklanmadığı
için exact başarısız komut biçimi henüz bilinmez. MSVC `/OUT:` ve düz
`.exe` çıktı adları artık hashli adlarla birlikte seçilir; aynı kayıt
testi bu biçimi ve quoted olmayan `/VERBOSE:LIB` Windows Kits yollarını
kanıtlar. Bu olası biçim farkının gerçek hatayı çözdüğü ancak yeni Windows
koşusuyla doğrulanabilir. Kayıt aracı hatasında iki build logu artık ayrı
yayınlanamaz tanı artefaktına korunur; sürüm kümesi üretilmez. Rust 1.93.1
Command debug çıktısı platforma bağlıdır: [Windows uygulaması](https://github.com/rust-lang/rust/blob/1.93.1/library/std/src/sys/process/windows.rs).
