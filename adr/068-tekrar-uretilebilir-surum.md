# ADR-068 — Tekrar üretilebilir sürüm artefaktı

- **Durum:** kabul
- **Tarih:** 5 Eylül 2026; K-191 araç zinciri revizyonu 12 Ekim 2026
- **İlgili kayıt:** K-168, ADR-006/038/048/066, V1-P1-25

## Bağlam

Zee paket yayını (`.zep`) K-094'ten beri byte-byte tekrar üretilebilir, imzalı
ve SBOM/provenance'lıdır; fakat derleyicinin kendisi için böyle bir söz yoktu.
`cargo build --release` çıktısının hangi commit'ten, hangi toolchain'le ve
hangi bağımlılıklarla üretildiği makine-okunur değildi; iki ayrı ortamın aynı
ikiliyi üretip üretmediği hiç ölçülmemişti. Üçüncü dış inceleme K-168 ile iki
temiz ortamda eş hash, SBOM, imza ve provenance istedi.

## Karar

1. **İki temiz klon kuralı.** `scripts/surum-artefakti.sh` aynı commit'i iki
   bağımsız `git clone` ağacına alır; her ikisini sabit `rust-toolchain.toml`
   ile, `--remap-path-prefix` (klon→`/zee`, Cargo home→`/cargo`,
   sysroot→`/rust`) ve commit zamanı `SOURCE_DATE_EPOCH` altında
   `cargo build --locked --release --bin dil --bin dillsp` ile derler.
   K-195 Windows MSVC dağıtımında exact host `--target` eklenir ve hedef
   Rustflags `-C target-feature=+crt-static -C link-arg=/Brepro` olur; çıktı target-triple/release
   dizininden alınır. Host build-script/proc-macro bayrakları ayrıdır. İki
   klonun SHA-256'sı eşit değilse artefakt üretilmez.
   K-191: host/rustc/sysroot sorguları da aynı exact pin altında çalışır.
   Pin üretilen commit'ten okunur; caller'ın varsayılanı veya kirli yerel
   pin provenance'a taşınmaz. Ortak seçici ve CI giriş regresyonları
   `scripts/rust-arac-zinciri.sh` ve `scripts/tests/rust-arac-zinciri-test.sh`.
2. **Artefakt kümesi.** Çıktı klasörü `dil`, `dillsp`, `SHA256SUMS`,
   `sbom.spdx.json` (SPDX 3.0.1: kök `dil` paketi, her ikili `software_File`,
   `Cargo.lock`taki her bağımlılık `dependsOn` ilişkisiyle), 
   `provenance.intoto.json` (in-toto Statement v1 / SLSA provenance v1; commit,
   rustc, platform, `SOURCE_DATE_EPOCH`, iki klonun `SHA256SUMS` özeti ve
   `ikiTemizKlonEsit` alanı) ve `GIT_SHA` taşır.
3. **İmza.** Anahtar verildiğinde `surum_artefakti imzala`, `SHA256SUMS`
   üzerinde `zee-surum-imza-v1` belgesi üretir: `"zee-surum-v1\0" || dosya`
   girdisinin Ed25519 imzası, açık anahtar ve `sha256:` kimliği.
   `dogrula` dosya özetini, kimlik↔açık anahtar bağını ve imzayı birlikte
   doğrular; beklenen kimlik verilirse farklı anahtarı reddeder. Anahtar biçimi
   paket yayınının `zee-ed25519-private-v1` dosyasıdır.
4. **CI.** `surum-adayi` workflow'u `v*` etiketinde ve elle koşar: önce
   birleşik güvenlik kapısı (sürekli kip), sonra Ubuntu'da iki klon derlemesi;
   `ZEE_SURUM_ANAHTARI` secret'ı varsa imzalar. Artefakt 90 gün saklanır.
   Platformlar arası (Linux/macOS/Windows) eşitlik iddia edilmez; her platform
   kendi çift-klon kanıtını üretir.
5. **Determinizm sözü.** SBOM ve provenance aynı girdide byte-byte aynıdır;
   zaman yalnız commit zamanından türetilir. Üretici ikili bakım aracıdır ve
   `dil` diline dokunmaz.

K-192: SBOM kök paketinin kaynak konumu ve provenance resolved dependency
URI'si aynı resmî `https://github.com/itwise-bilisim/zee` kaynağına ve exact
SHA'ya bağlanır. Eski `merbay-erp/zee` yönlendirmesi metadata kimliği değildir.
Fork/yerel dal commit'inin resmî depoda erişilebilirliği ayrı yayın kanıtıdır;
metadata üretimi uzak erişimin başarılı olduğunu iddia etmez. Bu bakım değişikliği
Zee kaynaklarının ayrıştırma/tür/yürütme davranışını değiştirmez.

