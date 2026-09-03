# ADR-061 — Binary yükleme ve dosya yaşam döngüsü sahipliği

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İş:** K-163/F030

## Karar

HTTP framing/content-length/timeout ve temp spool native web adaptörünün;
atomik publish/delete kalıcı dosya çekirdeğinin; tür/etki/yetkinlik intrinsic
kaydının; metadata durum makinesi ve uzlaştırma ise ürünün sahibidir.

Yorumlayıcı IO trait'i binary buffer taşımaz. Rota yalnız proje-kökü göreli
temp yolunu, hash'i ve boyutu görür. Böylece dil heap zarfı yükleme boyutuna
bağlanmaz, binary payload string'e dönüştürülmez ve ürün DB dışı yan etkiyi
açık saga olarak yönetir.

`ToplayanIo` ayrı modüle çıkarılarak kök yorumlayıcı bütçesi korunur. Yeni
işlemler deterministik adaptör, capability sargısı ve IO trace/replay boyunca
aynı sözleşmeyle taşınır. Genel bir dosya sistemi API'si veya gizli DB+dosya
transaction'ı açılmaz.
