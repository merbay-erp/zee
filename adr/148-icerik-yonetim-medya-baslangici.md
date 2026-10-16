# ADR-148: İçerik, yönetim ve medya başlangıcı

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): 14 gerçek PostgreSQL/HTTPS vakası ile dar yerel kabul; insan rehber kabulü, üretim/platform ve ürün benimsenmesi açık.
- İş/sınır: K-290, GB-106, P23/C6.
- Rehber: [Zee0.8 başlangıcı](../docs/versioned/0.8/icerik-paneli.md),
  [host API](../docs/versioned/0.8/starter-api.md), [cookbook](../docs/versioned/0.8/cookbook.md).

K-269 mevcut site/panel kopyasını sağladı; generic content CRUD, media upload /
processing / delivery kompozisyonu yoktu. Yeni ayrı şablon mevcut SQL eylemleri,
Argon2 PHC/session/role/CSRF, binary upload, image ve no-clobber file primitive'lerini
birleştirir. Eski iki şablon, compiler grammar/API ve `dil yeni` komutu değişmez;
fresh local creator mevcut CLI kilitle/denetle/dene çağrılarını yürütür.

Native HTTP yanıtı Metin'dir. Binary PNG cevabını ayrı owned HTTPS host verir;
compiler'a binary-response özelliği eklenmiş denmez. Host kernel-created loopback
listener'ını tutar, mevcut ready/nonce/Build/activate protokolüyle issued child'a
verir. Böylece boşaltılmış fixed worker portuna bağlanıp yabancı servisi yanlış
benimsemez. Bilinmeyen upstream/route/method yoktur. Native secure-cookie ve
Host/Origin/Forwarded bağları korunur; local test cert/key explicit operator
kaynağıdır, production PKI/anahtar/yayın veya installer değildir.

Host header/body/output/deadline sınırları ve strict framing uygular. Media
nofollow, tek-link/owner/private, lowercase hash-isim, exact SHA, RGBA8 PNG/320px
zarfındadır. Dosya hash'i içeriğin kökenini imzalamaz; trusted operator source/
config/runtime sınırı hostile same-UID sandbox değildir.

Unsafe request öncesi durable pending yazılır; belirsiz native/DB/filesystem
sonucu replay edilmez ve sonraki mutasyonlar kapalıdır. GET gözlemi sürer.
Açık operator review yeni readonly SQL gözlemi+pending SHA'ya bağlanır. Media
preparing/ready/deleting adımları bağımsızdır; DB+FS atomicity ve exactly-once
vaadi yoktur. Post-unlink fsync exit4, görünür tamamlanma/durability belirsizliğini
sahte paused durumundan ayırır. Orphan kaynaklar otomatik silinmez.

`lifecycle_core.py` byte-identical mevcut sahiplik helper'ıdır; yeni production
process owner kopyası icat edilmez. Orijinal Popen/WNOWAIT group authority,
TERM/KILL/reap sırası korunur; host SIGKILL veya authority kaybından sonra PID
adoption yapılmaz. Bu durumda cleanup geçmişe dönük tamamlanmış gösterilmez.

Yeni normatif dil/spec, compiler tanısı, production Rust modülü/katman kenarı,
public facade veya dependency yoktur. RFC-0012 genel FFI taslağı kalır. Yeni
Host biçimleri uyumluluk fixture kaydına eklendi. Benchmark
ve compiler/LSP kalıcı cache yoktur; kısa prototype süreleri performans tabanı
olmaz. Gerçek novice30min, P08/adoption, fiziksel/diğer platform, remote CI ve
canlı production deployment ayrı kapılardır; teknik starter bu kanıtları üretmez.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-308 — starter harness iptali ve özgün PG sahibi: dar kapsamda kapalı

K-308 / GB-117 dar kapsamda kapalı: Starter kabul harness sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e tam15/temiz commit makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/starter14 frozen9 ve özgün PG/readiness/host/dış owner kabulü doğrulandı. Diğer üç PG tüketicisi açık; P23 kısmi ve 39 plan statüsü kendi mevcut değerlerinde korunur.

**Gerçek kaynak kabulü:** Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e, tree b7f024e38f43bfb92714aa8db210ccc32bf79d49; tam15 kaynak kapısı 1146.264sn ve temiz tested-source commit/özgün owners doğrulandı. Faz998/1023 geçti,0 başarısız,25ignored kendi tam kaynak koşusudur. İlk app1 değişiklik/makbuz öncesi manifest-local-shadow TypeError/exit1/reaped olarak korundu; üç localname düzeltmesinden sonra app2 source4+OPENdocs11 gerçek uygulandı. FakeFD16, native sürücü pure18 ve integration35 ayrı hazırlık sonuçlarıdır; native/Cargo toplamına eklenmez.

