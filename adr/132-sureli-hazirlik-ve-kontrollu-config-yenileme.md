# ADR-132 — Süreli hazırlık ve kontrollü config yenileme (K-274/P13)

- **Durum:** kabul; dar owned Unix host profili, final exact temiz kaynak kabulü açık
- **İlgili güvenlik sınırı:** GB-090

K-251 DB hazırlık kaydı bir kez yazıldığında sonraki DB sapmasını kendiliğinden
izlemiyordu. K-246 host yalnız nesil tazeliğini kontrol ediyordu. K-274 yeni
compiler/grammar/facade ya da kalıcı state eklemeden bu yaşam döngüsünü daraltır.

Supervisor `--readiness-ttl-seconds` verilmezse eski davranışı korur. Explicit
1–3600 saniye TTL'de dosyanın nesil sonrası ve gelecekte olmayan mtime'ı hem
wall age hem ilk gözlemin monoton son tarihiyle sınırlanır. Aynı mtime tekrar
okunması süreyi uzatmaz; saat geri alınması eski monoton son tarihi yenilemez.
Daha eski makbuz yeni makbuzun yerine kabul edilmez. İki health200 tek başına
hazırlık kanıtı değildir. Ready dosyası bir yetkili host yazarın girdisidir;
mtime/hash imza ya da kaynağın güvenilirliği yerine geçmez.

`db-hazirlik --rounds` varsayılan1 tek seferlik eski true kaydını korur.
Explicit2–1000 turda aynı gate lock'ı tüm kampanyada tutulur; her tur önceki
exact kaynak/ikili/contract ve readonly şema sorgusunu yeniden doğrular.
Migration/schema/timeout retleri false yazar ve sonraki sınırlı turda tekrar
kontrol edilir; input/identity/gate/output retleri terminaldir. İki tur arası
2–60 saniye monoton beklemedir. Alt süreç total deadline/64KiB capture
sınırını korur. SIGTERM/SIGINT flag bırakır, child kill/wait sonrası false
kayıtla kapanır. Normal finite tamamlanma da false bırakır; kalıcı daemon veya
sonsuz retry değildir. Başarılı kontrol arasında gate true kalabilir; gecikme
üst sınırını supervisor TTL sağlar. Operatör interval+probe bütçesini TTL'ye
uydurmalıdır. SIGKILL dosyayı değiştiremez; tazelik dolunca hazır olma kapanır.

Config yenileme mevcut `zee.ayar` başlangıç okuması ve HUP nesil değişimiyle
sınırlıdır. Operatör geçerli config'i atomic replace eder, HUP verir; her yeni
worker mevcut CLI önceliğiyle dosyayı tekrar okur. Kaynak modüllerini yürürken
değiştirme ya da genel sıcak kod reload sözleşmesi eklenmez. Bellek oran
sayaçları restart'ta sıfırlanır; sayaç koruma/sıfır kesinti sözü yoktur.

Kanıt kapsamı: TTL6, finite watch10 ve eski strict10 hazırlık vakası; actual
owned iki native worker TTL expiry/recovery ve config100→101 HUP5 kısa vaka.
Owned PG16.15 kampanyası eski13 + tekrarlanan gerçek DB drift/recovery ve
SIGKILL/TTL/SIGTERM sınırlarını toplam15 native vakada ayrıca sınar. Kullanılan temiz native ikili
eski d42f147 kaynak kimliğidir; yeni kaynak commit'inin kabulü ileri sürülmez.
SIGKILL, hazır dosyasını veya o anda çalışan probe alt sürecini kendi başına
temizleyemez; servis sahibi outer owned group/cgroup temizliğini sağlamalıdır.
Bu dilim PID keşfi/adoption ya da anında crash cleanup sözü vermez.

900 saniye host, 1800 saniye soak ve final RC kampanyası ayrıca açıktır.

Bağımsız inceleme R1/R2 sonrasında EOF-terminal bekleme de iptal/deadline
kontrollerini sürdürür; selector kurulum ve close arızasında doğrudan çocuk
kill/wait ve pipe close garantili finally içinde kalır. Dört ek gerçek child
negatifi bu sınırları kanıtlar. Native fixture izleyiciyi kendi yeni owned
grubunda tutar, WNOWAIT ile lideri reap etmeden grubu temizler ve kalan
çocuk/pipe EOF kontrolünü kaydeder. Bu, CLI helperın tek başına SIGKILL
alınca cleanup yapabildiği anlamına gelmez.
