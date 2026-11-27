# 31 — DNS sorgusu ve posta kimlik doğrulama

Normatif kaynak: [RFC-0044](../rfcs/0044-dns-posta-kimlik-dogrulama-ve-teslim-guvenligi.md)
Aşama B1 (B1a, B1b), B2b ve B3, [ADR-169](../adr/169-dns-stub-cozucu-ve-tel-bicimi-siniri.md),
[ADR-177](../adr/177-kamu-son-ekleri-ve-dkim-dogrulama-ilkeli.md),
[ADR-174](../adr/174-dkim-imza-ilkeli.md).
Durum: §1–§3 **TANIMLI** (K-390), §4 **TANIMLI** (K-397 Dilim 2). Başlık
ayrıştırıcısı, SPF, DMARC politikası, ARC ve `Authentication-Results`
(B2a/B2c/B2d/B2e) ayrı bölüm olarak eklenir; teslim güvenliğinin B4a dışı
kalanı taslaktır.

## 1. DNS sorgusu (B1)

### 1.1 Bildirim

ZORUNLU: `dns` yetkinliği `dns_çözücü` ve `dns_dnssec` alanlarıyla birlikte bildirilir; üçünden biri ötekiler olmadan P015'tir. `dns_dnssec` yalnız `"zorunlu"` ya da `"bilgi"` olabilir; varsayılanı yoktur.

ZORUNLU: `dns_çözücü` aynı makinedeki çözücünün `IP:kapı` yazımıdır (`127.0.0.1:53`, `[::1]:53`); loopback dışı adres, ad (`localhost`) ve 0 kapısı P015 ile reddedilir. Uzak çözücünün AD biti güvenilir sayılmaz.

TANIMLI: `dns` yetkinliği bildirimsiz tek dosya geliştirici akışında ve `--güvenli` çocuk modunda kapalıdır; kalıbı kullanan program yetkinlik tanısıyla derlenmez.

ZORUNLU: `dns_dnssec "zorunlu"` bildirilmiş projede çalışma zamanı kaynak yürütülmeden kök `SOA` sorgusunda çözücünün `AD=1` döndüğünü sınar; sınanamazsa süreç `dns_dnssec zorunlu:` iletisiyle çıkış 2 verir.

### 1.2 Yüzey

TANIMLI: `AD için MX kayıtlarını sorgulamayı dene`, `AD için TXT kayıtlarını sorgulamayı dene`, `AD için TLSA kayıtlarını sorgulamayı dene` ve `AD için adres kayıtlarını sorgulamayı dene` ifadeleri `Sonuç<Metin sözlük listesi>` verir; `adres` önce A sonra AAAA sorar. Her satır `ad`, `ttl` ve `doğrulanmış` (`"doğru"`/`"yanlış"`) alanlarını; MX `öncelik` ve `hedef`, TXT `metin`, adres `aile` (`ipv4`/`ipv6`) ve kanonik `adres`, TLSA `kullanım`, `seçici`, `eşleme` ve küçük harf onaltılık `veri` alanlarını taşır.

TANIMLI: Satırlar kayıt verisine göre deterministik sıralı ve tekilleştirilmiştir (MX önce öncelik sonra hedef); TXT parçaları sırasıyla birleştirilir, UTF-8 olmayan bayt U+FFFD olur; adlar küçük harfli ve sondaki noktasızdır, kök (null MX hedefi) `.` yazılır. CNAME zinciri en çok 8 adım izlenir ve satır `ad`ı zincir sonundaki kanonik addır.

TANIMLI: `doğrulanmış` yalnız `dns_dnssec "zorunlu"` kipinde çözücünün AD bitini taşır; `"bilgi"` kipinde her zaman `"yanlış"`tır.

### 1.3 Sonuç ve hata

TANIMLI: Yetkili "alan yok" (NXDOMAIN) ve "kayıt yok" (NODATA) cevabı başarılı boş listedir.

