# ADR-168 — İstek boyut sınırları: multipart metin alanı, gövde aşımı rotası, rota bazlı gövde ve SQL parametre tavanı

- **Durum:** kabul — Dilim 1 (madde 1, 2, 6) ve Dilim 2 (madde 3, 4, 5)
  uygulandı
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-391, `urun-h` (ZM-F036, ZM-F037), ADR-059 (core freeze),
  ADR-064 (dil yüzeyi), ADR-163 / RFC-0041 katman B (multipart), ADR-061 /
  spec/26 (`.parca`), K-372 (yanıt yazım bütçesi), K-374 (istek artığı
  boşaltması), K-332 (yükleme ön kapısı), spec/11 §5, spec/25, GB-164, GB-165

## Bağlam

ZeeMail (`urun-h`) iki boyut sürtünmesi bildirdi:

- **ZM-F037.** Web gövdesi 64 KiB'tır (spec/11 §5). Tarayıcı formu
  `application/x-www-form-urlencoded` gönderir ve her Türkçe harfi
  (`ı` = `%C4%B1`) altı bayta şişirir; 25 000 harflik Türkçe-yoğun ileti
  rota çalışmadan 413 alır, kullanıcı taslağı kaybedebilir. Ürün geçici olarak
  30 000 harf sınırı koydu. K-378 multipart yolu metin alanlarını alan başına
  8 KiB ile sınırlıyor ve tam bir dosya parçası istiyordu; dosyasız yazışma
  formu bu yolu kullanamıyordu.
- **ZM-F036.** spec/25 tek SQL parametresini 64 KiB ile sınırlar; ürün 64 KB
  üstü gövdeyi 20 000 harflik parçalara bölüyor.

Canlı web işçileri bildirimsiz projelerdir; davranışları değişmemelidir.

## Karar

### Dilim 1 — uygulandı

1. **Multipart metin alanları sıradan form alanıdır (b).**
   `çok_parçalı_yükleme doğru olsun` projesinde multipart gövde dosya
   parçası taşımayabilir (yalnız metin alanlı form). Alan başına 8 KiB
   tavanı kalkar; bütün metin alanlarının ham UTF-8 baytları toplamı, aynı
   isteğin urlencoded form bütçesine (varsayılan 64 KiB, madde 3'te rota
   bazlı) tabidir; aşım 413'tür. Türkçe metin multipart'ta şişmez: 25 000
   Türkçe-yoğun harf ~30–35 KB'tır. Dosya parçası varsa `_csrf` ondan önce
   gelmek zorundadır (değişmedi); dosyasız formda CSRF yorumlayıcının olağan
   form denetimindedir (urlencoded ile aynı). Parça sayısı 8, başlık
   bloğu 8 KiB, zarf `16 MiB + form bütçesi` değişmez. Bildirimsiz projede
   multipart gövde bugünkü gibi 64 KiB metin yoludur.
2. **Gövde aşımı rotası (c).** `gövde_aşımı_rotası "/yol" olsun` yazan
   projede native adaptör, güvenli olmayan yöntemli bir isteği gövde boyutu
   yüzünden reddettiğinde (bildirilen `Content-Length` zarfı aşar, multipart
   dosya ya da metin bütçesi aşılır) ham 413 yerine aynı bağlantıda o yolu
   **GET** rotası olarak çalıştırır. Rota gövdesiz, isteğin çerezleri ve
   oturumuyla çalışır ve `govde_asimi_yolu` (sorgusuz özgün yol),
   `govde_asimi_siniri`, `govde_asimi_bayti` sorgu alanlarını görür. Rotanın
   200 metin/HTML yanıtı **413** durumu ve `Cache-Control: no-store` ile
   gider; yönlendirme, dosya ya da başka durum düz 413 metnine döner.
   Yanıttan sonra K-374 boşaltması uygulanır.
   - **Özgün rota çalıştırılmaz.** Unsafe rotanın CSRF belirteci okunmamış
     gövdededir; gövdesiz unsafe rota yürütmek CSRF'siz yazım olurdu.
     Kısmi gövdeyle yürütme spec/11'in kısmi kabul yasağını çiğner.
   - **GET rotası salt okumadır** (T045): aşım sayfası durum değiştiremez.
     Sorgu alanları güvenilmezdir (istemci aynı sayfayı doğrudan açabilir);
     yalnız bilgi amaçlıdır.
   - **Taslak:** rota taslak içeriğini göremez. Tarayıcı Geri ile form
     durumunu geri getirir; sayfa bunu söylemeli. Sunucu tarafı taslak
     kaydı ürünün sorumluluğudur (açık).
6. **Tanı P022.** K-391 proje alanlarının geçersiz değeri ya da eksik eşi
   P022'dir. Yeni alanlar K-167 dil yüzeyine `profil` satırı olarak girer
   (`govde-asimi-rotasi-v1`, `genis-govde-rotalari-v1`,
   `veritabani-parametre-siniri-v1`).

### Dilim 2 — uygulandı

