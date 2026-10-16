# ADR-144 — dene içinde sabit native web test köprüsü

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): dört native vaka/19 HTTP, iki fresh PostgreSQL ve altı admission ret vakası ile dar yerel kabul; fresh DB bütün HTTP commitlerini geri almaz, genel ürün/platform kabulü açık.
İş: K-286. Ürün gereksinimi ADR-142'de exact kaydedilen etkin `zee-cekirdek`
C5 talebidir. Genel kullanıcı betiği/SQL sandbox'ı değildir.

`dil dene` önce olağan Zee testlerini yürütür. Başarısızlıkta aynı hata exit'i
korunur, native suite başlamaz; ayrı unit receipt bunu `native_started:false`
ile kaydeder. Başarılı olağan test ve native suite sonuçları receipt'te ayrı
kalır. Native yeşil olağan test hatasını örtemez.

Altı açık operator rolü birlikte istenir: `--web-suite`,
`--web-suite-sha256`, `--web-fixtures`, `--test-python`,
`--test-postgres-bin`, `--test-output`. Suite en fazla64KiB normal nofollow
snapshot'tır ve aynı baytlardan hash/decode yapılır. Suite executable, shell,
argv, DSN veya inline SQL seçemez. Fixture SQL yalnız inventory'de pinli,
operator tarafından güvenilmiş fixture dosyasıdır; güvenilmeyen SQL sandbox'ı
yoktur. Python/PG executable yolları suite'ten gelmez.

Köprü yeni owned çıktı köküne derlemeye gömülü `owned-web-tests.py` ve tam
aynı `worker-supervisor.py` baytlarını yazar. Unix'te bu sabit Python kitine
`-I -B` ile exec olur; bir ikinci Rust süreç sahibi üretilmez. Önceden
capture edilmiş compiler absolute path/hash/Build ve Python hash korunur.
Kit her worker/PG sürecini kendi original Popen/WNOWAIT otoritesiyle tutar;
PID dosyasından adoption veya global hizmet yönetimi yapmaz. Flag-only iptal,
mutlak kit180sn, HTTP3sn, kaynak/capture bütçeleri mevcut K271 sınırlarıdır.
Çıktı/runtime retained kanıttır; tüm kaynakların silindiği iddia edilmez.

Her native worker açık `--kapsam` ile başlar; normal TERM'den sonra2MiB
bounded raporu, complete/Build ve aynı pinli fixture snapshot'ları doğrulanır.
Gömülü standart birim hash'leri compiler'ın exact gömülü baytlarından gelen
unit metadata'ya bağlanır; bu birimlerin çağrılmayan dalları paydadan atılmaz.
Normal K271 kullanımı ve kapsam kapalı worker davranışı değişmez.

Gerçek login/logout/PHC, session rotation, CSRF retleri ve byte snapshot
K271 kitinde yürür. Her DB vakası yeni cluster kullanır; bağımsız salt-okur
observer request içi rollback'i ve commit'i ayrı doğrular. Ayrı HTTP
isteklerinin commit'leri sonradan toplu rollback edilmiş gösterilmez.
Genel P08 ürün adoptionı, Windows native ownership, arbitrary suite execution
ve kaynak dışı Rust/HTTP guard branch coverage bu işin kapanış iddiası değildir.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-304 — Gömülü kitin finalizasyon etkisi

ADR-129 revizyonu cleanup/proof yazımı ve son kabul kontrolünde gözlenen
iptal/deadline sonucunu korur; GB-113 dar direct kit kapsamıyla kapalıdır.
Yeni derlemeye gömülen Python baytları değişir; Rust grammar/API/coverage
sözleşmesi değişmez. Temiz `abdc5cf36b4d55021df5edad08cee3a7ad953ebd` direct native4 kabulü, ayrı
`dene`/source-coverage sahibinin çalıştığı anlamına gelmez. S3 köprü kabulü
kendi exact kaynağında kalır; yeni köprü/ürün/platform kabulü ileri sürülmez.

## K-305 — Negatif harness iptal sahipliği

**GB-114 düşük/açık; kaynak hazırlığı.** Ayrı admission negatif harness'i
Supervisor sınıfını doğrudan kullanır; modül main'inin SIGTERM handler'ı
kurulmaz. K-305, ilk owned child öncesindeki flag-only handler ve final
cleanup/proof iptal sonucunun korunması için dar hazırlıktır. Original
Popen/WNOWAIT yetkisi korunur; PID adoption veya genel servis yönetimi yoktur.
Selector init'in iç run try'si önünde olması tek başına kanıtlanmış sızıntı
sayılmaz; main finally mevcut owner cleanup'ını zaten yürütür.

Hazırlık kaydı: Kontrollü hazırlıkta 15 Python metodu ve 27 senaryo geçti; eski sürümde 17 yapısal eksik, iki cleanup hatasında eksik makbuz ve bir terminal süre aşımında yanlış passed ayrıldı. Özgün dış owner eski exit1 ve yeni exit0 ile reaped; gerçek çocuk, OS sinyal teslimi, PG veya ağ kabulü değildir. `K305-PREPARATION.json`
SHA-256: `e445af8d0a82448488bd72d47e1360bba990751db0354f91061a93603c4918b1`.
Hazırlık türü/sonucu makbuzun kapsamıyla sınırlıdır; gerçek native köprü veya
OS sinyal kabulü otomatik çıkarılmaz. Mevcut normal CLI sahibi ve ayrı ignored
bridge sahibi korunur; yeni Rust test sahibi veya üretim compiler/kit değişimi
yoktur. Tam kaynak/faz ve güvenlik kapıları, yeni temiz exact kaynakta bridge
unit/source-coverage + native4 + altı admission negatifi ve son belge kapıları
beklenir. Bunlar tamamlanmadan K-305/GB-114 açık kalır. S3 ve K-304 kanıtları
yeni bridge sonucu yerine kullanılamaz; genel P22/P36/ürün/platform açıktır.
[Koşucu sınırı](../docs/dene-native-web-koprusu.md).