ZORUNLU: Başarısız Sonuç yalnız geçici hatadır ve C033 kodlu Hata taşır; Hata verisinin `tür` değeri `zaman_aşımı`, `sunucu_hatası`, `ret`, `ağ`, `boyut_aşımı`, `bütçe`, `geçersiz_ad` ya da `bozuk_yanıt`tır.

YASAK: SERVFAIL (DNSSEC doğrulaması başarısız yanıt dahil), REFUSED, zaman aşımı ya da bozuk yanıt hiçbir yolda boş liste veya başarılı Sonuç olamaz.

ZORUNLU: Sorgu adı ASCII harf, rakam, `-` ve `_` etiketlerinden oluşur (etiket 1–63 oktet, `-` ile başlayıp bitemez, toplam ≤ 253 oktet, sondaki nokta isteğe bağlı); aksi `geçersiz_ad`dır ve hiçbir paket gönderilmez. U-label (ASCII dışı) ad reddedilir.

### 1.4 Tel biçimi ve sınırlar

TANIMLI: Sorgu tek sorulu, RD=1, AD=1, EDNS0 OPT (1232 bayt UDP tamponu, DO=0) taşır; her sorgu yeni rastgele 16 bit kimlik ve işletim sisteminin rastgele geçici kapısıyla gönderilir.

ZORUNLU: Kimliği, QR biti, soru adı (büyük/küçük harf duyarsız), türü veya sınıfı sorguyla eşleşmeyen paket atılır ve yanıt beklemesi son tarihe kadar sürer; eşleşen ama gövdesi bozuk ya da sınırı aşan paket `bozuk_yanıt`/`boyut_aşımı` hatasıdır.

ZORUNLU: `TC=1` UDP yanıtında aynı sorgu TCP ile (en çok 65.535 bayt) yinelenir; kesik yanıt hiçbir zaman tam yanıt olarak kullanılmaz.

ZORUNLU: Yanıt ayrıştırıcısı cevap bölümünde 64, dört bölümde toplam 128 kayıttan fazlasını, bilinmeyen kayıt türünü, sondaki fazla baytı, rdata uzunluğuyla tutarsız kaydı, 4 KiB'ı aşan TXT ya da TLSA verisini, ileriye ya da kendi segmentine gösteren ya da ad başına 16'dan çok izlenen sıkıştırma işaretçisini ve 255 baytı aşan tel adını reddeder.

TANIMLI: Sorgu başına süre 5 saniyedir; etkin son tarihin kalanı daha kısaysa kalan kullanılır. Sorgudan önce ve sonra son tarih denetlenir; dolmuşsa `yetişmezse` kolu çalışır.

TANIMLI: Çalışma zamanı DNS önbelleği tutmaz; önbellek ve zehirleme direnci yerel çözücünündür.

TANIMLI: DNS sorgusu IO izine `dns_sorgula` olayı olarak argüman `[ad, tür]` ve yapılandırılmış sonuçla girer; replay ağa gitmez, argümanı farklı sorgu izle eşleşmez. Hermetik adaptörün sahte çözücüsünde kayıtlı olmayan ad "alan yok"tur.

AÇIK: Zee yüzeyinde NXDOMAIN/NODATA ayrımı, olumsuz yanıtın doğrulanmışlığı ve CNAME kanonik adının ayrı alanı (RFC-0044 B4); `dil dene` için statik bölge dosyası; PTR, SRV ve SVCB türleri; U-label alanlar; süreç ve oturum başına DNS bütçesi (B2).

## 2. Kurumsal alan (B1b)

ZORUNLU: `kamu_son_ekleri` (proje köküne göreli dosya yolu) ve `kamu_son_ekleri_özeti` (64 küçük harf onaltılık SHA-256) yalnız `dns` bildirimiyle ve birlikte geçerlidir; biri ötekisiz, `dns`siz kullanım, mutlak yol, `..` ya da ters bölü içeren yol ve biçimsiz özet P015'tir.