**Gerçek native14 kabulü:** Yalnız bu exact kaynakta canonical tek ignored Cargo sahibi/mevcut starter14 geçti; native sürücü 67.326sn. Kaynak346/production175, özgün native14 oracleları, typed release build ve exact tools/runtime pinleri frozen9/root/peer public raw kanıtlarıyla doğrulandı; kalan24 ignored Cargo vakası bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback, readiness özgün probe kapanışı ve ayrı host front_terminal exit0/reaped/stderr0 doğrulandı; private runtime kaldırıldı. Ayrı dış original owner0/reaped/errors[]/cancelled[]; iç nativechild normal0 bağımsız makbuzu iddia edilmez. Host’un iç native child cleanup’ı mevcut kaynakta H.stop_owned sınırıdır; bağımsız per-worker normal0 public makbuzu veya yeni worker hook kabulü iddia edilmez. Mevcut starter14 SQL/HTTPS/kimlik/medya/pending oracleları korunur. Tarihsel S3 native14, K307 SQL9 ve K306 dispatcher4/HTTP7 kendi exact kaynaklarında kalır. Yalnız bu yeni exact kaynakta bir ignored Cargo sahibi/starter14 native kabulü kullanılabilir; kalan24 ignored Cargo vakası bu kampanyada çalışmış sayılmaz.

**Tarihsel kontrollü kanıt:** Proposal5 dört test-harness kaynak dosyası bağımsız statik incelemede doğrulandı: sticky ledger/korumalı handler/publication, foreground StarterPgCampaign ve private-root cleanup; native14 kayıt adları/eski assertion çokluğu/özgün host Popen ve ordinary8 korundu. Bu kaynak/test/belge dilimi aynı atomik uygulama kapsamındadır; temiz kaynak veya native kabul sonucu değildir. Gerçek P5 focused18 PASS/exit0/reaped, unittest0.153sn; ayrı PG preflight23 PASS/exit0/reaped (0.224sn). PG23 helper/consumer/preflight baytları P4=P5 olduğundan özgün ilk makbuz korunur, tekrar koşulmuş sayılmaz. İlk focused18 ve combined27 koşuları birer selector TypeError/exit1/reaped olarak tarihsel kalır; bu model seçimi hatası native/runtime/OS sızıntısı değildir. Gerçek P5 fixture2 birleşik ordinary27 PASS/exit0/reaped, unittest0.499sn; yapı old8+focused18+tek nested PG23 wrapper’dır. Public fixture10 kaynak kopyası exact doğrulandı. Üç kabul özgün dış owner errors[]/cancelled[] makbuzlarına bağlıdır;18/23/27 sonuçları toplanıp Cargo/native vaka veya mevcut14’ün yeniden kabulü sayılmaz. Exact Proposal5 `/private/tmp/zee-k300/k308-starter-proposal-5/PREPARATION.json` SHA-256 `cddda1f8bbcd3973fb045fa974c7a6aa927f3e485f2416d2882272f29b585b33`; bağımsız static peer `75b005a3e0e01ad4a073970a49643fc2b66e8f2cce970669cc33448bb48522f8`; actual-results peer `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f`; selector düzeltme kaydı `6a9511ec68a6e9f8f702bb7bb865e443cdf22212e2c99a60ebe3252101c68589`. Dört source pini: `scripts/tests/starter-content-native.py`=`3dcb1da978a82ec9e2710d0d6de5e432a5c5fc2f4ce385c0ad79736fe3cbab50`, `scripts/tests/starter-content-test.py`=`93e7e23625eaf6a2732bcc99bdd86506e5e56a4d4238ba332d242e672cf1c80a`, `scripts/tests/starter-pg-owner-preflight-test.py`=`fd8c3d2b7e7c40feed710c93e410e3ce64f97535cf9a4213f43785acd70e4abe`, `compiler/tests/starter_content_testi.rs`=`d3471b0d129816c1e9c941409796172bb156a03fb905f53f979e9bb8d760d9d4`. Bu kayıtlar actual kaynak/test baytları ve bağımsız root/peer inceleme/özgün owner makbuzlarıyla bağlıdır. Ordinary baseline8, odaklı inert yöntemler ve birleşik ordinary wrapper sonucu ayrı kapsamdır; model sayıları Cargo/native sayısına eklenmez.

