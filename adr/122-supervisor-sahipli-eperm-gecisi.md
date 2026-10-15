# ADR-122 — Supervisor sahipli EPERM/WNOWAIT geçişi

- **Durum:** kabul, K-263 dar POSIX host correctness adayı
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-263/P13, GB-077; source ac077e31 tamamlandı; birleşik finalclean gate bekleniyor

## Gerçek arıza

BASE17b29da mevcut supervisor cleanup group_alive kontrolü killpg0 EPERM ve WNOWAIT henüz None durumunda hemen ownership reddi verir. Dış instrumented eski kaynakta gerçek native HUP2146ms sahipli worker EPERM/None/returncodeunset ile site0 Reject üretmiş, aynı held leader2174ms terminal0 olmuştur. Uygulama-gate TERM4227ms aynı geçişi,4334ms terminal0 gösterir. Minimal owned nativequery de gerçek OS EPERM/None ile aynı stop_child reddini iteration0 üretir; bütün liderler sonradan sahipli reap edilmiş, yabancı grup yaşamıştır. Bunlar mock veya müşteri kaynakları değildir. Dış stage ham/hash kanıtları root final kabulüne bağlanır.

## Minimal davranış ve yetki

K-263 yalnız scripts/worker-supervisor.py stop_child davranışını onarır; compiler src, grammar/API/CLI seçenekleri değişmez. Tek mutlak stop_grace deadline ilk probe öncesinde alınır. EPERM+nonterminal mevcut aynı unreaped Popen tutamacında1ms sınırlı yeniden gözlenir; deadline her helper çağrısında resetlenmez. TERM/KILL PermissionError aynı sahipli grup durumunu yeniden gözleyebilir; kalıcı belirsizlik deadline içinde ownership reddi ve unreaped kayıt bırakır. SIGKILL dispatch grace sonunda, başarılı dispatch sonrası mevcut2s finalwait sınırı korunur.

Sahipliği olmayan tutamaç, Popen.returncode-set veya ECHILD ilk signal öncesinde failclosed olur; PID/port/ps bulgusu yeni yetki vermez. Leader terminal olsa bile killpg0 True dönen canlı descendant grubu TERM/grace/KILL ile temizlenmeden leader reap edilmez. EPERM+terminal→gone çıkarımı mevcut Darwin aynıUID trusted child profiline aittir; genel Unix permission safety, setuid daemon containment veya yabancı süreç adoption sözü genişletilmez. Yeni diag/customer/argv/log içeriği yoktur; ownership/cleanup ret sözleşmesi korunur.

## Test ve kapılar

Ayrı supervisor-cleanup-test.py8 vaka actual heldchildren/foreigngroup ile controlled EPERMNone→terminal, persistentnonterminal deadline/noearlywait, TERM transient/persistent, KILL denied-aftergrace, terminalleader/live-descendant, returncode/ECHILD authorityloss ve50 gerçek Python sahipli boş helper yürütür. Ayrı Cargo wrapper1 eklenir; mevcut worker25 kampanya adı/sayısı değiştirilmez. Actual native query race sonrası50 seçili OS geçişi, HUP/TERM targeted repeati ve fullphase gate dış kanıtı beklenir. Katman/RustAPI/uyumluluk/performans/soak core davranışı değişmez; mevcut compiler1800s soak ayrıca sürer. K262 CLI source bug ve K257/259/260 featurewave bu adaya dahil değildir.
