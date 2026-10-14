# Rust tedarik zinciri ve offline derleme rehberi

Bu rehber K-141/ADR-038'in işletim sözleşmesidir. Zee paket registry'sinin
imzalı `.zep` zinciri RFC-0020/spec-18/19'dadır; burada yalnız Rust ile yazılmış
bootstrap compiler ve fuzz araçlarının üçüncü taraf bağımlılıkları ele alınır.

K-151/ADR-048 bu sınıra GitHub Actions yürütme kodunu da ekler. Cargo paketi
kilitli olsa bile hareketli bir action tag'i değişmeyen commit'te farklı kod
çalıştırabileceği için action pinleri ayrı immutable kayıt taşır.

## GitHub Actions pin sözleşmesi

Bütün `.github/workflows/*.yml|yaml` `uses:` değerleri, depo içi `./` action
hariç tam 40 küçük-hex commit SHA kullanır. İncelenen sürüm etiketi, SHA ve
resmî kaynak deposu
[`github-actions-pinleri-v1.tsv`](github-actions-pinleri-v1.tsv) içinde
workflow kullanımıyla birebirdir. `@v4`, `@stable`, `@main` ve kısa SHA
kapıdan geçmez.

`.github/dependabot.yml`, `github-actions` ekosistemini her pazartesi izler.
Dependabot PR'ı otomatik birleştirilmez. İnceleyen kişi:

1. action'ın resmî release/tag ref'inin önerilen commit'i gösterdiğini;
2. sürüm notu ve izin/değişen davranışını;
3. workflow'daki SHA ile pin TSV'sinin birlikte güncellendiğini;
4. `tedarik_kapisi_testi`, tam faz matrisi ve ilgili platform işlerini

doğrular. Checkout kalıcı GitHub kimliği bırakmaz (`persist-credentials:
false`); workflow token'ı yalnız `contents: read` yetkisindedir. Runner
image'ının `*-latest` hareketliliği bu pinin verdiği söz değildir ve K-169
cross-platform release tatbikatında ayrıca ele alınır.

## Sürekli kapı

`.github/workflows/tedarik.yml` her push, pull request ve günlük takvimde sabit
Rust 1.93.1 ve `cargo-deny 0.20.2` ile iki grafiği denetler:

```bash
cd compiler
cargo deny --locked check -D warnings
cargo deny --manifest-path fuzz/Cargo.toml --config deny.toml \
  --locked check -D warnings
```

Kapı güncel RustSec veritabanını, izinli SPDX lisanslarını, duplicate/wildcard
politikasını ve kaynak kökenini birlikte denetler. Advisory ignore kümesi
boştur. Yeni istisna exact crate+sürüm ve satır içi teknik gerekçe ister;
istisna grafikten düşünce `-D warnings` onu bayat kayıt olarak reddeder.

`deny.toml`deki lisans listesi yalnız kullanılan bağımlılıkların kabul
politikasıdır. Zee'nin kendi lisansını seçmez. Hukuk/topluluk kararı verilene
kadar compiler `publish = false` olduğu için yanlışlıkla crates.io'ya
yayımlanamaz.

## Lock ve checksum sözleşmesi

- `compiler/Cargo.lock` runtime/compiler grafiğinin,
  `compiler/fuzz/Cargo.lock` fuzz aracının exact çözümüdür.
- Her registry paketinin crates.io kaynağı ve 64 küçük-hex SHA-256 checksum'ı
  bulunur. Kaynaksız workspace paketi bu kurala girmez.
- CI'daki normal Cargo komutları daima `--locked` çalışır. Lock eksik, bayat
  veya değişmek zorundaysa iş hata verir; kilitsiz fallback yoktur.
- Bağımlılık güncellemesi elle ve ayrı incelemeyle yapılır; iki lock'taki fark,
  advisory/lisans/source sonucu ve release notu birlikte gözden geçirilir.
- Action güncellemesi de bağımlılık PR'ıdır; immutable SHA, insan-okur sürüm
  yorumu ve pin kaydı birlikte incelenmeden birleştirilmez.

