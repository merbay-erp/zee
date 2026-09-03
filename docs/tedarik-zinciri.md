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