Shared `native-pg-owner.py` exact6e5f korunur. `ZEE_STARTER_NATIVE_OUT` yalnız internal ignored testin taze mutlak output seçimi olup ürün CLI/API kapasitesi açmaz;0700 geçici-root varsayılanı korunur.

PHC/parola/cookie/request/cert/key/DSN özel baytları veya bunların hash’leri public kanıta yazılmaz. Public proof/progress/loglar yalnız izinli yaşam döngüsü/sonuç metadata’sıdır; eski private runtime/kök/PID/credential aranmaz veya benimsenmez. Mevcut route/PHC/session/CSRF/media güvenlik oracleları dar test-harness düzeltmesiyle gevşetilmez.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-gates-1/PROOF.json` | `4ce1698419f7d3aecdc2e13cbfdefaa159689e2d42e3acd79cdafae2e18471d8` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-gates-controller-1.json` | `a6f7950a3753c76839caaa818a9c9bcdda06101f65c3d33929ae89d9af732bb7` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-commit-1.json` | `d68bd9fd28a9bacafcdd5955b8c292ec99b3f0bc8b83c632de44c4c1e59abe29` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-commit-controller-1.json` | `df9be65ff3cd523f08a93617a973c679f1a4746be8e8489146951eb9b681bb97` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-1/PROOF.json` | `feb09e70e89b9946e30331c7ae3b044da0c77f9c5f48ce05c1ff0c18b376d558` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-controller-1.json` | `08471af35cade3240e94f09028f37ea98b7f4389dc29afc973b77ca49fa41138` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-1/native/proof.json` | `1b67d601d6f3fa2eb54c78a89c6041b2e7f8cf1ee5d5ac0629ebc7bbba1ca8d9` |
| root_native_audit | `/private/tmp/zee-k300/K308-NATIVE-ROOT-AUDIT-1.json` | `0fe921fc2c1994bae56773a1185a4fa6ca961a966ca502afc07e9de46d183b7b` |
| native_peer_review | `/private/tmp/zee-k300/K308-NATIVE-PEER-REVIEW-1.json` | `2acfbd8dba35ea13371407b288078f3df5c3277f33c83d11119b2dab2852c856` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-frozen-acceptance-1.json` | `9e7538a35c86a142bfe7de6892a490994c748a908e9ba3094967ba0ca4b5a1a4` |
| source_pins | `/private/tmp/zee-k300/k308-source-pins.json` | `2808178e2ebe9af8af556a7b3fc685d3005d23dd0eab763126dedeff2e762180` |
| closure_template | `/private/tmp/zee-k300/K308-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `e29d24881a61b806bb46eb16456d928c6cedaa9a293251fee8526e99d3595c08` |
| closure_template_review | `/private/tmp/zee-k300/K308-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `e97aca7c28c26b83623c12b714d575b7e5f681c465762982bbed39ff1d67cca8` |
| K308-ATOMIC-APPLICATION2-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-ATOMIC-APPLICATION2-ACTUAL-INDEPENDENT-REVIEW-1.json` | `f4668a08a0614528d8add1ed04570f76d7864b3ec82d3f6acd49f0f14306a2e1` |
| K308-CONTROLLED-RESULTS-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-CONTROLLED-RESULTS-INDEPENDENT-REVIEW-1.json` | `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f` |
| K308-ATOMIC-LOCAL-SHADOW-CORRECTION-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-ATOMIC-LOCAL-SHADOW-CORRECTION-INDEPENDENT-REVIEW-1.json` | `81641b09dd35f20c1017cadadea211abac7a6a06402e026cd820a029c706e784` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K308-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `94dba2a083f1f565de1ce69c83a683ec0d5027466be7faaa74b25628e0e7a328` |

Diğer üç PG tüketicisi, P20/P23 kısmi ve diğer açık ürün/insan/platform/dış girdiler, RC/soak/IDE ve C1/K288/P33/K293 ret sınırları korunur; 39 plan statüsü değişmez. Yeni compiler özelliği yoktur.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans iş yükü ve compiler/LSP kalıcı durum değişmez. Native tüketilen `docs/versioned/0.8/icerik-paneli.md` ürün komutları değişmediğinden bilinçli unchanged tutulur; kapanışta source-pinli rehber baytları değiştirilerek native kanıtı bayatlatılmaz. ADR-148 ve bu dar durum kayıtları atomik belge kapsamını taşır.
