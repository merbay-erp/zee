# Sahipli modül SQL yaşam döngüsü

K-285 / ADR-143 mevcut K-273 modül yöneticisini byte-identical helper olarak
kullanır. Yeni `scripts/modul-sql-yoneticisi.py` ayrı opt-in profildir. Eski
`modul-yoneticisi.py` ile SQL execute/CRUD yetkisi açılmaz. Compiler grammar,
Rust API, genel extension ABI veya RFC-0012 FFI gerçeklenmez.

## Şema ve sorgular

`zee-sql-module-1` tanımı id, label ve 1–8 alan taşır. Alan türü `text`,
`integer` veya `boolean`; SQL identifier yalnız küçük ASCII harf/rakam/alt çizgi,
en çok24 karakterdir. Ayrılmış id/state/schema ve tekrar alan reddedilir.
Kullanıcı değerleri SQL metnine girmez: mevcut Zee PostgreSQL parametreleri ve
sabit cast'ler kullanılır. Metin4096UTF8 byte, kimlik128 karakter, integer signed64
ve boolean gerçek JSON boolean ile sınırlıdır. Liste en çok100 satır döndürür. Yazı receipt’i gerçek affected0/1 sayısını taşır; bulunmayan update/delete değişmiş satır gibi sunulmaz.

Aynı uygulama kökünde modül registry/menü/içerik-türü ve GET liste adaptörü
oluşur; uygulama manifesti açık veritabanı yetkisi taşır. CLI `create`, `read`,
`update`, `delete`, `list` aynı şemanın gerçek Zee eylem/sorgularını çalıştırır.
Mutasyon HTTP endpoint'i/CSRF/auth katmanı eklenmez. Native HTTP yanıtı HTML olduğundan GET adaptörü JSON metnini HTML kaçışından geçirip sabit `<pre>` içinde sunar. CLI read/list stdout'u ayrı gerçek JSON verisidir; HTML-typed cevaba raw JSON koyulmaz.

Profil `zee-sql-module-profile-1`: açık loopback PostgreSQL target ve sabit
`ZEE_MODULE_DATABASE_URL` adı. DSN ve CA yalnız environment'tadır; profil ve
hata çıktısına parola kopyalanmaz. Operator yeni ve kendisine ait boş veritabanı
sağlar; ilk up public relation yokluğunu sorgular. Araç mevcut yabancı tabloyu
benimsemez; namespace çakışmasında CREATE transaction'ı geri alınır. Bu localhost
adresinden mülkiyet veya hostile same-UID sandbox sonucu çıkarılmaz.

## Göç ve veri koruma

Her up/down/reapply yeni pozitif sıralı migration ekler. `dil göçür` mevcut
transaction ve `_zee_gocleri` SHA-256 tarihçesini uygular. Uygulanmış kaynak
silinmez/yeniden yazılmaz; schema fingerprint ve gerçek sütun/type/not-null
metadata'sı denetlenir. İlk up CREATE; down active tabloyu archive adına SQL
ALTER TABLE ile taşır; reapply aynı tabloyu geri taşır. Satırlar korunur.
Down destructive drop/uninstall değildir. Bu profilde herhangi bir modül down
iken managed HTTP launch kapalıdır; diğer CLI modüllerinin durumu ayrı kalır.
Terminal kaldırma, schema evrimi ve keyfi kullanıcı migration SQL'i açılmaz.

DB ve dosya sistemi ortak atomik transaction değildir. EX lock ve active
marker koruması canlı managed host sırasında değişikliği reddeder. DB çağrısı
öncesinde durable `sql-pending.json` yazılır; bilinmeyen commit, hata veya kısmi
projection admission'ı kapalı tutar. `reconcile` göçü yeniden çalıştırmaz:
readonly gerçek metadata + exact bütün applied ledger hash'leri doğrulanınca
hazır projection journal'ını tamamlar. Uygulanmamış/başarısız göç pending'i
otomatik silinmez veya yeniden denenmez.

