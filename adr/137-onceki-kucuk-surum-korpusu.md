# ADR-137 — Gerçek önceki küçük sürüm korpusu ve açık göç (K-279)

- **Durum:** kabul, dar POSIX korpus/göç kanıtı; tam P04/P05/ürün kabulü açık.

P04/P05 için sentetik eski-consumer modeli yerine mevcut v0.7.0 exact
kaynağı d08a947713ddc38d2163360c17d07fbffcb1213d ayrı temiz klonda derlenir.
33 eski golden girdisi blob/SHA ile saklanır. Eski/yeni 32 denetleme sonucu
aynıdır; birim örneği K-086 public tür sözleşmesi nedeniyle T039 alır.
Bu açık istisna, yalnız birimde iki type annotation satırıyla göç edilir;
ana kaynak ve runtime çıktısı korunur. Kaynak korpusu gizlice güncellenmez.

15 sabit saf runtime çıktısı gerçek eski CLI'ya bağlıdır. Girdi/rastgele/
dosya/saat/ağ/web/donanım/async davranışı bu altküme dışında tutulur.
Yeni iki Cargo vakası modern CLI'da 49 alt kontrol ile beş actual bootstrap
reddini tekrarlar; bunlar unique libtest sayısı olarak şişirilmez. Exact
fixture/helper hash ve kaynak rolü bütün native çağrılarından önce denetlenir;
retained owned group/deadline/capture çekirdeği e76 aynen paylaşılır.

Gerçek eski CLI'nın capability sorgusu exit0 düz metindir ve config-reader
kabulü değildir. Mevcut seçicinin daha erken BuildJSON reddi gevşetilmez;
eski ikiliye yeni kimlik takılmaz. Yeni zee.ayar için 0.7 otomatik rollback
verilemez. GB-095 bu bilinçli destek sınırıdır, yeni compiler bulgusu değildir.

[Rehber](../docs/onceki-kucuk-surum-korpusu.md),
[mevcut karar](../spec/10-disari-acik-islem-sozlesmesi.md) ve
[ayar profili](../docs/zee-ayar.md) birlikte okunur. Yeni grammar, compiler/src,
Rust API, tanı kodu, dependency, owner/kenar, cache veya benchmark tarihçesi
değişmez. RFC/spec normları aynı kalır; eski açık rollback maddesi bu dar
kanıtla kapatılmaz. Fiziksel platform/insan/ürün/canal-yayın/finalclean kabulü
açıktır. Çalışma adayı short prototype, temiz finalS kampanyası değildir.

Linux üzerinde exact e76 helper ile gerçek `/usr/bin/true` çağrısı 3,001 saniye
tüketir: WNOWAIT ile tutulan bitmiş leader, grup temizliği sırasında görünür
kalır ve mevcut üç saniye grace süresi biter. Bu testte 52 komut vardır;
toplam test kampanya bütçesi bu nedenle 240 saniyedir. Komut başına sekiz
saniye ve sahipli temizlik davranışı değişmez; production timeout, SLA veya
benchmark eşiği gevşetilmez. Bu ölçüm tam Linux korpus kabulü değildir.
