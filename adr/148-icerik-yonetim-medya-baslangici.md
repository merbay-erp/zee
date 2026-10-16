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

## K-308 — starter harness iptali ve özgün PG sahibi: açık

K-308 / GB-117 açık: İçerik starter native harness’inin yapışkan iptal, publication ve özgün foreground PostgreSQL sahipliği için dar correctness dilimi hazırlanır. Gerçek ordinary baseline8 geçti; exact AST fakeTERM/fake PG-close karşı örneği tek beklenen failure/exit1/reaped verdi. Gerçek OS sinyali, PG/native14 veya gözlenen sızıntı kanıtı değildir. Ayrı focused18 ve PG23 ile birleşik ordinary27 gerçek PASS/exit0/reaped verdi; bu kontrollü sonuçlar native14 kabulü değildir. Tam kaynak kapıları, temiz exact kaynak ve fresh native14 kabulü bekler; diğer üç PG tüketicisi ile genel P23 kısmi kalır.

Bu kaynak/test/belge dilimi aynı atomik uygulama kapsamındadır; kaynak uygulaması veya kabul sonucu bu taslaktan türetilmez. Hedef; ayrı sticky cancellation ledger’ı, cleanup sırasında yalnız H maskesi ve garantili restore, korumalı kısmi handler kurulumu/tüm restore, callback registration ve temp-root sırası, ExitStack sonrası passed, publication iptal/deadline/failure guard’ı ve ilk hata nesnesinin korunmasıdır. Özgün foreground PG/readiness nesneleri, doğrulanmış private root kimliği ve unknown/no-replay sınırı korunmalıdır; belirsiz sahip/root kaldırılmış veya sonradan benimsenmiş gösterilemez.

**Kaynak ve kontrollü kanıt dondurması:** Proposal5 dört test-harness kaynak dosyası bağımsız statik incelemede doğrulandı: sticky ledger/korumalı handler/publication, foreground StarterPgCampaign ve private-root cleanup; native14 kayıt adları/eski assertion çokluğu/özgün host Popen ve ordinary8 korundu. Bu kaynak/test/belge dilimi aynı atomik uygulama kapsamındadır; temiz kaynak veya native kabul sonucu değildir. Gerçek P5 focused18 PASS/exit0/reaped, unittest0.153sn; ayrı PG preflight23 PASS/exit0/reaped (0.224sn). PG23 helper/consumer/preflight baytları P4=P5 olduğundan özgün ilk makbuz korunur, tekrar koşulmuş sayılmaz. İlk focused18 ve combined27 koşuları birer selector TypeError/exit1/reaped olarak tarihsel kalır; bu model seçimi hatası native/runtime/OS sızıntısı değildir. Gerçek P5 fixture2 birleşik ordinary27 PASS/exit0/reaped, unittest0.499sn; yapı old8+focused18+tek nested PG23 wrapper’dır. Public fixture10 kaynak kopyası exact doğrulandı. Üç kabul özgün dış owner errors[]/cancelled[] makbuzlarına bağlıdır;18/23/27 sonuçları toplanıp Cargo/native vaka veya mevcut14’ün yeniden kabulü sayılmaz. Exact Proposal5 `/private/tmp/zee-k300/k308-starter-proposal-5/PREPARATION.json` SHA-256 `cddda1f8bbcd3973fb045fa974c7a6aa927f3e485f2416d2882272f29b585b33`; bağımsız static peer `75b005a3e0e01ad4a073970a49643fc2b66e8f2cce970669cc33448bb48522f8`; actual-results peer `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f`; selector düzeltme kaydı `6a9511ec68a6e9f8f702bb7bb865e443cdf22212e2c99a60ebe3252101c68589`. Dört source pini: `scripts/tests/starter-content-native.py`=`3dcb1da978a82ec9e2710d0d6de5e432a5c5fc2f4ce385c0ad79736fe3cbab50`, `scripts/tests/starter-content-test.py`=`93e7e23625eaf6a2732bcc99bdd86506e5e56a4d4238ba332d242e672cf1c80a`, `scripts/tests/starter-pg-owner-preflight-test.py`=`fd8c3d2b7e7c40feed710c93e410e3ce64f97535cf9a4213f43785acd70e4abe`, `compiler/tests/starter_content_testi.rs`=`d3471b0d129816c1e9c941409796172bb156a03fb905f53f979e9bb8d760d9d4`. Bu kayıtlar actual kaynak/test baytları ve bağımsız root/peer inceleme/özgün owner makbuzlarıyla bağlıdır. Ordinary baseline8, odaklı inert yöntemler ve birleşik ordinary wrapper sonucu ayrı kapsamdır; model sayıları Cargo/native sayısına eklenmez.

