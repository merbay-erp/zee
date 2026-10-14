# Tekrar üretilebilir sürüm rehberi

K-168/ADR-068'in işletim sözleşmesi. Betik `scripts/surum-artefakti.sh`,
araç `compiler/src/bin/surum_artefakti.rs`, workflow
`.github/workflows/surum-adayi.yml`.

K-191: Host, rustc ve sysroot bilgisi de derlemeyle aynı exact Rust pininden
alınır. Ortak araç zinciri seçicisi caller'ın `RUSTUP_TOOLCHAIN` değerini
kayıtlı sürümle değiştirir; pin `git show HEAD:compiler/rust-toolchain.toml`
kaynağından gelir. `--kirli-kabul` HEAD'i üretir, yerel değiştirilmiş pini
kullanmaz. Yönlendirme regresyonu `scripts/tests/rust-arac-zinciri-test.sh`
ile CI'da çalışır. [Envanter ve açık sınırlar](derleme-ve-bagimliliklar.md).

## Yerel üretim

```bash
bash scripts/surum-artefakti.sh                      # temiz ağaç ister
bash scripts/surum-artefakti.sh --anahtar surum.zee-anahtar --cikti /tmp/surum
```

Betik iki bağımsız `git clone` ağacında aynı commit'i sabit toolchain,
`--remap-path-prefix` ve commit zamanı `SOURCE_DATE_EPOCH` ile derler; `dil` ve
`dillsp` SHA-256'ları eşit değilse hiçbir artefakt yazmaz. Çıktı:

| Dosya | İçerik |
|---|---|
| `dil`, `dillsp` | release ikilileri (ilk klondan) |
| `SHA256SUMS` | `<hex>  <ad>` satırları |
| `sbom.spdx.json` | SPDX 3.0.1 JSON-LD; kök paket, ikili dosyalar, `Cargo.lock` bağımlılıkları |
| `provenance.intoto.json` | in-toto/SLSA v1; commit, rustc, platform, iki klonun özeti, `ikiTemizKlonEsit` |
| `SHA256SUMS.zee-imza` | `zee-surum-imza-v1` Ed25519 imzası (anahtar verildiyse) |
| `GIT_SHA` | üretilen commit |

K-192: SBOM/provenance kaynak URI'si aynı kanonik
`git+https://github.com/merbay-erp/zee@<tam-sha>` değerini kullanır.
Sürüm yayımlanmadan exact commit'in bu resmî depodan erişilebilirliği ayrıca
kontrol edilir; iki yerel klon eşitliği uzak kaynak erişimi kanıtı değildir.

## Doğrulama

```bash
cd compiler
cargo run --locked --bin surum_artefakti -- dogrula \
  --dosya /tmp/surum/SHA256SUMS --imza /tmp/surum/SHA256SUMS.zee-imza \
  --kimlik sha256:<yayıncı-anahtar-kimliği>
sha256sum -c /tmp/surum/SHA256SUMS        # macOS: shasum -a 256 -c
```

## Anahtar

`dil anahtar üret surum.zee-anahtar` ile üretilen `zee-ed25519-private-v1`
dosyası kullanılır; CI'da `ZEE_SURUM_ANAHTARI` secret'ı bu dosyanın içeriğidir.
Açık anahtar kimliği (`sha256:…`) sürüm notunda ilan edilir.

## Sınır

Eşitlik aynı platform ve toolchain içindir; Linux/macOS/Windows artefaktları
ayrı ayrı çift-klon kanıtı taşır. SLSA L2/L3, hosted builder veya HSM imzası
iddia edilmez.

## Sistem bağımlılığı ve kurulu ikili provası

K-192 `calisma-ortami-denetle.py`, kurulu `dil`/`dillsp` checksum'ını ve exact
kaynak SHA'yı doğrular; doğrudan OS kitaplıkları, Rust'sız PATH merhaba-dünya
ve stdio LSP initialize/shutdown/exit sonucunu ayrı JSON raporunda üretir.
`kurulum-tatbikati` üç platform raporunu 90 gün saklar. Bu dosya release
artefakt kümesine eklenmez; SDK provenance veya temiz makine kanıtı yerine
geçmez. Komut/sınırlar [envanterde](derleme-ve-bagimliliklar.md).

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

## K-226 kaynak build kimliği

Build tarihi commit UTC günüdür. `compiler/build.rs` Git varsa
SOURCE_DATE_EPOCH=exact commit epoch ve explicit ZEE_SOURCE_SHA=HEAD ister.
Duvar saati alınmaz; yanlış beyan build'i durdurur. Git olmayan archive
explicit SHA+epoch alabilir; unknown veya kirli kaynak release değildir.
Kanal varsayılanı development/beta; dev seri explicit stable olamaz.
ADR-068 betiğinin aynı commit epoch kuralı ve remap iki-clone eşliği korunur.
