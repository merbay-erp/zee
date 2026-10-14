# K-226 iki temiz klon build kabulü

Kaynak `99f31d77451ee129a796104596da80ce6b928abc`; yalnız kanonik
`scripts/surum-artefakti.sh` iki bağımsız temiz kaynak klonunda release
ikilisini üretip karşılaştırdı. 14 Ekim 2026 yerel macOS/arm64 koşusu exit 0;
native-deployment ve build-environment kapıları da geçti.

- `dil`: `d5e6be03a138248240feb9f5ef8f74b0ec5fa1c1d9f56d2ebbe5d3744c1a44f9`.
- `dillsp`: `8e3d10f2c53c2dacdcce84f5b90e9c0bc00ebfcdbced9bf0e19088b524468e74`.

[SHA256SUMS](SHA256SUMS), [kaynak SHA](GIT_SHA),
[SBOM](sbom.spdx.json) üreticiden aynen korunur.
[Build kanıt özeti](build-proof.json) kaynak/araç/hedef/ikili hash bilgilerini
ve orijinal provenance SHA-256'sını taşır; yerel linker PATH alanlarını
içeren tam provenance depo dışındaki kanıt paketinde korunur. İkili dosyalar bu inceleme klasörüne kopyalanmadı.
Güvenilir yayıncı imza anahtarı verilmediği için çıktı imzasızdır;
mekanik tekrar üretim kanıtıdır. Daha sonraki commit, diğer hedef mimari,
temiz fiziksel platform, insan testi veya signed V1/RC kabulü değildir.