CRUD yazısında da ayrı durable `sql-write-pending.json` önce yazılır. Native
komut/receipt belirsizse yeni CRUD/launch kapalıdır. `reconcile-write --data`
operator review'u exact pending SHA ve yeni readonly satır gözlemiyle eşleştirir;
`accept-observed-state-no-replay` kararı yazıyı tekrar çalıştırmaz. Bu gözlem
hangi denemenin etkiyi ürettiğini veya exactly-once ispatlamaz. Post-unlink
fsync hatası exit4 görünür tamamlanma/durability belirsizliğidir; sahte paused
veya rollback raporu değildir.

## Kullanım

```sh
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil --profile /absolute/profile.json init 18080
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil up golden/module-sql/news.json
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil --data /absolute/new-item.json create news
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil list news
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil down news
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil reapply news
```

Hata sabit `sql-module-operation-rejected`, exit2; görünür tamamlanma/durability
belirsizliği ayrı sabit ileti/exit4. SQL/DSN/body/path echo edilmez. Başarılı
read/list kullanıcının talep ettiği kendi verisini stdout'a yazar.
Her native işlem private generated Zee project, exact binary başlangıç/son
hash kontrolü ve mevcut bounded lifecycle owner ile çalışır. İşlem başına
kilitle15s/native20s ve capture64KiB vardır; tüm çok adımlı transition için
sert toplam wall-time veya powerloss finally garantisi verilmez. İptal
bayrağı ve held WNOWAIT TERM/KILL/reap sınırı eski helper'dan gelir.

## Kanıt kapsamı

Normal stdlib contract testleri ve ayrı ignored fresh-PG native kampanyası
vardır. Native iki farklı şema, gerçek CRUD/HTTP liste, down/up veri korunumu,
applied drift/collision, commit sonrası kayıp receipt ve DB/FS publication fault
ile readonly recovery'yi ölçer. Alt senaryolar Cargo test toplamını çoğaltmaz.
Örnekler golden tasarım fixture'ıdır; gerçek ürün P08/adoption veya müşteri
migration'ı diye dogfood korpusuna kayıt edilmez. Yeni exact source için clean-S
kampanyası ve root gate'leri ayrıca gerekir.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-307 — SQL kabul harness’i ve foreground sahibi: dar kapsamda kapalı

Fresh exact ikinci kaynakta mevcut tek ignored Cargo sahibi1 başarılı/0 başarısız/0 ignored ve dokuz SQL native vaka geçti; frozen kaynak/native/özgün owner/raw zinciri bağımsız doğrulandı.

Tarihsel ilk sticky dilimde baseline10, ayrı inert13 ve birleşik23 geçti; exact AST fakeTERM/fakecluster.close karşı örneği tek beklenen failure/exit1/reaped kaldı. Gerçek OS/PG sızıntısı gözlemi değildir. Foreground takip2 standalone23 ve birleşik ordinary24 de kendi özgün owner0/reaped/errors[]/cancelled[] makbuzlarıyla geçti. Birleşik24 önceki23+tek nested foreground23 hazırlık yöntemidir;47, yeni Cargo sahibi veya native vaka toplamı değildir.

**İlk kaynak diliminin gerçek ayrı kanıtı:** Tam15 kaynak kapısı ve tam faz998/0/25 geçti; temiz kaynak `216d7e1a2d3012d8bef7f55030d42d83045b3fff`, tree `002ca4938767884b12b0cb395e94142059eedfbd`. Kaynakgate1/commit1 ve özgün owner makbuzları kendi kaynaklarında korunur; bu ilk kaynak kabulü SQL foreground takip2 native9 veya ikinci kaynak kapısı yerine geçmez.

