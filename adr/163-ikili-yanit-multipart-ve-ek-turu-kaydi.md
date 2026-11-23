# ADR-163 — İkili dosya yanıtı, multipart yükleme ve SMTP ek türü kaydı

- **Durum:** kabul
- **Tarih:** 23 Kasım 2026
- **İlgili kayıt:** K-378, K-373, K-355 (`urun-h`, ZM-F022), RFC-0041,
  RFC-0040 katman C, RFC-0036, ADR-059 (core freeze), ADR-064 (dil yüzeyi),
  ADR-096 (ekli e-posta), ADR-159 (imza okuma), ADR-160 (rota önbelleği),
  spec/11, spec/26, spec/23, GB-153

## Bağlam

Etkin dogfood ürünü `urun-h` (ZeeMail; ürün sürtünme kaydı ZM-F022) web
arayüzünde ek indirme, JavaScript'siz tarayıcı yüklemesi ve PDF/görsel
dışındaki eklerle e-posta gönderimi istiyor. RFC-0041 (K-373) üç eksiği
kaynakla doğruladı: rota yalnız UTF-8 metin/HTML yanıt verebiliyor; spec/26
yalnız `application/octet-stream` + `X-Zee-CSRF` kabul ediyor ve HTML formu
ikisini de gönderemiyor; `zee-smtp-mime-1` dört tür dışını yasaklıyor ve
genişletme yolu yok. 23 Kasım 2026 merkez kararıyla ZeeMail yüzeyleri
dogfood-change istisnasıyla sırayla açılıyor; bu ADR adım 1'dir. Canlı 17 site
ve resmî sitenin davranışı değişmemelidir: her yeni yüzey isteğe bağlıdır.

## Karar

### Ortak içerik türü kaydı (RFC-0040 katman C ile tek model)

`dil::icerik_turu` tek, kapalı tablodur. Her satır kanonik kısa ad, küçük harf
MIME, uzantılar, metin bayrağı (`charset=utf-8` ve UTF-8 strict), isteğe bağlı
içerik imzası ve kullanılabileceği yüzeyleri (web dosya yanıtı, SMTP
varsayılanı, SMTP adayı) taşır. İlk satırlar: pdf, png, jpeg, webp, txt, csv,
ics, docx, xlsx, pptx, zip ve yalnız web için `application/octet-stream`.
HTML, SVG, XHTML, betik, yürütülebilir ve makro taşıyan türler tabloda yoktur;
yeni satır K-167 `bicim icerik-turu.<ad>` kaydı ve ayrı karar ister. RFC-0040
katman C'nin metin sınıfı (`application/xml`, `text/vcard` …) açıldığında bu
tabloya satır olarak girer; ayrı kayıt açılmaz. RFC-0041'in son açık sorusu
böyle kapanır: DAV türleri ortak alt RFC'ye değil, kendi işlerinde satır
olarak girer.

### A — Dosyadan akışlı ikili yanıt

1. **Yüzey tek contextual Sonuç kalıbı:**
   `sonuç <yol> dosyasını <tür> türünde <ad> adıyla ek olarak göndermeyi dene olsun`
   → `TamSayı sonucu` (bayt). Yeni AST/değer türü yoktur; intrinsic
   `web.dosya_yaniti` (`dosya-okuma`, etki `WebAdaptoru` → eylemde T044).
   Yeni kalıp kelimeleri `türünde`, `adıyla`. Hata C013 başarısız Sonuç'tur
   (dosya yaşam döngüsü ailesi); yeni tanı kodu yoktur.
2. **Yalnız `attachment`.** `inline` (PDF/görsel önizleme) bu adımda
   açılmadı: ürün ihtiyacı indirmedir ve `inline` sandbox CSP'nin PDF
   önizlemesiyle etkileşimi ölçülmeden açılmaz. `ETag`, `Range`/206 ve koşullu
   GET de açılmadı (RFC-0040 katman B başlık okuma listesine bağlı).
