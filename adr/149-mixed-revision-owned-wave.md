# ADR-149 — K-291 mixed-code revision için ayrı gerçek kanıt

- **Durum:** K-300 GB-112 açık; hazırlık13 geçti; kaynak kapıları ve temiz exact native9 kabulü bekleniyor.
- **Tarihsel durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): dokuz native vaka/160 istek/dört dalga ile dar yerel kabul; genel SLA, DB şema devri ve ürün/platform kapsamı açık.
P14/B6'da inherited listener17 sabit application revision üzerinde kanıttı;
mixed-code yayınını veya evrensel kesintisizlik/SLA'yı kanıtlamıyordu.

Compiler/FD protocol genişletilmez. Yeni dedicated koşucu aynı owned project/root
ve atomic shared JSON state üzerinde eski loaded AST ve yeni source hash/revision
ile gerçek native process'leri birlikte yürütür. Yayın trusted anonymous fixture
source değişimidir, production artifact/DB schema deployment API'si değildir.
Ready/ACTIVATE fresh nonce korunur. Old admitted handler entry, new marker yanıtı,
TERM sonrası old exact body ve yeni generation'ın state/cookie/revoke/quota
koruması ayrı oracledır. Dört load wave/160istek/concurrency2 sonuçları request
retry veya quota reset/raise ile düzeltilmez. İlave state rollover ve mevcut
single-generation deadline kontrolü load sayısına katılmaz.

K-291 ilk kaynak diliminin [rehberi](../docs/mixed-revision-wave.md), hazırlık8
ve dedicated native9 ayrımını korur. Eski sabit-revision17 yeni kabul yerine geçmez. GB-107,
backlog, faz, uyumluluk ve makine planı ortak kayıtları güncellendi; compiler source/cache/API/grammar veya core soak semantiği
bu test diliminde değişmez. P14 gerçek ürün/şema/platform ve genel SLA açıktır.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-300 revizyonu — Kabul koşucusunda early setup ve cleanup

Canonical kök bekleyen Python koşucusuna Rust sarmalayıcısından `compiler/..`
aktarılması native kabulü başlangıçta reddeder. Kökün kanonikleştirilmesi bu
sözleşmeyi korur. İlk owned Git/kimlik komutundan önce handler kurulur; setup
hatası finally kapsamına alınır ve kısmen kurulan host açık sahiplikle temizlenir.
Cleanup sırasında alınan iptal korunur; tüm vaka kontrolleri daha önce geçmiş
olsa da iptal edilen koşu `passed` yazamaz. Sahiplik özgün handle üzerinden
sürer; üretim listener/worker/FD protokolü değiştirilmez.

Ek Python hazırlık vakaları mevcut normal Cargo sahibinde kalır. Native9 aynı
doğrulama koşullarıyla ayrı exact temiz kaynak kabulü ister; yeni sonuçlar henüz yoktur.
[GB-112 ve kapsam](../docs/mixed-revision-wave.md)
K-299'un C2 PDF/service kabulünden ayrıdır. Yeni compiler kapasitesi, grammar,
API, kalıcı durum veya normatif spec/RFC kararı eklenmez; P14 kısmi kalır.
