# K-227 — Mevcut paket ve rota birimiyle minimal modül kompozisyonu

Ürünleşme talebi `2026-10-14-zee-urunlesme.md` C3; özel ürün exact commit'i
`cd5d12f0008620819e6167af5146e83503fd2b38`. Harici özel kaynaklar 14 Ekim
2026'da HEAD ile aynı bulundu. İçerikleri kopyalanmadan şu parmak izleri
alındı:

| Ürün dosyası | SHA-256 |
|---|---|
| `araclar/site_modulu.py` | `0cdfd25f9499e2f3643448c4165d7fff5eacb71c9d532fb90376ef1f370874b2` |
| `moduller/pdf-katalog/modul.py` | `f100cc31d6b714cac088cdd829ee281e19ec9e456f01303578c880e4933c17c2` |
| `moduller/pdf-katalog/modul.json` | `0cb37d44e442b3f1ede10c91d9ce9830a5eb494049fdcedf97893c1184949ae2` |
| `moduller/pdf-katalog/kaynak/rota_sm_katalog.dil` | `b8721f8044ee976c3f1770bfe43d4788e0e962ecd6196be7845367c54d29fb59` |

Kaynak incelemesi `modul.py:75` rota eklemesini ve `:77–85` aynı panel menü
satırını iki modülün değiştirmesiyle ilgili not/tek-satır beklentisini
doğruladı. `site_modulu.py` birim ekleme, panel menüsü ve sayfa kancası
işlemlerinde dosya satırları/slice'ları üzerinden düzenleme yapıyor.
Gerçek çakışmış ürün dağıtımı yeniden çalıştırılmadı; eski olayın sonucu
ürün talebi olarak korunur. Bu fixture aynı müşterinin tam anonim site ağacı
değildir.

[Katalog paketi](katalog/ana.dil) mevcut `proje.dil`/exact yerel bağımlılık ve
tam public imzayı kullanır. [Host rota birimi](site/katalog_rotalari.dil)
paket tanımını açıkça alır; [site girişi](site/ana.dil)
`katalog_rotalari rotalarını kullan` ile onu kompoze eder. Kurulum kaynak
ankrajı aramaz veya başka bir modülün satırını yeniden yazmaz.

[Kabul testi](../../../compiler/tests/zee_cekirdek_modul_kabulu_testi.rs)
geçici kopyada mevcut `dil ekle`, `kilitle`, `paketler`, `dene`, `denetle` ve
`çıkar` komutlarını çalıştırır. Kurulu modül `/katalog` yanıtını açar;
çekirdek `/` yanıtı değişmez. Kaynak tüketilirken paket kaldırma P010 ile
reddedilir ve bildirim/kilit aynı kalır. Kaldırmada modüle ait host adapter
dosyası kaldırılır ve [modülsüz tam giriş](site/ana-modulsuz.dil) seçilir;
ardından `çıkar` başarılı olur, `/katalog` 404 olur, `/` korunur. Paket
kaynakları kur/kaldır boyunca aynı kalır. İşlem geçici fixture'dadır; private
ürün veya gerçek kurulum değiştirilmez.

Bu kabul yalnız rota/paket kompozisyonunun mevcut dilde yeterli alt
kapsamını gösterir. Tam ürün modül kurucusu/kaldırıcısı, modül manifestini
okuyan resmî araç, otomatik host adapter yaşam döngüsü, sürüm aralığı, ad
alanı, panel menüsü/içerik türü kayıt kancaları, göç aralığı ve iki modülün
veri yaşam döngüsü tamamlanmış değildir. Bunlar C3'ün açık kalan ürün
tasarımıdır. Yeni grammar, compiler özelliği veya API eklenmedi.
