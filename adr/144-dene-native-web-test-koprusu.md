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

**GB-114 dar kapsamda kapalı.** Supervisor sınıfını doğrudan kullanan admission
harness'i ilk child öncesinde kendi flag-only SIGTERM/INT handlerlarını kurar.
İptal, terminal süre sınırı ve cleanup/proof/son kontrol hataları failed/nonzero
sonucunda korunur. Original Popen/WNOWAIT yetkisi sürer; PID adoption veya genel
servis yönetimi eklenmez. Selector init yerleşimi tek başına gözlenmiş sızıntı
değildir; önceki main finally owner cleanup'ını zaten yürütüyordu.

Hazırlık kaydı: Kontrollü hazırlıkta 15 Python metodu ve 27 senaryo geçti; eski sürümde 17 yapısal eksik, iki cleanup hatasında eksik makbuz ve bir terminal süre aşımında yanlış passed ayrıldı. Özgün dış owner eski exit1 ve yeni exit0 ile reaped; gerçek çocuk, OS sinyal teslimi, PG veya ağ kabulü değildir. `K305-PREPARATION.json`
SHA-256: `e445af8d0a82448488bd72d47e1360bba990751db0354f91061a93603c4918b1`.

Mock türü ve gerçek OS sinyal teslimi ayrımı korunur. Mevcut normal CLI ve ayrı
ignored bridge sahibi değişmez; yeni Rust test sahibi veya production
compiler/kit/Supervisor davranışı eklenmez.

Kaynak öncesi15 kapı: 15/15 geçti; faz 998/1023 başarılı, 0 başarısız ve 25 ignored; faz komutu 839,411 sn, toplam 1045,134 sn; Clippy ve sürekli güvenlik geçti, 194 kaynak pini korundu, özgün dış owner exit0/reaped.
Temiz `40788a3d0a9f0d43083130bb0f062871861787db`, tree `9c1ea8dffa61d91f5946eff32a9c4cf1c124759e`, üzerinde tek mevcut bridge
Cargo sahibiyle unit kaynak kapsamı (ürün paydası14), native4 ve altı admission
negatifi birlikte geçti: Beş kaynak kapsamı raporu ham edge kayıtlarından yeniden hesaplandı: unit ürün 4/14; site_snapshot 0/12, panel_session_csrf 8/16, db_committed_and_request_rollback 3/6 ve db_fresh_case_isolation 0/6. Her raporda test paydası0 uygulanamaz; complete=true rapor bütünlüğüdür, yüzde100 kapsam değildir ve farklı raporların paydaları toplanmaz. Mevcut tek Cargo sahibi 1 başarılı/0 başarısız/0 ignored (4,63 sn); dört native senaryo, 19 HTTP isteği, altı SQL gözlemi ve iki ayrı fresh PG kümesi doğrulandı; native sürücü 20,169 sn, dış owner 21,076 sn.
Altı gerçek CLI admission negatifi geçti; installed=true, cancelled=false ve owned_groups_closed=true. Handler restoration ayrı kontrollü hazırlık kanıtıdır; gerçek OS iptal teslimi gözlemi ileri sürülmez. Özgün Popen/WNOWAIT süreç sahipliği ve boş worker/PG/komut grubu makbuzları doğrulandı; dış owner exit0/reaped. Her PG child için bağımsız normal SIGINT/exit veya private runtime dizini kaldırma gözlemi ileri sürülmez; 205 kaynak ve 17 runtime pini korundu
Bağımsız inceleme yedi özgün makbuzu, ham günlükleri, 12 public dosyayı ve beş kapsam raporunu doğruladı; E/k305-final-audits-1/INDEPENDENT-REVIEW.json SHA256=421facd52cc777ae0d34a480a3e971b2df6a6c63789eba64b05bcfb40d622d6e

Son 11 belge/güvenlik kapısı kayıt commit önkoşuludur; gerçek sonuç E/k305-final-gates-1/PROOF.json ve E/k305-final-gates-controller-1.json ile izlenir. Bu metin kapı sonucu yerine geçmez.
Unit/worker kaynak raporları ayrı kalır; C8 direct kit/S3 kanıtı yeni kaynak
sonucu sayılmaz. Genel P22/P36/gerçek ürün/platform kapsamı kısmi, C1/P33 reddi
korunur. [Koşucu sınırı](../docs/dene-native-web-koprusu.md),
[exact makbuzlar](../docs/urunlesme-k298-kabul-20261016.md).
