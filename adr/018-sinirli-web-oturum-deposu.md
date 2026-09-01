# ADR-018 — Sınırlı web oturum deposu ve mutlak ömür

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-106, B-046, V1-P0-16

## Bağlam

K-088 sunucu tarafı oturum kimliği, rol, CSRF, rotation ve expiry'yi kurdu.
Süresi dolan kayıtlar istek başında temizleniyordu; fakat geçerlilik süresi
içindeki anonim form oturumları process içi `HashMap`i sınırsız büyütebiliyordu.
Tek bir GET/CSRF akışı kalıcı bellek tüketimine dönüşmemelidir.

Ayrıca erişimin oturum ömrünü uzatıp uzatmadığı ve process-local deponun çok
süreçli deployment'taki anlamı açık sözleşme değildi.

## Karar

Native process içi depo şu sabit sınırları taşır:

- en çok 4096 toplam oturum;
- bu toplam içinde en çok 1024 anonim form oturumu.

Her istek önce süresi dolan kayıtları siler. Yeni anonim oturum anonim veya
toplam kotayı aşacaksa en uzun süredir kullanılmayan anonim kayıt tahliye
edilir. Aynı erişim anında oluşturulan kayıtlar artan oluşturma sırasıyla
deterministik ayrılır. Kimliği doğrulanmış kayıt anonim oturumdan önce tahliye
edilmez. Depo yalnız kimlikli oturumlarla doluysa yeni giriş fail-closed hata
olur; var olan kullanıcı rastgele düşürülmez.

10 dakikalık anonim ve 30 dakikalık kimlikli ömür **mutlaktır**. Geçerli
erişim yalnız tahliye sırasını günceller, son geçerlilik anını kaydırmaz.

## Deployment sınırı

Bu depo process-local'dır. Mevcut `--web-proxy` güvenlik profili tek bir zee
runtime process'i içindir. Birden çok runtime process'i ortak oturum ve anlık
revoke sözü veremez; production ölçekleme, aynı özet/expiry/rotation
semantiğini atomik sağlayan paylaşımlı depo adaptörü gelene kadar bu profilin
dışındadır. Sticky session bu güvenlik sözünün yerine geçmez.

## Sonuçlar

- Anonim CSRF istekleri process belleğini sınırsız büyütemez.
- Tahliye tekrarlanabilir, aktif kimlikli oturumu kurban seçmez.
- Mutlak ömür kullanıcının sürekli trafiğiyle sonsuza uzamaz.
- Per-IP/rate-limit, dağıtık ortak depo, depolama backend'i ve kota
  yapılandırması B-046'nın açık deployment dilimidir.
- Üç yeni test anonim LRU tahliyesini, anonimin kimlikli kayda yer açmasını ve
  yalnız kimlikli kayıtlarla dolu deponun fail-closed davranışını korur;
  expiry testi erişimin ömrü kaydırmadığını ayrıca kanıtlar.
