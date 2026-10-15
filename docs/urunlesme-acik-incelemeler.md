# Ürünleşmede açık teknik incelemeler

15 Ekim 2026 K-288/C1 ve K-293/P33 durum kaydıdır; tamamlanmayan iş kapanmış gösterilmez.

## K-288 / C1

Beş özel dil hatası için doğru öneri, `dil hata` örneği, LSP hızlı düzeltme ve regression zinciri tamamlanmadı. Ayrı bu görev için araç çağrısı da otomatik siber güvenlik incelemesince reddedildi; başka araca/ajana yönlendirerek tekrar edilmedi. K-287 opt-in lint, C1 beş kalıbının kabulü değildir.

## K-293 / P33 / GB-109

Salt okunur kaynak/test matrisi beş alanı ve20 mevcut test işlevini inceledi: dosya yetkinliği kontrolü ile açma arası; paket/cache yayın süresi; bağımsız HTTP parser karşılaştırması; Unicode gösterim/rename/registry; oturum kalıcılığının write/fsync/rename/yanıt kesitleri. Mevcut statik negatifler ve normal eşzamanlılık bu bütün kesitleri kanıtlamaz. Bu bir kanıt açığıdır; gözlenmiş dış erişim/istismar raporu değildir.

Koordineli yerel dosya değişimi reproducerı araç güvenlik incelemesince reddedildi ve çalıştırılmadı. Ret aşılmaya çalışılmadı. Yeni üretim savunması, gözlenmiş güvenlik açığı veya tam P33 kabulü ileri sürülmez. Matris tarihsel kirli aday hashlerini taşır; güncel temiz kaynak test sonucu değildir.

[Plan](urunlesme-plan-v1.tsv), [backlog](oncelikli-backlog.md) ve [güvenlik kaydı](guvenlik-bulgulari-v1.tsv) açık sınırı korur. Diğer yetkili teknik çalışmalar bağımsız sürer.