K-195: üç platform kurulum tatbikatı aynı çift-klon üreticisini kullanır;
Windows tek-derleme smoke artefaktı kaldırıldı. Provenance `cargoArgs`,
`crtLinkage` ve `targetRustflags` ile seçilen tarifi kaydeder. Windows
`static` sözü dumpbin listesindeki VCRUNTIME/MSVCP/UCRT importlarının reddiyle
sınanır; işletim sistemi DLL'leri gereklidir. CRT seçimi
[Rust linkage sözleşmesine](https://doc.rust-lang.org/stable/reference/linkage.html)
dayanır. SBOM lockfile/package ilişkileri ve artefakt dosya kümesi değişmez;
sistem bağımlılıkları ayrıca runtime raporundadır. CI eş-hash ve çalışma
kanıtı alınmadan K-195 kapatılmaz.

## Reddedilen seçenekler

- **Tek derlemenin hash'ini yayımlamak:** tekrar üretilebilirlik iddiası
  ölçülmeden verilmiş olur.
- **Harici SBOM/imza aracı (`cargo-sbom`, `cosign`) kurmak:** tedarik yüzeyini
  büyütür; mevcut serde/sha2/ed25519 bağımlılıkları ve `.zep` zincirinin
  şeması yeter.
- **İmzayı CLI'a `dil imzala` olarak eklemek:** CORE FREEZE altında yeni
  kullanıcı komutu açar; sürüm imzalama bakım ikilisinin işidir.

## Sonuçlar

- İlk yerel kanıt (macOS arm64, rustc 1.93.1): iki temiz klonda `dil` ve
  `dillsp` eş SHA-256 verdi; ADR ve günlük exact özetleri taşır.
- Sürüm adayı akışı: `guvenlik-kapisi.sh --surum-adayi` → `v*` etiketi →
  `surum-adayi` workflow artefaktı → `SHA256SUMS` ve imza sürüm notuna
  eklenir. K-169 kurulum/kaldırma tatbikatı bu artefaktı tüketir.
- Cross-platform eşitlik, SLSA L2/L3 iddiası ve managed imza servisi vaat
  edilmez; bunlar açık sınırdır.

K-195 başarısız çift-klon koşusunda isteğe bağlı `ZEE_SURUM_TANI_DIZINI`
ikili farkını araştırmak için a/b byte'larını ve `YAYINLANAMAZ` durumunu
saklar. Bu ayrı tanı kümesi kurulum/release artefaktı değildir; SBOM/imza
üreticisi eş-hash kapısını geçmeden çalışmaz. CI ayrı `yayinlanamaz-surum-farki`
adıyla 90 gün saklar; normal sürüm kümesi değişmez.

K-196 linker/SDK kayıt revizyonu: iki klonun gerçek Rust 1.93.1 link komutu,
çağrılan araç path/byte hash/version ve SDK referansları provenance
`buildEnvironment` içine girer. Unix `-v`, MSVC `/VERBOSE:LIB` ve Rust link
info izi tanı çıktısıdır; `targetRustflags` tarifle eşleşir. Python 3
standart kitaplığı build kanıtı aracıdır. Eksik iki-klon/iki-ikili kaydı veya
bozuk hash/source/schema release yayımlamadan reddedilir. Eski bakım CLI
çağrısı buildEnvironment olmadan çalışabilir; resmi script alanı daima ister.
Driver alt süreçleri ve bütün C build-script toolchain'i bu kısmi kaydın
ötesidir; exact üç platform üretim gözden geçirilmeden kapatılmaz.

GB-033: derleme kayıt aracı yalnız üreticinin kendi taze loglarına güvenir;
harici logdan sürüm programı çalıştırma yetkisi verilmez. Log shell olarak
değerlendirilmez, çözülen araca sürüm argümanı verilir. Kaydın kendisi
release imzasının ya da bütün native toolchain envanterinin yerine geçmez.

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

## K-197 — Temiz Windows runtime provasının sınırı

Release üretimi ve runtime tüketimi ayrı işlerdir. `kurulum-tatbikati`
`mode=clean-windows` ile aynı depodaki explicit source_run/source_sha
artefaktını indirir; ikili hash ve kaynak eşleşmeden Docker/ikili yürütmez.
Temiz Microsoft Nano Server LTSC 2025 digest'i
`4fe6f59e0aadb628a933345eae1f5ab43a2af802969ac0e2ef996c66a3188438`
manifestinde amd64/windows/10.0.26100.33438 gözlenmiştir. Host Docker
daemon ve image OS/mimarisi denetlenir; ContainerUser, network none,
system PATH, read-only iki ikili+kaynak mount'u ile program ve LSP koşar.
Container root geçici ve yazılabilirdir; process isolation host kernel'ini
paylaşır. Nano Server provasının kapsamı desktop/minimum OS veya bütün
IO davranışlarını kapsamaz. Host/prova aracı ve release source kimliği
raporda ayrı tutulur; başarısız koşu da raporla saklanır.

GB-034: unsigned SHA256SUMS yayımlayan kişinin özgünlüğünü kanıtlamaz.
Bu bakım provası yalnız kendi depomuzdaki incelenmiş üretim koşularından
indirilen byte'larla kullanılır; keyfi üçüncü taraf artefakt çalıştırma
ve Windows container'ını kötü niyetli kod için güvenlik sandbox'ı sayma
izni değildir. GH_TOKEN yalnız host download adımındadır; container'a
sadece PATH environment'ı taşınır. ACL yalnız geçici, kamuya açık release
byte'larına Users read/execute verir. Release artefakt kümesi/SBOM/imza
sözleşmesi değişmez. [Resmî Windows Docker kurulumu](https://github.com/actions/runner-images/blob/main/images/windows/scripts/build/Install-Docker.ps1).
Gerçek runtime sonucu gelmeden K-197 tamamlanmış sayılmaz.
