# ADR-130 — K-272 sabit TR/EN liste–detay HTTP golden

- **Durum:** kabul; dar sabit Unix golden; ortak entegrasyon ve final temiz kaynak kabulü açık
- **Tarih:** 15 Ekim 2026
- **İlgili:** P21/C4, K249/ADR109, GB088

## Karar ve gerekçe

Mevcut article fragmentini tam sayfa layout’una HTML Metin slotu olarak sokmak ya double-escape ya ham escape bypass gerektirirdi. Bu dilim bunun yerine ayrı sabit uygulama görünümü kurar: yalnız plain Metin named slotlar, fixed HTML tree/attribute/linkler, tr/en whitelist ve mevcut boolean hidden. Document başlığı yalnız sabit doctype literalidir. Kullanıcı verisi markup birleştirmesine girmez. Çeviri helperı mevcut byte-eş K249 kopyasıdır; eski fragment kaynakları/main ve7native+4Zee kabulü korunur.

Altı exact list/detail route mevcut GET handlerla gerçek HTTP200 verir. Unknown locale/path mevcut404 davranışıdır. Yeni grammar/runtime/API/HTTP parser veya dinamik router yoktur. Native host aynı existing owner/ready/activation/heldleader cleanup API’sini private snapshot ile kullanır. Yeni production owner/PID discovery kurulmaz.

## Güvenlik ve kanıt

GB088 sınırlı görünüm güvenlik sözleşmesidir: data yalnız text/title/aria-label bağlamlarında defaultescape; URL/tag/attribute/bodyHTML slotu yok. XSS negative exact snapshot+independent HTML parser ile character data ve quote sınırını doğrular. Hidden sadece görsel fixedanon seçimdir, erişim kontrolü değildir.

Dış actual9HTTP +5source +old7native/4source koşuları vardır. Current given debug binary BuildJSON ve dirty bilgisi değiştirilmez; temiz final/release veya bütün platform kabulü değildir. Rust wrapper önerisi tek testtir; ortak Cargo/faz/tazelik parent tarafından çalıştırılır. Native Linux/Windows/minOS ve canlı ürün kabulü açık.

C4 exact ürünkökü/hash kayıtları golden README’de, metadata-only tekrar git-show ile doğrulanır; özel source/body/etiketler taşınmaz. Somut anonim ürün sayfası eksik olduğundan yeni fixture golden örnektir; dogfood korpusu/P08/sayfa migration tamamlandı gösterilmez. K164/K249 mevcut kapasite karşılaştırması, yeni F/grammar talebi üretmez.

## Bilinçli etkiler

Compiler spec/RFC/tanı/ABI/API/katman/SCC değişmedi. ADR045/049 observational ölçüm politikası ve performans tarihçesi değişmez; yeni kalıcı compiler/LSP state veya cache olmadığı için bu dilim yeni soak geçmişi üretmez. JSON yalnız test raporu, production biçim/profil/CLI eklenmediği için compatibility yüzeyi değişmez. Golden/test counter ve faz kaydı actual root araçlarıyla güncellenir. Backlog/plan/release/gates/journal aynı atomik değişiklikte kısmi sınırları taşır.

İlk commitlenmemiş test-runner aday incelemesi sonrası stream okunurken64KiB capture, HTTP connect/send/recv/EOF/parse boyunca tek3sn bütçe ve tüm kampanya180sn bütçesi eklendi. Runner/legacy helper ve4legacy kaynak/3snapshot dahil begin/end pinler iki kampanyadan sonra denetlenir; eski kampanya da aynı private binary/input snapshot’larını kullanır. Ret makbuzu yalnız fixed tür/reason taşır. Başarılı Cargo makbuzu owned target altında saklanır; formatter nedeniyle ast ek faz kapsamıdır. Bu Python aday düzeltmesi compiler bug SHA/regression kaydı icat etmez.
