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
