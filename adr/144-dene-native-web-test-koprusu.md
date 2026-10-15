# ADR-144 — dene içinde sabit native web test köprüsü

- **Durum:** dar Unix uygulaması; final temiz kaynak kapıları beklenir.
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
