# ADR-080 — Özel Gitsrv CI ve Tier-1 kanıt sınırı

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıt:** K-217, K-216, ADR-043, ADR-071, ADR-088

## Bağlam ve karar

Ürün tarafının tek merkez talebi eski ci/tier1-gitsrv dalındaki Windows
MSVC çapraz derlemeyi değerlendirir. Eski kaynak geçmişi ve self-hosted
GitHub yönlendirmesi ana hatta alınmaz; özel yedekte korunur.

GitHub workflow dosyaları yalnız sabit GitHub-hosted hedeflere gider.
CI_RUNNER/fromJSON ile kurumsal koşucu seçimi yoktur. Ana GitHub ve Enterprise
ayna Actions'ı kullanıcı kararıyla kapalıdır; kurum işi yalnız özel Git2
Forgejo akışında çalışır. Kaynak fetch öncesi exact depo metadata'sında private=true
olmasını ister; açık, bozuk, yetkisiz veya yanlış depo yanıtı kod çalışmasını
reddeder. PR tetiklenmez; ADR-088 kaynak/token sınırı korunur.

Gitsrv Linux faz, kalite, güvenlik ve WASM kabulünü gerçekten çalıştırır.
Windows x86_64-pc-windows-msvc kodu cargo-xwin 0.23.1, LLVM ve sabit Rust
1.93.1 ile clippy ve dil/dillsp bağlama kapısından geçer. Bu CI derlemesi
sürüm dağıtım artefaktı değildir ve Windows runtime testi yerine geçmez.
Windows dosya/kilit/kurulum davranışı için gerçek Windows kanıtı açıktır.
macOS kabulü gerçek Mac'te kalır; çapraz sonuç Tier-1 çalışma sözü değildir.

## Etki ve kanıt

compiler/tests/tedarik_kapisi_testi.rs public runner allowlist'i ve özel
metadata kapısının olumlu/olumsuz gerçek Node yürütmesini korur. Exact
Forgejo/GitHub action pin envanterleri aynı değişiklikte yenilenir.
Normatif dil/spec, tanı, ABI/API ve production Rust katmanı değişmez;
yeni compiler özelliği veya regression dil kaynağı yoktur. Faz matrisi,
README canlı sayıları ve kanıt özeti güncellenir. Uzak çapraz/yenilenmiş CI
sonucu alınmadan geçmiş bir platform kabulü gibi yazılmaz.