K-163 PostgreSQL adaptörü root crate'in bağımlılık kapanışını genişlettiği için
fuzz path dependency'sinin ayrı lock'u da aynı geçişli PostgreSQL grafiğine
yenilendi. `cargo deny --locked` fuzz kapısı bu eşleşme olmadan fail-closed
durur; root lock güncelken fuzz lock'unun bayat kalması kabul edilmez.
F031'in `native-tls`, `postgres-native-tls`, `r2d2` ve `r2d2_postgres`
eklemesinden sonra iki lock yeniden çözüldü; iki gerçek vendor üretiminde 163
paket aynı
`6adafc91e5c1283cd030c21a36e9ab35e3f71ca95275ccaee32b6b590e79456e`
manifest özetini verdi ve boş Cargo home ile compiler+fuzz offline derlendi.

K-191: Vendor ve boş Cargo home derlemeleri de çalışma ağacındaki
`compiler/rust-toolchain.toml` exact pinini kullanır; caller'ın Rust
varsayılanına bırakılmaz. Araç zinciri/linker/SDK önceden kurulu olmalıdır;
vendor ağacı bu araçları içermez. [Derleme envanteri](derleme-ve-bagimliliklar.md).

## Gerçek offline vendor kanıtı

Depo kökünden:

```bash
bash scripts/offline-vendor-denetle.sh
```

Betik compiler ve fuzz manifestlerini birlikte iki kez
`cargo vendor --locked --versioned-dirs` ile geçici klasöre açar. Her dosyanın
göreli yolu ve SHA-256 özeti sıralı manifest olur; iki üretim byte-byte aynı
değilse kapı kapanır. Sonra boş bir `CARGO_HOME`, ayrı target klasörü,
`CARGO_NET_OFFLINE=true` ve `--offline --locked` ile iki proje gerçekten
derlenir. Kullanıcının önceden dolu Cargo cache'i bu kanıtı maskeleyemez.

Geçici vendor klasörleri iş sonunda silinir ve Git'e alınmaz. Offline release
artefaktı gerektiğinde aynı komutla üretilen vendor ağacı, iki lock dosyası ve
betiğin yazdığı manifest SHA-256 özeti birlikte saklanır. Artefaktın kendisi de
release checksum/imza zincirine girer; yalnız bir geliştirici klasöründen
kopyalanmaz.

## Bir advisory çıktığında

1. Advisory kimliğini ve etkilenen inclusion graph'ı kaydet.
2. Güvenli sürüme kontrollü yükselt; lock farkını incele.
3. İki `cargo deny` komutunu, offline vendor kapısını ve tam kalite matrisini
   çalıştır.
4. Doğrudan düzeltme yoksa risk, erişilebilir yüzey, geçici azaltım ve kesin
   kaldırma koşulu belgelenmeden ignore ekleme. Süresiz/genel ignore yasaktır.

Hareketli RustSec verisinin dün yeşil olan commit'i bugün kırması beklenen
güvenlik davranışıdır. Tekrar üretilebilir olan dependency çözümü ve vendor
içeriğidir; güvenlik bilgisini geçmiş tarihe dondurmak değildir.

## K-184: Public kaynakta sır taraması

Public görünürlük bilinçli tercihtir. `tedarik.yml` geçmişi tam indirir ve
Gitleaks 8.30.1 Linux x64 arşivini SHA-256
`551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb`
ile doğrular. [Resmî sürüm](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1)
checksum listesiyle karşılaştırılmıştır. `scripts/sir-taramasi.sh` tam erişilebilir
geçmişi ve çalışma ağacını ayrı tarar; `--staged` commit öncesi kullanım içindir.
Gerçek değerler maskelenir, repo içi inline allowlist/ignore bastırmaları kullanılmaz.
Yeni harici Action veya üretim Rust bağımlılığı eklenmez. GitHub secret scanning
ve push protection ek korumadır; tarayıcı tüm kodlanmış/ikili sırları bulma sözü vermez.

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

## K-204 — Immutable sır taraması yanlış pozitifleri

8d39c17 kaynak commit'inde dört belge satırındaki kamuya açık SBOM ve
provenance terimleri generic-api-key kuralıyla yanlış eşleşti.
[Exact kayıt](sir-yanlis-pozitif-v1.tsv) yalnız commit/dosya/kural/satır
fingerprintlerini, kaynak satırı SHA-256'sını ve gerekçeyi taşır.
`scripts/sir-yanlis-pozitif-denetle.py` immutable Git blob satırını doğrulamadan
scanner ignore dosyası üretmez; bozuk kayıt, tekrar, wildcard ve bayat hash
reddedilir. Kaynak satırı yeni belgelerde yeniden ifade edildi.

