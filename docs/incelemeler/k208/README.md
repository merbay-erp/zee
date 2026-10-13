# K-208 Node bcrypt kabul vektörleri

`node-vectors.json`, Node bcryptjs 3.0.3 `hashSync(password,cost)` ile
üretilmiş sentetik fixture'dır; gerçek kullanıcı hash/parolası yoktur.
Producer [vectors.cjs](vectors.cjs), exact [npm lock](bcryptjs-package-lock.json)
ve Node sürümü saklanır. Salt yalnız test için üretilir. Rust root
`guvenlik::legacy_bcrypt::tests` cost 11 üç önek, cost 14, yanlış parola, cost aşımı ve
bozuk biçim ile ASCII/UTF-8 72/73-byte sınırını kullanır. Üretim Zee
sadece `bcrypt::verify` çağırır, bcrypt üretim intrinsic'i/CLI'sı yoktur.

[Upstream doğrulama kaynağı](https://docs.rs/bcrypt/0.19.3/src/bcrypt/lib.rs.html)
digest için subtle::ConstantTimeEq kullanır. Toplam süre eşitliği iddia
edilmez. Node uyumlu 72 byte doğrulama için normal verify öncesinde
explicit byte sınırı vardır; non_truncating_verify ek NUL'u sayabilir.

RFC-0030/ADR-082/spec-12/katalog/sürüm/V1/master/backlog, public
API/compatibility yüzeyi, mimari owner/SCC ve kalıcı benchmark/soak etkisi
incelendi. Yeni owner, compiler/LSP cache veya kalıcı tampon yoktur.
T034 kullanıcı metni iki algoritmayı kapsar; kod ve tip reddi değişmez.
Yeni benchmark iş yükü açılmaz; ADR-045/049 ve performans tarihçesi değişmez.
Bağımlılık SBOM/provenance, offline vendor ve iki temiz klon kabulüyle
ayrıca kaydedilecektir. Sunucu dağıtımı/ürün hash migration'ı yapılmaz.

## Kapanış

Feature source `2c355efc0e1487dfc8de71579bcce72512ea5c2c`;
clean beyan/ikili/test kaynağı `daaa3ea0803baff5a22ff2a0ea432111bd33240f`. [JSON](acceptance.json),
[crypto](crypto.log), [gerçek worker/CLI/trace](native-ip.log),
[offline vendor](vendor.log) ve [iki-clone](release/accepted.log) kalıcıdır.
742/742 platform vakası geçti; tanı kalitesi 244 vaka/14 sınıf/0 ihlal.
Katman/SCC/API/uyumluluk/fmt/Clippy/spec/katalog/tazelik/güvenlik/sır
kapıları ve exact core-freeze/semantic beyanı geçti.
Bu yerel macOS kabulüdür; signed release, RC fuzz/Miri veya yeni üç
platform release sonucu değildir. Sunucu yükseltmesi ürün tarafında kalır.

İlk Linux CI job 115241425892 işlev eğilim raporunu bayat buldu;
rapor 51 kayıtla üretici tarafından yenilendi ve `--denetle` geçti.
Eşik/fixture değiştirilmedi; başarısız run 38381098416 tarihsel kalır.

Test loglarının yalnız EOF boş satırları kanonikleştirildi; test sonuç
içeriği değişmez. SBOM/provenance/GIT_SHA/SHA256SUMS byte’ları aynen korunur.
