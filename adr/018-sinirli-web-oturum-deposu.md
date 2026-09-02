# ADR-018 — Sınırlı web oturum deposu ve mutlak ömür

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-106, K-137, B-046, V1-P0-16, ADR-034

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

## Tarihsel deployment sınırı

K-106 anında bu adaptör process-local'dı ve `--web-proxy` profili tek bir zee
runtime process'iyle sınırlıydı. K-137/ADR-034 bu production sınırını kaldırdı:
process-local adaptör deneysel/öğretici kipte kaldı; production artık aynı
proje kökündeki kalıcı ortak depoyu ve N ayrı tek-worker süreci kullanır.
Sticky session hâlâ ortak revoke veya ortak rate-limit sözünün yerine geçmez.

## Sonuçlar

- Anonim CSRF istekleri process belleğini sınırsız büyütemez.
- Tahliye tekrarlanabilir, aktif kimlikli oturumu kurban seçmez.
- Mutlak ömür kullanıcının sürekli trafiğiyle sonsuza uzamaz.
- Per-IP/rate-limit ve süreçler arası kalıcı ortak depo K-137/ADR-034 ile
  gerçeklendi; çok-hostlu harici backend ve kota yapılandırması ileri
  deployment dilimidir.
- Üç yeni test anonim LRU tahliyesini, anonimin kimlikli kayda yer açmasını ve
  yalnız kimlikli kayıtlarla dolu deponun fail-closed davranışını korur;
  expiry testi erişimin ömrü kaydırmadığını ayrıca kanıtlar.
