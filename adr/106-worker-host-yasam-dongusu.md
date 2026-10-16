# ADR-106 — Worker host pilotu ve sahipli süreç temizliği

- **Durum:** kabul, sınırlı POSIX pilot
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-246/P13, GB-069

## Bağlam ve karar

Gerçek B5 talebi hizmet hazırlığı ve güvenli kapanış ister. Native worker'ın
mevcut health yanıtı uygulama DB/migration hazır oluşunu tek başına kanıtlamaz.
`scripts/worker-supervisor.py` açık trusted ikili/proje/source kimliğiyle
sahipli süreç grubunu başlatır, loopback HTTP probe'unda exact build header'ı
ve ayrıca application revision'lı readiness dosyasını birlikte ister.
Hazırlık dosyası health'in otomatik ürettiği başarı değildir; işletmeciye
ait ayrı kapıdır. Geliştirme/kirli build yalnız explicit `--pilot` ile
dürüstçe kabul edilir; expected source ve health identity kontrolü kalkmaz.

Host stdio DEVNULL ve sabit JSON lifecycle/probe sayacı üretir; request
URL/credential/header/body veya child log'u kopyalamaz. TERM, bounded
bekleme, gerekirse KILL ve wait/reap yalnız kayıtlı sahipli group içindir;
port/name aramasıyla yabancı süreç benimsenmez veya öldürülmez. Sonlu
ardışık restart aynı portta yapılır. K-245 credential köprüsüyle composition
ayrı actual test ister; yalnız iki aracın varlığı kabul sayılmaz.

## Public launcher bug ve güvenlik sınırı

Mevcut `dogfood/zeedili-com/scripts/yerel-baslat.sh` frontend'i `exec node`
ile başlatıyor, trap sahibi shell'i değiştiriyordu. Controlled actual önce
kanıtında frontend TERM sonrası owned native worker dinlemeyi sürdürdü.
K-246 Python WNOWAIT sahibini önde tutar, worker/frontend handle'larını
EXIT/INT/TERM ve startup başarısızlığında temizler. GB-069 kapanışı actual
owned lifecycle testi ister; yabancı servis veya müşteri deployment'ı yoktur.

Bu P13 host dilimidir: gerçek in-flight request drain, DB pool/request
metrics, HUP reload, systemd kurulumu ve uzun temiz host kabulü açıktır.
P14 socket devir/overlap/zero-downtime tamamlanmaz; ardışık restart kısa
kesinti taşır. Yeni `dil hizmet`, grammar, compiler signal handler, API,
dependency veya cache yoktur. POSIX capability eksikliği fail-closed'dur;
Windows veya minimum OS kabulü çıkarılmaz.

## Kanıt ve etki incelemesi

Normal POSIX Cargo kampanyası actual native worker, health/source identity,
readiness negatifleri, yabancı port, crash/TERM/KILL/reap ve credential
composition sınırını atlamasız yürütür. Exact case sayısı ve temiz kaynak
kampanyası [host rehberi](../docs/worker-supervisor.md) ve final kanıt
kaydında tutulur. Public launcher'ın owned süreç kanıtı aynı dalgadadır.

RFC/spec/tanı/API/ABI, production katman ve SCC, benchmark iş yükü ve
compiler state değişmez. ADR-045/049/046 etkileri bu nedenle aynı kalır.
Rehber, günlük/backlog, ürünleşme/kapılar, güvenlik/kanıt/faz haritaları ve
README sayıları birlikte güncellenir; gerçeklenmeyen hizmet kapasitesi açık
yazılır. Yeni release/RC veya canlı ürün kabulü verilmez.

Host metadata/readiness biçimleri giriş sürümüyle K-167 uyumluluk
fixture'ında aktiftir; bunlar compiler CLI/grammar/facade export'u değildir.

Bağımsız actual Bash probe, shell'in background child'ı explicit wait
öncesinde reap ettiğini gösterdi. Sayısal PID trap'i güven sınırı sayılmaz;
public launcher ortak Python WNOWAIT kontrolcüsünü kullanır. Lider PID
grup temizliği bitmeden serbest bırakılmaz. Startup probe başarılı olsa
bile mutlak başlangıç deadline'ı ve güncel generation gate'i ayrıca geçer.

## K-301 test kanıtı takibi

Mevcut inflight kesilme testinin dar doğruluk işi tamamlandı: handler giriş
işareti ile EOF/reset sonuna kadar sınırlı yanıt okuması doğrulanır. Explicit
0,2 saniye host grace ile üç saniyelik handler karşılaştırılır; üretim sinyal
davranışı, varsayılan grace ve mimari karar değişmez. V3 opt-in fixture,
`web-oturumu` altında CSRF GET/çerez/form POST akışını kullanır; dosya yazımı
`herkese açık` POST rotasından `eylem girişi kaydet` ile yapılır. CSRF atlanmaz.

İlk GET fixture'ının T045 reddi ve worker1 başarısızlığı korunur. V2 denetimi
geçti, worker2 yalnız inflight vakada başarısız oldu (25 test/1 hata);
assertion ayrıntısı yoktur. Eksik CSRF/çerez kaynak incelemesidir, gözlenmiş
HTTP403 değildir. V3 worker3 kabulü tek Cargo sahibi 1/0/0 ve wrapper'ın
25/0/0 doğrulamasıyla geçti; özgün controller exit0/reap ile kapandı.
V3 kontrollü 10 senaryo/20 karşılaştırma ayrı mock kanıtıdır; gerçek
HTTP/CSRF yardımcısı worker3'te sınandı.

25 Python test metodu ve mevcut tek Cargo sahibi korunur; test eklenmedi
veya taşınmadı. Genel drain/native DNS/tam fault/P34 ve GB-069 kapsamı
genişlemez; yeni GB/ADR kararı yoktur. K-300 tam faz sonucu K-301'e aktarılmaz.
Son 11 belge/güvenlik kapısının geçmesi kayıt commit'inin önkoşuludur.
Ayrıntı [host rehberindedir](../docs/worker-supervisor.md).
