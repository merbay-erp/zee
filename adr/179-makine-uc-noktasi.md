# ADR-179 — Makine uç noktası: CSRF muafiyeti yalnız kimliksiz rotaya

- **Durum:** kabul
- **Tarih:** 26 Kasım 2026
- **İlgili kayıt:** K-405, RFC-0046, ADR-034 (web güvenlik profili), ADR-059
  (core freeze), ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu), ADR-160
  (önbelleklenebilir rota), ADR-163 eki (K-385 dosya türleri), ADR-176
  (K-400 üretim sözleşmesi), spec/12 §1 ve §K-405, GB-180

## Bağlam

Ürün H (ZeeMail) klasik masaüstü Outlook için POX autodiscover
(`POST /autodiscover/autodiscover.xml`, `text/xml` gövde, `application/xml`
yanıt) sunamıyor: her POST rotası `_csrf` ister (spec/12 §3), `herkese açık`
CSRF'yi kapatmaz ve gövde her zaman form olarak ayrıştırılır. Sürtünme
`ZM-F047` gerçek ürün dosyasında duruyor (exact provenance
[dogfood kesitinde](../dogfood/urun-h/outlook-autodiscover/README.md)).
Thunderbird (GET autoconfig, K-385), Apple profili ve SRV zaten çalışıyor.

Core freeze yeni yüzeyi varsayılan olarak reddeder; bu iş merkezde 26 Kasım
2026'da güvenlik sınırlarıyla onaylandı. Mevcut yüzeyle karşılanamaz:
CSRF'yi rotada kapatan bir yol ve ham gövde okuması yoktur.

## Karar

1. **Yüzey.** Mevcut açık politikanın tek contextual biçimi:

   ```zee
   POST "/autodiscover/autodiscover.xml" adresine istek geldiğinde
       herkese açık makine uç noktası
       gövde isteğin ham gövdesi olsun
   ```

   AST'de `RotaPolitikasi.kip` (`Olagan` / `Onbellekli(N)` / `Makine`) tek
   alandır; K-335 önbellek biçimiyle birbirini dışlar. Başka yazım (oturumlu,
   yetkili, tek başına) **S050**. Yeni kalıp kelimeleri `makine`, `gövdesi`;
   yeni intrinsic `web.istek_ham_govdesi`.
2. **Derleme (etki geçişi).**
   - POST dışı yöntem, önekli ya da sabit olmayan yol **T060**;
   - rota gövdesinde `oturum`/`çerezler` okuması ya da gövdenin veya geçişli
     çağırdığı işlemlerin `csrf belirteci`, çerez yazma/silme, oturum
     açma/kapatma, web oturumu listeleme/iptal kullanması **T061** (K-335
     T057 ile aynı iz);
   - `isteğin ham gövdesi`nin makine rotası dışında (sıradan rota, işlem,
     eylem, üst düzey) okunması **T062**.
3. **Native adaptör** (gövde okunmadan, derlenen makine yolu kümesiyle):
   - proxy kipinde Host/Forwarded/HTTPS denetimi aynen, **Origin aranmaz**;
   - `Cookie` başlığı okunmaz; oturum anonim başlar, kayar yenileme yok;
     yanıtın çerez başlıkları bastırılır (hiçbir koşulda `Set-Cookie`);
   - tek `Content-Type`, medya türü `text/xml` / `application/xml` /
     `application/json`, `charset` varsa `utf-8`; aksi **415** (yoksa 415,
     yinelenen başlık 400);
   - Content-Length > 8 KiB gövde okunmadan **413**; geçersiz UTF-8 **400**;
   - yorumlayıcıya açık boş çerez satırlı zarf gider (gövde `çerez ` ile
     başlasa da çerez sayılmaz); iç oturum alanları eklenmez;
   - uç nokta oran sınırı (`POST <yol>` kovası) aynen; CSRF kovası
     kullanılmaz.
