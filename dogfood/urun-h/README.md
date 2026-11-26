# urun-h — e-posta sunucusu dogfood ürünü (K-355)

Ürün H, protokol motorları dahil bütünüyle zee ile yazılması hedeflenen bir
e-posta sunucusudur. Gerçek ürün özel depodadır; bu klasör
[RFC-0039](../../rfcs/0039-ag-sunucusu-yetkinligi.md) ağ sunucusu
yetkinliğinin depo içi kanıt aynasıdır. Köken: [KOKEN.md](KOKEN.md).

| Sürtünme | Kanıt | RFC-0039 katmanı |
|---|---|---|
| TCP dinleyicisi yok, tek bağlantı yüzeyi HTTP rotası (S036; K-381 ile çözüldü) | `korpus/f009-dinleyici-yok.dil` | A |
| Metin kaçışında `\r` yok, çıplak CR denetimi yazılamaz (S040) | `korpus/f004-cr-kacisi-yok.dil` | A (çalışma zamanı değişmezi) |
| Yerleşik büyük harf Türkçe yerel ayarlı (`MAİL`), ASCII katlama yok | `korpus/f001-yerel-ayarli-buyuk-harf.dil`, çözüm `korpus/f001-ascii-katlama-elle.dil` | D adayı |
| Adlı systemd soketi ve `Type=notify` yok; on soket sırasız geliyor (K-376, çözüldü: A1) | `dinleyici-devri/` | A1 (uygulandı) |
| Zee kodu adlı dinleyiciden bağlantı kabul edemiyor, satır okuma istem basıyor (F009/F010; K-381, çözüldü: A2a) | `baglanti-yankisi/`, `korpus/f009-dinleyici-yok.dil` | A2a (uygulandı) |
| Web arayüzü ek indiremiyor, JS'siz dosya yükleyemiyor, PDF/görsel dışı ek gönderemiyor (ZM-F022; K-378, çözüldü: RFC-0041 A, B, C) | `ek-akisi/` | RFC-0041 A, B, C (uygulandı) |
| Webmail e-posta istemcisi autoconfig XML'ini ve Apple yapılandırma profilini kayıtlı türle sunamıyor (ZM-F033; K-385, çözüldü: ADR-163 eki) | `istemci-yapilandirma/` | RFC-0041 A (tür kaydı satırı) |
| Bellekteki takvim metni `text/calendar` ile verilemiyor; GET rotası dosya yazamadığından takvim yayını `text/plain` iniyor (ZM-F048; K-406, çözüldü: ADR-163 eki) | `takvim-yayini/` | RFC-0041 A (tür kaydı, metin yanıtı) |
| Gelen iletinin ekleri çıkarılamıyor: blok dosyası parça parça okunamıyor, base64/QP gövde bayt olarak yazılamıyor, bellekte SHA-256 yok (ZM-F034/ZM-F024; K-389, çözüldü: RFC-0043 Dilim 1) | `ek-cikarma/`, `korpus/zm-r005-bellekte-sha256.dil` | D (RFC-0043 Dilim 1) |
| Ekleri yazan hizmetler ile sunan web ayrı proje kökleri; proje dosya sınırı paylaşılan depoyu engelliyor (ZM-F038; K-394, çözüldü: RFC-0045) | `paylasilan-depo/` | G (veri kökü yazımı) |
| Dosyayı bağlantıya RFC 2045 base64 satırlarıyla akıtan yüzey yok; 57 baytlık okuma döngüsü 16 MB ekte 29,6 sn + C024 (ZM-F040; K-396, çözüldü: RFC-0039 A2d) | `imap-govde-base64/` | G (yerleşik yüzey eksik) |
| `uzunluğu` harf sayar, UTF-8 oktet saymaz; IMAP literal/AUTH PLAIN/RFC 2047 bayt ve kod noktası işleri U+0000–04FF tablosuyla, tablo dışında U+FFFD ya da `?` (ZM-F023; K-402, çözüldü: RFC-0043 Dilim 2) | `korpus/zm-r004-uzunluk-oktet-saymaz.dil`, `korpus/zm-f023-*.dil` | D (RFC-0043 Dilim 2) |
| Türkçe-yoğun uzun ileti urlencoded formda 64 KiB'ı aşıp rota çalışmadan 413 alıyor (ZM-F037; K-391, çözüldü: ADR-168 Dilim 1); rota bazında 256 KiB form (ADR-168 Dilim 2) | `ileti-boyutu/` | RFC-0041 B (multipart metin alanı) |
| İleti gövdesi tek SQL parametresinin 64 KiB sınırı yüzünden 20 000 harflik parçalara bölünüyor (ZM-F036; K-391, çözüldü: ADR-168 Dilim 2) | `ileti-govdesi-sql/` | spec/25 kaynak sınırı |
| DNS sorgusu (MX/TXT/TLSA) yok; dış teslim, SPF/DKIM/DMARC ve DANE yanıtları dışarıdan verilen boş sözlükten geliyor (ZM-R006, S015; K-390, çözüldü: RFC-0044 B1a) | `korpus/zm-r006-mx-sorgusu-yok.dil` | RFC-0044 B1a (uygulandı) |
| Giden ileti DKIM imzasız çıkıyor; imza ilkeli yok (ZM-F042, S015; K-397 Dilim 2, çözüldü: RFC-0044 B3, anahtar `dkim_anahtarları` + credential) | `korpus/zm-f042-dkim-imza-yok.dil` | RFC-0044 B3 (uygulandı, ADR-174) |
| 256 KiB üstü gövdenin DKIM `bh=`'si hesaplanamıyor; 400 KB ekli imzalı ileti İstenmeyen'e düşüyor, ekli giden ileti imzalanamıyor (ZM-F046, S015; K-397 Dilim 2, çözüldü: dosyadan akışlı simple/relaxed gövde özeti) | `korpus/zm-f046-dkim-govde-dosyadan.dil` | RFC-0044 B3 (uygulandı, ADR-174 §8) |
| Ekli giden iletinin gövdesi tek dosyada kurulamıyor (ek parçaları yalnız SMTP'ye akarken oluşuyor); dosyadan `bh=` alınamadığından ekli dış ileti imzasız gidiyor (ZM-F049, kalıp yok; K-408, çözüldü: ADR-172 eki, satırlı base64'ü dosyaya akışla ekleme) | `korpus/zm-f049-ekli-govde-dosyada.dil` | G (yerleşik yüzey eksik) |
| Kurumsal alan (PSL) ve DKIM imza doğrulaması yok; DMARC hizalaması ve gelen imza denetimi yapılamıyor (ZM-F043, ZM-F024/F025, S015; K-390, çözüldü: RFC-0044 B1b/B2b; CRLF'li gövde özeti açık) | `korpus/zm-f043-dkim-dogrulama-yok.dil` | RFC-0044 B1b/B2b (uygulandı) |

Sürtünmelerin K-164 kataloğuyla karşılaştırması `kararlar/gunluk.md` K-355
kaydındadır. Korpus vakaları `dogfood/korpus-v1.tsv` içinde exact
beklentiyle kayıtlıdır.
