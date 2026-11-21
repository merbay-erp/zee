# Ürünleşmede açık teknik incelemeler

15 Ekim 2026 K-288/C1 ve K-293/P33 durum kaydıdır; tamamlanmayan iş kapanmış gösterilmez.

## K-288 / C1

Beş özel dil hatası için doğru öneri, `dil hata` örneği, LSP hızlı düzeltme ve regression zinciri tamamlanmadı. Ayrı bu görev için araç çağrısı da otomatik siber güvenlik incelemesince reddedildi; başka araca/ajana yönlendirerek tekrar edilmedi. K-287 opt-in lint, C1 beş kalıbının kabulü değildir.

## K-293 / P33 / GB-109

Salt okunur kaynak/test matrisi beş alanı ve20 mevcut test işlevini inceledi: dosya yetkinliği kontrolü ile açma arası; paket/cache yayın süresi; bağımsız HTTP parser karşılaştırması; Unicode gösterim/rename/registry; oturum kalıcılığının write/fsync/rename/yanıt kesitleri. Mevcut statik negatifler ve normal eşzamanlılık bu bütün kesitleri kanıtlamaz. Bu bir kanıt açığıdır; gözlenmiş dış erişim/istismar raporu değildir.

Koordineli yerel dosya değişimi reproducerı araç güvenlik incelemesince reddedildi ve çalıştırılmadı. Ret aşılmaya çalışılmadı. Yeni üretim savunması, gözlenmiş güvenlik açığı veya tam P33 kabulü ileri sürülmez. Matris tarihsel kirli aday hashlerini taşır; güncel temiz kaynak test sonucu değildir.

### K-362 dilimi (21 Kasım 2026)

Beş alandan ikisi zee'nin kendi koduyla test kanıtı aldı; davranış değişmedi.

| Alan | Durum | Kanıt |
|---|---|---|
| Bağımsız HTTP parser karşılaştırması | kanıtlı | `compiler/tests/http_kahini_testi.rs`, `compiler/tests/ortak/http_kahini.rs`, `http_istegi` fuzz hedefi, `web_profili_testleri::istek_baslik_siniri_exact_baytta_431_verir` |
| Oturum kalıcılığı write/fsync/rename/yanıt kesitleri | kanıtlı | `compiler/src/web_guvenligi/tests/kesit.rs`, `web_profili_testleri::kalici_oturum_commiti_dusunce_basari_yaniti_ve_cerez_gitmez` |
| Dosya yetkinliği kontrolü ile açma arası | açık | Koordineli yerel dosya değişimi reproducer'ı reddedildi; yeniden denenmez, etrafından dolaşılmaz. |
| Paket/cache yayın süresi | kanıtlı (K-363) | `compiler/src/artefakt_dogrulama/kurulum/testler.rs`, `compiler/src/registry/istemci/testler/yayin.rs` |
| Unicode gösterim/rename/registry | kanıtlı (K-363; altı açık düzeltildi, GB-146) | `compiler/tests/proje_testi.rs`, `compiler/tests/lsp_testi.rs`, `artefakt_dogrulama::testler::harf_katlama_cakismasi_uretici_ve_tuketicide_reddedilir`, `registry::testler::registry_adi_ve_kimligi_gorunmez_bicim_karakteri_tasiyamaz`, regression `native-unicode-kimlik` |

Kâhin sözleşmesi tek yönlüdür: `httparse` 1.10.1'in (yalnız dev/fuzz) reddettiğini zee kabul etmez; kâhinin kabul edip zee'nin reddettiği her girdi belgelenmiş katı ret sınıfına dayanır (istek öncesi boş satır, yalın LF, origin-form dışı hedef, obs-text başlık değeri, Transfer-Encoding, çoklu/geçersiz Content-Length, Content-Length dışı bayt, NUL/geçersiz UTF-8 gövde). Kesit testi her adım noktasında depo klasörünün görüntüsünü alır; görüntüler tam eski ya da tam yeni, yeniden başlatma kurtarır, `Ok` klasör eşzamanlamasından sonra gelir ve main.rs `Err`de başarı/çerez yerine 503 yazar. Ayrıntı ve sınırlar [backlog K-362](oncelikli-backlog.md) kaydındadır.

### K-363 dilimi (21 Kasım 2026)

Paket/cache yayın süresi davranış değişmeden kanıtlandı: eşzamanlı kurulum ve registry güncellemesi tek exact ağaç/içerik adresli nesne yayımlar; yarım geçici okunmaz; yarım ya da bozuk yayın ve durum öncesi çökme sessizce onarılmaz; süre sınırı kesin, eskiyen zincir durumu değiştirmez, çevrimdışı kip son kabul anını kullanır. Unicode alanında altı açık bulundu ve düzeltildi: paket adında homoglif/CJK/birleştirici im, `.zep`'te büyük/küçük harf çakışması ve eksik ALM bidi karakteri, büyük/küçük harf duyarsız dosya sisteminde farklı birimin bağlanması, LSP yeniden adlandırmasında lexer dışı ad, gerekçe/duyuru kimliğinde görünmez biçim karakteri. Ayrıntı [backlog K-363](oncelikli-backlog.md). GB-109 yalnız dosya yetkinliği kontrolü ile açma arası alanı kapanmadığı için `acik` kalır; o alanın reproducer reddi korunur.

[Plan](urunlesme-plan-v1.tsv), [backlog](oncelikli-backlog.md) ve [güvenlik kaydı](guvenlik-bulgulari-v1.tsv) açık sınırı korur. Diğer yetkili teknik çalışmalar bağımsız sürer.
