# ADR-131 — Yönetilen proje modül kompozisyonu (K-273)

- **Durum:** kabul; dar owned scaffold profili, gerçek ürün adoption ve DB lifecycle açık
- **Tarih:** 15 Ekim 2026
- **İlgili güvenlik sınırı:** GB-089

K-227 mevcut local paket/route kompozisyonu, iki anonim modülün mevcut dil ve
CLI ile çalışmasına yeterlidir. İlk yönetici yeni ve receipt sahibi olduğu
iskeletle sınırlıdır; mevcut üründe modul.py/anchor slicing yürütmez. Güncel
özel açıklayıcı modul.json'dan yalnız alan yapısı/provenance önceki tasarımda
incelenmiştir; yeni zee-modul-1 yapısal sözleşmedir, özel değerler taşınmaz.

Managed root sabit, .zee/appdata dışarıda, registry/menu/content metadata
uygulama sahibidir. Namespace ve SQL aralıkları kontrol edilir. Payload trusted
code'dur, metadata doğrulaması source sandboxı değildir. Candidate mevcut CLI
kilitle/denetle üzerinden doğrulanır; compiler/grammar/API değişmez.

SH lock managed launch ömrünü kapsar; mutation EX gerektirir. Parent SIGKILL
lockrelease'ini child-exit kanıtı saymayız: pre-Popen durable active marker
update'i explicit owned offline/cgroup review'a kadar kapalı tutar. Native
outer owned grubunda kalır; launcher yalnız doğrudan child handle'ına signal/
wait uygular. PID discovery/adoption ve yeni native FD rolü yoktur.

Per-file journal çok dosyalı atomik transaction diye sunulmaz. Partial error
pending state ve launch refusal bırakır; explicit finish yalnız exact old/new
byte durumlarıyla ilerler. Complete receipt+unlink sonrası directory fsync
hatası visible-publication/durability-unconfirmed exit4'tür; false rollback
veya false paused değildir. Exit0 yalnız tam fsync zinciridir.

SQL archive aktif migration dizininden ayrıdır; kaldırma range/namespace
mezar taşı ve arşiv bytes'ını korur. Manager SQL execute, applied-history
gözlemi, otomatik DROP/downmigration yapmaz. İçerik türü kaydı CRUD değildir.

İlk dış kanıt gerçek 23 Python vaka içinde mevcut dirty pilot binary ile
iki modül CLI/native HTTP ve remove/core/appdata/SQLarchive eşliğini çalıştırır.
Kontrollü IO/state testleri ayrı profildir; bu sonuçlar final clean kaynak,
Linux service/cgroup, güç kaybı veya gerçek özel ürün benimsenmesi değildir.
Bağımsız dar review ve keyed docs/compatibility/faz/security kayıtları bağlandı;
ortak Cargo ve final temiz kaynak kabulü ayrıca doğrulanır. Yeni uzun ömürlü compiler/LSP state yoktur.
