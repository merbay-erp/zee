# RFC-0039 — Zee ağ sunucusu yetkinliği (çatı RFC)

- **Durum:** taslak — 21 Kasım 2026. Core freeze (K-160A/ADR-059) altında
  uygulanmaz; her katman ayrı alt RFC ve dogfood kanıt kapısıyla açılır.
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
| systemd soket devri | Var ama dar: yalnız çok işçi grubu yöneticisi, `LISTEN_FDS=1`, fd 3, `127.0.0.1:worker-port`; bind'e geri dönüş yok | `compiler/src/cli/web_isci_grubu.rs` (`systemd_soketini_al`), K-343, ADR-161 eki |
| IPv6, ayrıcalıklı kapı, birden çok dinleyici | Yok | aynı |
| Bağlantı/akış değeri | Yok. Tek ağ sunucusu yüzeyi HTTP rota modelidir; Zee bağlantıya ya da bayt akışına dokunamaz | spec/11 |
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

### E — DNS

- MX, TXT, A, AAAA sorguları (PTR sonraki adım); sonuç `Sonuç` biçiminde ve
  hata **geçici** (zaman aşımı, SERVFAIL, ağ hatası) ile **kalıcı**
  (NXDOMAIN, NODATA) olarak sınıflandırılmış; SMTP 4xx/5xx kararı buna
  bağlıdır.
- Çözücü: `hickory-resolver` ya da sistem çözücüsüne (`res_query`) dar bağ;
  seçim alt RFC'de. Varsayılan hedef yerel çözücüdür (ZeeOS tarafı yerel
  doğrulayıcı çözücü sağlayabilir); DNSSEC doğrulaması bu RFC'de
  vaat edilmez.
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
transaction'ı. Kabul akışı bunlarla kurulabilir:

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
- Kalıcı dosya proje kökü sınırındadır; posta deposu bildirilmiş bir veri
  köküne (systemd `StateDirectory` karşılığı) yazmalıdır. Proje kökü dışı
  veri kökü bildirimi yeni yetkinlik kararıdır.
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

Bu sözleşme systemd `sd_listen_fds` uyumludur; ZeeOS'un servis yöneticisi
systemd ise ek iş gerekmez, değilse aynı ortamı kurar. ZeeOS tarafındaki
karar kayıtları ZeeOS deposunda tutulur.

**Yönetici (ZeeOS/systemd) sağlar:**

- Her dinleyici için `SOCK_STREAM`, `listen` durumunda, bildirilen aile
  (`AF_INET` veya `AF_INET6`), adres ve kapıya bağlı bir soket; IPv6
  soketlerinde `IPV6_V6ONLY` açık (çift yığın tek soketle değil, iki
  bildirimle).
- Dosya tanımlayıcıları 3'ten başlayıp ardışık; `LISTEN_PID` = hizmet
  sürecinin PID'i; `LISTEN_FDS` = soket sayısı; `LISTEN_FDNAMES` = her
  soketin bildirimdeki adı (`:` ayrılmış, sıra fd sırasıyla aynı). Ad
  **zorunludur**; adsız ya da yinelenen ad reddedilir.
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
- `LISTEN_*` ortam kayıtlarını doğrulamadan hemen sonra siler (alt süreçlere
  sızmaz); soketlere `FD_CLOEXEC` ve engellemeyen kipi kendisi koyar.
- Devir yoksa veya geçersizse **bind'e geri dönmez**.
- Bütün dinleyiciler doğrulanıp açılış işi bittikten sonra bir kez
  `READY=1`; SIGTERM'de `STOPPING=1`, yeni kabulü durdurur ve Katman C
  boşaltmasını durdurma süresinin altında bir boşaltma bütçesiyle yapar.
- Çok işçi süreç profilinde (RFC-0038) devralınan bütün dinleyicileri
  işçilere aynı adlarla aktarır.

Bu sözleşme bugünkü K-343 tek-soket devrinin (`LISTEN_FDS=1`, fd 3,
`127.0.0.1`) genellemesidir; K-343 davranışı değişmeden kalır.

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

Yok. Taslak; golden, regression, dogfood korpusu ve dil yüzeyi fixture'ı
değişmez.

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
- Proje kökü dışı veri kökü bildirimi ve izin modeli.
- Sertifika yenilemede süreç içi yeniden yükleme.
- Uzun oturumlu sunucularda yorumlayıcı başarımı Stage 0'da yeterli mi;
  değilse self-hosting/backend aşamalarıyla (ADR-008) ilişkisi.

## İlgili kayıt

K-354 (backlog ve karar günlüğü). Uygulama yok; bu belge yalnız sorunu,
katmanları, FD devri sözleşmesini ve kabul ölçütlerini önden bağlar.