ZORUNLU: Çalışma zamanı kaynak yürütülmeden dosyayı bir kez okur; okunamayan, SHA-256'sı bildirilen özetle tutmayan ya da ayrıştırılamayan liste (kural yok, bozuk ASCII kural, 4 MiB ya da 100 000 kural üstü) süreci `kamu_son_ekleri "…":` iletisiyle çıkış 2 ile durdurur.

TANIMLI: `AD alanının kurumsal alanı` Metin verir: publicsuffix.org "en uzun eşleşen kural" algoritmasıyla (istisna `!` kazanır, joker `*.` bir etiket tüketir, eşleşme yoksa varsayılan `*`) bulunan kamu son eki artı bir etiket, küçük harfli ve sonda noktasız. ICANN ve PRIVATE bölümlerinin ikisi de kullanılır.

TANIMLI: Ad geçersizse (boş etiket, baştaki nokta, 63 oktet üstü etiket, 253 oktet üstü ad, ASCII LDH/alt çizgi dışı karakter, U-label) ya da kamu son ekinin kendisiyse sonuç boş Metindir; ASCII dışı kural satırları yok sayılır.

TANIMLI: İfade `dns` yetkinliği ister; proje `kamu_son_ekleri` bildirmemişse yürütme C034 ile durur. Sonuç IO izine `kurumsal_alan` olayı (`[ad]` → metin) olarak girer; replay listeyi gerektirmez.

## 3. DKIM imza doğrulama ilkeli (B2b)

TANIMLI: `B dkim imzasını VERİ verisi için P anahtarıyla A olarak doğrulamayı dene` (`kriptografi`) `Sonuç<Metin sözlük>` verir; başarı `doğrulandı` (`"doğru"`/`"yanlış"`), `algoritma` ve `anahtar_biti` alanlarını taşır. İmzanın tutmaması başarılı Sonuç'tur (`doğrulandı` `"yanlış"`).