Mevcut starter14 SQL/HTTPS/kimlik/medya/pending oracleları ve mevcut ignored Cargo sahibi korunması gereken sözleşmedir; K-308 kaynağında yeniden geçtikleri iddia edilmez. Tarihsel S3 kaynak native14 sonucu kendi SHA’sında kalır; K307 SQL9 veya K306 dispatcher4/HTTP7 bu dilime aktarılmaz. Shared `native-pg-owner.py` helper’ı exact mevcut6e5f baytlarında korunur; starter adapter uyumu ayrıca incelenir. `ZEE_STARTER_NATIVE_OUT` planlanan yalnız internal ignored testin taze mutlak output seçimi olup ürün CLI/API kapasitesi açmaz; mevcut0700 geçici-root varsayılanı korunmalıdır.

PHC/parola/cookie/request/cert/key/DSN özel baytları veya bunların hash’leri public kanıta yazılmaz. Public proof/progress/loglar yalnız izinli yaşam döngüsü/sonuç metadata’sıdır; eski private runtime/kök/PID/credential aranmaz veya benimsenmez. Mevcut route/PHC/session/CSRF/media güvenlik oracleları dar test-harness düzeltmesiyle gevşetilmez.

**Açık kabul zinciri:** actual yeni kaynak+test+bu açık belgeler aynı batch; gerekli tam kaynak kapıları ve temiz tested-source commit/özgün owner makbuzları; ardından aynı yeni clean SHA/tree üzerinde fresh mevcut starter14 native; PG’nin özgün foreground sahibinde normal SIGINT/observed_exit=exit=0/reaped/noFallback ve readiness özgün probe kapanışı ile ayrı host front_terminal exit0/reaped/stderr0 oracleları, public audit ve bağımsız peer; son belge overlay’ine uygun final kapılar ve temiz kayıt commit’i gerekir. Host’un iç native child cleanup’ı mevcut kaynakta H.stop_owned sınırıdır; bağımsız per-worker normal0 public makbuzu veya yeni worker hook kabulü iddia edilmez. Kaynak SHA/tree, yeni model sayıları/sonuçları ve native/final sonuçları bilinmediği yerde uydurulmaz. K308/GB117 açık kalır; bu hazırlık PASS veya closure değildir.

Starter şu anda K307 sonrası açık dört PG tüketicisinden biridir; bu dar iş bittiğinde de kalan üç tüketicinin kapıları ayrı olacaktır. 39 plan statüsü mevcut değerlerinde korunur; P23 kısmi kalır. P20 ve diğer açık ürün/insan/platform/dış girdiler, RC/soak/IDE ve C1/K288/P33/K293 ret sınırları değişmez; yeni compiler özelliği açılmaz.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans iş yükü ve compiler/LSP kalıcı durum değişmez. Native tüketilen `docs/versioned/0.8/icerik-paneli.md` ürün komutları değişmediğinden bilinçli unchanged tutulur; kapanışta source-pinli rehber baytları değiştirilerek native kanıtı bayatlatılmaz. ADR-148 ve bu açık durum kayıtları atomik belge kapsamını taşır.