4. **Yorumlayıcı** (savunma derinliği, adaptörden bağımsız): makine yoluna
   giden POST'un çerez satırı atılır, gövde form olarak ayrıştırılmaz, 8 KiB
   ikinci kez denetlenir; makine rotasına boş `oturum`/`çerezler` verilir ve
   CSRF denetimi yapılmaz. Makine sınıfı istek makine olmayan bir rotaya
   düşerse ham gövde verilmez, o rota `_csrf` bulamaz ve 403 ile kapanır.
5. **Değişmeyenler.** CSP ve güvenlik başlıkları (`no-store` dahil), sıradan
   POST'un Origin + CSRF kapısı, başka rotaların gövde/alan sınırları,
   K-400 üretim sözleşmesi (deneysel kip yalnız localhost).

## Değerlendirilen seçenekler

- **`herkese açık` CSRF'yi kapatsın** — reddedildi; açık form rotalarının
  (giriş/kayıt) korumasını kaldırır.
- **Ayrı önsöz cümlesi** — reddedildi; kimlikli politikayla birleşebilir.
  Politika biçimi kimlikli rotayı yapısal olarak dışlar (ADR-160 gerekçesi).
- **Proje düzeyi muaf yol listesi** — reddedildi; karar rotadan uzakta,
  oturuma dokunmama derlemede kanıtlanamaz.
- **Yalnız çalışma zamanı boş oturum (statik hata yok)** — reddedildi;
  oturum kullanan makine rotası sessizce yanlış çalışırdı. Statik hata +
  çalışma zamanı boşaltma birlikte seçildi.
- **Her rotada ham gövde** — reddedildi; form rotasında gereksiz yüzey.
- **Dilde XML ayrıştırma** — reddedildi; XXE/varlık genişletmesi ve
  bağımlılık yüzeyi; ürün tek alan istiyor.
- **Tüm içerik türleri** — reddedildi; text/plain ve form türleri tarayıcının
  preflight'sız gönderebildiği türlerdir, 415 bunları dışarıda tutar.

## Sonuçlar

- Ürün H Outlook POX autodiscover'ı Zee kodunda sunar; yanıt durağan dosya
  yanıtıyla (K-385) ya da türlü metin yanıtıyla (K-406) yazılır.
- CSRF'siz rotanın tarayıcı kimliğine ulaşması dilde imkânsızdır: derleme
  T061, adaptör çerezi okumaz ve Set-Cookie yazmaz.
- Ürün sorumlulukları (RFC-0046 tehdit modeli): adresi günlüğe/tanıya
  yazmamak, alan adına göre farklı yanıt vermemek (sayım), XML'i metin
  bölmeyle sınırlı tutmak.
- Freeze yüzeyi: kalıplar `makine`, `gövdesi`; API `web.istek_ham_govdesi`;
  tanılar S050/T060/T061/T062 (`dil-yuzeyi-v1.tsv`, 1.0.0-dev).
- Kullanıcıya göre üretilen `application/xml` yanıtı K-406 türlü metin
  yanıtıyla (`… metnini "application/xml" türünde gönder`) yazılabilir.
- Açık: büyük/küçük harfe duyarsız yol, PUT/PATCH,
  CORS başlıkları, JSON ayrıştırma.

## Kanıt

- `compiler/tests/web_testi.rs::makine_uc_noktasi_yalniz_kimliksiz_sabit_post_rotasinda_derlenir`
- `compiler/tests/web_testi.rs::makine_uc_noktasi_csrfsiz_ham_govde_alir_siradan_post_degismez`
- `compiler/tests/web_cok_surec_testi.rs::makine_uc_noktasi_origin_csrf_cerezsiz_calisir_413_415_verir`
- `compiler/src/cli/makine_ucu.rs::makine_ucu_yalniz_xml_json_ve_utf8_icerik_turunu_kabul_eder`
- Dogfood: [`dogfood/urun-h/outlook-autodiscover`](../dogfood/urun-h/outlook-autodiscover/README.md)
