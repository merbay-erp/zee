# Önceki küçük sürüm korpusu — K-279/P04/P05

Gerçek `v0.7.0` kaynak kimliği `d08a947713ddc38d2163360c17d07fbffcb1213d`dir.
Ayrı temiz klonda Rust1.93.1 ile derlenen eski CLI, 33 eski golden kaynağı
üzerinde denendi. [Fixture manifesti](../compiler/tests/fixtures/onceki-kucuk-surum-v07/manifest.json)
her dosyanın exact Git blob ve SHA-256 kaydını tutar. Özel ürün korpusu veya
beş anonim tam site yerine geçmez.

Çalışma adayıyla karşılaştırmada 32/33 denetleme exit sonucu aynı kaldı.
`31-birimler.dil`, eski `hesap_araclari.dil` birimi açık parametre/dönüş türü
bildirmediğinden mevcut T039 ile reddedilir. Bu yeni değişiklik değildir:
[K-086 kararı](../spec/10-disari-acik-islem-sozlesmesi.md)
`2a59b9e49a07f142efa2f65d34d76872c26f8479` kaynak commit'indedir. Göç yalnız
birimde `fiyatı al` yerine `fiyatı Ondalık olarak al` ve `Ondalık döndürür`
yazar. Ana program değişmez; göçten sonra denetleme ve eski runtime çıktısı
birebir geçer. Ham korpusun tamamı değişmeden uyumlu gösterilmez.

Sabit 15 saf programda gerçek eski/yeni çalışma çıktıları eşittir. Girdi,
rastgelelik, dosya, saat, ağ, web, donanım ve async örnekleri bu runtime
altkümesinde yürütülmez; 33 dosyanın denetlemesi ayrıdır. Normal Cargo testi
33 denetleme +15 saf runtime +1 bilinen göç vakasını (49 alt kontrol) gerçek
aday CLI'da tekrarlar. İkinci test, kaynak/FIFO/duplicate manifest/ek dosya/
helper bozukluğunun native çalışmadan reddini beş gerçek subprocess ile sınar.
Bunlar iki Cargo vakasıdır; 54 yeni libtest vakası değildir. POSIX sahipli
süreç profili Windows'ta açıkça atlanır; çalışmış gösterilmez.

Eski gerçek 0.7 CLI `sürüm --ayar-semasi` sorgusunda exit0 olsa da normal
metin yazar; `zee-config-capability-1` JSON sözleşmesini sunmaz. Bu olumlu
rollback kabulü değildir. Mevcut seçici, legacy BuildJSON'u daha önceki
kimlik kapısında reddeder; config sorgusunun mevcut olması ayrıca gerekir.
Eski ikilinin kimliğini taklit eden shim ya da güvenli fallback yoktur.
Yeni `zee.ayar` taşıyan projeyi 0.7'ye otomatik geçirmek desteklenmez.
Mevcut [ayar rehberi](zee-ayar.md) ve [uyumluluk politikası](uyumluluk-politikasi.md)
korunur. Kanal yayınları, production trust, gerçek ürün/minOS/remote CI ve
final temiz kaynak kabulü ayrı kapılardır. İlk dış deneme current40b kirli
ikiliye aittir; temiz eski kaynak ile temiz yeni kaynak aynı şey değildir.

Test: `cd compiler && cargo test --locked --test onceki_kucuk_surum_testi`.
Compiler/grammar/spec normu, production Rust, supported API, dependency,
katman grafı ve performans iş yükü değiştirilmez. GB-095 bilinçli eski
config-reader/rollback sınırıdır; yeni compiler bugfix kaydı icat edilmez.

Linux üzerinde exact e76 helper ile gerçek `/usr/bin/true` çağrısı 3,001 saniye
tüketir: WNOWAIT ile tutulan bitmiş leader, grup temizliği sırasında görünür
kalır ve mevcut üç saniye grace süresi biter. Bu testte 52 komut vardır;
toplam test kampanya bütçesi bu nedenle 240 saniyedir. Komut başına sekiz
saniye ve sahipli temizlik davranışı değişmez; production timeout, SLA veya
benchmark eşiği gevşetilmez. Bu ölçüm tam Linux korpus kabulü değildir.