ZORUNLU: Algoritma yalnız `rsa-sha256` (RSASSA-PKCS1-v1_5, `p=` SPKI ya da PKCS#1, 1024–4096 bit) ve `ed25519-sha256` (RFC 8463: imza VERİ'nin SHA-256 özeti üzerinde, `p=` ham 32 bayt) olabilir; `rsa-sha1` (`algoritma_reddedildi`), bilinmeyen algoritma, boş `p=` (`anahtar_iptal`), çözülemeyen anahtar (`anahtar_bozuk`; RSA'da modül ve açık üs pozitif, en kısa DER INTEGER, modül tek, üs tek ve 3 ≤ e ≤ 2^33 − 1 olmalıdır — doğrulayıcının çözemediği anahtar imzayı sessizce `yanlış` yapmaz), 1024 bitten kısa ve 4096 bitten uzun RSA anahtarı ve base64 olmayan ya da boyu anahtara uymayan imza C034 `tür` verili rettir ve kriptografik işlemden önce elenir.

TANIMLI: VERİ'deki her LF imza girdisinde CRLF olur; VERİ CR içerirse C034 `veri_bozuk` rettir. Zee metninde CR yazılamaz ve SMTP alıcısı çıplak CR/LF'yi reddeder (spec/12 §9), bu yüzden dönüşüm kayıpsızdır. Kanonikleştirme (başlık seçimi, relaxed/simple kuralları, `b=` boşaltma) Zee kodudur. `B` ve `P` içindeki boşluk ve satır sonları atlanır.

TANIMLI: `X gövdesinin dkim gövde özeti` (`kriptografi`) Zee'nin kanonikleştirdiği gövdenin aynı LF→CRLF kuralıyla elde edilen baytlarının SHA-256'sını base64 Metin olarak verir (`bh=` biçimi; boş simple gövde `"\n"`, boş relaxed gövde `""`). Gövde CR içerirse yürütme C034 ile durur. İmza tarafı aynı özeti ve aynı imza girdisi kuralını kullanır.

TANIMLI: Gövde özetinin C034 duruşu yalnız o işi sonlandırır: bağlantı görevinde yalnız o bağlantı kapanır ve tanı günlüğe yazılır; sunucu, dinleyici ve diğer bağlantı görevleri etkilenmez.

ZORUNLU: Ürün kodu CR içeren girdiyi (ör. dosyadan okunmuş ileti) kanonikleştirmeden önce reddeder; gövde özeti ve doğrulama ilkeli CR'yi yalnız son savunma olarak yakalar (özette duruş, doğrulamada `veri_bozuk` ret Sonucu).

AÇIK: Başlık ayrıştırıcısı, `Authentication-Results`, SPF, DMARC, ARC (RFC-0044 B2a/B2c–B2e); `dil dene` için kamu son ek listesi; `psl` fuzz hedefi ve `dkim_anahtari` fuzz hedefinin RC fuzz kapısına alınması; IDN kuralları ve DMARCbis ağaç yürüyüşü.

## 4. DKIM imza ilkeli (B3)

ZORUNLU: `dkim_anahtarları` her öğesi `<alan> <seçici> <algoritma> <credential>` olan bir Metin listesidir ve yalnız `kriptografi` yetkinliğiyle geçerlidir. Alan ve seçici küçük harfli DNS adıdır. Algoritma `rsa-sha256` ya da `ed25519-sha256` olur. Credential K-339 güvenli adıdır. En çok 16 öğe yazılır. Aynı alan+seçici ya da aynı credential iki kez, projede başka bir gizli değer için de bildirilmiş credential adı, `rsa-sha1` ve biçimsiz öğe P015'tir.

ZORUNLU: Çalışma zamanı kaynak yürütülmeden her anahtarı K-339 sözleşmesiyle (önce credential dizini, yoksa ortam; ikisinde birden varsa ret) okur. Anahtar tek blok şifresiz PKCS#8 PEM olmalı ve bildirilen algoritmaya uymalıdır; RSA 2048–4096 bittir. Okunamayan, çözülemeyen ya da sınır dışı anahtar süreci çıkış 2 ile durdurur. Tanı alanı, seçiciyi ve sabit nedeni anar.

YASAK: Özel anahtar baytı ya da PEM metni Zee değeri, IO izi, tanı ya da günlük olamaz.

TANIMLI: `VERİ verisi için AD özel anahtarıyla A olarak imzalamayı dene` (`kriptografi`) `Sonuç<Metin>` verir. Başarıda değer imzanın base64 yazımıdır (`b=` değeri, satır kırma yok). AD, `dkim_anahtarları` içindeki credential adıdır. İmza girdisi §3'teki LF→CRLF kuralıyla kurulur. `rsa-sha256` RSASSA-PKCS1-v1_5 ile, `ed25519-sha256` RFC 8463'e göre SHA-256(girdi) üzerinde imzalanır.

ZORUNLU: İmza atılmadan önce VERİ RFC 6376 §3.7 imza girdisi olarak denetlenir (imza kâhini önlemi):
- En çok 256 KiB, CR yok (`veri_bozuk`), satır sonuyla bitmez.
- Başlık alanlarından oluşur ve son alan `dkim-signature`'dır.
- Etiketler tekildir. `v=1` vardır, `bh=` doludur, `h=` `from` içerir, `l=` yoktur, `b=` boştur ve son etikettir.
- Tam olarak bir `from` alanı vardır ve adresinin alanı `d=` ya da onun alt alanıdır.
- `i=` varsa onun alanı da `d=` altındadır.
- `a=` A ile aynıdır.

Bunlardan biri tutmazsa C034 `geçersiz` döner. `d=`, `s=` ya da `a=` AD'nin bildirimine uymazsa C034 `anahtar_uyuşmuyor`, AD bildirilmemişse ya da anahtar yüklenmemişse (ör. `dil dene`) C034 `anahtar_bilinmiyor` döner. `rsa-sha1` ve bilinmeyen algoritma §3'teki `tür` değerleriyle reddedilir.

TANIMLI: `YOL dosyasının BAŞ baytından KİP kipinde dkim gövde özetini almayı dene` (`kriptografi` + dosyayı okuma; `veri:` yolunda `veri-okuma`) `Sonuç<Metin>` verir. Değer, dosyanın BAŞ baytından sonuna kadarki gövdesinin KİP kanonikleştirmesiyle (`simple`: RFC 6376 §3.4.3; `relaxed`: §3.4.4) elde edilen CRLF baytlarının SHA-256'sının base64 yazımıdır. Satır sonu LF ya da CRLF olabilir; ikisi aynı özeti verir. Simple kipte LF'li kanonik gövde için değer `X gövdesinin dkim gövde özeti` ile aynıdır. Dosya akışla okunur, Metin oluşmaz. Gövde kanonikleştirmesi bu ilkede host'tadır; başlık kanonikleştirmesi Zee kodudur.

ZORUNLU: KİP `simple` ya da `relaxed` dışındaysa veya BAŞ negatifse ya da dosya boyunu aşıyorsa C034 `geçersiz` döner. Gövdede LF'siz CR varsa C034 `veri_bozuk`, işlenen gövde 32 MiB'ı aşarsa C034 `sınır`, dosya okunamaz ya da izin dışıysa C034 `dosya` ret Sonucu döner. Sonuç IO izine `dosya_dkim_govde_ozeti` olayı (`[yol, baş, kip]` → metin) olarak girer.

TANIMLI: İlke gövdenin tamamını özetler; `l=` desteklenmez. `l=` taşıyan gelen imzayı ürün kodu reddeder; imza tarafında `l=` yasaktır.

TANIMLI: İmza IO izine `dkim_imzala` olayı olarak argüman `[AD, algoritma, VERİ'nin SHA-256'sı (onaltılık)]` ve sonuç `b=` ya da `tür` ile girer. Replay anahtar gerektirmez.

ZORUNLU: Proje `dkim_anahtarları` ile birlikte `dns` yetkinliği ve çözücü bildiriyorsa çalışma zamanı kaynak yürütülmeden her anahtar için `<seçici>._domainkey.<alan>` TXT kaydını sorar. RFC 6376 §3.6.1 anahtar kaydı olarak okunan kayıtlardan biri (`v=` varsa ilk ve `DKIM1`; `k=` varsayılanı `rsa` ve anahtarın algoritmasına uymalı) özel anahtarın açık parçasını `p=` olarak taşımalıdır. Eşleşen kayıt yoksa ve iptal edilmiş (boş `p=`) ya da başka anahtarlı bir kayıt varsa yalnız o anahtar devre dışı kalır: süreç ve öteki anahtarlar çalışır, o anahtarla imza denemesi C034 `anahtar_iptal` ya da `anahtar_dns_uyuşmuyor` ret Sonucu döner. Açılışta stderr'e alanı, seçiciyi, kayıt adını ve nedeni anan `HATA:` satırı yazılır; anahtar baytı yazılmaz.

TANIMLI: Adda DKIM anahtar kaydı yoksa ya da DNS sorgusu geçici hata verirse stderr'e `UYARI:` yazılır ve anahtar etkin kalır. `dns` bildirimi olmayan projede yayımlanmış anahtar denetimi yapılmaz.

AÇIK: `dil dkim-denetle` komutu (K-415, backlog adayı) ve gerçek alıcıda `dkim=pass` kanıtı (yöntem K-416: [docs/dkim-alici-kaniti.md](../docs/dkim-alici-kaniti.md); ilk kayıt satırına kadar açık). (`dkim_imza_girdisi` fuzz hedefi 26 Kasım 2026'dan beri RC fuzz kapısındadır.)
