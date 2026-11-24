# RFC-0039 — Zee ağ sunucusu yetkinliği (çatı RFC)

- **Durum:** taslak — 21 Kasım 2026. Core freeze (K-160A/ADR-059) altında
  uygulanmaz; her katman ayrı alt RFC ve dogfood kanıt kapısıyla açılır.
  **Aşama A1 (adlı dinleyici devri + hizmet bildirimi) kabul edildi ve
  uygulandı — 23 Kasım 2026, K-376, [ADR-162](../adr/162-adli-dinleyici-devri-ve-hizmet-bildirimi.md);**
  **Aşama A2 tasarımı tamamlandı; A2a (bağlantı yüzeyi) ve A2b (sunucu
  TLS'i, STARTTLS) uygulandı — 23–24 Kasım 2026, K-381,
  [ADR-165](../adr/165-ag-dinleyici-baglanti-yuzeyi.md);** **A2c (blok
  okuma, bayt-kesin okuma/yazma, dosyadan aktarım, kısa süreli okuma)
  uygulandı — 24 Kasım 2026, K-384, ADR-165 A2c eki;** **A2d (dosyanın
  RFC 2045 satırlı base64 aktarımı ve kodlanmış aralığı) uygulandı —
  24 Kasım 2026, K-396, [ADR-172](../adr/172-baglantiya-satirli-base64-aktarimi.md);**
  katmanların geri kalanı taslaktır.
- **Tarih:** 21 Kasım 2026
- **İş:** K-354
- **İlgili belgeler:** [spec/08](../spec/08-kalici-dosya.md),
  [spec/09](../spec/09-son-tarih-ve-iptal.md),
  [spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md),
  [spec/14](../spec/14-yapilandirilmis-eszamanlilik.md),
  [spec/21](../spec/21-deterministik-io-izi.md),
  [spec/22](../spec/22-deterministik-io-profili.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [spec/24](../spec/24-kaynak-guvenlik-profili.md),
  [spec/26](../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [ADR-003](../adr/003-ilk-yurutme-modeli.md),
  [ADR-011](../adr/011-intrinsic-yetkinlik-siniri.md),
  [ADR-161](../adr/161-cok-isci-web-profili-ve-acilis-birincilligi.md),
  [RFC-0012](0012-ffi-ve-tehlikeli-sinir.md),
  [RFC-0016](0016-atomik-kalici-dosya.md),
  [RFC-0038](0038-cok-isci-web-profili.md)
- **Freeze sınıfı adayı:** her alt RFC için ayrı `dogfood-change`; çatı RFC
  kendi başına hiçbir yüzey açmaz.

## Özet

Bugün Zee programı yalnız devralınmış bir loopback IPv4 HTTP dinleyicisinin
arkasında web uygulaması yazabilir. Bu RFC, protokol motorları da dahil
olmak üzere **bütünüyle Zee ile yazılmış** bir ağ sunucusunun — ilk hedef:
SMTP (25/587/465), IMAP (143/993), dayanıklı kuyruk ve DKIM/SPF/DMARC içeren
bir e-posta sunucusu — hangi dil ve çalışma zamanı yetkinliklerini istediğini,
bunların bugün neden eksik olduğunu ve core freeze kapılarından tek tek
geçebilecek biçimde nasıl aşamalanacağını tanımlar. Kararlaştırılan ilke:
protokol durum makinesi, ayrıştırma ve politika Zee kodudur; kriptografi,
TLS, DNS tel biçimi ve soket olay döngüsü doğrulanmış Rust crate'lerine
dayanan **dar host intrinsic'leridir**. Ayrı bir Rust/Go servis motoru bu
RFC'nin kapsamında değildir.

## Motivasyon

ZeeOS (Debian tabanlı işletim sistemi ürünü) kendi e-posta sunucusunu Zee ile
yazmak istiyor. Ürün sahibi 21 Kasım 2026'da Rust/Go ile ayrı bir servis
motoru seçeneğini reddetti ve dil merkezinde bu RFC'nin açılmasını istedi.
Ürün bu RFC kabul edilene kadar yalnız faz-0 belgeleriyle (yetkinlik matrisi,
tehdit modeli, protokol kapsam matrisi) ilerler; Zee'de çalışan bir posta
motoru yoktur ve bu RFC onu varmış gibi göstermez.

E-posta sunucusu iyi bir zorlayıcı iş yüküdür: internete açık, uzun ömürlü,
binlerce eşzamanlı oturumlu, ikili veri taşıyan, kriptografi ve DNS'e bağımlı
ve "kabul edilen ileti kaybolmaz" sözü veren bir süreçtir. Burada yeterli olan
yetkinlik başka TCP sunucularına (ör. özel protokoller, LMTP, IRC benzeri)
genellenebilir; bu yüzden yüzey SMTP'ye özel değil, genel ağ sunucusu
yetkinliği olarak tasarlanır.

### Bugünkü durum (2b23547 üzerinde doğrulandı)

| Gerekli | Bugün | Kaynak |
|---|---|---|
| Port dinleme | Zee kodu port açamaz. Native adaptör yalnız `--deneysel-web` ile loopback'e bağlanır ya da devralınan dinleyiciyi kullanır; devralınan soket tam `AF_INET`/`127.0.0.1`/bildirilen kapı olarak doğrulanır | `compiler/src/web_listener.rs` (`endpoint`, `adopt_group`), spec/11 K-259 |
| systemd soket devri | Var ama dar: yalnız çok işçi grubu yöneticisi, `LISTEN_FDS=1`, fd 3, `127.0.0.1:worker-port`; bind'e geri dönüş yok. **K-376 (A1) sonrası:** `proje.dil` `dinleyiciler` bildirimiyle adla çoklu devir ve sd_notify | `compiler/src/cli/web_isci_grubu.rs` (`systemd_soketini_al`), K-343, ADR-161 eki; `compiler/src/cli/adli_dinleyici.rs`, ADR-162 |
| IPv6, ayrıcalıklı kapı, birden çok dinleyici | Yok. **K-376 sonrası:** devralınır, doğrulanır ve tutulur. **K-381 (A2a) sonrası:** bağlantı bloğu kabul eder, satır okur/yazar | aynı; `compiler/src/cli/ag_baglantisi.rs`, ADR-165 |
| Bağlantı/akış değeri | Yok. Tek ağ sunucusu yüzeyi HTTP rota modelidir; Zee bağlantıya ya da bayt akışına dokunamaz. **K-381 sonrası:** satır akışı; **K-384 sonrası:** blok/literal dosyaya, bayt-kesin okuma/yazma, dosyadan bağlantıya akış | spec/11; spec/12 §9 |
| Bayt değeri | Yok. Tür envanteri `TamSayı · Ondalık · Metin · Mantıksal · Liste · Sözlük · Seçenek ...`; binary yalnız diske akıtılan yükleme dosyası (yol + SHA-256 + bayt sayısı) olarak görünür | spec/04, spec/26 |
| Sunucu TLS'i | Yok. TLS yalnız istemci yollarında: SMTP istemcisi `native-tls` (465 implicit, 587 STARTTLS, TLS 1.2 alt sınır), HTTP istemcisi `ureq`+`rustls` | `compiler/src/eposta.rs` (K-210/K-232), `compiler/src/ag_istemcisi.rs` |
| Eşzamanlılık | Tek OS iş parçacığında işbirlikli, deterministik scheduler (`zee-esz-1`). Adaptör çağrısı önleyici olmayan atomik dilimdir; dinamik görev sayısı ve akış "bu sürümün sözü değildir"; çok çekirdek söz verilmez | spec/14 §2/§4, spec/09 |
| Ölçek | Çok çekirdek yalnız süreç düzeyinde: aynı dinleyiciyi paylaşan N işçi süreç (RFC-0038); her işçi istekleri sıralı işler | RFC-0038, ADR-161 |
| Yürütme modeli | Stage 0 ağaç-yürüyen yorumlayıcı; MIME/base64/özet gibi bayt-yoğun işler yorumlanan Zee'de pratik değildir | ADR-003 |
| Kripto | `sha2`, `ed25519-dalek`, `argon2`, `bcrypt`, `chacha20poly1305`, `getrandom` ağaçta; RSA yok; DKIM yok; base64 yalnız iç yardımcı (Zee yüzeyi değil) | `compiler/Cargo.toml` |
| DNS | Yalnız sistem çözücüsüyle A/AAAA (`to_socket_addrs`, ureq çözücüsü); MX/TXT yok; geçici/kalıcı hata ayrımı yok | `compiler/src/eposta.rs`, `compiler/src/ag_istemcisi.rs` |
| Dayanıklılık | Atomik metin yazma/ekleme `sync_all` + dizin `sync_all` ile; binary taşıma no-clobber + dizin eşzamanlama; güç kesintisi/disk dolu kapısı açık | spec/08, spec/26, RFC-0016 |
| Yetkinlik | `ağ-sunucusu` = native loopback web adaptörü; outbound `ağ` yalnız bildirilmiş origin'e | spec/23 |

Sonuç: bugün Zee ile SMTP/IMAP sunucusu yazılamaz; eksik tek bir özellik
değil, yedi bağımsız katmandır.

## Tasarım ilkeleri

1. **Motor Zee'dedir.** SMTP/IMAP durum makinesi, komut ayrıştırma, ESMTP
   yetenek ilanı, alıcı politikası, kuyruk durum makinesi, MIME yapısı ve
   SPF/DMARC değerlendirmesi Zee kodudur. Host intrinsic'i protokol kararı
   vermez ("SMTP sunucusu çalıştır" biçiminde bir intrinsic reddedilir).
   **Çapraz not (K-383, kabul ayrı):** [RFC-0044](0044-dns-posta-kimlik-dogrulama-ve-teslim-guvenligi.md)
   taslağı bu ilkeyi inceltmeyi öneriyor — DMARC politikası Zee'de kalır;
   SPF `check_host`, DKIM/ARC doğrulama ve kanonikleştirme DNS bütçesi ve
   ayrıştırıcı tutarlılığı gerekçesiyle host değerlendiricisi olur. RFC-0044
   kabul edilirse bu madde onunla güncellenecek.
2. **Kriptografi yazılmaz.** TLS, RSA/Ed25519, SHA-256, base64 ve DNS tel
   biçimi doğrulanmış crate'lerle; algoritma Zee'de ya da elle Rust'ta
   yeniden yazılmaz. Gizli anahtar hiçbir zaman Zee değeri olmaz.
3. **Fail-closed.** Eksik platform primitive'i, doğrulanamayan devralınmış
   soket, desteklenmeyen TLS sürümü ya da aşılan bütçe sessizce gevşetilmez;
   süreç başlamaz veya bağlantı reddedilir (spec/08 "zayıf sessiz fallback
   YASAK" kuralının ağ karşılığı).
4. **Grammar en son çaredir.** Yüzey önce mevcut kalıp ailesi, contextual
   ifade ve intrinsic kaydıyla (ADR-011) aranır; yeni anahtar kelime ancak
   bunlar yetmezse ve K-167 uyumluluk yüzeyine bilinçli kayıtla açılır.
5. **Deterministik kanıt.** Her yeni etki spec/21-22 IO izine girer; sahte
   ağ adaptörüyle golden test yazılabilir olmadan yüzey kabul edilmez.
6. **Ölçülmemiş söz yok.** "Binlerce oturum", gecikme ve bellek hedefleri
   ölçümden sonra sayıya bağlanır; bu RFC sayı uydurmaz.

## Katmanlar

Her katman ileride kendi alt RFC'si, ADR'si, spec bölümü, olumlu/olumsuz
testleri, tanı kodları ve güvenlik bulgusu kayıtlarıyla açılır. Aşağıdaki
sözdizimi örnekleri **yalnız fikir taslağıdır**; bağlayıcı değildir ve
freeze yüzeyine yazılmaz.

### A — Dinleyici ve bağlantı

**Amaç:** IPv4 ve IPv6 TCP dinleyicisinden bağlantı kabul etmek, sınırlı
satır okumak, bayt yazmak.

- **Dinleyici kaynağı.** Üretimde dinleyici yalnız dış yöneticiden devralınır
  (aşağıdaki FD devri sözleşmesi). Ayrıcalıklı kapı (<1024) için süreç kendi
  bind'ini yapmaz ve `CAP_NET_BIND_SERVICE` istemez; devir yoksa bind'e geri
  dönüş yoktur, süreç çıkış 2 ile durur. Geliştirme/test için yalnız açık
  bayrakla, yalnız loopback ve ≥1024 kapıya bind izinlidir (bugünkü
  `--deneysel-web` modelinin karşılığı).
- **Bildirim.** Dinleyiciler `proje.dil` içinde adla bildirilir (ör.
  `smtp`, `gonderim`, `smtps`, `imap`, `imaps`); her adın beklenen aile,
  adres, kapı ve TLS kipi (yok / implicit / STARTTLS izinli) vardır.
  Devralınan soket bildirimle tam eşleşmezse süreç başlamaz.
- **Satır okuma.** Tek okuma birimi CRLF ile biten satırdır. Çalışma zamanı:
  - satır başına en çok uzunluk dinleyici bildiriminden gelir (SMTP komut
    satırı için RFC 5321'in 512 oktet tabanı, metin satırı için 1000; IMAP
    için ayrı sınır); aşım ayrı ve sınıflandırılmış bir sonuçtur, satır
    kırpılıp kabul edilmez;
  - **çıplak LF ve çıplak CR reddedilir** (SMTP smuggling sınıfı:
    `<LF>.<LF>`, `<CR>.<CR>` ve karışık biçimler); ret, programın göreceği
    ayrı bir sonuç türüdür, sessiz normalleştirme yapılmaz;
  - komut satırı yalnız ASCII ise `Metin` olarak verilir; ASCII dışı bayt
    içeren komut satırı ayrı bir sonuçtur (Zee'ye geçersiz UTF-8 `Metin`
    olarak sızmaz).
- **Blok okuma.** SMTP `DATA` gövdesi ve IMAP literal'i Zee belleğine
  alınmaz: çalışma zamanı gövdeyi `CRLF.CRLF`'ye kadar (dot-unstuffing ile)
  veya bildirilen literal uzunluğu kadar, boyut tavanı altında, spec/26
  yükleme modelindeki gibi `0600` geçici dosyaya akıtır ve SHA-256'yı aynı
  akışta hesaplar. Zee yol + özet + bayt sayısı görür. Gövde içinde çıplak
  LF/CR politikası (ret ya da sınıflandırılmış işaret) alt RFC'de bağlanır;
  `CRLF.CRLF` dışındaki hiçbir dizi bitiş sayılmaz.
- **Yazma.** Satır yazma CRLF'yi çalışma zamanı ekler; Zee metni CR/LF
  içeremez (yanıt bölme/enjeksiyon). Dosyadan bağlantıya akıtma (IMAP
  FETCH, giden SMTP DATA) ayrı bir intrinsic'tir.
- **Geri basınç.** Giden tampon sınırlıdır; yazma tampon boşalana dek
  bağlantı görevini askıya alır (Katman C), kalan son tarih dolunca bağlantı
  kapanır. Sınırsız sıraya yazma yoktur.
- **Zaman ve sınırlar.** Bağlantı başına okuma boşta kalma süresi, komut
  başına süre, toplam oturum süresi ve TLS el sıkışma süresi; dinleyici
  başına en çok eşzamanlı bağlantı ve uzak adres başına bağlantı sınırı.
  Sınır dolduğunda kabul edilen bağlantıya protokolün kendi "geçici hata"
  satırını yazıp kapatmak Zee'nin işidir; çalışma zamanı yalnız sınırı
  zorlar ve sayar.
- **Uzak adres.** Bağlantının uzak IP/kapısı Zee'ye `Metin` olarak açılır
  (SPF ve hız sınırı için gerekir); proxy protokolü bu RFC'nin kapsamı
  dışındadır.

Fikir taslağı:

```text
"smtp" dinleyicisinde bağlantı geldiğinde
    bağlantıya "220 posta.example.test ESMTP" satırını yaz
    tekrarla
        komut bağlantıdan satır okumayı dene olsun
        ...
```

### B — Sunucu TLS'i

- **Sağlayıcı.** `rustls` (ağaçta zaten `ureq` üzerinden var). Kripto
  sağlayıcısı (`ring` ya da `aws-lc-rs`) Katman D ile tek seçim olarak
  bağlanır; iki ayrı kripto yığını taşınmaz. Sunucu tarafı için
  `native-tls` kullanılmaz (platforma göre değişen politika).
- **Politika.** En düşük TLS 1.2, tercih TLS 1.3; yalnız AEAD şifre
  takımları; yeniden müzakere yok; sıkıştırma yok. Politika dinleyici
  başına gevşetilemez (fail-closed); istemci sertifikası bu RFC'de yok.
- **Anahtar ve sertifika.** Sertifika zinciri ve özel anahtar dosya ya da
  systemd credential (K-339 modeli) olarak bildirilir; yükleyici izin/sahip
  denetimi yapar. Özel anahtar Zee değeri, IO izi içeriği veya tanı metni
  olmaz. Yeniden yükleme (sertifika yenileme) SIGHUP ile ya da süreç
  yeniden başlatmayla — açık soru.
- **Implicit TLS** (465, 993): el sıkışma Zee ilk satırı görmeden önce
  çalışma zamanınca yapılır.
- **STARTTLS** (25, 587, 143): `bağlantıyı TLS'e yükselt` benzeri tek
  intrinsic. Değişmezler:
  - yükseltme anında okunmuş ama tüketilmemiş düz metin tampon **boş
    olmak zorundadır**; değilse yükseltme reddedilir ve bağlantı kapanır
    (STARTTLS komut enjeksiyonu sınıfı);
  - yükseltmeden sonra çalışma zamanı önceki düz metinden hiçbir baytı
    TLS akışına taşımaz;
  - protokol durumunun sıfırlanması (EHLO'nun tekrarı, AUTH'un yalnız TLS
    sonrası ilanı) Zee kodunun işidir; çalışma zamanı `bağlantı TLS mi`
    sorgusunu güvenilir biçimde verir.
- **Giden TLS.** Giden SMTP (MX'e teslim) ayrı istemci profilidir:
  fırsatçı STARTTLS ve doğrulanmış TLS kipleri ayrıdır; mevcut `eposta.rs`
  istemcisi (`native-tls`) bu profile genişletilmez, aynı `rustls` yığınına
  taşınıp taşınmayacağı alt RFC kararıdır.

### C — Uzun ömürlü oturum ve eşzamanlılık

Bugünkü scheduler'da bloklayan adaptör çağrısı atomik dilimdir: bir
bağlantıdan satır beklemek bütün süreci durdurur. Binlerce boşta oturum
(IMAP IDLE) bu modelle mümkün değildir.

- **Bağlantı başına görev.** Her kabul edilen bağlantı, dinleyici
  kapsamının çocuğu olan bir görevdir. Kapsam spec/14 yapılandırılmış
  eşzamanlılığının uzantısıdır: dinleyici kapsamı bitmeden çocuk görev
  yaşamaz; görev hatası yalnız kendi bağlantısını kapatır, kardeşleri iptal
  etmez (bu, spec/14 §3 "ilk hata kardeşleri iptal eder" kuralından **bilinçli
  ayrılıştır** ve alt RFC'de ayrı kip olarak adlandırılmalıdır).
- **Dinamik görev sayısı.** spec/14 §4 bugün dinamik görev sayısını söz
  dışında bırakır; bu katman onu dinleyici kapsamı için, üst sınırı
  bildirilen bağlantı tavanı olarak açar.
- **Engellemeyen IO.** Soket okuma/yazma/kabul ve TLS el sıkışması
  scheduler noktası olur (`bekle` gibi): veri yoksa görev askıya alınır,
  çalışma zamanı OS olay bildirimiyle (epoll/kqueue; `mio` ya da `libc`
  üzerinden dar bir katman — alt RFC seçimi) hazır görevleri ilerletir.
  `zee-esz-1` gözlem sözü korunur: tek iş parçacığı, data race yok,
  aynı girdi izi için aynı sıra. Ağdan gelen olayların sırası dış girdidir
  ve IO izine yazılır (Katman G).
- **Son tarih ve iptal.** Bağlantı görevi spec/09 son tarihini miras alır;
  boşta kalma süresi dolunca okuma noktasında iptal edilir ve yalnız
  `yetişmezse` kolu çalışır (protokolün "zaman aşımı" satırını yazma şansı).
- **Diğer adaptörler.** PostgreSQL, dosya ve DNS çağrıları bugün senkron
  dilimdir. Bağlantı görevleri arasında adil ilerleme için en azından DNS ve
  dosya akıtma engellemeyen olmalı ya da ayrı iş parçacığı havuzunda
  yürütülüp sonucu scheduler noktasında teslim edilmelidir; PostgreSQL'in
  durumu açık sorudur (SMTP oturumu DB kilidine uzun süre bağlı olmamalı).
- **Çok çekirdek.** Dil sözü değildir. Ölçek, RFC-0038'deki gibi aynı
  devralınmış dinleyicileri paylaşan N işçi süreçle alınır; süreçler arası
  ortak durum (kuyruk, posta kutusu tek yazar kilidi) dosya kilidi veya
  PostgreSQL ile ürün düzeyinde kurulur.
- **Boşaltma.** SIGTERM: yeni kabul durur, `kapanıyor` sinyali her bağlantı
  görevine scheduler noktasında teslim edilir (Zee protokol uygun satırı
  yazar), bildirilen boşaltma süresi dolunca kalan görevler iptal edilir.
- **Bellek.** Boşta bağlantı başına bellek ölçülür ve spec/24 bütçesine
  bağlanır; yorumlayıcı çerçevesi (K-178: release ~11 KiB/düzey) ve görev
  ortamı ölçüm konusudur.

### D — Bayt ve kriptografi intrinsic'leri

Tür sistemi değişikliği (yeni `Baytlar` değer türü) yüksek eşiklidir. Bu
RFC önce **tutamak temelli** yolu önerir: bayt verisi diskte veya çalışma
zamanı tamponunda kalır, Zee yol/özet/uzunluk ve `Metin` sonuç görür.

- **Kodlama:** base64 ve quoted-printable kodlama/çözme, dosyadan dosyaya
  akışla ve bayt tavanıyla; küçük `Metin` girdiler (ör. AUTH PLAIN, DKIM
  `b=`) için sınırlı `Metin`→`Metin` biçimi. RFC 2047 encoded-word çözümü
  MIME alt RFC'sinde.
- **Özet:** SHA-256 dosya üzerinde zaten var (spec/23 `dosya-okuma`);
  akışlı özet bağlamı (parça parça besle, sonunda al) DKIM gövde özeti için
  eklenir. DKIM `simple`/`relaxed` gövde kanonikleştirmesinin Zee'de satır
  satır mı yoksa host dönüşümü olarak mı yapılacağı ölçüm sonrası açık
  sorudur.
- **İmza:** RSA-SHA256 (≥2048 bit) ve Ed25519 imzalama/doğrulama. Özel
  anahtar tutamakla (bildirilen dosya/credential adı) kullanılır, Zee'ye
  dönmez; açık anahtar DNS TXT'den gelen base64 `Metin`dir. RSA için
  RustCrypto `rsa` crate'i, zamanlama yan kanalı danışması (RUSTSEC-2023-0071)
  nedeniyle bu RFC tarihinde aday değildir; TLS ile aynı sağlayıcı (`ring`
  veya `aws-lc-rs`) tercih edilir. Seçim, uygulama günü RustSec ve
  `cargo-deny` kayıtlarıyla yeniden doğrulanır.
- **Rastgelelik:** güvenlik amaçlı rastgele değerler (`getrandom`, ağaçta
  var) spec/22 `zee-io-1` tohumlu rastgelelikten ayrı bir yüzeydir; IO
  izinde değeri maskelenir, replay'de deterministik değildir ve bunu açıkça
  söyler.
- **Ertelenen:** gerçek `Baytlar` değer türü; ihtiyaç tutamak yolunun
  ölçülmüş sürtünmesiyle kanıtlanırsa ayrı RFC.
- **Çapraz not (K-383):** kodlama/özet maddeleri
  [RFC-0043](0043-bayt-kodlama-ve-ozet-yuzeyi.md) taslağında
  ayrıntılanıyor (kodlanmış `Metin` + dosya tutamağı, `Baytlar` yine
  ertelendi; ilk dilim ZM-F034 dosya↔base64/QP). İmza maddesi RFC-0044 B3.

### E — DNS

- MX, TXT, A, AAAA sorguları (PTR sonraki adım); sonuç `Sonuç` biçiminde ve
  hata **geçici** (zaman aşımı, SERVFAIL, ağ hatası) ile **kalıcı**
  (NXDOMAIN, NODATA) olarak sınıflandırılmış; SMTP 4xx/5xx kararı buna
  bağlıdır.
- Çözücü: `hickory-resolver` ya da sistem çözücüsüne (`res_query`) dar bağ;
  seçim alt RFC'de. Varsayılan hedef yerel çözücüdür: **gereksinim**,
  loopback stub çözücünün (systemd-resolved) bildirimle izinli tek DNS
  hedefi olmasıdır; doğrulayıcı çözücü beklenmez, DNSSEC doğrulaması bu
  RFC'de vaat edilmez.
  **Çapraz not (K-383, kabul ayrı):** [RFC-0044](0044-dns-posta-kimlik-dogrulama-ve-teslim-guvenligi.md)
  B1 bu maddeyi güncellemeyi öneriyor — çözücü seçimi tel biçimi crate'i +
  bildirilmiş loopback **doğrulayıcı** çözücüye dar stub; `AD` biti yalnız
  loopback'ten güvenilir ve yalnız DANE kararına bağlanır;
  `hickory-resolver` ve `res_query` reddedildi. RFC-0044 kabul edilirse bu
  madde onunla güncellenecek.
  **Durum: ZeeOS sağlamalı; bugün yalnız statik laboratuvar çözücüsü var
  (QEMU user-net); sahada kullanılamaz (ZeeOS P0-Z3 açık).** Düzeltme
  (ZeeOS ADR-076, 21 Kasım 2026): imajın systemd-resolved taşıdığı yönündeki
  önceki yanıt izinli sürüm listesinden okunmuştu, kurulu paketlerden değil.
  r5 sürümünün dpkg durumunda (156 paket) systemd-resolved kurulu değildir ve
  ağ yapılandırması `[DHCPv4] UseDNS=no`'dur; imaj derlemede QEMU user-net
  statik çözücüsünü gösteren sabit bir `/etc/resolv.conf` yazar, bu yalnız
  laboratuvarda çalışır. ZeeOS önerisi (ZeeOS ADR-077, onay bekliyor):
  systemd-resolved stub, üst çözücü DHCP'den, `/etc/resolv.conf` →
  `stub-resolv.conf` sembolik bağı; yeni bir ZeeOS sürümü gerektirir.
- Sınırlar: sorgu son tarihi, yanıt boyutu, kayıt sayısı, TXT parça birleştirme
  kuralı (RFC 7208/6376'ya uygun), CNAME zinciri derinliği.
- Yetkinlik: yeni `dns` yetkinliği; spec/23 `ağ` origin listesinin yerine
  geçmez.
- **Giden teslim yetkinliği:** MX teslimi önceden bilinmeyen kamu
  adreslerine bağlantı ister; bu spec/23'teki bildirilmiş-origin modeline
  uymaz. Ayrı bir `smtp-teslim` yetkinliği: yalnız 25 kapısı, private /
  loopback / link-local / metadata adresleri varsayılan kapalı (SSRF),
  bağlantı ve hacim bütçesiyle.

### F — Dayanıklılık

**Soru:** spec/08 kalıcı dosya blob deposu ve outbox için yeterli mi?

Mevcut primitive'ler: atomik tek dosya yazma/ekleme `sync_all` + üst dizin
eşzamanlama; aynı klasörde süreçler arası sıralı yazarlar; spec/26'da
no-clobber atomik taşıma ve kalıcı dizin eşzamanlama; spec/25 PostgreSQL
transaction'ı. **PostgreSQL durumu: ZeeOS sağlamalı; bugün yok (ZeeOS
P0-Z3 açık)** — r5 imajında postgresql-17 kurulu değildir (aynı ADR-076
düzeltmesi); outbox'ın PostgreSQL kolu bu sağlanana kadar ürün hedefinde
kanıtlanamaz. Kabul akışı bunlarla kurulabilir:

1. `DATA` gövdesi geçici dosyaya akar, SHA-256 aynı akışta (Katman A);
2. geçici dosya `fsync`'lenir ve içerik adresli blob yoluna no-clobber
   taşınır, dizin eşzamanlanır;
3. kuyruk kaydı (outbox satırı veya PostgreSQL transaction'ı) commit edilir;
4. **ancak bundan sonra** Zee `250` yazar.

Eksikler (açılacak işler, bu RFC kapatmaz):

- RFC-0016'nın güç kesintisi/disk dolu kapısı açıktır; e-posta kabulü için
  her `fsync`/rename/commit/yanıt sınırında SIGKILL ve güç kesintisi
  enjeksiyon matrisi gereklidir.
- Disk dolu (`ENOSPC`) ve inode tükenmesi C013 ailesi içinde ayrı
  sınıflandırılmış sonuç olmalıdır; Zee bunu geçici `4xx`'e çevirir.
- Kalıcı dosya proje kökü sınırındadır. ZeeOS yanıtı (21 Kasım 2026): servis
  modeli yazılabilir veri bağlamasını proje kökü görünümünün içine kurar
  (kod salt okunur, veri alt ağaçları aynı bağlamada, rename `EXDEV`'siz).
  Bu yüzden ilk öneri posta deposunun proje kökü içinde bir alt ağaç
  (ör. `veri/posta`) olmasıdır; spec/08 proje kökü kuralı korunur ve yeni
  "kök dışı veri kökü" yetkinliği gerekmez. Ayrı bir bağlama seçilirse
  aynı dosya sistemi (EXDEV) denetimi kabul testinde tekrarlanır.
- Blob dosyası ile PostgreSQL kaydı birlikte atomik değildir; niyet kaydı +
  açılışta uzlaştırma (RFC-0038 açılış birincilliğindeki gibi) ürün
  protokolüdür, çalışma zamanı yalnız primitive'leri ve dizin listeleme /
  yaş bilgisini sağlar.
- Ağ dosya sistemleri kapsam dışıdır (spec/08 ile aynı).

### G — Güvenlik profili ve kanıt altyapısı

- **Yetkinlikler (spec/23):** `ağ-dinleyici` (devralınan ve bildirilen
  dinleyiciler), `tls-sunucusu`, `dns`, `smtp-teslim`, veri kökü yazımı.
  Bugünkü `ağ-sunucusu` (loopback web) anlamı değişmez; adlar alt RFC'de
  K-167 yüzey kaydına girer.
- **Kaynak bütçesi (spec/24):** satır uzunluğu, blok/literal boyutu,
  dinleyici ve uzak adres başına bağlantı, bağlantı başına bellek ve
  tampon, el sıkışma/boşta/komut/oturum süreleri, DNS sorgu bütçesi.
  Varsayılanlar sıkı; gevşetme bildirimle ve üst tavanla.
- **Fuzz hedefleri** (mevcut `lexer_parser`, `morfoloji`, `http_istegi`,
  `wasm_abi` yanına): `satir_okuyucu` (CRLF/çıplak LF-CR/uzunluk),
  `nokta_blogu` (dot-unstuffing ve bitiş), `starttls_gecisi` (yükseltme
  sınırında tampon), `dns_yaniti`; MIME ayrıştırıcısı Zee'de yazılacağı için
  Zee düzeyinde özellik tabanlı test. RC fuzz kapısına eklenmeleri alt RFC
  kararıdır.
- **IO izi (spec/21-22):** kabul, okunan satır/blok olayı, yazılan satır,
  TLS yükseltme işareti, DNS sorgu/yanıtı ve kapanış ize girer; TLS içindeki
  içerik ve kimlik doğrulama yükü maskelenir. Replay, sahte ağ adaptörüyle
  (ADR-003 IO soyutlaması) aynı sırayı ve çıktıyı üretir.
- **Sahte ağ golden testleri:** protokol durum makinesi gerçek soket
  olmadan, betiklenmiş istemci girdisiyle `dil dene` altında test
  edilebilmelidir.
- Her bulgu ve bilinçli sınır `docs/guvenlik-bulgulari-v1.tsv` GB satırı
  alır.

### H — Kabul ölçütleri

Bu çatı RFC ancak şunlar kanıtlandığında "geçici kabul"e aday olur:

1. **En küçük SMTP alıcısı Zee'de:** EHLO/HELO, MAIL, RCPT, DATA, RSET,
   NOOP, QUIT ve STARTTLS; yalnız gerçekten desteklenen uzantıları ilan eder.
   Loopback üzerinde `swaks` ve `openssl s_client -starttls smtp` ile geçer;
   implicit TLS dinleyicisi `openssl s_client` ile geçer.
2. **Saldırı vektörleri:** çıplak LF/CR ve karışık bitiş dizileri ile SMTP
   smuggling örnekleri reddedilir; STARTTLS komutuyla aynı pakette
   gönderilen ek komut TLS sonrasına taşınmaz; uzun satır ve boyut aşımı
   sınıflandırılmış sonuç verir.
3. **Dayanıklılık:** `250` sonrası her sınırda SIGKILL enjeksiyonunda kabul
   edilen ileti kaybolmaz; disk dolu geçici hata üretir.
4. **Uzun oturum yükü:** IMAP IDLE benzeri yük testinde ölçülen sayıda
   (hedef binler) boşta bağlantı uzun süre açık kalır; bildirim gecikmesi,
   RSS ve dosya tanımlayıcı kullanımı ölçülür ve spec/24 bütçesine
   bağlanır; K-173 soak koşusu tekrarlanır (uzun ömürlü tampon eklendiği
   için zorunlu).
5. **Boşaltma ve yeniden başlatma:** systemd soket birimiyle
   `systemctl restart` aralığında yeni bağlantı reddedilmez (K-343
   ölçümünün çok dinleyicili karşılığı).
6. **İzleme:** aynı oturum IO izinden replay edilir ve aynı çıktıyı üretir.

## FD devri sözleşmesi (ZeeOS'un uygulayacağı)

Bu sözleşme systemd `sd_listen_fds` uyumludur. ZeeOS'un servis yöneticisi
systemd'dir (21 Kasım 2026 yanıtı); ek yönetici gerekmez, ZeeOS soket
birimlerini `Accept=no`, `FileDescriptorName=` ve IPv6 için
`BindIPv6Only=ipv6-only` + ayrı IPv4 soketiyle sağlar. ZeeOS tarafındaki
karar kayıtları ZeeOS deposunda tutulur.

**Yönetici (ZeeOS/systemd) sağlar:**

- Her dinleyici için `SOCK_STREAM`, `listen` durumunda, bildirilen aile
  (`AF_INET` veya `AF_INET6`), adres ve kapıya bağlı bir soket; IPv6
  soketlerinde `IPV6_V6ONLY` açık (çift yığın tek soketle değil, iki
  bildirimle).
- Dosya tanımlayıcıları 3'ten başlayıp ardışık; `LISTEN_PID` = hizmet
  sürecinin PID'i; `LISTEN_FDS` = soket sayısı; `LISTEN_FDNAMES` = her
  soketin bildirimdeki adı (`:` ayrılmış; i'nci ad fd `3+i`'ye aittir). Ad
  **zorunludur**; adsız ya da yinelenen ad reddedilir. Ad kuralı
  `^[a-z][a-z0-9_]{0,31}$` (`:` ayraç olduğu için yasak).
- Ad fd başınadır: systemd'de `FileDescriptorName=` soket birimi başına tek
  addır ve IPv6 soketi (`BindIPv6Only=ipv6-only`) ayrı birim ister. Bu yüzden
  çift yığında her aile ayrı ad taşır; ZeeOS ilk tümleşik denemesinde aile
  sonekli adlar üretir (ör. `smtp_v4:smtp_v6:gonderim_v4:gonderim_v6:…`).
  Yukarıdaki bildirim örneğindeki adlar (`smtp`, `gonderim`…) tek aile
  içindir; `proje.dil` bildiriminin aile sonekli adları mı yoksa "aynı
  mantıksal dinleyicinin iki ailesi" kavramını mı taşıyacağı alt RFC A'da
  karar konusudur (Açık sorular).
- **Sıra garanti değildir.** "3'ten başlayıp ardışık" yalnız fd
  numaralarını ve ad↔fd eşlemesini tanımlar; soketlerin hangi sırayla
  geldiğini değil. Canlı gözlem (ZeeOS ADR-076, 21 Kasım 2026; Debian 13
  amd64, çekirdek 6.12, systemd 257.13; tek hizmete bağlı 10 soket birimi):
  `Sockets=` satırı `smtp_v4:smtp_v6:gonderim_v4:…` sırasındayken gelen
  `LISTEN_FDNAMES` `smtp_v4:imaps_v4:smtps_v4:imap_v6:smtp_v6:imap_v4:smtps_v6:imaps_v6:gonderim_v4:gonderim_v6`
  oldu. Her fd'nin adı, ailesi ve kapısı kendi içinde tutarlıydı; yalnız
  sıra bildirim sırasından farklıydı. Aynı koşuda `LISTEN_PID` eşleşti, IPv6
  soketlerinde `IPV6_V6ONLY=1`, hepsi `SOCK_STREAM` + `listen` idi;
  `Type=notify` ile `READY=1` ve SIGTERM→`STOPPING=1` çalıştı. İkinci
  kanıt: r5 imzalı çekirdek + kök dosya sistemiyle amd64 VM'de canlı
  denetimler 37/37 geçti ve fd'ler yine bildirim dışı sırayla geldi
  (`imap_v6:imaps_v6:imap_v4:…`). İmaj kökeni zinciri: Debian OpenPGP →
  `InRelease` → `Packages` → 156/156 paketin `.deb` SHA-256 eşleşmesi;
  kök dosya sisteminde paketinden ayrışan dosya 0.
- Hizmet süreci ayrıcalıksız kullanıcıyla, `CAP_NET_BIND_SERVICE` ve diğer
  yetkinlikler olmadan başlar.
- TLS sertifika ve anahtarları credential dizininden (`CREDENTIALS_DIRECTORY`)
  ya da yalnız hizmet kullanıcısının okuyabildiği dosyadan; Zee çalışma
  zamanı izin/sahip denetimi yapar.
- `Accept=no` (dinleyici kalıcı; yeniden başlatmada bağlantılar çekirdek
  kuyruğunda bekler).
- Hazırlık için `NOTIFY_SOCKET` (sd_notify) veya eşdeğeri; durdurma için
  SIGTERM ve bildirilen durdurma süresi.

**Zee çalışma zamanı garanti eder:**

- Kaynak ve yapılandırma okunmadan önce `LISTEN_PID` kendi PID'i değilse,
  `LISTEN_FDS` bildirilen dinleyici sayısına eşit değilse, herhangi bir ad
  bildirimde yoksa ya da bildirimdeki bir ad gelmemişse, herhangi bir
  soketin türü/durumu/ailesi/adresi/kapısı bildirimle eşleşmiyorsa çıkış 2
  ile durur ve düzeltme önerisini yazar; hiçbir bağlantı kabul edilmez.
- Dinleyicileri **yalnız `LISTEN_FDNAMES` adıyla** eşler, asla konumla:
  doğrulama "beklenen ad kümesi = gelen ad kümesi" artı her ad için
  ilgili fd'nin aile/adres/kapı eşleşmesidir. Sıra karşılaştırılmaz;
  sıra denetimi geçerli bir devri yanlışlıkla reddeder.
- `LISTEN_*` ortam kayıtlarını doğrulamadan hemen sonra siler (alt süreçlere
  sızmaz); soketlere `FD_CLOEXEC` ve engellemeyen kipi kendisi koyar.
- Devir yoksa veya geçersizse **bind'e geri dönmez**.
- Bütün dinleyiciler doğrulanıp açılış işi bittikten sonra bir kez
  `READY=1`; SIGTERM'de `STOPPING=1`, yeni kabulü durdurur ve Katman C
  boşaltmasını durdurma süresinin altında bir boşaltma bütçesiyle yapar.
- Çok işçi süreç profilinde (RFC-0038) devralınan bütün dinleyicileri
  işçilere aynı adlarla aktarır.

Bu sözleşme bugünkü K-343 tek-soket devrinin (`LISTEN_FDS=1`, fd 3,
`127.0.0.1`) genellemesidir; K-343 davranışı değişmeden kalır. K-343 tek
soket aldığı için sıra bulgusundan etkilenmez; ad tabanlı eşleme yalnız
çok soketli devirde (bu sözleşme) gerekir.

## Aşama A1 — adlı dinleyici devri (uygulandı, K-376)

23 Kasım 2026'da ürünün Faz 2 çalışma zamanı kararı (ürün deposunda kabul)
gerçek Zee çalışma zamanını `Type=notify`, ayrıcalıksız ve adla devredilen
dinleyicilerle kabul ölçütü yaptı. Katman A bu yüzden iki adıma bölündü:

- **A1 (uygulandı):** `proje.dil` `dinleyiciler "<ad> <adres>:<kapı>", …`
  bildirimi (P018); `dil çalıştır` yürütmeden önce `LISTEN_PID`/
  `LISTEN_FDS`/`LISTEN_FDNAMES`'i yalnız adla eşler, aile/adres/kapı ve
  IPv6 V6ONLY'yi doğrular, ihlalde çıkış 2; soketleri tutar. `NOTIFY_SOCKET`
  varsa `READY=1`/`STOPPING=1`. Atomik dosya değiştirmede gereksiz `fchown`
  kaldırıldı (`~@privileged` sandbox uyumu). Normatif maddeler
  [spec/12 §8](../spec/12-web-guvenlik-profili.md), karar
  [ADR-162](../adr/162-adli-dinleyici-devri-ve-hizmet-bildirimi.md),
  işletim [docs/adli-dinleyici-devri.md](../docs/adli-dinleyici-devri.md).
  Yukarıdaki FD devri sözleşmesinin "Zee çalışma zamanı garanti eder"
  maddelerinden boşaltma (Katman C) ve çok işçiye adla aktarım A1'de yoktur.
- **A2 (tasarım tamam, 23 Kasım 2026, K-381):** bağlantı kabulü, satır
  okuma/yazma, sınırlar, uzun oturum ve sunucu TLS'i; Katman A'nın geri
  kalanı Katman C'nin dar bir alt kümesiyle (yalnız bağlantı görevleri)
  birlikte açılır. Ayrıntı aşağıdaki "Aşama A2" bölümündedir.

## Aşama A2 — bağlantı yüzeyi (K-381)

A2 üç dilime bölünür; her dilim ayrı commit, ayrı güvenlik kaydı ve ayrı
kabul kanıtı taşır. Dilimler sırayla birleşir; sonraki dilim öncekinin
değişmezlerini gevşetemez.

| Dilim | Kapsam | Durum |
|---|---|---|
| **A2a** | Düz TCP: bağlantı olayı, satır okuma/yazma, kapatma, satır/boşta/ömür/bağlantı sınırları, kibar ret, bağlantı görevi scheduler'ı, `ağ-dinleyici`, SIGTERM boşaltma | uygulanır (K-381) |
| **A2b** | Sunucu TLS'i: implicit TLS dinleyicisi (465/993), STARTTLS (25/587/143), sertifika/anahtar yükleme, TLS 1.2+ AEAD politikası | uygulandı (K-381, ayrı commit) |
| **A2c** | Blok okuma: SMTP `DATA` nokta bloğu ve IMAP literal'i `0600` geçici dosyaya, SHA-256 aynı akışta; dosyadan bağlantıya akıtma; satır sonusuz yazma ve bayt-kesin okuma (ZM-F031); kısa süreli okuma (ZM-F032) | uygulandı (K-384) |
| **A2d** | Dosyanın RFC 2045 satırlı base64 kodlamasını bağlantıya akıtma ve kodlanmış çıktı üzerinde aralık (IMAP `BODY[n]`, `BODY[n]<o.n>`; ZM-F040) | uygulandı (K-396) |

### A2a — Zee yüzeyi

Yüzey mevcut kalıp aileleriyle kurulur; yeni anahtar kelime ya da grammar
kuralı yoktur (rota kalıbının bağlantı karşılığı, `dene` ailesi):

```text
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    bağlantıya "220 mx.ornek.test ESMTP" satırını yaz
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            ...
            bağlantıya "250 tamam" satırını yaz
        değilse
            açık yanlış olsun
    bağlantıyı kapat
```

- `"<ad>" dinleyicisinde bağlantı geldiğinde` + blok: üst düzey olay
  kaydıdır (`... adresine istek geldiğinde` gibi). Ad `proje.dil`
  `dinleyiciler` bildiriminde olmalıdır; değilse yürütme C030 ile durur.
  Aynı ada iki blok, web sunucusuyla (`sunucu başlat`) aynı programda
  bağlantı bloğu ve adlı devri olmayan süreçte bağlantı bloğu C030'dur.
  Blok her kabul edilen bağlantı için taze ortamda koşar ve yalnız
  `bağlantı` adlı Sözlük'ü görür: `dinleyici`, `uzak_adres` (IPv4 noktalı,
  IPv6 köşeli parantezsiz kanonik), `uzak_kapı`, `yerel_kapı` (Metin).
- `<ad> bağlantıdan satır okumayı dene olsun` → `Sonuç<Metin>`. Satır CRLF
  ile biter ve CRLF'siz verilir.
- `bağlantıya <Metin> satırını yaz` cümlesi ve
  `<ad> bağlantıya <Metin> satırını yazmayı dene olsun` → `Sonuç<TamSayı>`
  (CRLF dahil kuyruğa alınan bayt). Cümle biçiminin hatası bağlantı
  görevini C030 ile bitirir (yalnız o bağlantı kapanır).
- `bağlantıyı kapat`: giden tampon boşaltılıp bağlantı kapanır; sonraki
  okuma/yazma `kapandı` sonucudur. Blok bitince bağlantı kendiliğinden
  kapanır.
- Bağlantı kalıpları bağlantı bloğu dışında (ve onun çağırdığı işlemler
  dışında) çalışma zamanında C030'dur; işlemler içinden kullanılabilir,
  bağlantı değer olarak taşınmaz (o anki bağlantı görevinin bağlantısıdır).

### A2a — çalışma zamanı değişmezleri

- **Hata sınıfları.** Bütün beklenen bağlantı hataları C030 `Sonuç`'udur ve
  Hata verisinde `tür` taşır: `kapandı` (karşı taraf kapattı ya da
  bağlantı kapatıldı), `zaman_aşımı` (boşta süresi), `satır_uzun`,
  `satır_sonu` (çıplak CR ya da çıplak LF), `kodlama` (geçersiz UTF-8, NUL
  ya da TAB dışı denetim karakteri), `kapanıyor` (SIGTERM boşaltması),
  `yazılamadı` (CR/LF/NUL içeren ya da sınırı aşan Metin, giden tampon
  süresinde boşalmadı, karşı taraf okuma ucunu kapattı). Son tarih
  (`içinde`) dolarsa okuma noktasında Ç001 iptali olur ve yalnız
  `yetişmezse` kolu çalışır (spec/09; K-375 modeli).
- **Satır kuralı.** Tek satır sonu CRLF'dir. Çıplak LF, çıplak CR ve
  `CR CR LF` gibi karışık diziler satır sonu sayılmaz ve normalleştirilmez:
  bozuk satır sonuna kadar tüketilir ve `satır_sonu` döner (SMTP smuggling
  sınıfı). Sınırı (`bağlantı_satır_sınırı`, CRLF dahil) aşan satır kırpılıp
  kabul edilmez: `satır_uzun` döner, satırın kalanı sonraki CRLF'ye kadar
  atılır. Zee'ye geçersiz UTF-8 Metin sızmaz.
- **Zaman.** Bir okuma çağrısı tam satır gelene kadar en çok
  `bağlantı_boşta_süresi` bekler; süre çağrının başından **mutlaktır**,
  damla damla gelen bayt süreyi uzatmaz (slowloris; K-372 ilkesi). Yazma
  giden tampona sığmazsa aynı süre içinde boşalmalıdır. Bağlantı
  görevinin bütün ömrü `bağlantı_ömrü` mutlak son tarihine bağlıdır;
  dolunca görev iptal edilir ve bağlantı kapanır.
- **Bütçe.** Her bağlantı görevi kendi çalışma adımı/heap bütçesiyle başlar
  ve her başarılı satır okuması bütçeyi yeniler (komut başına bütçe); C023/
  C024 yalnız o bağlantıyı bitirir. K-388 (ZM-F035): protokol birimi
  sınırı girdi tüketen ya da süresini dolduran her okuma dönüşüdür
  (başarı, `zaman_aşımı`, `satır_uzun`, `satır_sonu`, `kodlama`, nokta
  bloğunda `boyut_aşımı`); anında dönen `kapandı` ve okumadan dönen
  `boyut_aşımı` yenilemez (spec/12 §9, ADR-165 eki).
- **Bellek.** Bağlantı başına gelen tampon ≤ satır sınırı + 4 KiB, giden
  tampon ≤ 64 KiB; sınırsız kuyruk yoktur (geri basınç: yazma görevi
  askıya alır).
- **Eşzamanlılık.** Bağlantı başına bir görev; görevler tek iş parçacıklı
  bağlantı scheduler'ında `poll(2)` ile ilerler, bekleyen okuma/yazma ve
  `bekle` scheduler noktasıdır (Katman C'nin yalnız bağlantı görevleri için
  dar alt kümesi). Görev hatası yalnız kendi bağlantısını kapatır,
  kardeşleri iptal etmez: **yalıtılmış bağlantı kapsamı**, spec/14 §3 "ilk
  hata kardeşleri iptal eder" kuralından bilinçli ayrılış. `zee-esz-1`
  sözü korunur: tek iş parçacığı, data race yok; ağ olaylarının sırası dış
  girdidir ve IO izine yazılır. Bağlantı içindeki PostgreSQL/dosya/HTTP
  çağrıları bugünkü gibi senkron dilimdir (açık; Katman C).
- **Sınırlar ve kibar ret.** `bağlantı_sınırı` (süreç geneli eşzamanlı) ve
  `uzak_adres_bağlantı_sınırı` (aynı uzak IP) dolduğunda çalışma zamanı
  bağlantıyı kabul eder, `bağlantı_dolu_yanıtı` bildirilmişse onu tek
  engellemeyen yazımla gönderir ve kapatır; Zee görevi kurulmaz. Ret
  sayılır ve günlüğe oran sınırlı yazılır.
- **Boşaltma.** SIGTERM'de yeni kabul durur, `STOPPING=1` gönderilir,
  bekleyen ve sonraki okumalar `kapanıyor` döner (Zee protokolün `421`
  satırını yazabilir); en çok 10 saniye sonra kalan görevler iptal edilir.
  Bağlantı scheduler'ı başlayınca `READY=1` gönderilir.
- **Yetkinlik.** Yeni `ağ-dinleyici` yetkinliği (spec/23); bildirimli
  projede `yetkinlikler` içinde olmalıdır, `--güvenli` çocuk modunda yoktur.
  `ağ-sunucusu` (loopback web) anlamı değişmez.
- **IO izi.** Bağlantı olayı bekleme ve her bağlantı işlemi (oku/yaz/kapat/
  TLS) ize girer; replay aynı sırayı ve çıktıyı üretir. Hermetik adaptör
  betiklenmiş bağlantıları parça parça teslim ederek eşzamanlı oturum
  sırasını deterministik sınar.
- **Platform.** Unix. Windows'ta bağlantı bloğu C030'dur (adlı devir de
  Windows'ta yoktur, ADR-162).

`proje.dil` alanları (P020; hepsi isteğe bağlı, bildirimsiz varsayılan):

| Alan | Aralık | Varsayılan |
|---|---|---|
| `bağlantı_sınırı` | 1–4096 | 64 |
| `uzak_adres_bağlantı_sınırı` | 1–`bağlantı_sınırı` | 8 |
| `bağlantı_satır_sınırı` (bayt, CRLF dahil) | 64–65536 | 1000 (RFC 5321 metin satırı) |
| `bağlantı_boşta_süresi` (saniye) | 1–3600 | 300 (RFC 5321 §4.5.3.2 komut süresi) |
| `bağlantı_ömrü` (saniye) | 1–86400 | 3600 |
| `bağlantı_dolu_yanıtı` | ≤ 512 bayt yazdırılabilir ASCII, CR/LF yok | yok (yalnız kapatılır) |

### A2b — sunucu TLS'i

- **Bildirim (P021).** `tls_sertifika_değişkeni "POSTA_TLS_ZINCIRI" olsun`
  ve `tls_anahtar_değişkeni "POSTA_TLS_ANAHTARI" olsun`: PEM zincir ve özel
  anahtar K-339 modeliyle önce systemd `CREDENTIALS_DIRECTORY` dosyasından
  (nofollow, düzenli dosya, sahibi süreç kullanıcısı/root, grup/diğer
  okuyamaz), yoksa ortamdan okunur; bildirim yalnız adları taşır.
  `tls_dinleyicileri "smtps_v4", "imaps_v4" listesi olsun` implicit TLS
  dinleyicileridir. Kimlik açılışta, kaynak yürütülmeden yüklenir; hata
  çıkış 2'dir. Anahtar baytı hiçbir zaman Zee değeri, IO izi içeriği,
  günlük ya da tanı metni olmaz. (Tasarım taslağındaki proje köküne göreli
  dosya yolu, sırrın proje ağacında durmasını özendirdiği için
  reddedildi.)
- **Politika.** `rustls` 0.23 + `ring` (ağaçtaki `ureq` yığını; doğrudan
  bağımlılık yapıldı, yeni crate yok, ADR-038), TLS 1.3 ve TLS 1.2, yalnız
  ileri gizlilikli AEAD takımları (TLS 1.3 üç takım; TLS 1.2
  ECDHE-ECDSA/RSA AES-GCM ve CHACHA20-POLY1305), yeniden müzakere ve
  sıkıştırma yok, istemci sertifikası yok. Politika bildirimle gevşetilemez.
- **Implicit TLS.** El sıkışma, Zee görevi kurulmadan reaktörde
  `bağlantı_boşta_süresi` içinde tamamlanır; başarısızlıkta görev hiç
  kurulmaz. `bağlantı` Sözlüğü `tls` = `evet`/`hayır` taşır.
- **STARTTLS.** `<ad> bağlantıda TLS başlatmayı dene olsun` →
  `Sonuç<Metin>` (ör. `TLSv1_3 TLS13_AES_128_GCM_SHA256`; `Received`
  başlığı için). Zee önce protokolün hazır satırını yazar. Değişmezler:
  - yükseltme anında okunmuş ama Zee'nin tüketmediği düz metin tampon
    **boş olmak zorundadır**; değilse yükseltme yapılmaz, tampon atılır,
    bağlantı kapanır ve C030 `tür=tls` döner (CVE-2011-0411 sınıfı);
    çekirdek tamponunda henüz okunmamış düz metin TLS kaydı olarak
    işlenir ve el sıkışmayı bozar, hiçbir zaman komut olarak görünmez;
  - düz giden tampon yükseltmeden önce boşaltılır; yükseltmeden sonra
    önceki düz metinden hiçbir bayt TLS akışına taşınmaz, satır okuyucu
    sıfırlanır;
  - TLS bağlantısında ikinci yükseltme ve kimliksiz süreçte yükseltme
    `tür=tls`'dir (bağlantı açık kalır); süresinde bitmeyen ya da başarısız
    el sıkışma bağlantıyı kapatır.
  - Protokol durumunun sıfırlanması (EHLO'nun yeniden istenmesi, AUTH'un
    yalnız TLS sonrası ilanı) Zee kodunun işidir; başarı Sonuç'u Zee'nin
    kendi durumunu güncellemesi içindir (ayrı bir sorgu kalıbı açılmadı).

### A2c — blok okuma ve bayt-kesin akış (K-384)

ZeeMail'in (`urun-h`) iki somut ihtiyacı: SMTP `DATA` gövdesi ve IMAP
`APPEND` literal'i satır satır belleğe alınamaz (1000 baytlık satır sınırı,
ikili içerik, 25+ MiB ileti); IMAP `+ ` devam yanıtı ve `{n}` literal'i
satır sonu eklemeden yazma ve tam `n` bayt okuma ister (ZM-F031; Zee
metninde `\r` kaçışı yok, S040). IMAP `IDLE` yeni iletiyi ancak boşta
süresi (ürünce 1800 sn) dolunca fark edebiliyordu (ZM-F032).

Yüzey (yeni anahtar kelime, değer türü ya da koşul grameri yok; `dene`
Sonuç ailesi ve var olan yazma cümlesi):

```text
ileti bağlantıdan nokta bloğunu dosyaya almayı dene olsun      # Sonuç<Sözlük>
ek bağlantıdan uzunluk baytı dosyaya almayı dene olsun          # Sonuç<Sözlük>
parola bağlantıdan uzunluk bayt okumayı dene olsun              # Sonuç<Metin>
bağlantıya "+ " parçasını yaz                                   # satır sonu yok
sonuç bağlantıya "+ hazır" parçasını yazmayı dene olsun         # Sonuç<TamSayı>
sonuç bağlantıya yol dosyasını aktarmayı dene olsun             # Sonuç<TamSayı>
satır bağlantıdan satır okumayı 30 saniye içinde dene olsun     # Sonuç<Metin>
```

Blok sözlüğü web yüklemesinin `yukleme_*` alanlarıyla aynı biçimdedir:
`yol` (`.zee/baglanti-bloklari/<64 hex>.parca`), `sha256`, `bayt` (Metin).
Zee dosyayı `atomik taşımayı dene` ile kalıcı yerine taşır.

Değişmezler (normatif metin spec/12 §9 A2c):

- **Nokta bloğu.** Yalnız birebir `CRLF . CRLF` biter; nokta doldurma
  çözülür, satır sonları CRLF korunur. Bloktaki çıplak CR/LF
  normalleştirilmez ve satır başı sayılmaz: blok `satır_sonu` ile atılır,
  akış gerçek sonlandırıcıya kadar tüketilir — kaçak `<LF>.<LF>` ardındaki
  baytlar asla komut olmaz (SMTP smuggling; CVE-2023-51764/51765/51766 sınıfı).
  Postfix'in `smtpd_forbid_bare_newline` seçeneğinin karşılığı varsayılan
  ve kapatılamaz. Sınır aşımı `boyut_aşımı` ile aynı biçimde tüketilir;
  oturum eşzamanlı kalır (Zee `552`/`554` yazar).
- **Literal.** Tam `n` bayt; fazlası sonraki komuttur. Sınırı aşan `n`
  hiç bayt okunmadan `boyut_aşımı` (Zee senkron literal'de `+` yazmadan
  `NO` döner; LITERAL+ istemcisinde bağlantıyı kapatmak Zee'nin işidir).
- **Dosya.** RFC-0041/spec/26 `.parca` modeli: `0600`, benzersiz ad,
  SHA-256 aynı akışta, başarıda `fsync`, yarım dosya her hatada silinir.
  Ayrı dizin (`.zee/baglanti-bloklari`): web süpürmesinin 15 dakikalık
  eşiği uzun oturumdaki canlı dosyaya dokunamaz; bağlantı süpürmesinin
  eşiği `bağlantı_ömrü` + 15 dk. Tamamlanan dosya bağlantıya aittir,
  sonraki blokta ya da bağlantı sonunda (taşınmadıysa) silinir.
- **Zaman.** Bütçe çağrının başından mutlak `boşta + bayt / 64 KiB/sn`
  (K-372 ilkesi; damla damla bayt uzatamaz). Yarıda kalan blok okuma
  yönünü kapatır (eşzamanı bozuk akış komut olarak okunmaz), yazma sürer.
- **Bayt-kesin yazma.** `parçasını yaz` satır kuralına uyar (CR/LF/NUL
  yok); satır sonunu yalnız `satırını yaz` üretir. Dil kaçış kümesi
  genişletilmedi: CRLF'yi her zaman çalışma zamanı üretir, Zee metni ham
  satır sonu taşıyamaz. Aktarım dosyayı 64 KiB geri basınçla akıtır,
  dosya baytı Zee değeri ya da IO izi içeriği olmaz.
- **Kısa süreli okuma.** Süre `bağlantı_boşta_süresi`ni aşamaz; zaman aşımı
  (bütün okumalarda) bağlantıyı kapatmaz ve yarım satırı atmaz. `IDLE`
  bu süreyle döngüde posta kutusunu yoklar.
- **Bağlam ipuçları.** Örtük `bağlantı` Sözlüğü sözcüksel kapsamda kalır
  (işleme dinamik geçiş reddedildi: gizli kapsam, işlemin başka bağlamda
  sessizce farklı davranması); A001 parametre önerir. Yerleşik `yaz`/
  `gönder` ile biten işlem adı cümle olarak çağrılamaz; A001/S037 ipucu
  verir. Tanı kodları ve ret davranışı değişmedi.

Reddedilenler: blok içeriğini Metin olarak vermek (ikili/8 bit ileti,
bellek), satır akışından blok kurmak (1000 bayt sınırı, smuggling yüzeyi
Zee'ye geçer), `\r` kaçışı eklemek (grammar/kaçış yüzeyi, CRLF dışı satır
sonu yazılabilir olur), çıplak LF'yi bloğun içinde normalleştirmek
(smuggling), okuma zaman aşımını Ç001 iptali yapmak (IDLE'de oturum
kaybolur), bağlantılar arası uyandırma (Katman C/E; adım 5).

#### A2c açık soruları — kapandı

- Blok dosyası nerede ve kime ait: **göreli IO kökünde ayrı dizin,
  bağlantıya ait** (yukarıda).
- Sınır: **`bağlantı_blok_sınırı`** (P020; 1 KiB–1 GiB, varsayılan
  32 MiB, SMTP `SIZE` ilanıyla aynı değer verilmeli).
- Zaman: **mutlak `boşta + bayt/64 KiB/sn`**; yarıda kalma okuma yönünü
  kapatır.
- Satır sonusuz yazma: **ayrı `parçasını yaz` kalıbı**, kaçış genişletilmez.
- IDLE: **kısa süreli okuma**; zaman aşımında oturum sürer.

### A2d — satırlı base64 aktarımı ve kodlanmış aralık (K-396)

ZeeMail (`urun-h`) ekleri çözülmüş hâlde paylaşılan veri kökünde tutar
(`veri:ekler/<kiracı>-<sha256>`, RFC-0045). IMAP istemcileri (Outlook,
Apple Mail, Thunderbird) `BODY[n]` gövdesini orijinal aktarım kodlamasıyla,
yani RFC 2045 base64 satırları (76 karakter + CRLF) olarak bir literal
içinde ister; kısmi getirme `BODY[n]<o.n>` kodlanmış çıktı üzerinde
konumlanır. Bugünkü geçici çözüm A2c ve RFC-0043 parçalarıyla 57 baytlık
`base64 okumayı dene` döngüsüydü: her satırda dosya yeniden açılır, her
satır Zee `Metin`i olur. 16 MiB'lık ek 29,6 sn sürüyor ve sonunda C024
(kümülatif değer zarfı) ile kesiliyordu (ZM-F040).

```text
n yol dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun                        # Sonuç<TamSayı>
n yol dosyasının base64 satırlarının konum baytından adet baytını bağlantıya aktarmayı dene olsun  # Sonuç<TamSayı>
```

Yeni anahtar kelime, değer türü, tanı kodu ya da koşul grameri yoktur:
iki kalıp kelimesi (`satırlarıyla`, `satırlarının`) ve iki intrinsic
(`ag.baglanti_base64_aktar`, `ag.baglanti_base64_aralik_aktar`) eklenir;
retler A2c'nin C030 `tür`leridir.

Değişmezler (normatif metin spec/12 §9 A2d):

- **Biçim ve uzunluk.** RFC 4648 §4 dolgulu alfabe; son satır dışında tam
  76 karakter, her satır (son satır dahil) CRLF ile biter; boş dosya sıfır
  bayt. `b` ham bayt için e = 4·⌈b/3⌉, toplam T = e + 2·⌈e/76⌉. Formül
  normatiftir: ürün IMAP literal boyunu (`{T}`) veritabanındaki ham boydan
  SQL ile hesaplar, dosyayı açmaz.
- **Aralık.** `konum`/`adet` T baytlık kodlanmış akış (CRLF dahil)
  üzerindedir; yazılan dilim [min(K,T), min(K+N,T)), dönen sayı
  min(N, max(0, T−K)). K ≥ T boş başarıdır (RFC 3501/9051 kırpması);
  negatif değer dosya açılmadan `geçersiz`. Aralık satır başına hizalı ham
  konuma (`⌊K/78⌋·57`) atlayarak başlar; dosyanın baştan kodlanması
  gerekmez.
- **Akış.** Kodlanmış çıktı Zee değeri olmaz ve değer zarfına sayılmaz;
  bellekte en çok bir ham parçanın (57·256 bayt) kodlaması durur. A2c
  aktarımının 64 KiB geri basıncı, 256 KiB tur dilimi, mutlak
  `boşta + bayt/64 KiB/sn` bütçesi (bayt = yazılan çıktı) ve kırık
  bağlantı/`yazılamadı` kuralları aynen geçerlidir. `bağlantı_blok_sınırı`
  ham dosya boyuna uygulanır.
- **Yol ve yetkinlik.** Kaynak, dosya okumasının merkezî yol denetiminden
  geçer: proje dosyası `dosya-okuma`, `veri:` yolu `veri-okuma`; `veri:`
  yaprağı A2c gibi bağ izlenmeden açılır.

Neden RFC-0039 (A2d), RFC-0043 değil: davranışın güvenlik sınırı
bağlantı akışıdır — geri basınç, süre bütçesi, kısmi yazımda eşzamanın
bozulması ve C030 sınıflandırması RFC-0039 A2c'ye aittir. RFC-0043
süreç içi dosya ↔ `Metin` dönüşümünü ve değer zarfını yönetir; A2d tam
olarak o zarfı devre dışı bırakır. Saf kodlayıcı RFC-0043 Dilim 1'in
`kodlama` modülünü paylaşır, ikinci bir base64 gerçeklemesi yazılmaz.

Reddedilenler: base64'ü `Metin` olarak üretip satır yazmak (C024, ZM-F040'ın
kendisi); RFC-0043'e "kodlayarak oku" eklemek (zarf sorunu sürer); ham
dosya konumuyla aralık (IMAP partial kodlanmış çıktı üzerindedir, ürün
satır hesabını Zee'de yeniden yazmak zorunda kalırdı); satır uzunluğunu
parametre yapmak (RFC 2045 76'yı sabitler, yüzey büyür); quoted-printable
kodlama (ihtiyaç yok, ürün yalnız base64 ek saklar).

### A2 açık soruları — kapandı

- Engellemeyen IO katmanı: **`libc` `poll(2)` üzerinden dar reaktör**
  (yeni bağımlılık yok; bağlantı sayısı `bağlantı_sınırı` ≤ 4096 ile
  sınırlı olduğundan `poll` yeterli); Windows'ta yetkinlik reddi.
- Bağlantı görevinde hata yalıtımı kipinin adı: **yalıtılmış bağlantı
  kapsamı** (yukarıda).
- Tek kripto sağlayıcısı: sunucu TLS'i için **`ring`** (ağaçta, `ureq`
  ile aynı `rustls`); istemci `native-tls` yollarının taşınması ayrı iş.
- Sertifika yenilemede süreç içi yeniden yükleme: **yok**; yenileme süreç
  yeniden başlatmasıyla (`Accept=no` dinleyici kuyrukta bekletir, K-343
  ölçümü).

### A2 açık kalanlar

- Katman C'nin genel hali (PostgreSQL/dosya/DNS'in
  engellemeyen yürütülmesi); çok işçi grubuna adla aktarım; `dil dene`
  içinden betiklenmiş bağlantı cümlesi (bugün hermetik adaptör yalnız Rust
  testlerinden beslenir); binlerce boşta oturum ölçümü (H4); geliştirme
  için açık bayraklı loopback bind (bugün `systemd-socket-activate`);
  bağlantılar arası uyandırma (yeni ileti → IDLE oturumu; adım 5);
  ham dosya baytlarının kısmi aktarımı (base64 kodlu kısmi getirme A2d'de).

## Freeze ve kanıt kapısı

Core freeze altında yeni compiler özelliği varsayılan olarak reddedilir. Bu
RFC'nin hiçbir katmanı şu zincir kurulmadan uygulanmaz:

1. Ürün `docs/dogfood-projeleri-v1.tsv` içinde etkin ürün olarak, exact
   harici provenance ve depo içi kanıt köküyle kayıtlıdır.
2. Ürün, eksik yetkinliğe çarpan **gerçek Zee kodu** taşır (faz-0 belgesi
   yeterli değildir); sürtünme K-164 kataloğuyla karşılaştırılmıştır.
3. Her katman için: alt RFC → ADR → spec maddesi + `spec-madde-kaniti`
   satırı → olumlu/olumsuz test → depo içi reproducer → K-167 yüzey kaydı →
   güvenlik bulgusu kaydı aynı değişiklikte.
4. Katman A ve C (dinleyici + engellemeyen scheduler) ilk alt RFC'dir; B, D,
   E, F onların üzerinde çalışır. Bu sıra değişirse gerekçe ADR'ye yazılır.
5. Grammar değişikliği ayrıca RFC-0028 ve syntax freeze kurallarına tabidir;
   V1 syntax freeze penceresi içinde açılmaz.

### Faz-0 ürün girdisi ve dogfood kaydı (21 Kasım 2026)

ZeeOS e-posta ürünü K-355 ile `urun-h` olarak etkin dogfood ürünüdür
(`docs/dogfood-projeleri-v1.tsv`, exact harici köken
`91fcfc76a81f08b7551bf0e78c35c972d376e695`); depo içi kanıt kökü
[`dogfood/urun-h`](../dogfood/urun-h/README.md). Bu kayıt yukarıdaki 1.
adımı ve 2. adımın gerçek Zee kodu + K-164 karşılaştırması kısmını
karşılar; 3. adım her alt RFC'nin kendi işidir.

- Ürün Zee kodu: saf SMTP komut durum makinesi (soket yok, stdio deneme
  aracı), 19 `dil dene` testi. Reproducer'lar `dogfood/korpus-v1.tsv`
  içinde exact beklentiyle kayıtlıdır.
- Katmana bağlanan sürtünmeler (K-355/Fnnn, karşılaştırma karar
  günlüğünde):
  - **F009 — `... dinleyicisinde bağlantı geldiğinde` → S036:** bugün yalnız
    HTTP rota kalıbı var; Katman A yüzeyinin doğrudan reproducer'ı
    (`korpus/f009-dinleyici-yok.dil`). **K-381 ile çözüldü:** dosya artık
    temiz derlenen en küçük karşılayıcıdır.
  - **F004 — metin kaçışında `\r` yok** (S040): çıplak CR denetimi Zee'de
    yazılamıyor; Katman A'nın "çıplak LF/CR reddi çalışma zamanı
    değişmezidir" kararını destekler. Kaçış eksikliği K-164/F011 sınıfının
    üçüncü ürün tekrarıdır (K-356 tasarım maddesi).
  - **F001 — ASCII harf katlama yok:** yerleşik büyük/küçük harf dönüşümü
    bilinçli olarak Türkçe yerel ayarlıdır (`mail` → `MAİL`); SMTP/IMAP
    fiilleri ve DNS adları ASCII harf duyarsızdır (RFC 5321 §2.4). Hata
    değil eksik yüzeydir; Katman D'ye ASCII katlama adayı. Ürün çözümü
    26 değiştirmeyle elle katlamadır (`korpus/f001-*.dil`).
  - **F010 — istemsiz satır okuma yok:** `"" diye sor` her okumada boş
    istem satırı basar; ağ yüzeyinde Katman A satır okuyucusu karşılar
    (K-381: `bağlantıdan satır okumayı dene`).
- Diğer tekrarlar (sıra erişimi K-180, listede `içermeli` K-357, boş
  koleksiyon türü K-358, örtük çoğul döngü kaynağı K-359) ağ yetkinliğinden
  bağımsızdır ve bu RFC'nin kapsamında değildir.

V1 kapı listesine (`docs/v1-surum-kapilari.md`) bu RFC eklenmez: ağ sunucusu
yetkinliği V1 sözü değildir.

## Dört soru süzgeci

1. **Türkçe doğal mı?** — Rota modelinin "`... adresine istek geldiğinde`"
   biçimi bağlantı için "`... dinleyicisinde bağlantı geldiğinde`" olarak
   uzanabilir; okuma/yazma mevcut `dene` ailesiyle. Kesin yüzey alt RFC'de
   kullanılabilirlik kartıyla seçilir.
2. **Deterministik mi?** — Scheduler tek iş parçacıklı kalır; ağ olay sırası
   dış girdidir ve ize yazılır; aynı izle aynı çıktı. Güvenlik rastgeleliği
   bilinçli istisnadır ve izde maskelidir.
3. **Öğrenilebilir mi?** — Protokol kodu satır oku / satır yaz / blok al /
   TLS'e yükselt dört kavramıyla yazılır; olay döngüsü, tampon ve TLS
   kullanıcıya görünmez.
4. **Profesyonel ölçekte savunulabilir mi?** — Kripto ve TLS doğrulanmış
   crate'lerde, saldırı sınıfları (smuggling, STARTTLS enjeksiyonu, SSRF,
   kaynak tüketimi) çalışma zamanı değişmezi olarak, kabul ölçütleri gerçek
   istemci ve hata enjeksiyonuyla.

## Alternatifler

- **Ayrı Rust/Go servis motoru, Zee yalnız politika:** ürün sahibi 21 Kasım
  2026'da reddetti; ayrıca motor Zee'de olmadığı için dilin kendi ağ
  yetkinliğini kanıtlamaz.
- **Host'ta SMTP/IMAP motoru intrinsic'i:** protokol kararını host'a taşır,
  aynı ret gerekçesi; ADR-011 alan adaptörü sınırını da şişirir.
- **Genel FFI (RFC-0012):** tehlikeli sınırı açar, Faz 4/5 işidir; ağ
  sunucusunun ihtiyacı dar intrinsic setiyle karşılanabilir.
- **Görev başına OS iş parçacığı:** binlerce oturumda bellek ve bağlam
  geçişi maliyeti; `zee-esz-1` data race yokluğu sözünü bozar.
- **async/await yüzeyi:** büyük grammar değişikliği; mevcut `bekle`
  scheduler noktası aynı işi örtük yapar.
- **Uygulama içi bind (CAP_NET_BIND_SERVICE):** saldırı yüzeyini büyütür ve
  kesintisiz yeniden başlatmayı vermez; FD devri her ikisini çözer.
- **Önce `Baytlar` değer türü:** tür sistemi değişikliği, tutamak yolunun
  yetmediği ölçülmeden açılmaz.

## Korpus etkisi

Taslağın kendisi golden, regression ve dil yüzeyi fixture'ını değiştirmez.
Dogfood korpusuna K-355 ile ürün H'nin dört vakası girdi; katman alt
RFC'leri kabul edildiğinde S036/S040 vakaları ret/çözüm çiftine dönüşür.

## Açık sorular

- Engellemeyen IO katmanı: `mio` mu, `libc` üzerinden dar epoll/kqueue mu;
  Windows (IOCP) desteği mi yoksa Windows'ta yetkinliğin reddi mi?
- PostgreSQL çağrıları scheduler'ı bloklamadan nasıl yürür (ayrı iş
  parçacığı + scheduler noktasında teslim)?
- Bağlantı görevinde hata yalıtımı kipinin adı ve spec/14 ile ilişkisi.
- DKIM gövde kanonikleştirmesi Zee'de mi host dönüşümünde mi (yorumlayıcı
  ölçümü)?
- Tek kripto sağlayıcısı: `ring` mi `aws-lc-rs` mi; mevcut `native-tls`
  istemci yollarının geleceği.
- Posta deposu proje kökü içinde mi kalır (ZeeOS önerisi), yoksa ayrı
  bağlama + EXDEV kanıtı mı gerekir?
- ~~Dinleyici bildiriminde çift yığın: aile sonekli iki ayrı ad mı
  (`smtp_v4`, `smtp_v6`), yoksa tek mantıksal ad + ailelere göre
  `LISTEN_FDNAMES` eşlemesi mi (alt RFC A)?~~ **Kapandı (K-376/ADR-162):**
  aile sonekli iki ayrı ad.
- Sertifika yenilemede süreç içi yeniden yükleme.
- Uzun oturumlu sunucularda yorumlayıcı başarımı Stage 0'da yeterli mi;
  değilse self-hosting/backend aşamalarıyla (ADR-008) ilişkisi.

## İlgili kayıt

K-354 (backlog ve karar günlüğü). Çatı belge sorunu, katmanları, FD devri
sözleşmesini ve kabul ölçütlerini önden bağlar. Uygulanan tek dilim Aşama
A1'dir (K-376).