Beş test-harness kaynağı SQL tüketicisini `OwnedLocalCluster` foreground özgün Popen sahibi ve `SqlPgCampaign` adapter’ına bağlar. Ayrı yapışkan iptal ledger’ı, cleanup sırasında H maskesi/garantili restore, korumalı handler kurulumu, tüm handler restore, ExitStack sonrası passed ve publication iptal/deadline/failure kontrolleri korunur. Cleanup90 kabul600 sınırını uzatmaz. Readiness/data-directory/version pinleri ve özgün probe/PG nesneleri korunur; belirsiz sahip veya private root kimliği değişiminde kök silinmez ve sonradan benimsenmez. Dokuz SQL oracle, pending/unknown no-replay ve DB+FS atomik değil sınırı aynı kalır.

`ZEE_MODULE_SQL_NATIVE_OUT` yalnız internal ignored testin taze mutlak çıktı yolu seçimidir; yokluğunda mevcut0700 geçici dizin varsayılanı korunur. Ürün CLI/API kapasitesi açılmaz.

**Yeni exact ikinci kaynak:** `9df1b4abb52372dce86c92255392032b6fa08574`; tree `e7de0cfc2dce9e9c271979cc963c9629c49263e8`. Yeni exact ikinci kaynak tam15 kapısı geçti (1118.889 sn); gerçek tam faz998/1023 geçti,0 başarısız,25 ignored. İlk216d7 kaynak15 sonucu bu sonuca eklenmez. Temiz ikinci kaynak 9df1b4abb52372dce86c92255392032b6fa08574, tree e7de0cfc2dce9e9c271979cc963c9629c49263e8; sourcegate2 tested17 overlay ve sourcecommit2 özgün owner makbuzları eşleşti; kaynak344/production175 korundu.

**Fresh SQL native kabulü:** Fresh exact ikinci kaynakta mevcut tek ignored Cargo sahibi1 başarılı/0 başarısız/0 ignored ve dokuz SQL native vaka geçti; frozen kaynak/native/özgün owner/raw zinciri bağımsız doğrulandı. SQL/HTTP/DDL/down-up/pending/unknown no-replay oracleları değişmedi. İki genel şema, parametreli CRUD/hostile veri, bağımsız readonly gözlem, HTTP/host kilidi, gerçek down-archive-up, kalıcı pending/reconcile ve collision rollback dokuz mevcut vakada doğrulandı; DB+FS atomikliği iddiası yok. Gerçek Rust sürüm günlüğü macOS aarch64-apple-darwin/rustc1.93.1; aynı foreground PG16.15 profili doğrulandı. Aynı kaynak release ikili/typed build JSON ve explicit Cargo/Rust/Python/git4 tool pini, public PG/runtime17 pini başlangıç/son denetiminde korundu. Foreground PostgreSQL aynı özgün Popen nesnesiyle normal SIGINT/observed_exit=exit=0/reaped; readiness kimliği ve tüm özgün probe sahipleri kapandı, fallback yok, private kök kaldırıldı. Native sürücü 60.761 sn, harness 3.672 sn; özgün dış owner0/reaped/errors[]/cancelled[]. Root native audit ve bağımsız peer issues=[]; frozen exact9 öncül kayıt ve sıralı public artifact seti gerçek hash/baytlarla eşleşti.

Yalnız yeni exact ikinci kaynakta mevcut tek ignored Cargo sahibi/SQL9 dar kabulü sayılabilir; diğer24/25 ignored Cargo bu kampanyada çalışmadı. Python kontrollü yöntemler ve farklı SHA’ların sonuçları Rust/native toplamına eklenmez. Diğer dört PG tüketicisi açık; SQL dar diliminin kabulü tüm beş PG sahibinin kapanışı değildir. P20 kısmi ve39 plan statüsü korunur; ürün/adoption/platform ve C1/K288/P33/K293 retleri açık kalır. RC/soak/IDE/diğer fiziksel platform/ürün sonuçları bu kaynağa aktarılmaz.

Bilinçli etki incelemesi: production Rust175/bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü veya compiler/LSP kalıcı durum değişmez.

