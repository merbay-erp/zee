# zee — V1 öncesi öncelikli mühendislik backlog'u

Bu belge 1 Eylül 2026 tarihli ayrıntılı dış incelemenin depo içindeki kalıcı,
sıralı iş karşılığıdır. Yeni dil özelliği P0 omurga işleri kapanmadan öne
alınmaz. Bir madde yalnız kodla değil; gerekiyorsa RFC/ADR, normatif spec,
olumlu/olumsuz test, sürüm notu ve v1 kapısı birlikte güncellendiğinde kapanır
(ADR-010 ve kök [AGENTS.md](../AGENTS.md)).

Durumlar: **SIRADA** · **AÇIK** · **KISMEN** · **KAPALI**.

## Uygulama sırası

1. Tamamlanan önkoşul: K-095 registry metadata güveni (378 test).
2. İnsan kanıtı bekleyen kapılar: B-001/K-096 + B-002/K-093.
3. Tamamlanan compiler omurgası: B-003/K-097 ve B-004/K-098 (393 test).
4. Sıradaki makine işi: B-005; ardından B-006 →
   B-010/B-018/B-019/B-020 → B-014–B-017.
5. Üçüncü sprint: B-027/B-028 → B-030/B-031 → B-043/B-044.
6. Sonraki işler aşağıdaki öncelik ve bağımlılık sırasını korur.

## P0 — V1 öncesi dil ve derleyici omurgası

- **B-001 · KISMEN (K-096 deney hazır) — K-016 işlem çağrısı sözdizimini
  kesinleştir.** Serbest üretim, kör A/B/C kartları, dengeli sıra, anonim form
  ve alt grup+teknik karar eşikleri
  [karar paketinde](k016-cagri-karar-paketi.md) önden bağlandı. Gerçek 10 çocuk
  + 5 profesyonel verisi gelmeden tek yüzey seçilmiş veya iş kapanmış sayılmaz.
- **B-002 · AÇIK — K-093 gezme zihinsel modelini kullanıcıyla doğrula.**
  Değer-sonuç imleci makine tarafında tamamdır; V1-P1-05 için 10 öğrenci +
  5 profesyonel eşiği [usability kitinde](usability-kiti.md) bekler.
- **B-003 · KAPALI (K-097) — expression grammar büyüme mimarisini
  kararlaştır.** Primary → erişim/postfix → çağrı → aritmetik → birleştirme →
  karşılaştırma → boolean sırası, tam bölge tüketimi ve yeni yüzey uzatma
  protokolü RFC-0021/spec-20/ADR-002'de bağlandı. Sekiz bağımsız conformance
  testi güçlü birleşimleri, işlem-adı kuyruğunun postfix'i gölgelememesini,
  tam sıfır-argüman çağrısını, en uzun çağrıyı ve fail-closed sınırları korur;
  fiziksel fonksiyon/modül parçalama B-005'in davranış-korumalı işidir.
- **B-004 · KAPALI (K-098) — domain özelliklerini core AST'den ayır.**
  HTTP, sensör, CSRF ve parola yüzeyleri kaynak yazımı değişmeden tek generic
  `Intrinsic { kimlik, argumanlar }` düğümüne indirildi. Tür imzası,
  yetkinlik ve etki ADR-011'deki merkezi kayıtta birleşti; kapalı sensör koşulu
  genel olumsuzlamayı kullanır. Yedi lowering/imza testi ve mevcut davranış
  korpusuyla V1-P0-09 kapandı. Fiziksel runtime/parser handler ayrımı B-005,
  proje/paket izin politikası B-023 kapsamındadır.
- **B-005 · AÇIK — mega fonksiyon büyümesini durdur.** Parser, checker ve
  runtime handler'larını domain/faz sınırlarına ayır; yeni özellik doğrudan
  yüzlerce satır ekleyemez.
- **B-006 · AÇIK — type checker'ı katmanlaştır.** Sembol çözümü, tür çıkarımı,
  flow, çağrı/sözleşme, etki/capability ve dönüş/control-flow ayrılmalıdır.
- **B-007 · AÇIK — yerel çağrı kaynaklı inference'ı sıra bağımsız yap.** Aynı
  çağrıların kaynak sırasını değiştirmek yerel işlem türünü değiştirmemelidir.
