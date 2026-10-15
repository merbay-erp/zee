# Typed modül SQL tasarım örnekleri

K-285 / ADR-143: news(text+boolean) ve stock(text+integer+boolean) iki generic
schema örneğidir. Yeni Zee grammar veya gerçek ürün korpusu değildir. Host,
mevcut Zee eylem/sorgu ve göç project'ini üretir; runtime `.dil` dosyaları sabit
renderer'dan gelir. SQL değerleri her zaman bind parametresidir.

[Owned lifecycle rehberi](../../docs/module-sql-lifecycle.md) pending/unknown,
SQL down/archive/reapply ve explicit readonly reconciliation sınırlarını açıklar.