**Kapanışın koşulu:** K307/GB116 dar kapanışı yalnız exact belge overlay son11 kapısı ve sonraki tested-overlay temiz kayıt commit’i ile iki özgün owner makbuzuna bağlıdır; bu metin kendi başına kapı/commit kanıtı değildir. Dar kapanış yalnız bu exact belge overlay’inin son11 kapısı ve o test edilmiş overlay’i kaydeden sonraki temiz commit/özgün owner makbuzlarıyla geçerlidir. Bu metin kendi commit SHA’sını üretmez veya kendi başına kapı/commit kanıtı sayılmaz.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-2/PROOF.json` | `374b2c08411e4a56950b03c8f927d3c8002743920bfb0862aa1776de84af5a86` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-controller-2.json` | `9656b67c69241742721c657f2a1ed0c92d0ba7604057b896245293b97e4f4251` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-2.json` | `877b4ba036b9d35a4f0275b2709925e4a6841682e2bd7c6c67087835435eb226` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-controller-2.json` | `e647bf72ae26de16717a6b8aa5d1c71ff56e4282c3aa8c4da11d19c3c5364584` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-1/PROOF.json` | `b62fd8cf03bfd4d14fb0453e06ef1a1f0c83abddbf94464928acb0b34b13b98b` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-controller-1.json` | `4496e9ea5bf4812782d756f7f5ce80c86944c7c44a84b97f08879a0e7a62c012` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-1/native/proof.json` | `3dac8e6964993a3b4dcbe8ef36e6aae8ad3832be2e7424b735a143898abece32` |
| root_native_audit | `/private/tmp/zee-k300/K307-NATIVE-ROOT-AUDIT-1.json` | `de1632e288fa29940c29ad3e9bd785ad9013abf44f511448dd5aa31605a263cd` |
| native_peer_review | `/private/tmp/zee-k300/K307-NATIVE-PEER-REVIEW-1.json` | `1020a23986c7a998acedaf99d0efa6289bc0a395b8fa7ce81e00f019ff5ff961` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-frozen-acceptance-1.json` | `e542370e8d951ec0921e268412aeee459eca371645d4d69bca4ed9819293fb1a` |
| source_pins | `/private/tmp/zee-k300/k307-source-followup-pins.json` | `42cab50fc8be5935dc2ac71c115b7e7ef8a011dc5f67887543a6881ca6c97aad` |
| template_record_application_ready_documents | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-DOCS-APPLICATION-READY-1.json` | `ba40001b8cd92e20a2026c980b080fc74d0a995c12187148263a1720b4a65b55` |
| template_record_application17 | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-APPLICATION-1.json` | `8cee65e19ae75e8057755690ed9b58178818fc7e52381c51378de1c93c1abcde` |
| template_record_application_independent | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-APPLICATION-INDEPENDENT-REVIEW-1.json` | `8c353ca282c03e5c1af32d0718d8b685390a0d5a70139901fd11c70934370628` |
| template_record_source1_commit_root_audit | `/private/tmp/zee-k300/K307-SOURCE-COMMIT-ROOT-AUDIT-1.json` | `f8397d4f59a1c72f1d963b0bd3b7d1e0cf4f1e6eb7ac0e8f0df73ceac38971fb` |
| template_record_source1_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-1.json` | `7cd3434d0de144d09249128262025ad082a0c3d46228b175fc64c0ba597a5b86` |
| template_record_source1_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-1/PROOF.json` | `1db8c354449b0d96fa0209afd3b8761f00b142c31b6f082b6c86e1ec73d266a1` |
| template_record_first_source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-controller-1.json` | `6536aa77e7b1f65a2a817de4ef61761c270f5149b6ff3b9c3a0ef4de298205c9` |
| template_record_first_source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-controller-1.json` | `20e9306f6251a5a938a2990002877ee12f0bf8b20024e84087f06b5bdc4a35a6` |

**Kalan yerel işler:** Backlog bağımlılık sırasındaki diğer dört PG tüketicisinin özgün foreground sahiplik/iptal kabulü ve ayrı açık teknik işler sürer; genel P20, ürün/adoption, fiziksel platform ve dış girdiler açık kalır.
