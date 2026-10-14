# K-250–K-252 temiz ortak kabulü

Exact kaynak `021bff47839092706c7698d360ae5face9cd2a19`, öncesinde ve
sonrasında temiz no-hardlinks klonda Rust 1.93.1 ile doğrulandı.
[Bağımsız kabul özeti](independent-review.json) 58 ham dosyanın hash
incelemesini, gerçek ikili kimliğini ve ayrı ortam testlerini kaydeder.
Root ayrıca proof, 18 komut günlüğü, faz referansı ve ikili hashlerini
yeniden doğruladı. Ham proof SHA-256
`6530c7b74d4a2e4460c33a4516c807e4bcccb5ed518485f7664f34e99146ba55`.
Ham günlükler ve owned fixture çıktıları depo dışı kalıcı kanıt dizinindedir.

[Faz](faz-test-matrisi.md): 899/907 başarılı, sıfır başarısız, sekiz ayrı
ortam atlaması. Ayrı PostgreSQL koşusunda TLS/statement-fault7, göç1,
parola/COMMIT11 ve SQL queue11 olmak üzere 30 vaka, sıfır atlamayla geçti.
DB hazırlık13, credential14, worker25 ve Python3.9/3.13'ün her birinde
TR–EN7 native+4 Zee kabulü tamamlandı. Python alt vakaları Cargo toplamına
yeniden eklenmez. fmt/spec204/clippy/sürekli güvenlik/sır ve exact
semantic/core-freeze kapıları geçti.

İlk harness eksik USER nedeniyle macOS ACL testini, ikinci harness uzun
TMPDIR nedeniyle PostgreSQL Unix socket kurulumunu başarısız kaydetti.
Başarısız kanıtlar korunur. Ortam düzeltildi; ikinci koşunun başarılı fazı
aynı exact SHA ile referanslanır, yeniden kaynak kabulü diye etiketlenmez.

K-252 kaynak düzeltmesi `c0f79aded235266f75cda1bda9244d766d8a32cc`;
exact regression fixed_by ve semantic/core-freeze kaydı takip 021bff4
commit'indedir. SMTP slow-trickle testleri normal fazda gerçek native
sender ile yürür. Sistem resolver iptali veya sert toplam wall-time sözü
yoktur. Bu kabul daha sonra eklenen K-253–K-256 kaynaklarına taşınmaz;
onların temiz kontrol noktası ayrıca kaydedilir. RC ASan/Miri, fiziksel
platform, uzak CI, gerçek ürün ve release kabulü değildir.
