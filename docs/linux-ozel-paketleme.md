# Linux özel paketleme hazırlığı — K-276

Bu dar host aracı P06'nın özel değerlendirme dilimidir. Compiler/grammar/API
özelliği eklemez. Public dağıtım, yayıncı/lisans, apt repository/imza ve
apt-native otomatik activation/upgrade protokolü açık kalır.

Önce kanonik `scripts/surum-artefakti.sh` ve iki temiz exact klon çıktısı gerekir:
ikili eş hashleri, SHA256SUMS, GIT_SHA, SBOM/provenance, kaynak ZIP eşliği ve
Git bundle restore/fsck. Tek Linux build bu sözleşmenin yerine geçmez.

`scripts/linux-package-preflight.py --sha TAM_SHA --delivery MUTLAK_TESLIM
--delivery-sha256 RAPOR_SHA256 --arch arm64` sadece sonlu kayıt doğrular.
ELF64 header+mimari tanıması ikilinin çalıştığını veya Linux ABI uyumunu
kanıtlamaz. Teslim raporunun hashini veren çağıran güvenilir kaynaktır;
bu hash bir üretici imzası ya da authenticated release trust değildir.

`scripts/linux-private-carrier.py --repo TEMIZ_REPO --sha TAM_SHA
--delivery MUTLAK_TESLIM --delivery-sha256 RAPOR_SHA256 --arch arm64 --out YENI_DIZIN`
yalnız native Linux/nonroot süreçte çalışır. Fresh out sahibi ve modu denetlenir;
repo/teslim içinde çıktı açılamaz. Teslim/kurulum betikleri hash snapshot ve
Git blob ile bağlanır, başlangıç/son kaynak temizliği tekrar denetlenir.
Dpkg-deb bounded build/info/contents/raw-extract exact inventory/hash/mode
eşliğinden sonra BUILD.json üretir. Eski out üzerine yazılmaz; bir hata
olursa oluşmuş owned out inceleme için korunur, yanlış tamamlandı denmez.

Paket `zee-private-carrier-<fullSHA>` olup yalnız
`/usr/share/zee-private-carrier/<fullSHA>/` altına altı kanonik dosyayı,
exact kur.sh/kaldir.sh ve CARRIER.json'u inert veri olarak taşır. Paket
Maintainer alanı `Private evaluation placeholder <noreply@invalid.invalid>`
olarak açıkça placeholder'dır; kimse yayıncı ilan edilmez. Kod/servis/trigger,
conffile veya `/usr/bin` wrapper eklenmez. Araç dpkg install, activation veya
ikili/container çalıştırmaz. Yapılandırılmış kaynak commit zamanı,
xz6 ve en çok iki compression thread kullanılır.

Sadece sahipli disposable Linux kabulünde ve ayrı gözden geçirilmiş BUILD/deb
hashiyle install/remove yapılabilir. Önce exact paket kaydı/hedef yol yokluğu ve
yabancı sentinel kaydedilir; var olan Zee yolları üzerine yazılmaz. Kullanıcı
ikilisi activation'ı embedded kur.sh ile fresh owned prefix ve kurulum-v1.tsv
üzerinden; kaldırma embedded kaldir.sh ile olur. BuildJSON/LSP/repeat/tamper ve
sentinel kanıtı ayrıca gerekir. Taşıyıcıyı kaldırmadan önce user-prefix manifest
kaldırılır; yalnız exact paket kaldırılır, autoremove/foreign purge yapılmaz.

`linux-runtime.Dockerfile.in` optional template'tir; image builder değildir.
BASE resmî per-arch immutable digest olmalı; runtime-rootfs gerçek readelf
PT_INTERP/DT_NEEDED/version, package/hash/license inventory ile onaylanmalıdır.
OpenSSL/glibc/libgcc/CA closure bir base tag'den tahmin edilmez. Offline build,
nonroot USER65532, kapalı ağ CLI/LSP ve sahipli runtime cleanup gerçek kanıt
ister. Rootless namespace prerequisite veya base manifest/config doğrulaması,
Zee container kabulü değildir. Hiçbir privileged container/sysctl/host mount
veya host Docker/global service kurulumu bu recipe'nin parçası değildir.
