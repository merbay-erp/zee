# ADR-134 — Linux özel değerlendirme paket taşıyıcısı

- **Durum:** kabul, K-276/P06 dar hazırlık; final exact kaynak kabulü açık
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-276, GB-092; halka yayın/üretici/lisans kararı değildir

## İhtiyaç ve sınır

Kanonik iki temiz klon sürümünün Linux ortamında taşınması ve doğrulanması için
compiler değişikliği gerekmez. `linux-package-preflight.py` kanonik teslim raporu,
exact kaynak/ikili/SBOM/provenance ve ELF64/mimari başlığını doğrular; başlık tek
başına ELF geçerliliği, çalıştırma veya taşınabilir ABI kanıtı değildir.

`linux-private-carrier.py` yalnız native Linux kullanıcı sürecinde temiz exact
repo, hash ile verilen kanonik teslim ve fresh owned out kabul eder. Altı
artefakt dosyası aynı rapor hashlerine; kur.sh/kaldir.sh exact Git bloblarına
bağlanır. Sabit `zee-private-carrier-<tamSHA>` paketi yalnız
`/usr/share/zee-private-carrier/<tamSHA>/` altında inert veri taşır. Publisher
olmayan açık placeholder Maintainer kullanılır. `/usr/bin`, service/trigger,
conffile veya maintainer script eklenmez; canonical kur/kaldir ve manifest
activation sahipliğini korur. Source commit zamanı SOURCE_DATE_EPOCH, xz6 ve
en çok iki compression thread seçilir. Dpkg-deb build/info/contents/raw-extract
sonrasında exact dosya kümesi/hash/mode eşitliği olmadan başarılı rapor yoktur.

Üretici ikiliyi, `.dil` kaynağını, kurulum betiğini veya container çalıştırmaz;
dpkg install/remove yapmaz. Çalıştırılabilir apt-native upgrade/autoactivation
protokolü eklenmemiştir. Public apt repo, imza, lisans ve dağıtım kararı açıktır.
Host güvenilir/cooperative süreç sınırıdır, saldırgan aynı UID için sandbox
sözü yoktur. Snapshotlar nofollow/nonblock/regular ve gözlenen byte/entry
sınırı taşır; helper aynı hashlenmiş bytes'tan yüklenir, FIFO sessiz reddedilir.

## Kanıt ve açık işler

Portable12 preflight +9 taşıyıcı hazırlık/negatif vaka, tek Cargo wrapper
önerisidir; bu vakalar native dpkg kurulum veya container kabulü değildir.
Compiler CLI/parser/runtime/LSP fazı tahminle ek kapsam yapılmaz.

Ayrı sahipli ARM64 VM'de resmî checksum Rust1.93.1, temiz eski1367 release
compiler/LSP, gerçek CLI/LSP ve PostgreSQL16.15 bootstrap geçti. Bunlar son
aday S değildir; kanonik iki klon release yerine konulamaz. Gerçek baseline
readelf PT_INTERP/DT_NEEDED ve library hashleri yalnız bu konuk ortamını
kanıtlar. Resmî Debian per-arch manifest/config digestleri doğrulandı; image
katmanı, container ve güvenlik açığı kabulü bunlardan türetilmez.

Final temiz S verilince kanonik release, `.deb` gerçek build/extract ve yalnız
sahipli guest package install/remove, canonical user-prefix activation,
BuildJSON/LSP/repeat/tamper/foreign-sentinel kanıtları ayrı çalışır. Rootless
container için exact image digest, audited runtime closure, owned storage,
subordinate mapping ve gerçek namespace/nonroot testi gerekir. Hiçbir ürün
benimsemesi, fiziksel sunucu veya final P06 kapanışı hazırlıkla iddia edilmez.
