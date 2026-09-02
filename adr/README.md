# ADR süreci

Güvenlik/mimari sınır değişikliği ADR ister (master plan bölüm 25).
Şablon: [000-sablon.md](000-sablon.md).

## Planlanan ilk ADR'ler (master plan bölüm 38)

| No | Başlık | Durum |
|---|---|---|
| ADR-001 | [Bootstrap dili: Rust + küçük/kilitli bağımlılık yüzeyi](001-bootstrap-dili.md) | **kabul** (K-088/K-092/K-117 revizyonu) |
| ADR-002 | [Parser: elle yazılmış, yüklem-sonlu dağıtım](002-parser-stratejisi.md) | **kabul** (K-097 katmanlar, K-119 formatter eşdeğerliği) |
| ADR-003 | [İlk yürütme: ağaç-yürüyen yorumlayıcı + IO soyutlaması](003-ilk-yurutme-modeli.md) | **kabul** |
| ADR-006 | [Paket registry güven modeli](006-paket-registry-guven-modeli.md) | **kabul/gerçeklenmiş** (K-094 yayın; K-095 metadata; K-117 kanonik yol; K-135 taşıma/cache/offline; K-136 exact proje/kilit/CLI) |
| ADR-009 | [Dilin adı: zee](009-dil-adi.md) | **kabul** (kurucu yetki devriyle) |
| ADR-010 | [Normatif otorite ve değişiklik bütünlüğü](010-normatif-otorite-ve-degisiklik-butunlugu.md) | **kabul** (K-118 kanıt/sayı tazelik kapısı) |
| ADR-011 | [Core AST intrinsic/yetkinlik sınırı](011-intrinsic-yetkinlik-siniri.md) | **kabul** (K-098/B-004) |
| ADR-012 | [Derleyici fiziksel faz modülleri](012-derleyici-faz-modulleri.md) | **kabul** (K-099/B-005) |
| ADR-013 | [Checker semantik katmanları](013-checker-katmanlari.md) | **kabul** (K-100 katmanlar, K-121 çağrı çıkarımı) |
| ADR-014 | [Semantic kimlikler](014-semantic-kimlikler.md) | **kabul** (K-101 temel, K-120 LSP tüketimi) |
| ADR-015 | [Derleyici faz tipleri](015-derleyici-faz-tipleri.md) | **kabul** (K-102/B-018, K-121 checker-içi keşif sınırı) |
| ADR-016 | [Typed HIR çekirdeği](016-typed-hir-cekirdegi.md) | **kabul** (K-103/K-104 runtime, K-120 LSP, K-121 nihai çıkarım) |
| ADR-017 | [Native ağ I/O kaynak sınırları](017-native-ag-kaynak-sinirlari.md) | **kabul** (K-105/B-025) |
| ADR-018 | [Sınırlı web oturum deposu](018-sinirli-web-oturum-deposu.md) | **kabul** (K-106; production uzantısı K-137/ADR-034) |
| ADR-019 | [LSP girdi sınırları](019-lsp-girdi-sinirlari.md) | **kabul** (K-107/B-047) |
| ADR-020 | [Zorunlu HIR kaynak aralığı](020-zorunlu-hir-kaynak-araligi.md) | **kabul** (K-108/B-020) |
| ADR-021 | [Production panic politikası](021-production-panic-politikasi.md) | **kabul** (K-109/B-014) |
| ADR-022 | [Lexer/parser fuzz politikası](022-lexer-parser-fuzz-politikasi.md) | **kabul** (K-110/B-015) |
| ADR-023 | [AST/HIR invariant doğrulama politikası](023-ast-hir-invariant-politikasi.md) | **kabul** (K-112/B-017) |
| ADR-024 | [Parser hata kurtarma politikası](024-parser-hata-kurtarma-politikasi.md) | **kabul** (K-113/B-021) |
| ADR-025 | [Tanı kimliği kararlılık politikası](025-tani-kimligi-kararlilik-politikasi.md) | **kabul** (K-114/B-022) |
| ADR-026 | [Deterministik IO izi mimarisi](026-deterministik-io-izi-mimarisi.md) | **kabul** (K-115/B-027) |
| ADR-027 | [Sürümlü deterministik IO profili](027-surumlu-deterministik-io-profili.md) | **kabul** (K-116/B-028) |
| ADR-028 | [Kanonik `.zep` yol ve Unicode güvenlik profili](028-kanonik-zep-yol-profili.md) | **kabul** (K-117/B-030/B-031) |
| ADR-029 | [Ondalık ile binary float arasında örtük köprü yoktur](029-ondalik-binary-float-siniri.md) | **kabul** (K-125/B-012) |
| ADR-030 | [Bütün AST ifadelerinde kesin kaynak aralığı](030-kesin-ast-kaynak-araliklari.md) | **kabul** (K-126/B-050) |
| ADR-031 | [Merkezî yetkinlik politikası ve native outbound istemci](031-merkezi-yetkinlik-ve-outbound-istemci.md) | **kabul** (K-127/B-023/B-049) |
| ADR-032 | [Atomik replace metadata koruma politikası](032-atomik-replace-metadata-politikasi.md) | **kabul** (K-128/B-048) |
| ADR-033 | [Merkezî ve değişmez kaynak bütçesi](033-merkezi-kaynak-butcesi.md) | **kabul** (K-129/K-130/K-131/K-132; B-025 kapalı) |
| ADR-034 | [Kalıcı ortak web deposu ve tek-worker süreç modeli](034-web-ortak-depo-ve-worker-modeli.md) | **kabul** (K-137/B-046) |
| ADR-035 | [Protokol-kesin LSP JSON-RPC sınırı](035-protokol-kesin-lsp-json-rpc.md) | **kabul** (K-138/B-051) |
| ADR-036 | [Kanonik web proxy origin sınırı](036-kanonik-web-proxy-origin-siniri.md) | **kabul** (K-139/B-052) |
| ADR-037 | [Byte tabanlı HTTP/1.x istek sınırı](037-byte-tabanli-http-istek-siniri.md) | **kabul** (K-140/B-053) |
| ADR-038 | [Rust tedarik zinciri ve offline vendor kapısı](038-rust-tedarik-zinciri-kapisi.md) | **kabul** (K-141/B-054) |
| ADR-039 | [Hasım hosta karşı sürümlü WASM C ABI](039-hasim-hosta-karsi-wasm-c-abi.md) | **kabul** (K-142/B-055) |
| ADR-040 | [Playground girdi ön-tahsis bütçesi](040-playground-girdi-on-tahsis-butcesi.md) | **kabul** (K-143/B-056) |
| ADR-041 | [Kritik işlev boyutu ve karmaşıklık eğilim kapısı](041-kritik-islev-egilim-kapisi.md) | **kabul** (K-144/B-035) |
| ADR-042 | [Kanonik Rust biçim kapısı](042-kanonik-rust-bicim-kapisi.md) | **kabul** (K-145/B-037) |
| ADR-043 | [Faza özgü ve tam sahipli test matrisi](043-faza-ozgu-test-matrisi.md) | **kabul** (K-146/B-038) |
| ADR-044 | [Sürümlü semantic regresyon korpusu](044-surumlu-semantic-regresyon-korpusu.md) | **kabul** (K-147/B-039) |
| ADR-004 | Bellek yönetimi prototip kararı (GC / ARC benchmark) | bekliyor |
| ADR-005 | Native backend seçimi (Cranelift / LLVM) | bekliyor — Faz 4 |
| ADR-007 | [Telemetri ve gizlilik: araçlar veri toplamaz](007-telemetri-ve-gizlilik.md) | **kabul** |
| ADR-008 | [Self-hosting aşamaları ve geçiş kapıları](008-self-hosting-asamalari.md) | **kabul** |
