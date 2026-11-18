# ADR-160 — Herkese açık rota için dar önbellek başlığı izni

- **Durum:** kabul
- **Tarih:** 18 Kasım 2026
- **İlgili kayıt:** K-335, RFC-0037, ADR-059 (core freeze), ADR-064 (dil
  yüzeyi), ADR-067 (dogfood korpusu), ADR-034 (web güvenlik profili),
  ADR-158 (kayar oturum), spec/12 §1 ve §K-335, GB-132

## Bağlam

Çalışma zamanı K-088'den beri her yanıta sabit `Cache-Control: no-store`
yazar (spec/12 §5). zee-cekirdek (exact ürün commit'i
`abaca3aba0e90f57d26435c4befffa903346c425`) ~9.300 adresli içerik sitesinin
oturumsuz sayfalarını (`GET /robots.txt`, `GET /sitemap.xml` ve site
sayfaları) paylaşılan önbellekte tutmak için ön kapıda `Cache-Control`
başlığını gizleyip yol eşlemesiyle yeniden yazıyor. "Bu yanıt oturumdan
bağımsız" kararı kaynaktan uzakta, bir yol listesinde duruyor: eşleme
yanılırsa Set-Cookie'li ya da oturuma bağlı yanıt paylaşılan önbelleğe girer
ve oturum başka kullanıcıya sızar. Talep W3 (P1): rota düzeyinde dar izin,
yalnız GET/HEAD ve Set-Cookie/oturum okuması yoksa `public`.

Core freeze yeni kalıbı varsayılan olarak reddeder. İhtiyaç gerçek ürün
dosyasında (`cekirdek/kaynak/rota_site.dil`) duruyor ve mevcut yüzeyle
karşılanamıyor: dilde yanıt başlığı yazılamaz, `no-store` sabittir.

## Karar

1. Rota önsözünün mevcut açık politikası tek bir biçimle genişler:

   ```zee
   GET "/sitemap.xml" adresine istek geldiğinde
       herkese açık önbelleklenebilir 300 saniye
   ```

   N 1–86400 arası sabit TamSayıdır; başka biçim S043. Cümle erişim
   bakımından `herkese açık`tır, "ilk ve tek politika" kuralına uyar. Yeni
   kalıp kelimesi yalnız `önbelleklenebilir`dir.
2. Derleme (etki geçişi, ad çözümlemesinden önce):
   - GET/HEAD dışı yöntemde **T056**;
   - rota gövdesinde `oturum`/`çerezler` okuması ya da gövdenin veya geçişli
     çağırdığı işlemlerin `csrf belirteci`, çerez yazma/silme, oturum
     açma/kapatma, web oturumu listeleme/iptal kullanması **T057**.
   Talepteki "uyarı" yerine hata seçildi: önbelleklenebilir rotada oturum
   kullanımı her zaman sızıntı adayıdır; uyarı üretime taşınabilirdi.
3. Çalışma zamanı (savunma derinliği, statik kapıdan bağımsız):
   - önbelleklenebilir rotaya boş `oturum` ve `çerezler` sözlüğü verilir;
   - `Cache-Control: public, max-age=0, s-maxage=N` yalnız GET/HEAD,
     gövdeli 200, **hiç Set-Cookie yok** ve istek oturum yüzeyine
     dokunmadıysa yazılır; aksi hâlde `no-store`. Karar yanıt başlıkları
     kurulurken, çerez başlıkları hesaplandıktan sonra verilir; böylece
     kayar oturum yenilemesi (ADR-158) gibi rotanın görmediği Set-Cookie de
     `public`i engeller.
   - Diğer güvenlik başlıkları (nosniff, CSP, HSTS…) değişmez.
4. Adaptör arayüzü `GirdiCikti::yanit_onbellegine_izin_ver` varsayılan
   olarak izni yok sayar; yalnız native web adaptörü başlığı değiştirir.
   Deterministik IO izi bu çağrıyı kaydetmez (girdi değil, başlık ayrıntısı).

## Değerlendirilen seçenekler

- **Ayrı önsöz cümlesi (`300 saniye önbelleklenebilir`, politikadan
  sonra)** — reddedildi. `oturum gerekli` ile birleşebilir ve "önce
  politika, sonra alan" sırasına üçüncü bir tür ekler; birleşimi ayrıca
  yasaklamak gerekirdi. Politika biçimi kimlikli rotayı yapısal olarak
  dışarıda bırakır.
- **Proje düzeyi yol listesi (`proje.dil`)** — reddedildi; bugünkü ön kapı
  eşlemesinin aynısıdır, karar yine rotadan uzakta kalır.
- **Genel yanıt başlığı yazma (`"Cache-Control" başlığına … yaz`)** —
  reddedildi; güvenlik başlıklarını rota denetimine açar, `private`/
  Set-Cookie birleşimini dil kuralı olmaktan çıkarır.
- **`max-age=N` (tarayıcı önbelleği)** — reddedildi; çıkış/girişten sonra
  bayat kişisel olmayan sayfa sorun değildir ama ürün ihtiyacı yalnız
  paylaşılan önbellek; `max-age=0` ön kapı temizliğini tek yerde tutar.
- **Uyarı (`dil denetle` lint)** — reddedildi (yukarıda).
- **Üst sınır yok / bir yıl** — reddedildi; ürün 60–300 sn kullanıyor, bir
  gün dar ve gerekirse genişletilebilir.

## Sonuçlar

- Ürün ön kapıdaki `proxy_hide_header` eşlemesini kaldırıp başlığa
  güvenebilir; Set-Cookie'li yanıtın `public` olması dilde imkânsızdır.
- Önbelleklenebilir rota oturum içeriği gösteremez; kişiselleştirilmiş
  parça ayrı rotaya (ör. istemci tarafı istek) taşınmalıdır.
- Kayar oturumlu projede oturumu olan ziyaretçinin yanıtı Set-Cookie
  taşıdığı için `no-store` döner; paylaşılan önbellek bu yanıtı tutmaz,
  oturumsuz ziyaretçinin yanıtı tutulur. Bu bilinçli fail-safe'tir.
- `Vary`, ETag ve önbellek temizleme açık sınırdır.
- Freeze yüzeyi: kalıp `önbelleklenebilir`, tanılar T056/T057
  (`dil-yuzeyi-v1.tsv`, 1.0.0-dev).

## Kanıt

- `compiler/tests/web_testi.rs::onbelleklenebilir_rota_yalniz_oturumsuz_get_head_rotasinda_derlenir`
- `compiler/tests/web_testi.rs::onbellek_izni_yalniz_eslesen_onbelleklenebilir_rotada_adaptore_gider`
- `compiler/tests/web_cok_surec_testi.rs::onbelleklenebilir_rota_set_cookie_uretince_asla_public_olmaz`
- `compiler/src/main.rs::onbellek_izni_yalniz_no_store_satirini_public_s_maxage_yapar`
- Dogfood: [`dogfood/zee-cekirdek/onbellek-basligi`](../dogfood/zee-cekirdek/onbellek-basligi/README.md)
