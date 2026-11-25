# RFC-0044 — DNS, posta kimlik doğrulama ve teslim güvenliği

- **Durum:** taslak — 24 Kasım 2026. Aşama **B1a kabul edildi ve
  uygulandı** (K-390, ADR-169, spec/31 §1); **B1b (PSL) ve B2b (DKIM
  doğrulama ilkeli) kabul edildi ve uygulandı** (K-390 ikinci dilimi,
  ADR-175, spec/31 §2–§3); **B4a uygulandı** (K-397 Dilim 1, ADR-173);
  B2a/B2c/B2d/B2e, B3 (K-397 Dilim 2, ADR-174) ve B4'ün kalanı taslak. Core
  freeze (K-160A/ADR-059) altında her aşama ayrı `dogfood-change` K-işi,
  ADR'si ve depo içi reproducer'ıyla açılır.
- **Uygulama:** K-397 Dilim 1 (B4a giden TCP + istemci STARTTLS, ADR-173)
  ve Dilim 2 (B3 DKIM imza ilkeli, ADR-174) — bkz. "Uygulama kararı — K-397".
- **Tarih:** 24 Kasım 2026
- **İş:** K-383 (taslak), K-390 (B1a uygulandı), K-397 (B4a/B3 uygulaması)
- **İlgili belgeler:** [spec/08](../spec/08-kalici-dosya.md),
  [spec/09](../spec/09-son-tarih-ve-iptal.md),
  [spec/12](../spec/12-web-guvenlik-profili.md) (§9 ağ dinleyicisi),
  [spec/21](../spec/21-deterministik-io-izi.md),
  [spec/22](../spec/22-deterministik-io-profili.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [spec/24](../spec/24-kaynak-guvenlik-profili.md),
  [ADR-011](../adr/011-intrinsic-yetkinlik-siniri.md),
  [ADR-038](../adr/038-rust-tedarik-zinciri-kapisi.md),
  [ADR-165](../adr/165-ag-dinleyici-baglanti-yuzeyi.md),
  [RFC-0024](0024-merkezi-yetkinlik-ve-outbound-guvenligi.md),
  [RFC-0034](0034-ekli-eposta.md),
  [RFC-0039](0039-ag-sunucusu-yetkinligi.md) (katman D, E, G),
  [RFC-0041](0041-ikili-yanit-ve-ek-akisi.md),
  [RFC-0042](0042-postgresql-sorgu-son-tarihi.md),
  [RFC-0043](0043-bayt-kodlama-ve-ozet-yuzeyi.md)
- **Freeze sınıfı adayı:** her aşama için ayrı `dogfood-change`
  (`urun-h`; ZM-F025 ve ZeeMail motor güvenlik kapsamı). Gelen posta
  doğrulaması (B2) güvenlik önceliklidir.

## Özet

ZeeMail (`urun-h`) bugün Zee ile SMTP/IMAP sunucusu olarak çalışıyor
(RFC-0039 A1/A2: adlı dinleyici, satır/blok akışı, sunucu TLS'i ve
STARTTLS). İnternete açık bir posta sunucusu olmak için eksik kalan
yüzeyler DNS'e ve posta kimlik doğrulamasına bağlıdır: MX/TXT/TLSA sorgusu,
DNSSEC doğrulamalı yanıt, kurumsal alan (Public Suffix List), gelen postada
SPF/DKIM/DMARC/ARC değerlendirmesi ve sahte `Authentication-Results`
temizliği, giden postada DKIM imzası ve anahtar döndürme, uzak MX'e
teslimde MTA-STS/DANE ile indirgemeye dayanıklı zorunlu TLS ve TLS-RPT.
Bu RFC dört aşama önerir: **B1** DNS çözücü + DNSSEC + PSL, **B2** gelen
doğrulama, **B3** giden DKIM imzası, **B4** MTA-STS/DANE/TLS-RPT. İlke
RFC-0039'dan gelir: **politika Zee'dedir; kriptografi, DNS tel biçimi,
kanonikleştirme ve kaynak sınırı gerektiren değerlendiriciler dar host
intrinsic'leridir.** Gizli anahtar hiçbir zaman Zee değeri olmaz.

## Motivasyon

### Ürün ve güvenlik önceliği

ZeeMail motorunun 23 Kasım 2026 güvenlik kapsamı (merkez kaydı) şunları
zorunlu sayar: gelen postada SPF + DKIM doğrulama + DMARC hizalama ve
politika uygulama (reject/quarantine), ARC değerlendirmesi, dışarıdan gelen
`Authentication-Results` başlığının silinmesi; giden postada DKIM
(RSA-2048/Ed25519) + anahtar döndürme; DNSSEC doğrulamalı çözücü; giden
teslimde MTA-STS (RFC 8461), DANE/TLSA (RFC 7672), TLS-RPT (RFC 8460).
Ürünün ZM-F025 maddesi DMARC kurumsal alan hesabı için sürümlü bir Public
Suffix List veri dosyası ister.

Bunlar olmadan:

- gelen tarafta ZeeMail, görünen `From:` alanı sahte iletileri ayırt
  edemez; kullanıcıya kimlik avı iletisini meşru gibi teslim eder ve
  dışarıdan eklenmiş `Authentication-Results: …; dkim=pass` başlığı
  istemcide (MUA) güven göstergesi olarak görünebilir;
- giden tarafta büyük posta sağlayıcıları (Gmail ve Yahoo'nun Şubat
  2024'ten beri uyguladığı gönderen gereksinimleri: tüm gönderenler için
  SPF ya da DKIM, toplu gönderenler için SPF + DKIM + DMARC) ZeeMail'in
  iletilerini reddeder ya da istenmeyen klasörüne koyar;
- teslim yolunda etkin bir ağ saldırganı STARTTLS ilanını silerek
  (indirgeme) ya da MX yanıtını sahteleyerek iletiyi düz metin okuyabilir
  veya başka sunucuya yönlendirebilir.

### Bugünkü durum (bd8d44f üzerinde doğrulandı)

| Gerekli | Bugün | Kaynak |
|---|---|---|
| DNS sorgusu | **Kısmi:** yalnız sistem çözücüsüyle A/AAAA (`to_socket_addrs`; HTTP'de ureq çözücüsü). MX, TXT, TLSA, PTR yok; geçici/kalıcı hata ayrımı yok; yanıtın DNSSEC durumu görünmez | `compiler/src/eposta.rs:341`, `compiler/src/ag_istemcisi.rs`; RFC-0039 tablo "DNS" |
| DNSSEC | **Yok.** Ne süreç içi doğrulama ne doğrulayıcı çözücünün `AD` bitine erişim | — |
| DNS crate'i | **Yok** (hickory, domain vb. kilit dosyasında yok) | `compiler/Cargo.lock` |
| Public Suffix List | **Yok** (veri dosyası, ayrıştırıcı ya da crate) | — |
| SPF / DKIM doğrulama / DMARC / ARC | **Yok** | — |
| `Authentication-Results` | **Yok** (yazma da silme de) | — |
| DKIM imzası | **Yok.** SMTP istemcisi bildirilmiş bir aktarıcıya (`smtps://`/`smtp://` origin) kimlik bilgisiyle teslim eder; imza yok | `compiler/src/eposta.rs` (K-210/K-232), spec/23 |
| Uzak MX'e teslim | **Yok.** Giden yol yalnız bildirilmiş origin'e (spec/23); önceden bilinmeyen kamu MX adreslerine 25 kapısı bağlantısı yok; RFC-0039 katman E `smtp-teslim` yetkinliğini taslak olarak anar | spec/23 §Origin; RFC-0039 E |
| MTA-STS / DANE / TLS-RPT | **Yok.** SMTP istemci TLS'i `native-tls` (DANE için özel sertifika doğrulayıcı takılamaz); HTTPS istemcisi `ureq` + `rustls` + `webpki-roots` (MTA-STS ilke indirmesi için uygun) | `compiler/Cargo.toml`, `compiler/src/eposta.rs` |
| İmza kriptosu | `ring 0.17.14` (rustls üzerinden; RSA PKCS#1 v1.5 SHA-256 imza/doğrulama, Ed25519), `ed25519-dalek 3.0.0` (paket imzası), `sha2 0.11.0`. RustCrypto `rsa` yok | `compiler/Cargo.lock` |
| Sıkıştırma (TLS-RPT gzip) | `flate2 1.1.9` geçişli (image → png) | `compiler/Cargo.lock` |
| Gizli anahtar kaynağı | **Var:** systemd credential / ortam okuma sözleşmesi (K-339), boyut sınırlı, sembolik bağsız, öncelik belirsizliği reddi | spec/23 §systemd credential |
| Gelen ileti tutamağı | **Var:** A2c ile `DATA` gövdesi `0600` dosyaya, SHA-256 aynı akışta (`yol`, `sha256`, `bayt`) | spec/12 §9 A2c |
| Bayt/özet yapı taşları | **Yok** → RFC-0043 | RFC-0043 |

## Amaç

1. B1: Zee kodunun MX, A, AAAA, TXT ve TLSA kayıtlarını sınırlı, son
   tarihli, geçici/kalıcı hata ayrımlı ve DNSSEC durumunu açıkça taşıyan
   biçimde sorgulayabilmesi; sürümlü PSL ile kurumsal alan hesabı.
2. B2: Gelen iletinin SPF, DKIM, DMARC ve ARC sonucunu tek ve tutarlı bir
   başlık ayrıştırıcısıyla üretmek; politikayı Zee'de uygulamak; sahte
   `Authentication-Results` başlığını teslimden önce silmek.
3. B3: Giden iletiyi bildirilmiş anahtarlarla DKIM imzalamak; anahtar
   Zee'ye, IO izine ya da günlüğe hiç girmeden; birden çok seçiciyle
   kesintisiz anahtar döndürme.
4. B4: Uzak MX'e teslimde DANE ve MTA-STS ile indirgemeye dayanıklı zorunlu
   TLS; başarısızlıkların TLS-RPT ile raporlanması.

## Kapsam dışı

- DNS sunucusu (yetkili ya da özyinelemeli) olmak; kendi bölge kayıtlarını
  yayımlamak (DKIM açık anahtarı, MTA-STS TXT'si, TLSA, SPF kaydı operatör
  ve DNS sağlayıcısı işidir; Zee bunları **denetleyen** bir komut önerebilir).
- Süreç içinde DNSSEC doğrulayıcısı olmak (ilk sürüm; bkz. B1 karar).
- DMARC toplu (`rua`) ve hata (`ruf`) raporu göndermek/almak — mahremiyet ve
  hacim nedeniyle ayrı aşama (B5 adayı). TLS-RPT raporu **almak** da dışarıda.
- ARC mühürleme (aracı/liste sunucusu olarak `ARC-Seal` eklemek); yalnız
  değerlendirme.
- BIMI, VBR, ADSP, Sender ID, DomainKeys (eski), `rsa-sha1`.
- İstenmeyen posta puanlaması, itibar listeleri (DNSBL/RBL) — sonraki ayrı
  RFC; B1 sorgu yüzeyi onların da temeli olur ama bu RFC söz vermez.
- SMTPUTF8 / IDN (U-label) alan adları: ilk sürüm yalnız ASCII LDH
  (A-label) alanları kabul eder; U-label reddedilir (açık soru).
- DoH/DoT ile uzak çözücüye gitmek (loopback doğrulayıcı çözücü tercih
  edilir; bkz. B1).

## Tasarım ilkeleri

1. **Politika Zee'de, değerlendirici host'ta.** RFC-0039 ilke 1 "SPF/DMARC
   değerlendirmesi Zee kodudur" der. Bu RFC onu şöyle inceltir: **DMARC
   politikası** (hizalama sonucu, `p`/`sp`/`pct` uygulama, yerel istisnalar,
   reddetme/karantina kararı, `Authentication-Results` içeriği) Zee
   kodudur. **SPF `check_host`, DKIM doğrulama/imza, ARC zinciri doğrulama
   ve kanonikleştirme** host intrinsic'idir; çünkü (a) DNS sorgu bütçesi
   (RFC 7208 §4.6.4 on mekanizma, iki boş yanıt) ve kanonikleştirme bayt
   düzeyinde güvenlik sınırıdır ve merkezî uygulanmalıdır, (b) yorumlanan
   Zee'de satır satır kanonikleştirme pratik değildir (ADR-003), (c) aynı
   ayrıştırıcının DKIM, DMARC ve MIME arasında paylaşılması ayrıştırıcı
   farkı saldırılarını kapatır. Bu sapma B2 ADR'sinde RFC-0039'a ek olarak
   kaydedilir.
2. **Kriptografi yazılmaz** (RFC-0039 ilke 2): RSA/Ed25519/SHA-256 `ring`
   (ağaçta), DNS tel biçimi doğrulanmış crate.
3. **Fail-closed, ama posta semantiğiyle.** DNS'in geçici hatası hiçbir
   zaman "kayıt yok" ya da `pass` sayılmaz; SMTP tarafında geçici sonuç
   `4xx` (yeniden dene) olur. Doğrulanamayan DNSSEC (`bogus`) geçici hatadır,
   "imzasız" değildir.
4. **Gizli anahtar Zee değeri değildir.** Özel anahtar yalnız bildirilmiş
   credential adıyla tutamak olarak kullanılır; içerik IO izine, hata
   mesajına, günlüğe ve çekirdek dökümüne (mümkün olduğunca) girmez.
5. **Her dış etki bütçelidir.** Sorgu sayısı, yanıt boyutu, kayıt sayısı,
   imza sayısı, anahtar boyutu, zaman — hepsi önceden sayıdır; saldırganın
   denetlediği alan adı sunucuyu sınırsız işe sokamaz.
6. **Deterministik kanıt.** Her DNS sorgusu ve yanıtı IO izine girer;
   `dil dene` için sahte çözücü (statik bölge dosyası) ile golden test
   yazılabilir olmadan aşama kabul edilmez. Saat (`t=`, `x=`, MTA-STS
   `max_age`) IO izindeki saat olayından gelir.
7. **Önbellek yok (B1–B3).** DNS önbelleği yerel doğrulayıcı çözücünündür;
   süreç içi önbellek K-173/ADR-069 soak kapısını ve zehirleme yüzeyini
   getirir. B4'ün MTA-STS ilke önbelleği zorunludur (RFC 8461 §5) ve kalıcı
   dosyadır; soak ve spec/08 kuralları B4'te uygulanır.

## B1 — DNS çözücü, DNSSEC ve kurumsal alan

> **B1a kabul (K-390, 24 Kasım 2026, [ADR-169](../adr/169-dns-stub-cozucu-ve-tel-bicimi-siniri.md),
> [spec/31 §1](../spec/31-dns-ve-posta-kimlik-dogrulama.md)).** Uygulamada
> taslaktan üç fark vardır: (1) **yeni crate yok** — tel biçimi kendi sınırlı
> `dns_teli` ayrıştırıcımızdır, `hickory-proto` yalnız dev/fuzz kâhinidir
> (aşağıdaki "Bağımlılık seçimi" satırı bu kararla değişti); (2) Zee yüzeyi
> ürün reproducer'ının biçimidir: `AD için MX|TXT|TLSA|adres kayıtlarını
> sorgulamayı dene` → kayıt başına satırlı `Sonuç<Metin sözlük listesi>`
> (yeni tür yok; `durum`/`ttl`/`ad` sonuç sözlüğü yerine satır alanlarıdır,
> NXDOMAIN/NODATA boş liste, ayrımları host iç API'sinde); (3) hata kodu
> C033, `tür` listesine `bozuk_yanıt` eklendi. B1b (PSL, `kurumsal alanı`)
> B2 DMARC ile birlikte gelir.

> **B1b + B2b kabul (K-390, 25 Kasım 2026, [ADR-175](../adr/175-kamu-son-ekleri-ve-dkim-dogrulama-ilkeli.md),
> [spec/31 §2–§3](../spec/31-dns-ve-posta-kimlik-dogrulama.md)).** Merkez kararı
> (b) dogfood-minimal; taslaktan farklar:
> (1) PSL bildirimi tek sözlük değil iki proje alanıdır
> (`kamu_son_ekleri`, `kamu_son_ekleri_özeti`; yalnız `dns` ile, P015), sürüm
> alanı yok — kimlik exact SHA-256'dır; ifade `alanın kurumsal alanını almayı
> dene` Sonuç'u yerine `<ad> alanının kurumsal alanı` Metin'idir (geçersiz ad
> boş Metin; saldırgan From'u yürütmeyi durduramaz).
> (2) B2b tam DKIM değerlendiricisi değil **doğrulama ilkelidir**:
> kanonikleştirme, `h=` seçimi, `bh=` karşılaştırması, `x=` ve politika
> Zee'dedir; host yalnız `p=` ayrıştırma, bit sınırı ve imza doğrulamasıdır
> (`<b> dkim imzasını <veri> verisi için <p> anahtarıyla <a> olarak doğrulamayı
> dene`, C034). B2a'dan (başlık ayrıştırıcısı) önce alındı; "Ayrıştırıcı
> tutarlılığı" riski B2a gelene dek Zee kodundadır (GB-168).
> (3) Zee'de `\r` olmadığından veri LF'si imza girdisinde CRLF'dir; aynı
> kuralla `<x> gövdesinin dkim gövde özeti` `bh=` biçiminde gövde özetini
> verir (DKIM'e özgü, genel özet API'si değil).
> (4) Kriptografi rustls'in kilitli `ring`'idir, yeni crate yok.

### Çözücü modeli — karar

| Seçenek | Artı | Eksi | Karar |
|---|---|---|---|
| a. Sistem çözücüsü (`res_query`/`libresolv`, Windows `DnsQuery`) | Yeni crate yok | Platform farkı (glibc/musl/macOS/Windows), son tarih ve yanıt boyutu denetimi zayıf, `AD` bitine erişim platforma göre (glibc `trust-ad`), iş parçacığı güvenliği; deterministik iz zor | Ret |
| b. Tam özyinelemeli/önbellekli çözücü crate'i (`hickory-resolver`) | Hazır özellik seti | Büyük ağaç, iç önbellek (K-173), kendi zamanlayıcı/çalışma zamanı; politikası bizim değil | Ret (ilk sürüm) |
| c. **Tel biçimi crate'i + dar stub istemci → bildirilmiş loopback doğrulayıcı çözücü** | Küçük yüzey; zaman/boyut/kayıt bütçesi bizde; `AD` biti yalnız güvenilir yerel kanaldan; IO izi doğal | Doğrulayıcı çözücü ortamdan (ZeeOS/operatör) sağlanmalı | **Seçilen** |
| d. Süreç içi DNSSEC doğrulama (ör. `hickory-proto` + `dnssec-ring`) | Ortamdan bağımsız | Güven çapası yönetimi (RFC 5011 devri), NSEC/NSEC3 doğrulama karmaşıklığı ve CPU DoS sınıfı (KeyTrap, CVE-2023-50387), algoritma politikası | Ertelenir (B1 sonrası ayrı karar) |
| e. Kendi DNS tel ayrıştırıcımız | Bağımlılık yok | İlke 2'ye aykırı; sıkıştırma işaretçisi döngüleri, sınır hataları tarihsel CVE kaynağı | Ret |

Seçilen model, Postfix'in `smtp_dns_support_level = dnssec` ve Exim'in
DANE modeliyle aynı güven varsayımıdır: **`AD` biti yalnız aynı makinedeki
(loopback) doğrulayıcı çözücüden geldiğinde anlamlıdır** (RFC 4035 §4.9.3,
RFC 6698 §4.1 ilkeleri). Uzak çözücünün `AD` biti güvenilmez; bildirim
loopback dışı adresi reddeder.

### Bildirim

```text
# proje.dil (fikir taslağı)
yetkinlikler: ["dns", ...]
dns_çözücü: "127.0.0.1:53"            # yalnız loopback; IPv6 ::1 da olur
dns_dnssec: "zorunlu"                 # "zorunlu" | "bilgi" (varsayılan yok)
```

- `dns_dnssec: "zorunlu"`: açılışta çözücünün doğruladığı sınanır (ör.
  kök `.` `SOA`/`DNSKEY` yanıtında `AD=1`); sınanamazsa süreç başlamaz
  (fail-closed). DANE (B4) yalnız bu kipte etkinleşir.
- `"bilgi"`: `AD` biti sonuçta taşınır ama hiçbir güvenlik kararı DNSSEC'e
  bağlanmaz; DANE kapalıdır.
- **Ortam bağımlılığı:** RFC-0039 katman E'deki gibi ZeeOS'un doğrulayıcı
  yerel çözücü sağlaması gerekir (systemd-resolved `DNSSEC=yes` ya da
  unbound); bugün r5 imajında yoktur (ZeeOS P0-Z3 açık). Hangi çözücünün
  önerileceği ZeeOS ile açık sorudur; öneri: DNSSEC olgunluğu nedeniyle
  unbound.

### Zee yüzeyi (fikir taslağı)

```text
kayıtlar alanın mx kayıtlarını sorgulamayı dene olsun     # Sonuç<Sözlük>
metinler alanın txt kayıtlarını sorgulamayı dene olsun    # Sonuç<Sözlük>
adresler alanın adres kayıtlarını sorgulamayı dene olsun  # A + AAAA
tlsa "_25._tcp.mx.ornek.test" adının tlsa kayıtlarını sorgulamayı dene olsun
kurum alanın kurumsal alanını almayı dene olsun           # Sonuç<Metin>, PSL
```

Başarılı sonuç `Sözlük`tür:

| Alan | Tür | Anlam |
|---|---|---|
| `durum` | Metin | `var` · `kayıt_yok` (NODATA) · `alan_yok` (NXDOMAIN) |
| `kayıtlar` | Liste | MX: `{öncelik, hedef}` öncelik + ad sırasıyla; TXT: parçaları birleştirilmiş `Metin` (RFC 7208 §3.3, RFC 6376 §3.6.2.2); A/AAAA: kanonik IP `Metin`; TLSA: `{kullanım, seçici, eşleme, veri_onaltılık}` |
| `doğrulanmış` | Mantıksal | Çözücü `AD=1` ve `dns_dnssec: "zorunlu"` |
| `ttl` | TamSayı | En küçük TTL (bilgi) |
| `ad` | Metin | CNAME zinciri sonrası kanonik ad |

`alan_yok` ve `kayıt_yok` **başarıdır** (kalıcı, yetkili cevap); SPF/DMARC
bunları "kayıt yok" anlamında kullanır. **Başarısız `Sonuç` yalnız geçici
hatadır:** zaman aşımı, `SERVFAIL` (DNSSEC `bogus` dahil), `REFUSED`, ağ
hatası, kesik yanıt TCP'de de alınamadı, bütçe aşımı. Hata verisi
`tür: zaman_aşımı | sunucu_hatası | ret | ağ | boyut_aşımı | bütçe | geçersiz_ad`.
SMTP tarafında geçici hata `451 4.4.3` sınıfına eşlenir; eşleme Zee'dedir.

### Sınırlar

| Sınır | Önerilen | Gerekçe |
|---|---|---|
| Sorgu son tarihi | 5 sn/sorgu, çağrı son tarihi spec/09 ile sınırlı | RFC 7208 §4.6.4 toplam ≥ 20 sn önerir; toplam bütçe değerlendiricide |
| UDP EDNS tamponu | 1232 bayt; `TC=1` → TCP | DNS Flag Day 2020 parçalanma önerisi |
| TCP yanıt | ≤ 64 KiB (tel biçimi üst sınırı) | — |
| Kayıt sayısı | ≤ 64/yanıt (MX ≤ 10 kullanılır, RFC 7208 §4.6.4 MX sınırı) | Amplifikasyon |
| TXT toplam | ≤ 4 KiB birleştirilmiş/kayıt | DKIM 4096 bit anahtar sığar |
| CNAME zinciri | ≤ 8 | Döngü |
| Ad | ≤ 253 oktet, etiket ≤ 63, ASCII LDH + `_` (hizmet etiketleri) | RFC 1035; U-label ret |

### Tehdit modeli (B1)

| Tehdit | Önlem | Kalan risk |
|---|---|---|
| **DNS sahteciliği / önbellek zehirleme** (Kaminsky 2008, SAD DNS 2020 kaynak kapısı yan kanalı) | Yalnız loopback çözücü (yol dışı saldırgan aynı makinede değil); her sorguda `getrandom` ile 16 bit kimlik + rastgele kaynak kapısı; soru bölümü, sınıf, tür ve kaynak adresi exact eşleşmeli, eşleşmeyen paket atılır (yanıt bekleme sürer); 0x20 kodlama gereksiz (loopback). Üst yoldaki zehirleme çözücünün sorumluluğu: DNSSEC `zorunlu` kipinde `doğrulanmış` alanı bunu görünür kılar | İmzasız bölgeler için zehirleme yerel çözücünün dayanıklılığına bağlı; DANE yalnız imzalı bölgede |
| **İndirgeme** (saldırgan DNSSEC'i kaldırıp yanıtı imzasız gösterir ya da `SERVFAIL` zorlayıp "kayıt yok"a düşürür) | `bogus` → `SERVFAIL` → **geçici hata**, asla `alan_yok`; doğrulayıcı imzalı bölgenin imzasının sökülmesini zaten `bogus` sayar; `doğrulanmış=yanlış` yanıt DANE'yi etkinleştirmez ama MTA-STS önbelleğini de silmez (B4) | İmzasız bölgede gerçek "kayıt yok" ile sahte "kayıt yok" ayırt edilemez → DMARC/MTA-STS pratiği |
| **Yansıtma/amplifikasyon** (bizim sunucumuzun saldırganın alanı için çok sorgu atması: SPF include bombası, çok DKIM imzası, MX zinciri) | İleti/bağlantı başına sorgu bütçesi (B2), değerlendirici başına RFC 7208 sınırları, kayıt sayısı sınırı; çözücüye eşzamanlı sorgu sınırı (süreç başına) | Çözücü önbelleği ısınmamış alanlar için gecikme |
| **Zaman aşımı / son tarih** (yavaş yetkili sunucu SMTP oturumunu bekletir; slowloris benzeri) | Sorgu ve değerlendirme son tarihleri spec/09 iptaline bağlı; son tarih aşımı geçici hata → `4xx`; bağlantı bütçesi (A2) ayrıca geçerli | — |
| **Ayrıştırıcı hataları** (sıkıştırma işaretçisi döngüsü, sınır taşması) | Doğrulanmış tel biçimi crate'i; fuzz hedefi `dns_yaniti`; kayıt/ad sınırları ayrıştırmadan önce | Crate CVE'leri → RustSec/cargo-deny kapısı |
| **Ad enjeksiyonu** (Zee'nin kullanıcı verisinden sorgu adı kurması; `\0`, nokta kaçışı, çok uzun ad) | Ad doğrulaması host'ta; geçersiz ad yakalanabilir hata, tel biçimine hiç girmez | — |

### Kurumsal alan — Public Suffix List (ZM-F025)

- **Veri dosyası, derleyiciye gömülü değil.** PSL'i derleyici ikilisine
  gömmek güncellemeyi derleyici sürümüne bağlar ve ürünün DMARC
  davranışını derleyici yükseltmesine gizler. Öneri: proje, `dosya-okuma`
  sınırındaki bir veri dosyasını **exact SHA-256 ile** bildirir:

  ```text
  kamu_son_ekleri: { dosya: "veri/public_suffix_list.dat",
                     sha256: "<64 hex>", sürüm: "2026-11-20" }
  ```

  Açılışta özet tutmazsa süreç başlamaz; dosya `proje.kilit` kapsamına
  girer (yeniden üretilebilirlik).
- Ayrıştırma host'ta (biçim basit: kural, `*.` joker, `!` istisna; ICANN ve
  PRIVATE bölümleri; IDN satırları A-label'a çevrilmeden ilk sürümde
  yok sayılır — açık soru). Fuzz hedefi `psl`.
- Algoritma publicsuffix.org "en uzun eşleşen kural" algoritmasıdır;
  kurumsal alan = kamu son eki + bir etiket (RFC 7489 §3.2).
- **DMARCbis notu:** IETF DMARC çalışma grubunun DMARCbis taslağı
  (draft-ietf-dmarc-dmarcbis) PSL yerine DNS ağaç yürüyüşü (tree walk)
  önermektedir; yayın durumu ve sağlayıcı benimsemesi uygulama günü
  doğrulanır. Öneri: B1'de PSL (RFC 7489 ile uyumlu, yaygın), ağaç
  yürüyüşü B2 DMARC'ta seçenek olarak açık soru.
- PSL güncelliği operatörün işidir; `dil denetle` benzeri bir uyarı
  (dosya sürümü N günden eski) çalışma zamanında saate bağlı belirsizlik
  getirmemek için yalnız CLI/lint düzeyinde önerilir.

### B1 kabul ölçütleri

1. Statik bölge dosyalı sahte çözücüyle `dil dene` golden'ı: MX sıralaması,
   TXT parça birleştirme, NXDOMAIN/NODATA ayrımı, CNAME zinciri, TC→TCP.
2. Olumsuz: eşleşmeyen kimlikli/sorulu paket atılır; 65 kayıtlı yanıt,
   9 CNAME, 254 oktet ad, U-label ad, loopback dışı `dns_çözücü` reddi.
3. `SERVFAIL` hiçbir yolda `alan_yok`a dönüşmez (özellik testi).
4. `dns_dnssec: "zorunlu"` + doğrulamayan çözücü → süreç başlamaz.
5. Gerçek doğrulayıcı çözücüyle (unbound) imzalı/imzasız/bogus test
   alanlarında `doğrulanmış` ve hata sınıfı (ağa bağlı entegrasyon testi,
   CI'da isteğe bağlı).
6. IO izinden replay aynı sonuçları verir; iz, sorgu adı/türü ve yanıtın
   kanonik özetini taşır.
7. PSL: publicsuffix.org resmî test vektörleri (`tests.txt`) geçer; özet
   uyuşmazlığında açılış reddi.

## B2 — Gelen doğrulama: SPF, DKIM, DMARC, ARC ve `Authentication-Results`

### Akış (SMTP alıcısında)

1. `MAIL FROM` sonrası (ya da `DATA` sonunda) **SPF** `check_host(ip, alan,
   gönderen)`; HELO kimliği de değerlendirilir (RFC 7208 §2.3).
2. `DATA` A2c ile dosyaya akar.
3. Host **ileti başlığı ayrıştırıcısı** dosyadan başlık bölümünü bir kez
   ayrıştırır ve sonraki adımların hepsi aynı ayrıştırmayı kullanır.
4. **DKIM** doğrulama: ilk N imza.
5. **ARC** zinciri doğrulama (varsa).
6. **DMARC**: `From:` alanı → `_dmarc` kaydı → hizalama → politika (Zee).
7. Gelen `Authentication-Results` (ve isteğe bağlı diğer güven başlıkları)
   temizlenir, yeni `Authentication-Results` başa eklenir, ileti yerel
   teslime/kuyruğa gider ya da `DATA` sonunda `550 5.7.1`/`451 4.7.x`
   yazılır.

### Zee yüzeyi (fikir taslağı)

```text
spf istemci_adresi, helo ve dönüş_yolu için spf sonucunu almayı dene olsun
# → Sonuç<Sözlük>{sonuç: pass|fail|softfail|neutral|none|temperror|permerror,
#                 alan, mekanizma, sorgu_sayısı}

başlıklar ileti dosyasının başlıklarını çözümlemeyi dene olsun
# → Sonuç<Sözlük>{from_alanı, from_sayısı, başlık_sayısı, sorunlar: Liste}

imzalar ileti dosyasının dkim imzalarını doğrulamayı dene olsun
# → Sonuç<Liste<Sözlük>>{d, s, algoritma, sonuç: pass|fail|neutral|temperror|permerror|policy,
#                         neden, l_var, anahtar_biti}

arc ileti dosyasının arc zincirini doğrulamayı dene olsun
# → Sonuç<Sözlük>{sonuç: none|pass|fail, örnek_sayısı, son_imzacı}

kayıt "_dmarc." + alan alanının dmarc kaydını almayı dene olsun   # B1 TXT + host ayrıştırıcı
# → Sonuç<Sözlük>{p, sp, adkim, aspf, pct, rua_var, kurumsal_alandan}

yeni ileti dosyasından güven başlıklarını temizleyip sonucu ekle …   # tek host işlemi
```

DMARC hizalaması (`alanın kurumsal alanı` ile `relaxed`, exact ile
`strict`), politika seçimi (`p` / alt alan için `sp`), `pct` (rastgelelik
IO izindeki tohumlu olmayan güvenlik rastgeleliğinden değil, ileti
kimliğinin özetinden deterministik — açık soru), yerel istisna listesi
(güvenilir yönlendiriciler, ARC ile) ve SMTP yanıtı **Zee kodudur**.

### Değerlendirici sınırları

| Sınır | Değer | Kaynak |
|---|---|---|
| SPF DNS sorgulu mekanizma | ≤ 10 (`include`, `a`, `mx`, `ptr`, `exists`, `redirect`) | RFC 7208 §4.6.4 |
| SPF boş yanıt | ≤ 2 → `permerror` | RFC 7208 §4.6.4 |
| SPF `mx` başına adres sorgusu | ≤ 10 | RFC 7208 §4.6.4 |
| SPF `ptr` | desteklenir ama önerilmez; ≤ 10 ad | RFC 7208 §5.5 |
| SPF toplam süre | ≤ 20 sn | RFC 7208 §4.6.4 |
| DKIM imza sayısı | ilk 5 doğrulanır, fazlası `policy` (açık soru) | DoS |
| DKIM RSA anahtarı | 1024–4096 bit doğrulanır; < 1024 geçersiz | RFC 8301 §3.2 |
| DKIM algoritma | `rsa-sha256`, `ed25519-sha256`; `rsa-sha1` geçersiz | RFC 8301 §3.1, RFC 8463 |
| Başlık bölümü | ≤ 256 KiB, ≤ 1000 alan | ayrıştırıcı DoS |
| ARC örneği | ≤ 50 (RFC 8617 üst sınırı); pratikte daha az | RFC 8617 §4.2.1 |
| İleti başına DNS bütçesi | SPF kendi RFC 7208 sınırıyla; DKIM + DMARC + ARC ayrıca ≤ 10 sorgu | amplifikasyon; ürün görüşü |

### Ayrıştırıcı tutarlılığı

"Composition Kills" (Chen ve ark., USENIX Security 2020) ve sonraki
çalışmalar, SPF/DKIM/DMARC bileşenlerinin ve MUA'nın aynı iletiyi farklı
ayrıştırmasının (çift `From:`, katlama farkları, encoded-word içinde
adres, boş/NUL baytlı alan, birden çok adresli `From:`) kimlik
doğrulamayı atlattığını gösterdi. Bu nedenle ZORUNLU tasarım:

- tek host başlık ayrıştırıcısı; DKIM `h=` seçimi, DMARC `From:` çıkarımı
  ve MIME (RFC-0039 MIME alt RFC'si) aynı ayrıştırmayı kullanır;
- birden çok `From:` alanı, birden çok adresli `From:`, ayrıştırılamayan
  `From:`, başlıkta NUL/çıplak CR/çıplak LF, 998 oktet üstü satır →
  `sorunlar` listesinde; öneri: DMARC `permerror` ve politika
  uygulanabilen alanlarda **reddet** (RFC 7489 §6.6.1 çoklu `From:`
  uyarısı);
- DKIM `h=` içinde `From:` yoksa imza geçersiz (RFC 6376 §5.4); imzalanan
  başlığın birden çok örneği "aşağıdan yukarı" seçilir (RFC 6376 §5.4.2)
  ve fazla imza (`h=from:from`) önerilir (B3).

### `Authentication-Results` temizliği

- Proje `yetkili_kimlik` (authserv-id, RFC 8601 §2.5) bildirir.
- **ZORUNLU (önerilen):** dışarıdan gelen ve bizim authserv-id'mizi taşıyan
  her `Authentication-Results` alanı teslimden önce silinir (RFC 8601 §5).
  Seçenek: sınır MTA'sı olarak **tüm** gelen `Authentication-Results`
  alanlarını silmek (varsayılan öneri, çünkü MUA'lar authserv-id'yi
  denetlemeyebilir); açık soru.
- `ARC-Authentication-Results` ARC zincirinin parçasıdır; silinmez,
  yalnız değerlendirilir.
- Yeni alan host tarafından **sözdizimi doğrulanarak** yazılır (Zee
  yalnız yapılandırılmış sonuç verir: yöntem, sonuç, özellik); CR/LF ve
  `;` enjeksiyonu yapısal olarak imkânsızdır.
- Temizlik ve ekleme yeni bir dosya tutamağı üretir (kaynak dosya
  değişmez); aynı akışta SHA-256 (A2c ilkesi).

### Tehdit modeli (B2)

| Tehdit | Önlem |
|---|---|
| **Görünen gönderen sahteciliği** (`From:` taklidi) | DMARC hizalaması + politika; hizalanmamış SPF/DKIM `pass` DMARC `pass` değildir |
| **Sahte `Authentication-Results`** (saldırganın kendi eklediği `dkim=pass`) | Teslimden önce silme; yeni alan başta |
| **Ayrıştırıcı farkı** (çift `From:`, katlama, encoded-word) | Tek ayrıştırıcı + `sorunlar` + reddetme önerisi |
| **DKIM `l=` (gövde uzunluğu) saldırısı** — imzalı gövdeye içerik eklenmesi | `l_var` raporlanır; öneri: `l=` ile kısmi kapsanan gövdede imza `pass` sayılmaz ya da DMARC'ta yok sayılır (politika Zee'de; varsayılan katı) |
| **DKIM tekrar (replay)** — meşru imzalı iletinin toplu yeniden gönderimi | Bu RFC'de çözülmez; `x=` süre denetimi ve oran sınırı Zee politikası; not edilir |
| **DNS amplifikasyonu / CPU DoS** (SPF include bombası, çok imza, 16384 bit anahtar) | §Değerlendirici sınırları; ileti başına bütçe; RSA ≤ 4096 |
| **İndirgeme / geçici hata istismarı** (SERVFAIL zorlayarak `none`a düşürme) | `temperror` asla `pass`/`none` değildir; DMARC `temperror` → `451` (Zee) |
| **Zaman aşımı** | Toplam değerlendirme son tarihi `DATA` yanıt bütçesine bağlı; aşım `451` |
| **Başlık enjeksiyonu** (yeni A-R'ye CR/LF) | Host'ta yapısal yazım |
| **Gizlilik** (sonuçların günlüğe gönderen adresiyle düşmesi) | Günlük politikası ürünün; IO izi ileti içeriğini değil özetini ve sonuçları taşır |

### B2 kabul ölçütleri

1. SPF: RFC 7208 test paketi (openspf `rfc7208-tests.yml`) geçer;
   on bir `include` → `permerror`; iki boş yanıt → `permerror`;
   `SERVFAIL` → `temperror`.
2. DKIM: RFC 6376 / RFC 8463 örnek imzaları doğrulanır; `rsa-sha1`,
   1023 bit, `h=` içinde `From:` yok, değiştirilmiş gövde/başlık, `x=`
   geçmiş → beklenen sonuç; 6. imza `policy`.
3. Diferansiyel test: aynı korpus bağımsız bir uygulamayla (ör. OpenDKIM
   / bir Rust posta kimlik doğrulama kitaplığı dev-dependency kâhini,
   K-362 `httparse` emsali) karşılaştırılır.
4. DMARC: `relaxed`/`strict` hizalama, `sp`, kurumsal alan geri düşüşü,
   `pct`, çoklu `From:` reddi; Zee golden'ı sahte çözücüyle.
5. ARC: RFC 8617 örnekleri; bozuk zincir `fail`; güvenilir imzacı listesi
   DMARC `fail`ı yalnız Zee politikasında geçersiz kılar.
6. A-R: dışarıdan gelen, bizim kimliğimizi taşıyan alan teslim edilen
   dosyada yoktur; yeni alan RFC 8601 ABNF'sine uyar (fuzz).
7. Hiçbir yolda `temperror` → `pass`.

## B3 — Giden DKIM imzası ve anahtar döndürme

### Anahtar modeli

```text
# proje.dil (fikir taslağı)
dkim_anahtarları: [
  { alan: "ornek.test", seçici: "z2026a", algoritma: "ed25519-sha256",
    gizli: "DKIM_Z2026A", durum: "etkin" },
  { alan: "ornek.test", seçici: "r2026a", algoritma: "rsa-sha256",
    gizli: "DKIM_R2026A", durum: "etkin" },
  { alan: "ornek.test", seçici: "r2025b", algoritma: "rsa-sha256",
    gizli: "DKIM_R2025B", durum: "emekli" }
]
```

- Özel anahtar PKCS#8 (PEM ya da DER) olarak K-339 credential/ortam
  sözleşmesiyle okunur; yeni ad listesi `http_gizli_değişkenleri` gibi
  kapalıdır. Okunan bayt Zee değeri olmaz; host'ta `zeroize` edilen tampon
  ve `ring` anahtar çifti olarak süreç belleğinde kalır.
- RSA < 2048 bit ve `rsa-sha1` yüklenmez (RFC 8301 §3.2 SHOULD 2048 →
  burada ZORUNLU); Ed25519 RFC 8463.
- Çift imza (Ed25519 + RSA) desteklenir (RFC 8463 alıcı uyumu için önerir).
- `durum`: `etkin` (imzalar), `emekli` (imzalamaz, açılış denetiminde
  DNS'te hâlâ yayımlı olması beklenir), kaldırılmış anahtar listeden çıkar.

### Zee yüzeyi (fikir taslağı)

```text
imzalı ileti dosyasını "ornek.test" alanı adına dkim ile imzalamayı dene olsun
# → Sonuç<Sözlük>{yol, sha256, bayt, imzalar: [{seçici, algoritma}]}
```

- İmza yalnız `From:` alanının alanı (ya da hizalanan üst alan) bildirilmiş
  bir `alan` ile eşleşirse atılır; aksi `Sonuç` hatası (**imza kâhini
  önlemi**: Zee kodu rastgele içeriği başka alan adına imzalatamaz).
- Kanonikleştirme `relaxed/relaxed`; imzalanan başlık kümesi sabit varsayılan
  (From, To, Cc, Subject, Date, Message-ID, MIME-Version, Content-Type,
  Reply-To, In-Reply-To, References, List-* …) ve `From`, `Subject`,
  `Date`, `To` fazla imzalanır (oversigning).
- `t=` IO izindeki saatten; `x=` isteğe bağlı (açık soru).
- Gövde özeti dosya üzerinde akışla (RFC-0043 bellekte özet küçük bloklar
  içindir).

### Anahtar döndürme

Çalışma zamanı döndürmeyi **planlamaz**; birden çok seçiciyi güvenle
taşır ve denetler. Önerilen operatör akışı (runbook'a):

1. Yeni anahtar üret (çalışma zamanı dışında; öneri `openssl genpkey`
   komutları runbook'ta — CLI üreticisi açık soru), credential olarak koy.
2. Açık anahtarı `seçici._domainkey.alan` TXT olarak yayımla; TTL kadar bekle.
3. `proje.dil`'de yeni seçiciyi `etkin`, eskiyi `emekli` yap; yeniden başlat
   (ya da RFC-0038 kesintisiz yeniden başlatma).
4. Eski açık anahtarı en az birkaç gün (yoldaki iletilerin doğrulanması)
   yayımda tut, sonra `p=` boş ile iptal et, credential'ı sil.

Açılış denetimi (öneri, `dns` yetkinliği varsa): her `etkin`/`emekli`
seçicinin DNS TXT'sindeki açık anahtar, özel anahtarın açık parçasıyla
eşleşmeli; eşleşmezse `etkin` için süreç başlamaz (yanlış anahtarla
imzalanmış ileti DMARC'ta düşer), `emekli` için uyarı. Ayrı bir CLI
komutu (`dil dkim-denetle` adayı, K-167 yüzeyi) açık sorudur.

### Tehdit modeli (B3)

| Tehdit | Önlem |
|---|---|
| **Anahtar sızıntısı** — günlük, IO izi, hata mesajı, panik çıktısı, `/proc/<pid>/environ`, çekirdek dökümü | Credential dosyası öncelikli (ortam değişkeni `environ`'da görünür; uyarı), anahtar baytı hiçbir Zee değerine, IO izine (yalnız seçici + açık anahtar özeti) ya da tanıya girmez; `zeroize`; systemd birimine `LimitCORE=0` önerisi (ZeeOS); hata mesajları anahtar içeriğini değil yalnız seçiciyi anar (K-236 maskesi) |
| **İmza kâhini** — ele geçirilmiş/hatalı Zee kodu ya da gönderim yolu üzerinden başka alan adına ya da keyfî içeriğe imza | Alan eşleşmesi ZORUNLU; imza yalnız ileti dosyası tutamağına, ham `Metin`e değil; oran sınırı ürünün gönderim politikası |
| **Zayıf anahtar / algoritma indirgemesi** | RSA ≥ 2048, `rsa-sha1` yok |
| **Yanlış anahtarla imza (döndürme hatası)** | Açılış eşleşme denetimi |
| **Zamanlama yan kanalı** (RSA imzasında) | `ring` sabit zamanlı uygulama; RustCrypto `rsa` (RUSTSEC-2023-0071, Marvin) aday değildir |
| **Replay için imzalı içerik üretimi** | `x=` seçeneği; gönderim yetkilendirmesi ürünün (SMTP AUTH) |

### B3 kabul ölçütleri

1. Zee'nin imzaladığı ileti B2 doğrulayıcısı **ve** bağımsız doğrulayıcı
   (OpenDKIM `opendkim-testmsg` ya da eşdeğeri) tarafından `pass`.
2. Gerçek alıcıda (ör. Gmail'e test gönderimi, insan adımı) `dkim=pass`
   ve DMARC hizalaması `pass` — sürüm kanıtında elle kayıt.
3. 1024 bit RSA, `rsa-sha1`, bozuk PKCS#8, aynı ad hem ortamda hem
   credential'da → açılış reddi.
4. `From:` alanı bildirilmemiş alan → imza reddi.
5. IO izi ve tüm tanı çıktılarında anahtar baytının (PEM gövdesinin
   herhangi bir 16 karakterlik dilimi) bulunmadığını doğrulayan test.
6. Döndürme senaryosu: iki `etkin` seçici, bir `emekli`; açılış denetimi
   sahte çözücüyle eşleşme/uyuşmazlık.

## B4 — Teslim güvenliği: MTA-STS, DANE/TLSA, TLS-RPT

### Önkoşul: uzak MX'e teslim (`smtp-teslim`)

Bugün giden SMTP yalnız bildirilmiş origin'edir (spec/23). B4, RFC-0039
katman E'nin `smtp-teslim` yetkinliğini **önkoşul** alır: yalnız 25
kapısına, B1 ile çözülen MX'lere (MX yoksa RFC 5321 §5.1 örtük A/AAAA),
spec/23 IP kuralları (private/loopback/link-local/metadata yasak) ile;
bağlantı, eşzamanlılık ve hacim bütçeli. Bu yetkinlik ayrı bir K-işinde
(B4a) açılır; MTA-STS ilke indirmesi Zee'nin `ağ_hedefleri` listesinden
geçmez, `smtp-teslim` içindeki dar HTTPS alt işlemidir.

Giden SMTP istemcisinin TLS'i `native-tls`'den, A2b sunucu TLS'iyle aynı
`rustls` yığınına taşınır: DANE eşlemesi özel `ServerCertVerifier` ister
ve `native-tls` bunu desteklemez. Mevcut aktarıcıya gönderim yolunun
(K-210) taşınıp taşınmayacağı ayrı karardır (açık soru).

### Teslim TLS politikası (öncelik sırası)

1. **DANE** (`dns_dnssec: "zorunlu"`, MX RRset ve MX konağının TLSA RRset'i
   `doğrulanmış`): RFC 7672. Yalnız `DANE-TA(2)` ve `DANE-EE(3)`
   kullanımları; `PKIX-TA(0)`/`PKIX-EE(1)` kullanılamaz sayılır (RFC 7672
   §3.1.3). `DANE-EE(3)` için ad denetimi yapılmaz, `DANE-TA(2)` için MX
   konak adı (RFC 7672 §3.2.2). Kullanılabilir TLSA varken TLS ya da
   eşleme başarısızsa teslim **ertelenir** (düz metne düşülmez). TLSA
   sorgusu `bogus`/geçici hatada da ertelenir (indirgeme engeli).
2. **MTA-STS** (`enforce`): RFC 8461. TXT `_mta-sts.<alan>` (`v=STSv1;
   id=…`), ilke `https://mta-sts.<alan>/.well-known/mta-sts.txt`
   (yönlendirme izlenmez, RFC 8461 §3.3; PKIX doğrulamalı, `webpki-roots`;
   boyut ≤ 64 KiB; son tarih). MX adı ilkedeki `mx:` kalıplarıyla
   eşleşmeli; sertifika MX adı için PKIX geçerli olmalı; değilse teslim
   ertelenir. `testing` kipinde yalnız TLS-RPT'ye raporlanır.
3. **Fırsatçı STARTTLS**: TLS sunulursa kullanılır, sunulmazsa düz metin
   (RFC 3207 pratiği); bu yalnız DANE ve MTA-STS yokken.

Her ikisi de yayımlıysa DANE uygulanabildiğinde öne alınır (RFC 8461 §2).

### MTA-STS ilke önbelleği

- RFC 8461 §5 önbelleği zorunlu kılar: ilk başarılı indirmeden sonra
  `max_age` (≤ 31 557 600 sn) boyunca, DNS TXT'si kaybolsa ya da
  sahtelense de **önbellekteki `enforce` ilkesi geçerlidir** (ilk kullanımda
  güven — TOFU — indirgeme engeli).
- Önbellek kalıcı dosyadır (spec/08 atomik yazma, proje veri kökü); süreç
  yeniden başlasa da korunur. Yeni `id` görülünce ilke yeniden indirilir;
  indirme başarısızsa eski ilke `max_age` bitene kadar geçerli kalır.
- Önbellek kalıcı durum eklediği için K-173/ADR-069 soak koşusu B4b'de
  ZORUNLUDUR; önbellek kayıt sayısı ve dosya boyutu bütçelidir.

### TLS-RPT

- TXT `_smtp._tls.<alan>` (`v=TLSRPTv1; rua=mailto:…` ya da `https://…`,
  RFC 8460 §3).
- Çalışma zamanı teslim girişimlerinin TLS sonuçlarını (başarı sayısı,
  hata türü: `starttls-not-supported`, `certificate-host-mismatch`,
  `validation-failure`, `sts-policy-fetch-error`, `tlsa-invalid`,
  `dnssec-invalid` …) alan başına günlük toplamlar olarak kalıcı dosyada
  biriktirir.
- Günlük rapor RFC 8460 §4 JSON'udur; gzip (`flate2`, ağaçta) +
  `application/tlsrpt+gzip`. JSON'u kim kurar — Zee (`json metni` var) mı,
  host mu — açık soru; öneri: host kurar (şema doğruluğu), Zee gönderimi
  zamanlar ve `mailto:` için kendi giden kuyruğunu, `https:` için
  `smtp-teslim` içindeki dar POST'u kullanır.
- Rapor içeriği istemci IP'si ve MX adları içerir ama ileti içeriği ve
  alıcı adresi içermez (RFC 8460 §7 mahremiyet).

### Tehdit modeli (B4)

| Tehdit | Önlem | Kalan risk |
|---|---|---|
| **STARTTLS soyma (indirgeme)** | DANE ya da önbellekli MTA-STS `enforce` varken düz metne düşülmez, teslim ertelenir | Hiçbirini yayımlamayan alıcı için fırsatçı TLS (sektör pratiği) |
| **MX sahteciliği** | DANE: DNSSEC doğrulamalı MX + TLSA; MTA-STS: HTTPS (PKIX) ile gelen `mx:` kalıpları DNS'ten bağımsız | İmzasız bölgede ilk MTA-STS indirmesinden önceki TOFU penceresi |
| **MTA-STS TXT silme** | Önbellekteki ilke `max_age` boyunca geçerli | İlk temas |
| **İlke indirmesinde SSRF** (`mta-sts.<alan>` private IP'ye çözülür) | spec/23 IP kuralları; yönlendirme izlenmez; yalnız 443 | — |
| **Yavaş/dev ilke sunucusu, yavaş MX (DoS)** | İndirme son tarihi ve 64 KiB; bağlantı/okuma bütçesi; alan başına eşzamanlılık | Teslim gecikmesi (kuyruk yeniden dener) |
| **TLS-RPT amplifikasyonu** (saldırgan `rua`yı üçüncü tarafa yönlendirir) | Alan başına günde tek rapor, rapor boyutu sınırı, `rua` hedef sayısı ≤ 2; `https:` hedefi için spec/23 IP kuralları | Düşük hacimli istenmeyen rapor |
| **DANE ayrıştırma/eşleme hataları** | TLSA `seçici`/`eşleme` yalnız RFC 6698 değerleri (0/1, 0/1/2); bilinmeyen değer kullanılamaz kayıt; fuzz | — |
| **Zaman / son tarih** | Her DNS, HTTPS ve SMTP adımı son tarihli; aşım geçici hata → kuyrukta yeniden deneme | — |
| **Önbellek zehirleme (MTA-STS önbelleği)** | Yalnız PKIX doğrulamalı HTTPS yanıtı yazılır; atomik dosya; dosya bozuksa önbellek yok sayılmaz, teslim o alan için ertelenir ve operatöre tanı (fail-closed) — açık soru | — |

### B4 kabul ölçütleri

1. `smtp-teslim` (B4a): private/loopback/metadata MX reddi; MX'siz alanda
   örtük A/AAAA; bütçeler; IO izi.
2. DANE (B4c): `DANE-EE(3) SPKI SHA-256` ve `DANE-TA(2)` eşleşen/eşleşmeyen
   sertifika; `PKIX-EE(1)`-yalnız TLSA kümesi "kullanılamaz" → DANE yok
   davranışı; `bogus` TLSA → erteleme; RFC 7672 Ek örnekleri.
3. MTA-STS (B4b): `enforce` + eşleşmeyen MX → erteleme; TXT silinmiş +
   önbellek geçerli → `enforce` sürer; yönlendirmeli ilke → indirme hatası;
   65 KiB ilke → hata; `testing` → teslim + rapor.
4. TLS-RPT (B4d): RFC 8460 JSON şemasına uyan, gzip'li rapor; günde tek;
   `rua` iki hedef üstü kırpılır (ya da reddedilir — açık soru).
5. Gerçek internette (insan adımı): DANE yayımlayan ve MTA-STS yayımlayan
   birer alıcıya teslim, oturum kaydı sürüm kanıtında.
6. Soak (K-173) 1800 sn önbellek büyümesi sınırında.

## Uygulama kararı — K-397 (24 Kasım 2026)

ZeeMail'in `zeemail/disteslim` hizmeti (ZeeOS'ta 25 kapısına çıkışı olan tek
hizmet) teslim motorunu Zee'de taşır: SMTP istemci durum makinesi, MX sırası,
yeniden deneme, MTA-STS/DANE/TLS-RPT kararı, DSN ve ileti kurma. Dilden
istediği yalnız dört dar yapı taşıdır. K-397 bunları iki dilimde açar
(görev metnindeki "B3 TCP+STARTTLS / B4 DKIM" adlandırması bu RFC'nin
**B4a** ve **B3** aşamalarına karşılık gelir):

| Dilim | RFC aşaması | Yüzey | ADR |
|---|---|---|---|
| 1 | B4a | `ağ_hedefleri` `tcp://` biçimi, giden TCP, istemci STARTTLS, MTA-STS ilke indirmesi için `https://mta-sts.*` | ADR-173 |
| 2 | B3 | DKIM imza ilkeli (`rsa-sha256`, `ed25519-sha256`), anahtar credential'dan | ADR-174 |

### Taslaktan bilinçli sapmalar

1. **`smtp-teslim` yetkinliği açılmaz.** Giden bağlantı mevcut `ağ`
   yetkinliği ve `ağ_hedefleri` listesine iki yeni biçimle bağlanır; tek
   outbound politika noktası (RFC-0024) korunur, yeni yetkinlik/P015 bağı
   gerekmez. Teslim politikası (MX sırası, MTA-STS, DANE, fırsatçı TLS)
   ürünün Zee kodudur; dil yalnız "hangi IP'ye, hangi kapıya" sınırını ve
   kriptografiyi taşır.
2. **MTA-STS indirmesi `smtp-teslim` alt işlemi değildir.** Mevcut HTTP
   istemcisi zaten yönlendirme izlemez, çözümleme sonrası IP'yi denetler,
   PKIX doğrular, gövdeyi ve süreyi sınırlar. Eksik olan yalnız önceden
   bilinmeyen `mta-sts.<alan>` origin'idir; bunun için dar joker
   `https://mta-sts.*` eklenir (yalnız 443, yalnız
   `/.well-known/mta-sts.txt` yolu, yalnız public IP).
3. **DKIM imzası kanonikleştirilmiş bayt üzerinde atılır.** Kanonikleştirme,
   başlık seçimi ve `bh=` ZeeMail'dedir (ürünün B2 doğrulayıcısıyla aynı
   kod). İmza kâhini önlemi taslaktaki "yalnız dosya tutamağı" yerine imza
   girdisinin kendisi üzerinden kurulur (Dilim 2, aşağıda).

### Dilim 1 — giden TCP (B4a)

```text
# proje.dil
yetkinlikler "ağ" listesi olsun
ağ_hedefleri "tcp://*:25", "https://mta-sts.*" listesi olsun

# Zee
sonuç "mx1.alici.ornek:25" adresine bağlanmayı dene olsun   # Sonuç<Sözlük>
selam bağlantıdan satır okumayı dene olsun
bağlantıya "EHLO gonderen.ornek" satırını yaz
…
tls bağlantıda TLS istemcisi başlatmayı dene "mx1.alici.ornek" ile olsun
bağlantıyı kapat
```

- **Hedef biçimleri.** `tcp://<konak>:<kapı>` tam hedeftir (kapı zorunlu,
  varsayılan yok); çözülen IP'ler spec/23 kuralına göre denetlenir
  (`yerel-ağ` yetkinliği varsa yerel adres de olabilir — test ve iç röle
  için). `tcp://*:25` **yalnız 25 kapısında** geçerli jokerdir: her public
  IP, `yerel-ağ` açık olsa bile yalnız public. Başka kapıda joker P-hatası.
- **SSRF ve yeniden bağlama.** Hedef ad ise süreç çözücüsüyle bir kez
  çözülür ve en çok 16 adres sınıflandırılır; **biri bile** izinsizse
  `hedef_yasak` (fail-closed, HTTP istemcisinin kuralıyla aynı). Bağlantı **denetlenen IP'ye** açılır (ad ikinci kez
  çözülmez); TOCTOU penceresi yoktur. Public sayılmayan aralıklar:
  0/8, 10/8, 100.64/10 (CGNAT), 127/8, 169.254/16, 172.16/12, 192.0.0/24,
  192.0.2/24, 192.88.99/24, 192.168/16, 198.18/15, 198.51.100/24,
  203.0.113/24, 224/4 ve üstü (multicast/ayrılmış/yayın); IPv6'da
  2000::/3 dışı her şey, ::1, fc00::/7, fe80::/10, 2001::/23, 2001:db8::/32,
  2002::/16 (6to4), 3fff::/20 ve **IPv4-mapped `::ffff:0:0/96` (her
  profilde ret)**; 64:ff9b::/96 (NAT64) 2000::/3 dışında olduğu için ret.
  Metadata/link-local her profilde yasak (mevcut kural).
- **Son tarih ve sınırlar.** Bağlanma `min(Zee son tarihi, 30 sn)`; okuma,
  yazma ve TLS el sıkışması mevcut bağlantı ayarlarıyla (`bağlantı_boşta_süresi`
  varsayılan 300 sn, satır sınırı 1000, giden tampon 64 KiB). Görev başına
  aynı anda **bir** giden bağlantı (ikinci çağrı `zaten_bağlı`); süreç
  başına **en çok 16** eşzamanlı giden bağlantı (`sınır`). Görev ya da
  program biterken açık giden bağlantı kapatılır.
- **Kalıp yeniden kullanımı.** Bağlantı değer değildir (ADR-165): başarılı
  bağlanma o görevin bağlantısı olur ve A2/A2c kalıpları (satır oku/yaz,
  süreli okuma, bayt okuma, parça yazma, dosya aktarma, nokta bloğu) aynen
  çalışır. Bu kalıplar artık `ağ-dinleyici` **ya da** `ağ` yetkinliğiyle
  derlenir; gerçek kapı bağlantıyı açan adımdır. Dinleyici görevinin
  içinden giden bağlantı açılamaz (`geçersiz`): gelen oturumun bağlantısı
  sessizce değiştirilemez.
- **Sonuç ve hatalar.** Başarı `Sözlük{uzak_adres, uzak_kapı, yerel_kapı}`
  (Metin). Ret `C030` Sonuç'u, `tür` ∈ `geçersiz` (biçim), `hedef_yasak`
  (politika/IP), `çözülemedi`, `bağlanamadı` (ret/erişilemez), `zaman_aşımı`,
  `zaten_bağlı`, `sınır`.

### Dilim 1 — istemci STARTTLS (B4a)

```text
tls bağlantıda TLS istemcisi başlatmayı dene "mx1.alici.ornek" ile olsun
# → Sonuç<Sözlük>{doğrulandı: evet|hayır, doğrulama_hatası, sürüm, takım,
#                 sunucu_adı, sertifika_sha256, spki_sha256,
#                 zincir_sha256, zincir_spki_sha256}
```

- `rustls 0.23` + `ring`, sunucu tarafıyla aynı sabit sürüm ve takım
  politikası (TLS 1.2/1.3, ileri gizlilikli AEAD). Kök deposu
  `webpki-roots` (kilitli ağaçta; doğrudan bağımlılığa yükseltme).
- **Doğrulama sessiz değildir.** PKIX sonucu (zincir, süre, ad) kayda
  ayrı alan olarak yazılır: `doğrulandı` `evet` ya da `hayır`;
  `hayır` ise `doğrulama_hatası` ∈ `ad_uyuşmuyor`, `süresi_dolmuş`,
  `henüz_geçerli_değil`, `bilinmeyen_yayıncı` (kendinden imzalı dahil),
  `iptal`, `geçersiz_sertifika`. El sıkışma PKIX başarısızken de tamamlanır,
  çünkü DANE-EE(3) ve fırsatçı TLS (RFC 7672, RFC 3207 pratiği) bunu ister;
  **el sıkışma imzası her durumda doğrulanır**: karşı taraf raporlanan
  SPKI'nin özel anahtarına sahiptir. MTA-STS `enforce` için ürün
  `doğrulandı = evet` olmadan teslim etmez; TLSA eşlemesi ürünün kodudur.
- **Özetler.** `sertifika_sha256` uç sertifikanın DER'inin, `spki_sha256`
  onun `SubjectPublicKeyInfo` DER'inin SHA-256'sı (TLSA `3 0 1`/`3 1 1`);
  `zincir_*` sunucunun gönderdiği sırayla bütün sertifikaların aynı
  özetleri, boşlukla ayrık (DANE-TA(2) `2 0 1`/`2 1 1`). Eşleme türü 2
  (SHA-512) ve 0 (tam) sağlanmaz; ürün bunları "kullanılamaz" sayar.
  SPKI çıkarımı yeni, sınırlı bir DER yürüyücüsüdür (aşağıda fuzz notu).
- **Sunucu adı.** ASCII DNS adı olmalı (IP, boş, 253 üstü → `geçersiz`);
  SNI ve PKIX ad denetimi bu addır. Yanlış SNI `ad_uyuşmuyor` verir.
- **Yanıt enjeksiyonu.** STARTTLS'e verilen `220`den sonra ara bellekte
  düz metin varsa (CVE-2011-1575 sınıfı) yükseltme yapılmaz, bağlantı
  kapanır, `tür=tls`. Zaten TLS olan bağlantı `tür=tls`.

### Dilim 2 — DKIM imza ilkeli (B3)

```text
# proje.dil
yetkinlikler "kriptografi", … listesi olsun
dkim_anahtarı "ornek.test" "z2026a" "ed25519-sha256" "zeemail_dkim_anahtari" olsun

# Zee
b dkim_girdisi metninin "ornek.test" alanı "z2026a" seçicisiyle dkim imzasını almayı dene olsun
# → Sonuç<Metin>: `b=` değeri (base64, satır kırma yok)
```

- Girdi, RFC 6376 §3.7'nin imza girdisidir: kanonikleştirilmiş imzalı
  başlıklar ve sonda `b=` boş, CRLF'siz `DKIM-Signature` başlığı.
  Kanonikleştirme ve `bh=` ZeeMail'dedir.
- **İmza kâhini önlemi (host'ta, kanonikleştirmeyi bilmeden):** girdinin
  son başlığı `dkim-signature` olmalı ve onun `d=`, `s=`, `a=` etiketleri
  bildirilmiş anahtarla birebir eşleşmeli, `h=` `from` içermeli, `b=` boş
  ve son etiket olmalı; girdide tam bir `from:` başlığı bulunmalı ve
  adresinin alanı `d=` ya da onun alt alanı olmalı (gevşek hizalama). Aksi
  `geçersiz`; anahtar başka alan adına ya da keyfî bayta imza atmaz.
- **Anahtar.** PKCS#8 PEM, K-339 sözleşmesiyle credential dizininden (yoksa
  ortam; aynı ad ikisinde → ret). Açılışta bir kez okunur ve doğrulanır:
  RSA 2048–4096 bit, Ed25519; `rsa-sha1`, 1024 bit, bozuk PEM, eksik
  credential → çıkış 2, tanı yalnız seçiciyi anar. Anahtar baytı Zee
  değeri, IO izi, günlük ya da tanı olmaz; okuma tamponu `zeroize` edilir.
- B2 doğrulayıcısı (K-390 B2) ve K-389 `sha256` ile tek kripto yüzeyi:
  hepsi `ring`; DKIM özeti `ring::digest::SHA256`, base64 RFC-0043 kodlayıcısı.

### Tehdit modeli (K-397)

| Tehdit | Önlem | Kalan risk |
|---|---|---|
| **SSRF** — Zee kodu (ya da MX kaydını denetleyen saldırgan) giden bağlantıyı iç hizmete (127.0.0.1:25, 10.x, metadata, unix socket) yöneltir | Hedef yalnız `ağ_hedefleri`; joker yalnız 25 ve yalnız public IP; IPv4-mapped, NAT64, 6to4, Teredo ve dokümantasyon aralıkları ret; metadata her profilde ret | Public internetteki keyfî bir 25 kapısına bağlanma — hizmetin işi budur; oran/hacim ürünün kuyruğu |
| **DNS yeniden bağlama** — ad denetimde public, bağlanırken private çözülür | Tek çözümleme; bağlantı denetlenen `SocketAddr`'a açılır | — |
| **Kapı kaçırma** — joker 25'i 22/6379'a genişletme | Joker kapısı sabit 25; başka kapıda P-hatası; tam hedefte kapı zorunlu | — |
| **STARTTLS soyma** | Dil yalnız mekanizmadır; DANE/MTA-STS varken düz metne düşmeme ürün politikası (B4 tablosu) | Ürün hatası; ZeeMail testleri |
| **Sessiz doğrulama hatası** | `doğrulandı`/`doğrulama_hatası` her sonuçta; doğrulanmamış el sıkışma da imzayı doğrular | Ürün alanı okumazsa fırsatçı TLS'e düşer — RFC 8461 testlerinde ürün kanıtı |
| **Anahtar sahipliği taklidi** (DANE için yanlış SPKI raporu) | SPKI uç sertifikadan, el sıkışma imzası o anahtarla doğrulanmadan kayıt verilmez | — |
| **Yanıt enjeksiyonu** (CVE-2011-1575) | `220` sonrası tampon boş değilse kapanış | — |
| **Kaynak tüketimi** (yavaş MX, sonsuz satır, dev zincir) | Bağlanma 30 sn, boşta süresi, satır sınırı, 64 KiB giden tampon, TLS kayıt tamponu 64 KiB; süreç başına 16 giden bağlantı; zincir en çok 10 sertifika, DER yürüyücüsü derinlik/uzunluk sınırlı | Teslim gecikmesi |
| **MTA-STS indirmesinde SSRF/exfil** | Joker yalnız `mta-sts.` ön ekli ad, 443, sabit yol, public IP, yönlendirme yok | Saldırganın alanındaki `mta-sts.` sunucusuna sabit yollu istek — RFC 8461 gereği |
| **DKIM imza kâhini** | Girdi `d=`/`s=`/`a=` ve From hizalaması host'ta denetlenir | Ürün kendi alanı adına yanlış içeriği imzalayabilir — gönderim yetkilendirmesi ürünün |
| **Anahtar sızıntısı** | Credential tercih, `zeroize`, tanılarda yalnız seçici; IO izinde yalnız seçici ve girdi özeti | Süreç belleği ve çekirdek dökümü — ZeeOS birimi `LimitCORE=0` |
| **Zayıf anahtar** | RSA < 2048 ve > 4096, `rsa-sha1` açılışta ret | — |

### Deterministik IO ve fuzz

- Giden bağlanma (hedef, seçilen IP, sonuç), TLS istemci sonucu (bütün kayıt
  alanları) ve DKIM imzası (seçici, girdi SHA-256'sı, imza) IO izine girer;
  replay ağa gitmez. `dil dene` hermetik adaptörü betiklenmiş sahte uzak
  sunucu ve sahte TLS kaydı verir; politika denetimi hermetikte de aynı
  koddur.
- Yeni ayrıştırıcılar: X.509 SPKI DER yürüyücüsü (Dilim 1) ve DKIM imza
  girdisi etiket denetimi (Dilim 2). İkisi saf modüldür; birim testleri
  bozuk/kesik/aşırı uzun DER'i ve etiket kenar durumlarını kapsar. Fuzz
  hedefleri (`x509_spki`, `dkim_imza_girdisi`) aday olarak ADR-173/174'te
  kayıtlıdır; RC fuzz kapısına alınmaları açık iştir (ayrıştırıcılar
  saldırganın gönderdiği sertifikayı yalnız rustls/webpki kabul ettikten
  sonra görür).

### Dilim 1 kabul ölçütleri

1. `tcp://` tam hedef ve `tcp://*:25` joker: bütün özel aralıklar, IPv4-mapped,
   izinsiz kapı ve listede olmayan hedef reddi (hermetik + birim).
2. Yeniden bağlama: bağlantının denetlenen IP'ye açıldığını gösteren test.
3. Gerçek TCP: loopback yankı sunucusuyla satır/parça/süreli okuma/aktarım.
4. Gerçek TLS: geçerli zincir `doğrulandı=evet`; kendinden imzalı, süresi
   dolmuş, yanlış SNI `doğrulandı=hayır` + doğru neden; SPKI özeti
   `openssl` ile bağımsız hesaplanan değere eşit.
5. Görev başına ikinci bağlantı `zaten_bağlı`; dinleyici görevinde ret.
6. MTA-STS joker: yol dışı istek, 443 dışı kapı, `mta-sts.` önekisiz ad reddi.

## Bağımlılık seçimi

| İhtiyaç | Seçim | Ağaçta mı | Gerekçe |
|---|---|---|---|
| RSA-SHA256 imza/doğrulama, Ed25519 | `ring 0.17.14` | Evet (rustls) | TLS ile aynı sağlayıcı (RFC-0039 D); `RsaKeyPair::from_pkcs8` 2048+ bit, PKCS#1 v1.5 SHA-256 doğrulama 1024–8192 (biz 4096'ya sınırlarız). RustCrypto `rsa` RUSTSEC-2023-0071 nedeniyle aday değil |
| Ed25519 alternatifi | `ed25519-dalek 3.0.0` | Evet | Tek sağlayıcı ilkesi için `ring` öncelikli; ikisinden biri seçilir (açık soru) |
| SHA-256 | `sha2` / `ring::digest` | Evet | — |
| base64 | RFC-0043 kararı (`base64 0.23.1`) | Evet (geçişli) | — |
| DNS tel biçimi | **Kendi sınırlı ayrıştırıcımız** (`dns_teli`, K-390/ADR-169); `hickory-proto =0.26.3` yalnız dev/fuzz kâhini | — | Taslak `hickory-proto` ya da `domain` öneriyordu; uygulama günü değerlendirmesi: `hickory-proto` +46 paket (icu4x/idna/url/tracing), `domain` +10 paket ama `unsafe` yoğun ve RUSTSEC-2026-0310. Üretim ağacı değişmez; Zee ⊆ hickory diferansiyel testi ve `dns_yaniti` fuzz hedefi (RC kapısında) |
| DNSSEC doğrulama | **Yok** (yerel çözücü) | — | B1 karar (d) ertelendi |
| PSL | **Yeni crate yok**; host ayrıştırıcı (biçim basit) + veri dosyası | — | Alternatif `psl`/`publicsuffix` crate'leri listeyi gömer ya da kendi sürüm döngüsünü dayatır; ZM-F025 sürümlü dosya ister |
| SPF/DKIM/DMARC/ARC değerlendirici | **Kendi host kodu** (`ring` + paylaşılan başlık ayrıştırıcı) | — | Hazır kitaplıklar (ör. Rust posta kimlik doğrulama crate'leri) genelde async çözücü + tokio ve kendi ayrıştırıcısını getirir; ayrıştırıcı tutarlılığı ilkesine aykırı. Yalnız **dev-dependency kâhini** olarak diferansiyel testte değerlendirilir (K-362 emsali); lisans ve ağaç uygulama günü doğrulanır |
| HTTPS (MTA-STS) | `ureq 3.4.0` + `rustls` | Evet | Yönlendirme kapalı, boyut sınırı mevcut istemci deseninde |
| SMTP istemci TLS'i (DANE) | `rustls 0.23.46` özel doğrulayıcı | Evet | `native-tls` DANE'ye uygun değil |
| gzip (TLS-RPT) | `flate2 1.1.9` | Evet (png) | Doğrudan bağımlılığa yükseltme; yeni crate yok |

Net yeni üretim crate'i: **yok** (K-390 kararı; taslak yalnız DNS tel biçimi
kitaplığını öngörüyordu). Diğer her şey mevcut kilitli ağaçtan doğrudan
bağımlılığa yükseltmedir.

## Yetkinlik etkisi

| Yetkinlik | Aşama | Açtığı sınır | Bağ |
|---|---|---|---|
| `dns` (yeni) | B1 | Bildirilmiş loopback çözücüye MX/A/AAAA/TXT/TLSA sorgusu; PSL dosyası okuma | `dns_çözücü` bildirimi ZORUNLU (P015 sınıfı ihlal) |
| `ağ-dinleyici` (var) | B2 | Değişmez; doğrulama `DATA` dosya tutamağı üzerinde | B2 → `dns` |
| `kriptografi` (var) | B3 | DKIM imzası; `dkim_anahtarları` bildirimi | B3 → (açılış denetimi için) `dns` isteğe bağlı |
| `smtp-teslim` (yeni, RFC-0039 E) | B4 | Kamu MX'lerine 25 kapısı; MTA-STS ilke HTTPS'i; TLS-RPT gönderimi | `smtp-teslim` → `dns` |
| `ağ` / `ağ_hedefleri` | — | Değişmez; MTA-STS indirmesi buradan geçmez | — |

Yetkinlik ekleme spec/23 tablosu, P015 bağları, K-167 `yetkinlik`/proje
alanı satırları ve RFC-0024 ile aynı committe yapılır.

## Deterministik IO

- DNS sorgu/yanıtı, SPF/DKIM/DMARC/ARC değerlendirici sonuçları (girdi
  özeti + sonuç), DKIM imza olayı (seçici + ileti özeti, **anahtar yok**),
  MTA-STS indirme ve TLS-RPT gönderimi IO izine girer.
- Replay ağa gitmez; izdeki yanıtı verir. `dil dene` için sahte çözücü
  statik bölge dosyasıdır (proje `test` verisi), gerçek ağ yoktur.
- DKIM imzası deterministiktir (Ed25519 deterministik; RSA PKCS#1 v1.5
  deterministik); bu nedenle replay imza baytlarını da karşılaştırabilir.

## Core freeze gerekçesi ve kanıt kapısı

1. **Ürün kaydı.** `urun-h` etkin (K-355). Karşılanmış.
2. **Ürün sürtünmesi.** ZM-F025 ve ZeeMail motor güvenlik kapsamı ürünün
   kendi kaydındadır (depo dışı). Her aşama açılmadan önce ürün, gerçek
   Zee kaynağını (`dogfood/urun-h/korpus/`) exact beklentiyle koyar: B1 MX
   sorgusu yokluğu, B2 sahte `From:` iletisinin teslim edilmesi, B3 imzasız
   giden iletinin alıcıda reddi, B4 düz metne düşen teslim.
   **Ürün durumu (24 Kasım 2026):** ürün commit'i
   `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac` korpusunda
   `korpus/zm-r006-mx-sorgusu-yok.dil` (blob `4af3d1b1…`, SHA-256
   `b645c2230562c1cd37e4eae221b916ad75a69f801946038753036e5a50f97d08`;
   bugün S015) ve `korpus/zm-r007-dnssiz-sahte-gonderen.md` (blob
   `161816a4…`, SHA-256
   `cbfe3b4235180401d208b82dc0bac9bde6c4f220922c74e857a1a2f03e5f2e35`;
   DMARC `p=reject` yayımlayan iş ortağı alanının taklidi DNS'siz teslim
   ediliyor, yalnız kurumun kendi alanı reddediliyor) hazır. Aşama açılırken `dogfood/urun-h/korpus/`
   altına exact beklentiyle aynalanır.
3. **Güvenlik önceliği.** B2 olmadan ZeeMail kimlik avına açık bir posta
   kutusu sunucusudur; B3 olmadan giden posta teslim edilemez. Bu iki aşama
   `correctness/security` gerekçesini de taşır; ama yeni yüzey açtıkları
   için sınıfları yine `dogfood-change`dir ve ADR/spec/RFC zinciri ister.
4. **Minimal değişiklik.** Grammar değişmez; yüzey mevcut kalıp ailesinin
   (`… sorgulamayı dene`, `… doğrulamayı dene`, `… imzalamayı dene`)
   uzantısıdır; yeni tür yok (RFC-0043 tür kararı).
5. **Zincir (aşama başına).** Alt ADR → spec maddesi (`spec/23` yetkinlik,
   yeni posta güvenliği spec bölümü adayı) + `spec-madde-kaniti` → olumlu/
   olumsuz test → reproducer → K-167 satırları → GB kayıtları → fuzz
   hedefi → `docs/surumler.md`.
6. **RC fuzz kapısı.** Yeni ayrıştırıcılar (`dns_yaniti`, `psl`,
   `ileti_basligi`, `dkim_imzasi`, `spf_kaydi`, `dmarc_kaydi`,
   `mta_sts_ilkesi`, `tlsa`) AGENTS.md RC fuzz listesine aday; hangilerinin
   RC kapısına gireceği aşama ADR'sinde.

V1 kapı listesine eklenmez; V1 sözü değildir.

## RFC-0039, RFC-0041 ve RFC-0043 ile ilişki

- **RFC-0039 katman D/E/G** bu RFC'nin çatısıdır: D'nin imza maddesi (RSA
  `ring`, anahtar tutamakla) B3'te, E'nin DNS ve `smtp-teslim` maddeleri
  B1/B4'te ayrıntılanır. Bir fark vardır: RFC-0039 E "DNSSEC doğrulaması bu
  RFC'de vaat edilmez" der; bu RFC doğrulamayı **yerel doğrulayıcı
  çözücüye** bırakır ve `AD` bitini güvenlik kararına (yalnız DANE) bağlar.
  RFC-0039 ilke 1'in SPF incelmesi yukarıda gerekçelidir. RFC-0039 kabul
  edilirken bu iki nokta çatıya işlenir.
- **RFC-0039 A2b** sunucu TLS'i `rustls 0.23 + ring` ile; B4 istemci TLS'i
  aynı yığına taşır (tek TLS sağlayıcı).
- **RFC-0039 A2c** blok dosyası tutamağı B2/B3'ün girdisidir.
- **RFC-0041 C** giden ek türü kaydı ve MIME kodlaması host'ta; B3 imzası
  bu MIME çıktısının son hâli üzerinde atılır (imzadan sonra iletiyi
  değiştiren adım yasak).
- **RFC-0043** bayt/kodlama/özet yapı taşlarını verir (DKIM `p=`/`b=`
  base64, DMARC/SPF kayıtlarında oktet uzunlukları, küçük blok özeti);
  B2/B3'ten önce uygulanır.

## Dört soru süzgeci

1. **Türkçe doğal mı?** — `alanın mx kayıtlarını sorgulamayı dene`,
   `ileti dosyasının dkim imzalarını doğrulamayı dene` mevcut
   `… almayı dene` ritmini izler; protokol adları (MX, SPF, DKIM, DMARC)
   çevrilmez, çünkü operatörün aradığı terimlerdir. Kesin kelimeler
   kullanılabilirlik kartıyla.
2. **Deterministik mi?** — Evet; DNS/HTTPS/saat IO izinde, imza
   deterministik, sahte çözücüyle hermetik test.
3. **Öğrenilebilir mi?** — Posta kimlik doğrulaması karmaşıktır; Zee bunu
   saklamaz ama ayırır: değerlendiriciler tek çağrı ve yapılandırılmış
   sonuç, politika okunur Zee kodu. Öğretici rehber (ZeeMail örneği)
   aşamalarla birlikte.
4. **Profesyonel ölçekte savunulabilir mi?** — Yerel doğrulayıcı çözücü +
   `AD` biti (Postfix/Exim DANE modeli), RFC 7208 sınırları, RFC 8301
   algoritma politikası, tek ayrıştırıcı ilkesi, RFC 8601 temizliği,
   MTA-STS TOFU önbelleği ve DANE-TA/EE sınırı yerleşik MTA'ların (Postfix,
   Exim, OpenSMTPD + rspamd/OpenDKIM/OpenDMARC) yapılandırma
   pratikleriyle uyumludur.

## Alternatifler

- **Ayrı bir milter/rspamd sürecine devretmek** — ürün sahibi motorun Zee
  olmasını istedi (RFC-0039 motivasyon); dış süreç ayrıca ayrıştırıcı farkı
  ve dağıtım yükü getirir. Ret.
- **Tüm değerlendirmeyi Zee'de yazmak** (yalnız DNS ve kripto intrinsic) —
  RFC-0039 ilke 1'e en yakın; ama bütçe ve kanonikleştirme güvenlik
  sınırları Zee programcısının doğruluğuna kalır ve her ürün yeniden yazar.
  Ret (DMARC politikası hariç).
- **Hazır posta kimlik doğrulama crate'i üretimde** — async/tokio çözücü,
  kendi ayrıştırıcı, geniş ağaç; dev-kâhin olarak tutulur.
- **Süreç içi DNSSEC doğrulama** — ertelendi (B1 tablo).
- **PSL'i derleyiciye gömmek** — güncelleme derleyici sürümüne bağlanır;
  ret.
- **DANE yerine yalnız MTA-STS** — DNSSEC'siz ortam için yeterli, ama ürün
  güvenlik kapsamı ikisini de istiyor ve DANE TOFU penceresi taşımaz; ikisi.

## Korpus etkisi

Golden değişmez. Aşamalarla `dogfood/urun-h/korpus/` altında reproducer'lar
ve sahte çözücülü proje testleri eklenir.

## Açık sorular

**Ürün görüşü (`urun-h`, 24 Kasım 2026).** ZeeMail şu yanıtları verdi;
aşama ADR'lerinde karar girdisidir, bu taslağı bağlamaz:

- (1) Çözücü: **unbound**, `127.0.0.1:53`, yalnız loopback, QNAME
  minimisation açık; systemd-resolved'ın DNSSEC'i (`allow-downgrade`)
  indirgemeye açık olduğu için DANE'ye uygun bulunmadı.
- (4) A-R: MX sınırında gelen **tüm** `Authentication-Results` silinsin;
  dış A-R hiçbir yerde tüketilmiyor, güvenilen yönlendiriciler ARC ile
  geliyor. Ürün motoru bugün yalnız kendi authserv-id'li olanları siliyor;
  B2a entegrasyonu kısa.
- (5) DKIM üst sınırı: ileti başına en çok **5** imza doğrulansın, fazlası
  `neutral` (`temperror` değil); RSA 1024–4096 bit (< 1024 kalıcı hata,
  RFC 8301); Ed25519 (RFC 8463) sayılır.
- (6) `l=`: `l=` taşıyan imza **DMARC hizalamasına sayılmaz**; sonuç A-R'ye
  `policy`/`neutral` olarak yazılır; imzalarken `l=` hiç kullanılmaz (B3).
- (9) DNS bütçesi: ileti başına **~10** sorgu önerildi (SPF'in 10 sınırıyla
  aynı mantık). Not: SPF tek başına RFC 7208 §4.6.4 uyarınca 10 sorgu
  mekanizmasına ve `mx` başına 10 adres sorgusuna izin verdiği için toplam
  bütçe SPF + DKIM (≤ 5 anahtar) + DMARC (≤ 2) + ARC'yi kapsamalıdır;
  ürün "SPF kendi RFC 7208 sınırıyla, DKIM + DMARC + ARC ayrıca ≤ 10"
  ayrımına katıldı; B2 ADR'sinin önerilen bütçesi budur.
- Önerilen sıraya katıldı.

1. ~~**Doğrulayıcı çözücü**~~ — **kapandı (K-390/ADR-169):** dil yalnız
   loopback çözücüyü ve `zorunlu` kipte açılış AD sınamasını sözleşir;
   öneri unbound (ürün görüşü: systemd-resolved `allow-downgrade` DANE'ye
   uygun değil). ZeeOS paketlemesi ZeeOS kararıdır.
2. ~~**DNS crate'i**~~ — **kapandı (K-390/ADR-169):** ikisi de değil; kendi
   sınırlı tel biçimi ayrıştırıcımız, `hickory-proto` yalnız kâhin.
3. **SPF değerlendirmesi** host'ta mı (bu RFC'nin önerisi) Zee'de mi
   (RFC-0039 ilke 1'in lafzı)?
4. **A-R temizliği:** yalnız kendi authserv-id'mizi taşıyanlar mı, tüm
   gelenler mi (öneri: tümü, sınır MTA'sı olarak)?
5. **DKIM imza üst sınırı** 5 mi; fazlası `policy` mi, `neutral` mı
   (ürün: 5, `neutral`)?
6. **`l=` politikası** varsayılanı: imzayı `pass` saymamak mı, yalnız
   raporlamak mı?
7. **DMARC `pct`** deterministik örnekleme kaynağı (ileti kimliği özeti?).
8. **DMARCbis ağaç yürüyüşü** PSL'in yanında ne zaman?
9. **İleti başına DNS bütçesi:** öneri SPF RFC 7208 sınırıyla + DKIM/DMARC/
   ARC ≤ 10 (ürün katıldı); SMTP oturumu başına bütçe açık.
10. **CLI:** `dil dkim-denetle` (yayımlı anahtar ↔ credential eşleşmesi) ve
    anahtar üretici komutu açılsın mı (K-167 yüzeyi)?
11. **Aktarıcıya gönderim yolu** (K-210 `native-tls`) da `rustls`'e taşınsın
    mı, yoksa yalnız `smtp-teslim` mi?
12. **TLS-RPT JSON'u** host mu Zee mi kurar?
13. **U-label/IDN** alan adları (SMTPUTF8 ile birlikte) hangi aşamada?
    (K-390: B1a A-label dışını `geçersiz_ad` ile reddeder; açık.)
14. **Bozuk MTA-STS önbellek dosyası** davranışı: alan için erteleme mi,
    önbelleği yok sayıp fırsatçı TLS mi (öneri: erteleme + tanı)?

## Önerilen uygulama sırası ve K-işi bölümlemesi

Numaralar açılışta verilir; her satır ayrı K-işi, ayrı ADR (ya da aynı
ADR'nin eki) ve ayrı PR'dır.

| Sıra | Aşama | İş | Bağımlılık | Not |
|---|---|---|---|---|
| 0 | — | RFC-0043 bayt/kodlama/özet | — | En küçük, önce |
| 1 | B1a | `dns` yetkinliği, stub çözücü, MX/A/AAAA/TXT, hata sınıfları, sahte çözücü | — | Tek yeni crate burada (ADR-038) |
| 2 | B1b | PSL veri dosyası + `kurumsal alanı` | B1a | ZM-F025; **uygulandı** (K-390, ADR-175) |
| 3 | B2a | Paylaşılan başlık ayrıştırıcı + `Authentication-Results` temizliği/yazımı | RFC-0043 | **Güvenlik önceliği**; DNS'siz bile değerli (sahte A-R silme) |
| 4 | B2b | DKIM doğrulama (`rsa-sha256`, `ed25519-sha256`) | B1a, B2a | İlkel olarak B2a'dan önce **uygulandı** (K-390, ADR-175); kanonikleştirme Zee'de |
| 5 | B2c | SPF `check_host` | B1a | — |
| 6 | B2d | DMARC kaydı + hizalama yardımcıları; politika Zee'de | B1b, B2b, B2c | ZeeMail reject/quarantine |
| 7 | B3 | DKIM imzası + çoklu seçici + açılış denetimi | B2b (doğrulayıcıyla test) | Giden kabul |
| 8 | B2e | ARC değerlendirme | B2b | Deneysel RFC 8617; daha düşük öncelik |
| 9 | B4a | `smtp-teslim` (RFC-0039 E) + istemci TLS'i `rustls` | B1a | MX teslimi önkoşulu |
| 10 | B4b | MTA-STS + kalıcı ilke önbelleği + soak | B4a | — |
| 11 | B4c | DANE/TLSA | B4a, B1a `zorunlu` kip, ZeeOS çözücüsü | — |
| 12 | B4d | TLS-RPT | B4b, B4c | — |

B2a'nın B1'den önce alınabilmesi bilinçlidir: dışarıdan gelen sahte
`Authentication-Results` silmek DNS istemez ve ürünün en acil güvenlik
açığını kapatır.

## İlgili kayıt

- K-383 (bu taslak), K-355 (`urun-h`), K-354/RFC-0039, K-381/ADR-165
  (A2 bağlantı + sunucu TLS'i), K-384 (A2c blok dosyası), K-339
  (credential), K-127/RFC-0024 (yetkinlik), K-173/ADR-069 (soak),
  ADR-038 (tedarik zinciri).
