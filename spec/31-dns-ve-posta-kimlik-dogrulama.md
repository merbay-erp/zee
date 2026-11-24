# 31 — DNS sorgusu ve posta kimlik doğrulama

Normatif kaynak: [RFC-0044](../rfcs/0044-dns-posta-kimlik-dogrulama-ve-teslim-guvenligi.md)
Aşama B1 (B1a), [ADR-169](../adr/169-dns-stub-cozucu-ve-tel-bicimi-siniri.md).
Durum: §1 **TANIMLI** (K-390). Gelen posta doğrulaması (B2) ayrı bölüm
olarak eklenir; giden DKIM imzası (B3) ve teslim güvenliği (B4) taslaktır.

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

ZORUNLU: Başarısız Sonuç yalnız geçici hatadır ve C031 kodlu Hata taşır; Hata verisinin `tür` değeri `zaman_aşımı`, `sunucu_hatası`, `ret`, `ağ`, `boyut_aşımı`, `bütçe`, `geçersiz_ad` ya da `bozuk_yanıt`tır.

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

AÇIK: Zee yüzeyinde NXDOMAIN/NODATA ayrımı, olumsuz yanıtın doğrulanmışlığı ve CNAME kanonik adının ayrı alanı (RFC-0044 B4); `dil dene` için statik bölge dosyası; PTR/SRV/SVCB türleri; U-label alanlar; süreç ve oturum başına DNS bütçesi (B2).