Default Gitleaks kuralları, --redact ve --ignore-gitleaks-allow korunur.
Kayıt bütün dosyayı/dizini/kuralı veya gelecek commit'i hariç tutmaz. Geçmiş
ve çalışma ağacı taraması geçti; runtime üretilen sentetik anahtar hâlâ
generic-api-key bulgusu ile exit 1 verir ve çıktı REDACTED'dır. Dört
Python ret/pozitif test CI sır-taraması işinde Gitleaks kurulduktan sonra
koşar. Gerçek sır döndürme/kaldırma süreci SECURITY politikasında değişmez.

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

Dependency retention ref `dependency-retention/postgres-protocol-c4e9d03616fe`
clean kabul kaynak ab8e97e'yi tutar; dependency c4e9d03 bunun ancestor'ıdır.
Squash/branch temizliği bağımlılık commitini reachable kümeden çıkaramaz.
Ref silinmez/taşınmaz; Zee `v*` etiketi veya binary publish değildir.

### K-214: Anonim açık depo kanıtı

Müşteri ve altyapı kimliği anonimleştirildi; yeni içerik hash politikasıyla
CI içinde denetlenir. Eski geçişler maskeli dökümle ayrı kullanıcı kararını
bekler. Tarihsel ölçümler eski kaynak kimliklerini korur; yeniden açılış ve
GitHub PR önbelleği temizliği ayrıca onay/teyit ister. [Kayıt](incelemeler/k214/README.md).

### K-215: Path yamanın build kimliği

Önceki exact self Git pin tarihsel tedarik kanıtıdır. Güncel compiler/fuzz
`postgres-protocol` 0.6.12+zee.2'yi depo içi path yamasıyla derler; paket
baytları ve `patches-v2` kanonik SHA-256'sı aynıdır. Kayıtlı kaynak SHA'sı
anonim soy ağacı bilgisidir. Güncel SBOM URL'si gerçek build SHA'sını taşır
ve hem o commit ağacı hem derlenen disk ağacı kayıtlı hash'le eşleşmeden
üretilmez. Git kaynakları deny istisnası olmadan reddedilir. Eski ikililer
ve imzaları geçmiş kaynaklarına aittir; yeni kimlik adına sunulmaz.

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

## K-220 — Exact sentetik TLS test kimliği

[Fixture kaydı](sentetik-tls-fixture-v1.tsv) ve ADR-091/GB-054: directory taramasındaki pkcs12-file eşleşmesi yalnız bilinen localhost test kimliğidir. scripts/sir-yanlis-pozitif-denetle.py tek sabit dosya/kuralı, immutable Git blob’u ve kaynak/kopya SHA-256 eşitliğini doğrular; yalnız o geçici kopyanın exact fingerprint’i eklenir. Geniş yol, kural kapatma veya başka key dosyası istisnası yoktur. Yedi olumsuz/olumlu dedektör testi kurulu Gitleaks ile çalışır; başka PKCS12 ve API anahtarı hâlâ reddedilir. Action pinleri, Rust/Cargo graph’ı ve native TLS doğrulaması değişmez.

## K-238 — Güncel rustls advisory yaması

RUSTSEC-2026-0346/0347/0348/0349 ortak patched alt sınırı rustls 0.23.46dır;
compiler ve fuzz lock'ları birlikte güncellenir. Ureq 3.4.0, manifest ve
advisory ignore kümesi korunur. [ADR-101](../adr/101-rustls-advisory-yamasi.md)
custom Debug-session, ECH, Brotli certificate decompression ve hybrid KX
koşullarını gerçek ring feature-tree'den ayırır; dört ayrı Zee exploit'i
veya genel sandbox iddiası verilmez. İki deny ve offline vendor kabulü yeni
kilide aittir. Canonical artifact/SBOM mevcut üreticiyle yeniden üretilir;
eski imzasız iki-klon/provenance kaydı tarihsel olarak kalır.