- **B-008 · KISMEN — `zee-tr-1` profilini immutable koru.** RFC-0018/spec-13
  kırıcı değişikliği `zee-tr-2`ye yönlendirir; bağımsız uyumluluk denetimiyle
  bu kural CI'da görünür olmalıdır.
- **B-009 · AÇIK — morfoloji conformance korpusunu compiler'dan bağımsızlaştır.**
  Yüzey→kök→ek zinciri→belirsizlik/hata→profil makine-okunur fixture olmalıdır.
- **B-010 · SIRADA — semantic ID modelini kur.** `YapiId`, `IslemId`,
  `SymbolId` newtype'ları indeksleri semantic identity olmaktan çıkarmalıdır.
- **B-011 · KISMEN — gözlenebilir concurrency determinizmini V1 garantisi yap.**
  spec/14 tek-thread semantiği tanımlar; gelecekte multicore yürütmenin gözlenen
  sıra/sonucu değiştiremeyeceği açık compatibility sözüne bağlanmalıdır.
- **B-012 · KISMEN — Ondalık↔binary float dönüşümünü yalnız açık ve kayıplı yap.**
  Bugün implicit dönüşüm yoktur; RFC-0012'deki eski `GerçekSayı↔double`
  kalıntıları FFI gerçeklenmeden temizlenmelidir.

## P1 — Compiler sağlamlığı

- **B-013 · KAPALI (K-096 hazırlığı) — stale v0 parser yorumlarını temizle.**
  `Ayristirici::islem_adlari` artık dosya/birim başlıklarının ön-tarandığını,
  tanım sırasından bağımsız çağrı ve karşılıklı özyinelemeyi doğru açıklar;
  tarihsel karar günlüğü eski davranışı açıkça tarihsel diye korur.
- **B-014 · AÇIK — production `unwrap/expect` audit'i.** Matematiksel invariant,
  malformed AST ve IO/external state sınıflarını ayır; son ikisini tanıya çevir.
- **B-015 · AÇIK — lexer/parser fuzzing.** Her UTF-8 girişte panic-free sözünü
  Unicode, emoji, combining im, girinti, dev sayı ve virgül saldırılarıyla kanıtla.
- **B-016 · AÇIK — morfoloji property/fuzz testini büyüt.** Üret→çöz,
  belirsizliğin sessiz seçilmemesi ve normalizasyon varyantları.
- **B-017 · AÇIK — AST invariant doğrulayıcı ekle.** Test/debug aşamasında
  çözülmüş ad, yapı kimliği ve imkânsız ifade durumlarını doğrula.
- **B-018 · SIRADA — compiler faz sınırlarını kodda görünür yap.** Source →
  Tokens → Parsed AST → Resolution → Typed HIR → Execution/Lowering.
- **B-019 · SIRADA — typed HIR tasarla.** Runtime kaynak belirsizliği yerine
  açık SymbolId ve tür taşıyan gösterimi tüketmelidir.
- **B-020 · SIRADA — her semantic node'da source span garanti et.** `Node<T>`
  ya da eşdeğeri spansiz düğümü yapısal olarak zorlaştırmalıdır.
- **B-021 · AÇIK — LSP odaklı error recovery planı.** Cümle sınırı ve girinti
  güvenilir synchronization point olarak birden çok tanıyı desteklemelidir.
- **B-022 · KISMEN — diagnostic code stability kapısını güçlendir.** Katalog
  birebir testi vardır; sürümler arası identity değişimini fixture ile koru.

## P1 — Runtime ve güvenlik

- **B-023 · SIRADA — web/sensör/HTTP'yi capability modeline bağla.** Ağ,
  sandbox dosya sistemi ve sensör yetkisi merkezi compile/runtime politikası olsun.
- **B-024 · KAPALI İLKE — HTTPS/TLS'yi elle yazma.** Gerektiğinde kilitli,
  battle-tested backend kullan; Zee kriptografi/TLS gerçeklemeye dönüşmez.
- **B-025 · AÇIK — ortak `KaynakSinirlari` modeli.** Recursion, input/body,
  allocation, koleksiyon, görev, deadline ve output bütçelerini merkezileştir.
- **B-026 · AÇIK — cancellation-safety audit'i.** Dosya temp'i, web yanıtı,
  oturum mutation'ı ve diğer yan etkilerin iptal/yarım kalma davranışını testle.
