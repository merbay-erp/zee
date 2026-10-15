# ADR-127 — Kopyalanabilir site ve panel başlangıcı

- **Durum:** kabul; K-269 kopyalanabilir pilot; final temiz kaynak ve gerçek kullanıcı kabulü açık
- **Tarih:** 15 Ekim 2026
- **İlgili:** P23/C6, K-203, GB-085, ADR-089/094

P23 talebindeki site/panel başlangıcı için mevcut `projeler/mini-site.dil` ve
`projeler/girisli-panel.dil` kaynakları `sablonlar/` altında kopyalanabilir
proje bildirimiyle sunulur. Mevcut komutlar kilidi oluşturur, kaynağı denetler,
testleri ve yerel HTTP sunucusunu çalıştırır. Yeni compiler/grammar/API,
installer, ortam yetkisi veya outbound hedef eklenmez. Bu örnek ürünün anonim
beş tam site korpusu değildir; `dogfood/` kaydı icat edilmez.

Panel sabit demo PHC özetini kaldırır. Operatör mevcut gizli terminal
`dil parola-özeti` komutuyla 0600 yerel `panel-phc` dosyasını hazırlar; Bash
pipefail/noclobber/umask ve newline kaldırma tarifi kayıtlıdır. Boş veya bozuk
dosya giriş vermez. Hash üretimi başarısızsa kalan boş dosya örtük başarı ya
da otomatik kurtarma sayılmaz. Üretim scripti eklenmez; normal dosya okuma
credential launcher'ın nofollow/tek-link/izin/secret IO redaction profiline
eşdeğer değildir. Hash/not/runtime dosyaları kaynak kontrolünden dışlanır;
notlar yerel veridir, eşzamanlı çok-worker veri yönetimi vaat edilmez.

Gizli ortam okuma alternatifi mevcut P015 sözleşmesinde boş olmayan outbound
origin gerektirir. Yalnız şablonun parolası için ilgisiz ağ yetkisi veya
hedefi açılmaz. Eksik/bozuk normal PHC dosyası native doğrulamada failclosed
kalır; site ve panelin iki farklı capability bildirimi vardır.

Tek Unix Cargo vakası geçici kopyalarda gerçek CLI kilitle/denetle/dene ve
owned HTTP süreçlerini yürütür. Site ana/hakkında/hesap/404; panel anonim
rol reddi, eksik/bozuk PHC, CSRF reddi, yanlış parola, doğru giriş/oturum
rotation, kayıtlı HTML kaçışı ve logout/revoke doğrulanır. Kısa prototype2
kampanyası dirty d53 ikilisine aittir, final temiz kaynak kanıtı değildir.
HTTP helper kalan süreyi recv ve son EOF/parse kontrolünde uygular; kaynak,
ikili ve destek dosyası hashleri başta/sonda eşlenir. Başarısız cohort sabit
failed/type receipt bırakır; credential veya response gövdesi kaydedilmez.
SIGTERM/INT yalnız bayrak koyar ve bounded kontrol owned cleanup'a ulaşır;
SIGKILL/güç kaybında cleanup garantisi yoktur. Signal yolu kaynak incelemesidir,
ayrı actual signal negatif koşusu yapılmış gösterilmez.

GB-085 bilinçli düşük riskli yerel dosya sınırıdır; compiler bugfix ve yeni
tanı yoktur. RFC/spec normları ve 214 normatif kanıt sayısı değişmez; supported
Rust facade, production Rust katman grafiği ve dependency listesi aynı kalır.
ADR-045/049 performans tarihçesine kısa test süreleri taban olarak yazılmaz;
compiler/LSP cache veya yeni kalıcı buffer eklenmez. Gerçek yeni kullanıcı,
Windows/minOS, HTTPS/production dağıtımı, P25 file-native sır aktarımı ve tam
P23 ürün kabulü açıktır. Şablon varlığı insan veya canlı ürün kapısını kapatmaz.
