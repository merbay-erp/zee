# ADR-098 — Yerel exact compiler seçicisinin güven sınırı

- **Durum:** kabul
- **Kapsam:** K-233/P07/A4 minimal pilot dilimi.
- Bağlam: ürün sürüm başına/site başına elle symlink ve systemd yamasıyla
  farklı compiler taşır. Paket `proje.kilit` v3 compiler seçimini içermez.
- İlgili kararlar: ADR-068 tekrar üretilebilir sürüm, ADR-071 kurulum,
  K-226 BuildJSON, K-234 runtime ayar capability; GB-059 kabul sınırı.

## Karar

Proje `derleyici.kilit.json` sürümlü ayrı pin taşır. Pin yalnız exact
source SHA, SemVer, kanal ve platformu seçer. Proje executable path,
publisher/key veya verifier yönlendiremez. Bilinmeyen pin/kanal/platform,
yinelenen kayıt, symlink/path escape ve eksik paket reddedilir; latest,
global veya eski compiler fallback'i yoktur.

Kaynak seçim yalnız yerel paket deposundadır; network/kurulum yapılmaz.
İmzaya dayalı ilk güven kökü operatörün dışarıdan verdiği katalog key
kimliği ve trusted `surum_artefakti` executable hash'idir. Crypto mevcut
Rust Ed25519 aracına bırakılır; Python yeni crypto gerçeklememiştir.

Mevcut release imzası binary SHA256SUMS'a bağlıdır; SBOM/provenance ve
kanal yalnız bu imzaya dayanarak authenticate edilmiş sayılmaz. İmzalı
operatör katalogu bu sidecar'ların exact hash'lerini ve kanalı bağlar.
Katalog publisher allowlist'i bu authenticated politika içindedir.

Binary hash/signature doğrulanmadan aday compiler çalıştırılmaz. Exact
SPDX/SLSA metadata eşliği sonrası bytes özel snapshot'a alınır; compiler
own BuildJSON source/version/date/channel kimliğiyle seçilen kaydı
doğrular. Snapshot source paketinin sonradan değiştirilmesinden ayrıdır.
Unknown/dirty/development yalnız explicit pilotla, bütün authentication
kapıları korunarak kabul edilir. Stable unknown/dirty kabul edilmez.

`zee.ayar` bulunan seçilmiş projede, aynı authenticated compiler'ın
K-234 capability query'si `zee-ayar-1` ve unknown-security reject
sözleşmesini bildirmeden launch yapılmaz. Pilot bu kapıyı bypass edemez.
Çalıştırma hedefi proje köküne bağlanır; başka proje hedefi geçilemez.

## Kabul edilen ve açık sınır

GB-059: trusted publisher/catalogue key'nin doğruluğu, catalogue yayınının
freshness/revocation ve gerçek iki-clean-clone provenance üretimi bu yerel
seçicinin kendi kanıtı değildir. Seçici hash, signature ve eşlik kontrolünü
yapar; authenticated yayıncı beyanını bağımsız build kanıtı diye yeniden
üretmez. Anahtar çalınması veya yetkili imzacıya karşı koruma iddia edilmez.

Production release/platform, signed release katalogunun işletimi,
indirme/kurulum/apt/container, global `dil` shim, `dil kur/kullan`, package
lock birleşimi ve gerçek0.7 rollback ayrıca açıktır. Yeni grammar ve
desteklenen `api::v1` yüzeyi bu araçla genişlemez.

## Kanıt

17 çevrimdışı vaka: iki synthetic signed paket, gerçek mevcut Rust
Ed25519 doğrulayıcısı, ayrıca actual local debug compiler/LSP kopyası.
Test anahtarı geçicidir; production key/sır kullanılmadı. Synthetic
SBOM/provenance yalnız protocol binding'i kanıtlar, release geçişi değildir.

Bağımsız review sonrası readonly nonblock/identity/bounded read, role göre
JSON sınırı/recursion ret, loader ortam temizliği, host bayrağı ret ve
pin temp hata temizliği 25 gerçek kabul/retle kanıtlandı. Cargo kapısı
`compiler/tests/derleyici_secici_testi.rs` aynı derlenmiş verifier ve compiler
üzerinden izolasyonlu Python (`-I`) ile kampanyayı sıfır atlamayla yürütür; Windows pilot fixture açıkça
ignored olup fiziksel/native kabulü sağlanmış gösterilmez.
