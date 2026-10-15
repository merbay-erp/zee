# Zee-çekirdek sabit SQL niyet dilimi — K-258

Gerçek B1 ihtiyacının anonim SQL-only reproducer'ı mevcut dil kalıplarını kullanır.
`ana.dil` sabit handler niyetini ve eşleşen lease/generation tamamlamasını aynı
transaction'a bağlar. Payload veri olarak saklanır; SQL, shell veya callback
olarak yürütülmez. 0001/0002 mevcut değişmez göçlerdir; 0003 niyet tablosunu ekler.

[Host rehberi](../../../docs/job-dispatcher.md) sonlu 1..100 tick dispatcher'ı,
COMMIT belirsizliğinde replay yapılmamasını, ayrı heartbeat ve süreç grubu
sahipliğini açıklar. `denetle` korpus kaydı derleme kanıtıdır; actual 11 vakalı
owned PostgreSQL kampanyası dedicated Cargo kapısındadır. Gerçek katalog,
calendar/cron, SMTP/PDF etkileri ve tam P09 benimsemesi açıktır.