3. **Rota bazlı gövde sınırı (a).** `geniş_gövde_rotaları "/posta/gonder",
   "/taslak/" listesi olsun` ve `geniş_gövde_sınırı 262144 olsun` birlikte
   yazılır. Liste ögesi `/` ile başlar; `/` ile biten öge önek, diğeri tam yol
   eşleşmesidir. Yalnız POST/PUT/PATCH/DELETE isteğinde, sorgusuz yol
   eşleşirse urlencoded gövde ve multipart metin bütçesi `geniş_gövde_sınırı`
   olur. Alt sınır 65 537, **tavan 1 MiB**; en çok 32 öge. Eşleşme başlıktan,
   gövde okunmadan yapılır; böylece sınır `Content-Length` zarfında uygulanır.
   Alan sayısı (100) değişmez. Yorumlayıcının ikinci denetimi aynı bütçeyi
   iç çerezle alır. Hermetik `dil dene` adaptöründe rota bazlı bütçe yoktur
   (açık).
4. **Orantılı multipart okuma bütçesi.** Multipart isteğin okuma son tarihi
   `kabul + 10 sn + okunan_bayt / 128 KiB/s`'dir; bütçe yalnız gerçekten
   alınan baytla büyür ve en çok `10 sn + Content-Length / 128 KiB/s`'e
   (≤ 146 sn) ulaşır. K-372 yazım hızıyla aynı taban hız seçildi.
   Urlencoded ve ikili (`application/octet-stream`) yol 10 sn mutlak kalır.
5. **SQL parametre tavanı (d).** `veritabanı_parametre_sınırı 1048576 olsun`
   tek parametreyi 65 536–4 194 304 bayt arasında yükseltir; tüm
   parametrelerin toplamı her durumda `100 × 64 KiB = 6 400 KiB`'ı (eski
   örtük en kötü durum) aşamaz. Sorgu metni 64 KiB, parametre sayısı 100,
   sonuç 16 MiB değişmez. Tavan PostgreSQL oturumuna bağlantıdan sonra
   verilir (`VeritabaniBildirimi` değişmez); hermetik/replay adaptörü
   parametre tavanı uygulamaz (değişmedi).

## DoS gerekçesi

- **Bellek:** form gövdesi isteğe 1 MiB'a kadar bellekte kalır; çalışma
  heap'i 64 MiB, metin 16 MiB. Multipart yeniden kodlanan iç gövde en çok
  3 × metin bütçesidir. SQL toplamı eski en kötü değerdedir.
- **Zaman:** başlıklar hâlâ 10 sn mutlak; damlatan istemci bütçe kazanmaz
  (1 bayt 7,6 µs). İşçiyi 10 sn'nin ötesinde tutmak için en az 128 KiB/s
  gerçek veri göndermek gerekir; tutma süresi bant genişliğiyle orantılı ve
  146 sn ile sınırlıdır. Dosya parçası öncesi CSRF ön kapısı ve oturum çerezi
  şartı (K-378) korunur. Urlencoded/ikili yol 10 sn mutlak kalır.
- **Canlı `.parca` (Dilim 2):** canlı ömür 10 + 30 + 139 + 2 = 181 sn'den
  146 + 30 + 139 + 2 = 317 sn'ye çıkar; yetim eşiği 4 kat payı korumak için
  15 dakikadan 30 dakikaya çekilir (derleme zamanı denetimi).
- **Aşım rotası:** ek iş bir GET rotasıdır; uç nokta oran sınırına ve
  bağlantı iznine tabidir, sonra K-374'ün 2 sn / 16 MiB boşaltması gelir.

## Varsayılanlar

Alan yazmayan projede: urlencoded 64 KiB, multipart kapalı, 413 ham, SQL
parametre 64 KiB, ikili yükleme 10 sn — bayt bayt aynıdır; yorumlayıcıya
giden istek zarfı da değişmez (iç `__zee_ic_govde_siniri` çerezi yalnız
varsayılan dışı bütçede eklenir, istemcinin aynı adlı çerezi atılır).
Davranış yalnız `çok_parçalı_yükleme` açık projelerde gevşer (dosyasız form
ve > 8 KiB metin alanı artık kabul; metin bütçesi aşımı 400 yerine 413).
Dilim 2'nin tek gözlenebilir genel değişikliği yetim `.parca` eşiğinin
30 dakika olmasıdır.

## Reddedilen seçenekler

- **Rota cümlesinde boyut bildirimi** (`… adresine 256 KiB gövdeyle istek
  geldiğinde`): gramer değişikliği; core freeze eşiğini aşar ve rota yolu
  çalışma zamanı ifadesi olabildiği için gövde okunmadan eşleşemez.
- **Genel 64 KiB yükseltmesi:** canlı işçilerin DoS profilini değiştirir.
- **413'te özgün rotayı işaretle çalıştırmak:** CSRF'siz unsafe yürütme.
- **Akışlı/parçalı SQL API:** 4 MiB'a kadar metin tek parametreyle sığar;
  daha büyük içerik dosyaya (`.parca`, K-384) aittir. Açık iş olarak kalır.

## Açık

Hermetik `dil dene` adaptöründe aşım rotası (bayt düzeyi zarf yok); sunucu
tarafı taslak kaydı; ikili (`application/octet-stream`) yolun orantılı
bütçesi; 4 MiB üstü SQL metni.