3. **Başlığı çalışma zamanı kurar.** `Content-Disposition: attachment;
   filename="<ASCII geri düşüş>"; filename*=UTF-8''<RFC 8187>`; SMTP'nin
   mevcut `yuzde_kodla` kodlaması ortak `icerik_turu::rfc8187_kodla`'ya
   taşındı. Ad doğrulayıcısı SMTP'ninkinden sıkıdır: CR/LF dahil denetim,
   çift tırnak ve Unicode yön denetimi de reddedilir.
4. **Dosya yanıtı başlıkları sabittir:** `nosniff`, `no-store`,
   `Content-Security-Policy: default-src 'none'; sandbox`,
   `Cross-Origin-Resource-Policy: same-origin`. K-335 önbellek izni dosya
   yanıtına uygulanmaz.
5. **Yol ve tavan.** spec/26 proje dosya sınırı + SMTP ek okuyucusunun
   `dosyayi_guvenle_ac`'ı (readonly, `O_NOFOLLOW`, açılış öncesi/sonrası kimlik
   karşılaştırması). Tavan 16 MiB (`yukleme_bayti`, yüklemeyle simetrik);
   proje bildirimiyle yükseltme açılmadı.
6. **Yazım bütçesi (RFC-0041 açık sorusu).** Ayrı formül açılmadı: K-372
   mutlak bütçesi `10 sn + bayt / 128 KiB/s` (16 MiB için ~138 sn) ve 10 sn
   ilerlemesizlik kesmesi başlık + gövdenin tamamına tek son tarih olarak
   uygulanır. Gövde 16 KiB parçalarla akar, belleğe ve heap zarfına girmez.
   Akış sırasında kısalan ya da son parçada uzunluğu değişen dosya bağlantıyı
   eksik `Content-Length` ile keser.
7. **İlk yanıt kazanır.** Dosya yanıtı isteğin yanıt taslağıdır; yanıtı
   belirlenmiş istekte başarısız Sonuç'tur. Taslak açık dosya tanıtıcısını
   taşır; istek geri alınırsa tanıtıcı kapanır, commit sonrası yazılır
   (K-327 kopma kuralı aynen).
8. **İz ve hermetik adaptör.** IO izi `dosya_yaniti_gonder(yol, tür, ad)` →
   bayt sonucu; içerik ize girmez. `ToplayanIo` aynı kayıt/ad/tavan kuralını
   uygular, gövdeyi metin dosyası olarak yazar ve (tür, disposition) çiftini
   kaydeder.

## Sonuçlar

- ZeeMail ek, vCard ve CSV indirmesini rotadan, yetki kararı Zee kodunda
  kalarak verebilir. Bildirimsiz/kalıpsız projenin yanıtı bayt bayt aynıdır.
- Kullanıcı kaynaklı dosya hiçbir koşulda origin'de belge/betik olarak
  yorumlanmaz (tür kaydı + sandbox CSP + `attachment` + `nosniff`).
- IDOR (hangi dosyanın kime verildiği) ürün sorumluluğudur; dil yetki kararı
  vermez (GB-153 bilinçli sınır).

## Değerlendirilen seçenekler

- **Genel başlık/yanıt builder'ı:** V1 kararıyla reddedildi.
- **`Metin` yanıtta base64/data URL ya da `X-Accel-Redirect`:** RFC-0041
  Alternatifler'deki gerekçeyle reddedildi.
- **Ayrı dosya yanıtı yazma bütçesi:** K-372 formülü 16 MiB'ı zaten
  kapsıyor; ikinci formül iki ayar ve iki kanıt yolu yaratırdı.
- **Hata kodu C022 (web güvenlik adaptörü):** ad/tür reddi dosya yaşam
  döngüsü Sonuç ailesiyle (C013) aynı yakalama kalıbını kullanır; ürün tek
  `başarısızsa` dalıyla 404/415 verir.
