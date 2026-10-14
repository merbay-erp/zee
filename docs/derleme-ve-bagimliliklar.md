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

K-241 sürüm loglarını/native raporlarını kaynak klonunun dışında tutar.
İki klon derleme öncesi ve sonrası temiz kalmalı; her derlenmiş ikili exact
SHA ve `dirty: false` bildirmelidir. Eş hash tek başına bu kimlik kapısını
ikame etmez. [ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md).

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
temiz fiziksel makine ve minimum OS/tüm native C araçları provenance sınırları açıktır.

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
tüm native C araçlarının üretim kaydı K-192 içinde ayrıca açıktır.

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
minimum OS/tüm native C araçları ve temiz Windows/macOS sınırları ayrı açıktır.

## K-196 gerçek linker ve SDK izleme — exact üç platform kanıtıyla kapalı

Release derlemesi sabit Rust 1.93.1 `RUSTC_LOG=rustc_codegen_ssa::back::link=info`
izini alır. [Rust compiler tracing](https://rustc-dev-guide.rust-lang.org/tracing.html)
ve exact [1.93.1 link yürütme kaynağı](https://github.com/rust-lang/rust/blob/1.93.1/compiler/rustc_codegen_ssa/src/back/link.rs)
gerçek komutun ne zaman kaydedildiğini tanımlar. Unix `-C link-arg=-v`,
MSVC `-C link-arg=/VERBOSE:LIB` gerçek SDK/library aramasını görünür kılar.
Provenance `targetRustflags` aynı tarifi taşır; release dosya kümesi değişmez.

`scripts/derleme-ortami-kaydet.py` yalnız Python 3 standart kitaplığıyla
iki klonda iki binary komutunu seçer. Log shell olarak çalıştırılmaz;
quoted Debug String değerleri çözülür. İzdeki PATH/programdan gerçek araç
çözülür; executable byte SHA-256 ve o programın `--version`/Windows FileVersionInfo çıktısı
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
Exact b153064 kaynağı/run 38252942210 üç platformda geçti; K-196 bu
linker driver/SDK referansı kapsamında kapandı.

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

Tarihsel K-196 Linux tatbikatı kuruluş koşucusunda başlamadan kuyrukta
kalmıştı. O sıradaki linux_runner geçişi K-217 ile kaldırıldı; GitHub
workflow'ları sabit hosted hedefler taşır ve mevcut repo Actions'ı kapalıdır.
Kurum işi özel Git2/Gitsrv akışında yürür (ADR-080/088). Önceki run/SHA
kanıtları tarihsel kalır; güncel çapraz derleme gerçek Windows runtime
kabulü yerine konmaz.

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

### K-192 glibc 2.34 kullanıcı alanı kanıtı

Exact `e0f53630fe02c3239b9fc248e07abcedbe7ca681` Linux dağıtım ikilileri
AlmaLinux 9.8 temiz minimal image üzerinde program ve gerçek LSP
initialize/shutdown/exit provasını geçti. `glibc-2.34-275.el9_8`,
`libgcc-11.5.0-14.el9.alma.1`, `openssl-libs-3.5.8-2.el9_8` kaydedildi.
Base digest `8e56ca192f602eb1c486b870f268730f4b08179b43845184a3623c6634f5d01a`;
Dockerfile yalnız openssl-libs/libgcc ekler. Rust paketleri ve PATH
araçları yoktur. Koşu user 10001:10001, network none, read-only root,
cap-drop ALL, no-new-privileges ve geçici /tmp ile yapıldı. [Ham kanıt](temiz-almalinux-runtime-kaniti-2026-10-12.json)
exact image/base, tüm paket envanteri, Docker argümanları, ikili hash ve
LSP yanıtlarını taşır. Host macOS arm64, hedef amd64 emülasyonudur; kernel
6.10.14-linuxkit hostundur. Bu seçili runtime provası glibc 2.34 kullanıcı
alanını doğrular; bütün dil IO özellikleri, fiziksel Linux minimum kernel,
Windows/macOS temiz kurulum ve genel minimum OS desteği ayrı açıktır.

İlk K-196 run 38250463153 Linux job 114815113812 geçti; GCC 13.3.0 dört
gerçek link kaydı kalıcı ham provenance ile eşleşir. Aynı run Windows
kayıt kapısında kaldı; koşu bütünü başarılı gösterilmez.

K-196 ikinci Windows job 114819400810 gerçek linker komutlarını seçti,
fakat MSVC `link.exe /?` sürüm çağrısı nonzero verdi. Ham build logları
yayınlanamaz tanı artefaktında korunmuştur. Windows MSVC kimliği artık
PowerShell 5/.NET `FileVersionInfo.GetVersionInfo` ile executable dosyasının
FileVersion/ProductVersion kaynağından okunur; `version_source` bunu
açıkça belirtir. Yol statik komuta eklenmez, ayrı environment alanındadır.
Unix/LLVM araçları `--version` kullanır; boş sürüm veya başarısız metadata
yine release kanıtını reddeder. [API kaynağı](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.fileversioninfo.getversioninfo).
Bu Windows build kanıtı PowerShell gerektirir; Zee runtime bağımlılığı
değişmez. Yeni exact üç platform doğrulaması gelmeden K-196 açık kalır.

İndirilen ikinci Windows a/b build loglarının komut/SDK ayrıştırması
incelendi: gerçek `/OUT:` sonları hash'siz `dil.exe`/`dillsp.exe`; SDK
verbose satırları `Windows Kits/10/lib/10.0.26100.0` taşıyor. Ayrıştırma
deneyi resolver'ı yalnız yerel Python executable'ına yönlendirerek
indirilen komuttan hiçbir araç yürütmedi; bu Windows üretim sürüm
kanıtı değildir. Gerçek FileVersionInfo üretim kapısı ayrı beklenir.

## K-197 temiz Windows runtime provası — exact kanıtla kapalı

`scripts/temiz-windows-prova.py` yalnız Python standart kitaplığı kullanır.
Exact release SHA/hash/yol/tekillik denetimi Docker veya ikili yürütmeden
önce yapılır; staged byte'lar da yeniden hash'lenir. Docker Windows
daemon ve pinned image mimarisi denetlenir. Microsoft Nano Server digest
`4fe6f59e0aadb628a933345eae1f5ab43a2af802969ac0e2ef996c66a3188438`
amd64/10.0.26100.33438 manifestidir. Container'da Rust kurulmaz; PATH
system dizinleriyle sınırlıdır ve rustc/cargo/rustup byte varlığı reddedilir.
Program exact çıktı, LSP yanıt kimliği/capabilities/shutdown koşullarıyla
denetlenir. Windows Python/Docker/PowerShell/gh bakım hostunun araçlarıdır,
Zee runtime bağımlılığı değildir. Ham JSON UTF-8, log özeti ASCII'dir.
Host/prova kaynak ve araç hash'i release kaynağından ayrı raporlanır.

ContainerUser/network none/read-only bind kullanılır; root geçici ve
yazılabilirdir, process isolation host kernel'ini paylaşır. Trusted kendi
release byte'ları dışında sandbox sözü verilmez (GB-034). Desktop/minimum
Windows, bütün IO davranışları ve temiz fiziksel macOS ayrı açıktır.
Release c7e3d04/prova 27c0fa9 run 38253777857 job 114826470199 gerçek
Windows Nano Server program/LSP kanıtıyla K-197 kapandı. [Ham rapor](temiz-windows-runtime-kaniti-2026-10-12.json)
ve [defter](temiz-windows-platform-kaniti-v1.tsv) kaynak/image/prova aracı
hash'i ve ikili byte'larını bağlar; Cargo kurulum kanıtı veya desktop
Windows desteği bu dar kapının kapsamı değildir.

K-196 exact kapanma kaynağı `b1530641a3f902047b57ad6e7c0c004ec5f1c420`,
[run 38252942210](https://github.com/itwise-bilisim/zee/actions/runs/38252942210).
Üç platform eş iki-klon SHA-256 ve kur/program/LSP/kaldır geçti. Her a/b
klonunda dil/dillsp gerçek araç path/byte hash/sürüm/komut kaydı vardır.
Linux GCC 13.3.0, macOS Apple Clang 21.0.0 (clang-2100.1.1.101)/SDK 26.5,
Windows MSVC linker FileVersion 14.51.36260.0/SDK 10.0.26100.0. Windows
araç dizin adı 14.51.36231 iken dosya metadata sürümü 14.51.36260.0
olduğundan klasör adı sürüm kanıtı sayılmaz. [Kalıcı defter](derleme-platform-kaniti-v1.tsv)
ve fixture raw JSON byte'ları indirilen artefakt/hash/source ile eşleşir.
İlk iki Windows ret koşusu saklıdır; başarı yeni exact kaynak içindir.

## K-198 temiz Windows kurulum/kaldırma — exact kanıtla kapalı

`scripts/temiz-windows-kurulum-prova.py` stock Server Core LTSC 2025
immutable digest'inde mevcut PowerShell ile kur/program/LSP/kaldır
zincirini yürütür. Rust veya harici VC runtime kurulmaz. Exact artefakt
SHA/hash, staged iki ikili ve installer byte hash denetlenir. Aynı
ContainerUser container'ında Türkçe boşluklu hedefe kurulur; manifest
iki exact path/hash/source kaydı ister. Program/LSP ardından manifestli
kaldırma ve dosya yokluğu doğrulanır. Kapanışta yalnız bu provanın UUID
container'ı kaldırılır. Source/helper/installer/prova hash'leri ham
raporda ayrı tutulur. GB-035 güven sınırı ADR-068'dedir.

Yerel macOS Parallels CLI inventory çağrısı servise bağlanamadı; guest
listesi/kanıtı elde edilmedi. Kullanıcının Mac'i yeniden başlatılmadı.
Fiziksel/VM macOS temiz runtime ve minimum OS kanıtı açık kalır.

K-198 release b153064/prova 33176cf run 38255788989/job 114833388588
ile geçti: stock Server Core 10.0.26100.33438, Docker 29.7.2, ContainerUser
ve Türkçe boşluklu kurulum hedefi. Manifest exact iki path/hash/source ile
eşleşir; program/LSP ve kaldırma sonrası dosya yokluğu doğrulandı. Prova,
helper ve iki installer byte hash'i ayrıca kaynakla eşleştirildi.
[Ham rapor](temiz-windows-kurulum-kaniti-2026-10-12.json),
[defter](temiz-windows-platform-kaniti-v1.tsv) kalıcıdır. Bu kapsamta
K-198 kapandı; fiziksel/desktop/minimum OS ve tüm IO ayrı K-192 sınırıdır.

## K-199 Cargo compiler/wrapper pin sınırı — exact üç platform kanıtıyla kapalı

Yerel gerçek probe RUSTUP_TOOLCHAIN=1.93.1, RUSTC=kurulu 1.97.1 ile
helper'ın 1.93.1 version sorgusu ve Cargo'nun 1.97.1 invocation'ını
birlikte gösterdi. Tam Zee release bu farklı compiler ile üretilmedi;
bu küçük gerçek Cargo reproducer'ı pin seçim kusurunun kanıtıdır.
Helper exact rustup executable'ını doğrular ve Cargo RUSTC'ye bağlar;
Windows native yol normalize edilir. İki wrapper boş string'i config ve
ENV wrapper önceliğini kapatır. Installer/runtime byte'ları, SBOM/provenance
schema ve profile bayrakları değişmez; compiler seçimi sağlamlaştırılır.

Gerçek depsiz Cargo/build-script/ikili probe'u wrong compiler/wrapper
ENV ve Cargo home config ile koşar. Actual compiler sürümü yürütülen
ikiliden, tool path/hash ve log exact prova commit'iyle kaydedilir.
Bu tam Zee release veya tüm Cargo profil/linker/native C ayarlarının
envanteri değildir. Exact 22ea2b8/run 38259341995 üç platformda geçti.
Performans profil/baseline'ı değişmedi; ADR-045/049/history incelendi,
izlenen benchmark tarihçesine yeni satır yazılmadı. GB-036/ADR-068.

K-199 wrapper kapatma caller'ın derleme cache davranışını değiştirir;
derleme süreleri etkilenebilir. Runtime profile/iş yükü değişmez;
ADR-045/049/history incelemesi `docs/olcumler.md` kaydındadır. Küçük
probe raporu source tree cleanliness, helper/prova byte hash ve exact
Git SHA taşır; dirty yerel deneme exact kaynak kapanma kanıtı değildir.

K-199 source `22ea2b8d4d8425788c1ef5c49874d66df14eb7f5`,
[run 38259341995](https://github.com/itwise-bilisim/zee/actions/runs/38259341995)
üç platformda geçti. Rust/Cargo 1.93.1, compiler absolute path/hash, boş
wrapper alanları, poisoned config ve source tree temizliği raw raporda
vardır. Helper/prova/pin hash'leri ayrıca kaynakla eşleşir. [Defter](rustc-pin-platform-kaniti-v1.tsv)
source/run/job/artifact/zip/raw hash'lerini bağlar. K-199 bu sınırlı
compiler seçimi kapsamında kapandı; tam Zee release profil/native C
envanteri değildir. K-192 kalan makine sınırları açık kalır.

## K-200 macOS C/Rust deployment uyumu — kapalı

Gerçek local Cargo -vv + CC_ENABLE_DEBUG_OUTPUT locked cc 1.4.4/Ring
C bayrağı `-mmacosx-version-min=27.0` gösterdi. otool native curve25519.o
minos 27.0, final dil minos 11.0 bildirdi. Bu final başlığın native
object deployment uyumunu tek başına kanıtlamadığını gösterir; macOS 11
çalıştırma testi yapılmadı. Export MACOSX_DEPLOYMENT_TARGET=11.0 sonrası
24/24 build-script object minos 11.0, program/LSP Rust'sız PATH ile geçti.

Release exact Rust default'unu caller ENV dışında alır ve C/Rust için
ortak export eder. Native .o başlığı doğru Apple platform/üst minos/
en az bir toplam version kanıtıyla denetlenir. Object relative path/
byte hash/ham başlık ve target iki-klon buildEnvironment metadata'sına
girer. Başlıksız assembly açık sınırlıdır; gerçek minimum OS/runtime
desteği veya bütün C alt süreçlerinin araç envanteri değildir. Release
dosya kümesi, SBOM/imza sözleşmesi ve dil API değişmez. Compiler/native
compile bayrağı değişti; ADR-045/049/history etkisi `olcumler.md` içindedir.
K-200 exact kaynak `44fba404c79762aff1698a07978c79ea069a59df`, run
38263264133 ile kapandı: Linux/macOS/Windows iki-klon eş hash, kurulum,
program/LSP ve kaldırma geçti. macOS a/b ayrı ayrı 24 nesne minos 11.0
bildirdi. Ham runtime/provenance, job ve artefakt hash kayıtları
[platform kanıtı](derleme-platform-kaniti-v1.tsv) içindedir.

## K-201 — Native araç çağrı kaydı

İki release klonu verbose Cargo ve locked cc debug çıktısını kaydeder.
Compile ve archive çağrı kanıtı zorunludur. Kayıt, her native driver
çağrısının paket sahibini, argümanlarını ve çözülen executable path/hash
kimliğini `zee-native-tools-1` alt şemasında provenance'a bağlar.
Windows cl/lib/ml/link FileVersionInfo, diğer araçlar --version ile
sorgulanır. Desteklenmeyen sürüm sorgusu `unavailable` ve gerekçesiyle
görünür; byte hash araç sürümü yerine sunulmaz. Yerel gerçek cc izinde
38 çağrı ve cc/xcrun/ar görüldü; Apple ar --version desteklemedi.
Bu eski yerel log denemesi exact yeni release kapanma kanıtı değildir.

Parser Rust Debug Command'ın quoted alanlarını çözer; shell/eval yoktur.
Eksik compile/archive, bozuk command biçimi veya kayıp executable release'i
durdurur. Probe komutunun başarısız olması Cargo'nun başarılı derlemesiyle
karıştırılmaz; kayıt tek tek compile exit sonucu eşlemesi vaat etmez.

K-201 exact kaynak `25b6882b3f148f536ea81079d98991370999cdd8`,
merbay-erp/zee run 38349601833 üç platform iki-klon eş hash/kurulum/
program/LSP/kaldırma kapısını geçti. Ham runtime/provenance ve job/artifact
hash kayıtları `derleme-platform-kaniti-v1.tsv` içindedir. pkg-config/vcpkg,
driver iç subprocess'leri ve işletim sistemi process tracing ayrı
K-192 işidir. Release dosya kümesi/SBOM/imza değişmez; provenance additive
alandır. Dil grammar/spec/RFC/tanı/API ve production Rust katmanı değişmez.

## K-202 — Ana depo ve enterprise ayna

13 Ekim 2026 kurucu kararıyla ana geliştirme/yayın kaynağı
`https://github.com/merbay-erp/zee` olur. CI, manual tatbikat, fuzz, soak
ve release koşuları yalnız bu depoda yürütülür.
`itwise-bilisim/zee` kaynak geçmişinin aynasıdır; repository Actions
permissions `enabled=false` olarak API üzerinden doğrulandı. O anda
çalışan/bekleyen koşu bulunmadı. Ayna push'u geçmişi korur; force push yoktur.

SBOM/provenance kaynak URI değeri yeni ana depoya bağlanır; aynı exact
Git SHA her iki depoda erişilebilir olmalıdır. Önceki itwise koşu bağlantısı
ve ham fixture kaydı tarihsel kanıttır, yeniden adlandırılmaz. Dil semantiği,
ABI ve public API sözleşmesi, SBOM, provenance şeması ve release dosya kümesi değişmez.
Git `origin` merbay-erp, `enterprise-mirror` itwise-bilisim olarak adlandırılır.
Aynadaki PR #9 eski çalışma kaydıdır; bundan sonraki işler/koşular ana
depoda takip edilir. Kaynak testleri ve beyanları K-202 kapsamında güncellenir.

K-206 native okuma akış toplamasına geçti: sonuç sınırından sonra çıktı
biriktirilmez; borrowed metin kopyalanmadan denetlenir. Hata önceliği ve
mevcut read retry/transaction/API korunur. Tek wire satırı/driver RSS
bütçesi GB-041'de açık; exact native kabul ve RSS kaydı aşağıdaki kapanışta saklıdır.

K-206 akış toplama dilimi kapalı: source f2ba654, temiz beyan/ölçüm
6463a7f; 731/731 faz ve gerçek native oracle/transaction kabulü geçti.
64×1 MiB sonuçta native süreç RSS 128,77→23,81 MiB, tek 64 MiB satırda
136,77→72,72 MiB; üç vakada sonraki sorgu başarılıdır. Bir örnek/sıfır
ısınma gözlemi, genel RSS garantisi değildir. [Ham kanıt](incelemeler/k206/README.md).
**K-206 KISMEN:** tek wire satırı/driver sınırı ve GB-041 açık kalır.

## K-206 yerel bağımlılık yaması kimliği — mesaj sınırı henüz açık

PostgreSQL sürücüsü uzunluk başlığında bir üst sınır sunmaz. İzole 32 MiB
prototipi gövdeyi almadan reddeder; fakat bağlantıyı kapatır ve aktif
transaction rollback'i de başarısız olur. Bu davranış **üretimde etkin
değildir**; GB-041 açık kalır. Önce tedarik/artefakt kimliği hazırlanır.

`compiler/third_party/postgres-protocol` upstream 0.6.12 arşivinin
Cargo.lock ile doğrulanmış kopyasıdır; Rust src byte'ları aynıdır. Sürüm
kimliği yalnız kaynak ayrımını göstermek için `0.6.12+zee.1` olur.
Compiler/fuzz patch ve iki lock aynı yerel kaynağı seçer; diğer paket
sürümleri değişmez. MIT/Apache-2.0 lisansları, upstream Git SHA ve arşiv
SHA-256 `UPSTREAM.json` içinde korunur.

`compiler/third_party/patches-v1.tsv` exact ad/sürüm/repo yolu/kanonik
tree SHA-256/upstream sürüm/arşiv hash/Git SHA taşır. Hash; sıralı UTF-8
relative path uzunluğu (u64 BE), path byte'ları, içerik uzunluğu (u64 BE)
ve içerik byte'larının SHA-256'sıdır. Mtime, absolute path, izin/UID
karar girdisi değildir. Symlink ve normal olmayan dosya reddedilir.
Registry paketinin checksum'u ile yerel dosya ağacının hash'i aynı
anlamdaymış gibi gösterilmez: SPDX downloadLocation yerel yamada exact
merbay-erp/zee Git SHA ve repo yolu, verifiedUsing bu kanonik tree'dir.
GB-042 düşük/kapalı: kayıtsız kaynak, bozuk checksum ve yanlış kök
paket ret testleriyle tedarik sınırı doğrulanır.

Yeni yerel kayıt ve fork içeriği değişince tree hash aynı committe
incelenerek yenilenir; sırf kapıyı susturmak için güncellenmez. SBOM
şeması aynı SPDX 3.0.1; kurulum dosya kümesi/ABI/API ve native C araç
bayrakları değişmez. İki-clone release/offline vendor kabulü beklenir.

## K-206 tekrar üretilebilir dependency kimliği

Path patch hazırlığı ba547f4 kaynakta iki temiz klonun farklı ikili hash
üretmesine yol açtı; artefakt reddedildi. Bu kaynak başarılı release kanıtı
değildir. Etkin compiler/fuzz patch artık ana merbay-erp/zee deposundaki
exact c4e9d03616fefa3d4aa9edcdef5c3ab8af7a8c75 commitine pinlidir; dal/tag
kullanılmaz. Cargo.lock iki graph'ta aynı git URL/rev/commit taşır. Rust
protokol src byte'ları hâlâ upstream ile aynıdır; wire sınırı etkin değildir.

`patches-v1.tsv` ilk path kayıt sözleşmesinin tarihidir; etkin sözleşme
`compiler/third_party/patches-v2.tsv` sekizinci alan olarak Zee source Git
SHA taşır. SBOM tam kaynak eşleşmesini, kayıtlı SHA-256'yı, yerel ağacı ve
seçilen commitin gerçek Git blob ağacını birlikte denetler. Package URL
build HEAD'i yerine dependency commitini gösterir. Git geçmişi eksikse
metadata üretimi durur; release/CI full history kullanır. cargo-deny yalnız
ana repo URL'sine izin verir; SBOM ayrıca tam kayda/pine birebirlik ister.

[Cargo Git dependency sözleşmesi](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories)
repo içindeki crate'i bulur ve `rev` pinini destekler. Yalnız sürüm build
metadata'sı pinin yerini tutmaz. Offline vendor ve iki-klon kabulü bu exact
Git kaynağıyla yeniden alınacaktır; başarısız path koşusu tarihten gizlenmez.

K-206 dependency kimliği dilimi kapalı: source 9dae3e6, temiz beyan/
ikili üretimi ab8e97e. 735/735 faz, git pin/blob/tree SBOM, offline vendor
163 paket ve iki temiz klonda eş dil/dillsp SHA-256 geçti. Yerel macOS
kabulüdür; [ham kanıt](incelemeler/k206/source-identity/README.md).
Path kaynaklı başarısız iki-clone koşusu da saklanır. Protokol src byte'ları
upstream ile aynıdır; GB-041/32 MiB erken wire guard hâlâ açık kalır.

### K-215: Path yamanın build kimliği

Önceki exact self Git pin tarihsel tedarik kanıtıdır. Güncel compiler/fuzz
`postgres-protocol` 0.6.12+zee.2'yi depo içi path yamasıyla derler; paket
baytları ve `patches-v2` kanonik SHA-256'sı aynıdır. Kayıtlı kaynak SHA'sı
anonim soy ağacı bilgisidir. Güncel SBOM URL'si gerçek build SHA'sını taşır
ve hem o commit ağacı hem derlenen disk ağacı kayıtlı hash'le eşleşmeden
üretilmez. Git kaynakları deny istisnası olmadan reddedilir. Eski ikililer
ve imzaları geçmiş kaynaklarına aittir; yeni kimlik adına sunulmaz.

K-215 path paketleri için Cargo’nun kanonik kaynak yolu remap kapsamındadır:
geçici klon kökü fiziksel yola açılır. İki ayrı temiz klon ve bağımsız hedef
ağacı korunur; hash farkı artefaktı reddeder. Mantıksal sembolik kök
(`/var` gibi) fiziksel kaynak yolunu maskeleyemediği için kabul edilmez.

### K-216 — Git2 / Gitsrv ana CI koşusu

Kullanıcı isteğiyle GitHub Actions kapalıdır; ana kaynak GitHub'da,
Enterprise yalnız aynadır. Özel Git2'de `ci/zee-gitsrv` dalına exact
kaynak gönderimi veya `zee-ci.yml` manuel başlatması Linux kapılarını
çalıştırır. Git2 erişim adresi/kimliği özel ortamda kalır. Örnek:

```sh
git push git2 HEAD:refs/heads/ci/zee-gitsrv
```

`.forgejo/workflows/zee-ci.yml` yalnız seçili dalın tam geçmişini alır;
PR kodu ve ilgisiz özel dallar çekilmez. İki action SHA'sı ayrı
`docs/forgejo-actions-pinleri-v1.tsv` envanterinde resmî kaynağa bağlıdır.
Faz/kalite, gizlilik/sır, tedarik/offline vendor, WASM ve eşiksiz shared
performans gözlemi aynı kaynakta çalışır; raporlar 90 günlük özel artefakt
olur. Uzak başarısızlık veya başlamamış iş geçiş sayılmaz. Windows runtime
ve release candidate fuzz/Miri/kurulum kanıtları bu Linux CI'ın yerine
geçmez; ilgili sürüm runbook kapıları ayrıca korunur. [ADR-088](../adr/088-git2-gitsrv-kosulari.md).
