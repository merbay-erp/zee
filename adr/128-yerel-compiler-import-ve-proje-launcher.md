# ADR-128 — K-270 yerel compiler import ve proje launcher

- **Durum:** kabul, dar yerel araç pilotu; final temiz kaynak/platform/live kabulü açık.

## Gerekçe

P07 mevcut K-233 signedcatalog seçicisi exact compiler pinini zaten uygular. Açık yerel sürtünme, operatörün doğrulanmış bundle’ı owned store’a güvenli yerleştirmesi ve her proje için bütün trust parametrelerini elle tekrar vermesidir. Dependency-lock’u compilerpin sanmak veya global fallback eklemek bu sorunu çözmez.

## Karar

Yeni `scripts/derleyici-yerel.py` yalnız explicit açılmış yerel bundle import’u ve owned proje-local `.zee-araclar/zee` launcher kurulumudur. Mevcut selector authenticated_snapshot zincirine optional source bundle ve bounded inventory eklenir; imza/katalog/metadata/privatebyte/build/config kabulü kopyalanmaz veya gevşetilmez. Launcher current derleyici.kilit.json ile aynı seçiciyi yürütür. `proje.kilit` v3 gerçek compiler tarafından doğrulanır, araç bunu yazmaz.

Same-filesystem private staging, owned namespace, exclusive peridentity lock ve target recheck publish protokolüdür. Hash eş existing import idempotent; farklı hedef/foreign launcher overwrite edilmez. Acquired lock cleanup exact held inode kimliğine bağlıdır. Postrename fsync failure visible published=true state’i bildirir ve hedefi silmez. Stale/foreign lock kendiliğinden alınmaz. Güç kaybı/sameUID kötü niyetli namespace saldırısı için mutlak dayanıklılık/sandbox sözü yoktur.

Launcher explicit local çağrıdır, global PATH shim değildir. Operator policy/Python/selector/verifier trust sınırı sürer; private hash-eş snapshot runtime attestation değildir. Yeni receipt/publication biçimleri ve POSIX profil compatibility kayıtlarına eklenir. Secret path/argv alıntılayan yeni tanı yoktur. Uygulama stderr’si mevcut sözleşmeyle aktarılır.

## Kanıt ve sınırlar

Mevcut25 fixture değişmeden ve yeni14 süreç/filesystem/native lock/LSP vakası dış stage gerçek koşusunda geçti; ortak gate kabulü ayrı gereklidir. Gerçek local debug byte’ları ephemeral signed catalog altında denenir; production publisher/two-clean-clone/prod adoption değildir. Root Rust wrapper ve ortak çalışma ağacı fullphase kabulü 933/945 geçti,
0 başarısız/12 explicit ignored sonucundadır; bu temiz exact kaynak kampanyası değildir. İndirme/update/globalinstall/productionkey/Windows/minOS kapsam dışı ve P07 açık kalır. Yeni Rust production veya grammar yoktur; compiler spec/katman/performans normları değiştirilmez.

İlgili: [seçici](../docs/derleyici-secici.md), [yerel araç](../docs/derleyici-yerel.md), [paket normu](../spec/07-birimler.md). Güvenlik kaydı GB-086 aynı atomik değişikliğin owned publish/launcher sınırıdır.