- **B-027 · SIRADA — deterministik IO trace/replay biçimi tasarla.** Event,
  argüman, sonuç ve sıra sürümlü/kanonik bir formatta olmalıdır.
- **B-028 · SIRADA — saat/rastgele semantiğini sürümle.** Seed, zaman ilerleme
  ve gözlenebilir fake-IO davranışı spec sözleşmesi olmalıdır.

## P1 — Paketleme ve supply chain

- **B-029 · KISMEN — registry protokolünü önce normatifleştir.** RFC-0020,
  spec/18 ve spec/19 wire/imza/expiry'yi koddan önce bağladı; K-095 metadata
  doğrulayıcısı çalışır. Taşıma/cache/CLI hâlâ açıktır.
- **B-030 · KISMEN — platformlar arası kanonik paket testi.** `.zep` sıralama,
  metadata'sızlık ve byte tekrarı testli; Unicode dosya normalizasyonu ve gerçek
  iki platform fixture'ı eklenmelidir.
- **B-031 · KISMEN — adversarial archive korpusunu büyüt.** Traversal, mutlak
  yol, symlink, duplicate/fazladan byte ve dev metadata testli; Unicode ayraç
  benzerleri ve ayrı kalıcı saldırı corpus'u eklenmelidir.
- **B-032 · KAPALI — lockfile formatını sürümle.** `proje.kilit` baştan
  `kilit_sürümü 2` taşır; sonraki formatlar migration testi istemelidir.

## P2 — Tooling ve bakım

- **B-033 · KISMEN — archive hijyeni.** `.DS_Store` Git dışında; kaynak/yayın
  arşivleri `__MACOSX`, target ve geçici dosyaları yapısal olarak dışlamalıdır.
- **B-034 · AÇIK — tekrar üretilebilir compiler source snapshot komutu.** Yalnız
  gerekli kaynak/test/Cargo/belgeleri alan, özetli geliştirme arşivi üret.
- **B-035 · AÇIK — function size/complexity trend bütçesi.** Kör hard limit
  yerine kritik modüllerde büyüme raporu ve gözden geçirme eşiği koy.
- **B-036 · KAPALI — Clippy `-D warnings` kapısı.** CI ve yerel toplu doğrulama
  bunu uygular; release işlerinde korunur.
- **B-037 · AÇIK — `cargo fmt --check` kapısı.** Mevcut geniş format borcu
  kontrollü tek seferlik committe temizlenmeden hard gate açılamaz.
- **B-038 · AÇIK — faza özgü test matrisi.** Lexer/parser, tür, morfoloji,
  runtime, concurrency, güvenlik, package ve LSP ayrı raporlanmalıdır.
- **B-039 · AÇIK — semantic regression corpus.** Düzeltilen her compiler bug'ı
  minimal kalıcı `.dil` success/fail fixture'ına dönüşmelidir.
- **B-040 · KISMEN — performans baseline arşivi.** `src/bin/olcum.rs` vardır;
  parser/checker/runtime p50/p95 CI artefact ve trend olmalıdır.
- **B-041 · SIRADA — LSP'yi SymbolId/HIR'a bağla.** Rename/definition özellikle
  morfolojili adlarda parser metin tahmininden kurtulmalıdır.
- **B-042 · KISMEN — formatter parse-equivalence property.** İdempotence vardır;
  `parse(format(x))` semantiği `parse(x)` ile eşit olmalıdır.
- **B-043 · SIRADA — spec↔code kanıt haritası.** Her RFC/ADR/spec'in test
  dosyalarını makine-okunur tek tabloda izle.
- **B-044 · SIRADA — hareketli README sayılarını tek kaynaktan üret.** Golden,
  tanı, RFC/ADR ve test sayıları script/xtask çıktısı olup tazelik testinde
  doğrulanmalıdır.
- **B-045 · KAPALI İLKE — self-hosting'e erken atlama.** P0 omurga ve semantik
  V1 yaklaşmadan Rust bootstrap'tan ikinci compiler'a borç kopyalanmaz.

## Bir sonraki somut kapı

K-095'in atomik güvenlik dilimi kapandı; sıradaki iş B-001'dir. B-001,
“tek syntax seçildi” iddiasıyla değil; doldurulmuş gerçek usability formları,
önceden ilan edilmiş eşik, kabul edilen RFC-0006 revizyonu, grammar/spec ve
golden/anti-example kanıtıyla kapanır.
