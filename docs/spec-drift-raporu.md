# Spec maddesi ↔ test kanıtı drift raporu

<!-- `cd compiler && cargo run --locked --bin spec_drift -- --rapor-yaz` üretir. Elle değiştirme. -->

Bu rapor K-171/ADR-065 kapısının güncel görünümüdür. Madde; `spec/` altında ZORUNLU/ZORUNDA/YASAK/TANIMLI/AÇIK işaretçisi taşıyan paragraf, liste öğesi ya da başlıktır. Kimlik dosya adı + sadeleştirilmiş metnin parmak izidir; metin değişince kimlik değişir ve kanıt yeniden incelenir.

| Bölüm | Madde | Kanıtlı | Kısmi | Açık |
|---|---:|---:|---:|---:|
| `spec/01-sozcukleme.md` | 8 | 8 | 0 | 0 |
| `spec/02-dizim.md` | 7 | 7 | 0 | 0 |
| `spec/03-adlar-ve-kapsam.md` | 12 | 11 | 0 | 1 |
| `spec/04-turler.md` | 15 | 15 | 0 | 0 |
| `spec/05-degerlendirme.md` | 9 | 9 | 0 | 0 |
| `spec/06-hata-modeli.md` | 7 | 7 | 0 | 0 |
| `spec/07-birimler.md` | 23 | 20 | 1 | 2 |
| `spec/08-kalici-dosya.md` | 9 | 8 | 1 | 0 |
| `spec/09-son-tarih-ve-iptal.md` | 1 | 1 | 0 | 0 |
| `spec/10-disari-acik-islem-sozlesmesi.md` | 4 | 4 | 0 | 0 |
| `spec/11-uygulama-eylemleri-ve-web-adaptoru.md` | 26 | 26 | 0 | 0 |
| `spec/12-web-guvenlik-profili.md` | 87 | 80 | 0 | 7 |
| `spec/13-surumlu-morfoloji-profili.md` | 9 | 8 | 0 | 1 |
| `spec/14-yapilandirilmis-eszamanlilik.md` | 2 | 2 | 0 | 0 |
| `spec/15-yapilandirilmis-hata-degeri.md` | 3 | 3 | 0 | 0 |
| `spec/16-keyfi-hassasiyetli-ondalik.md` | 2 | 2 | 0 | 0 |
| `spec/17-deger-semantigi-ve-gezme.md` | 11 | 11 | 0 | 0 |
| `spec/18-paket-yayini.md` | 14 | 13 | 1 | 0 |
| `spec/19-registry-metadata-guveni.md` | 10 | 10 | 0 | 0 |
| `spec/20-ifade-grameri.md` | 9 | 9 | 0 | 0 |
| `spec/21-deterministik-io-izi.md` | 6 | 6 | 0 | 0 |
| `spec/22-deterministik-io-profili.md` | 5 | 4 | 1 | 0 |
| `spec/23-yetkinlik-ve-outbound-guvenligi.md` | 53 | 50 | 1 | 2 |
| `spec/24-kaynak-guvenlik-profili.md` | 10 | 10 | 0 | 0 |
| `spec/25-postgresql-veri-erisimi.md` | 12 | 11 | 0 | 1 |
| `spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md` | 20 | 19 | 0 | 1 |
| `spec/27-uyumluluk-ve-surumleme.md` | 13 | 11 | 1 | 1 |
| `spec/28-gorunum-blogu.md` | 2 | 2 | 0 | 0 |
| `spec/29-test-kapsami.md` | 5 | 5 | 0 | 0 |
| `spec/30-editor-analizi-ve-lint.md` | 13 | 12 | 0 | 1 |
| `spec/31-dns-ve-posta-kimlik-dogrulama.md` | 42 | 39 | 0 | 3 |
| **Toplam** | **449** | **423** | **6** | **20** |

## Kısmi kanıtlı maddeler

- `spec/07-birimler.md#533cdfd1a42d5de3` — TANIMLI: Çalışan programın göreli dosya IO kökü giriş kaynağının klasörü… — K-382: çalıştır <proje>/<giriş dosyası> ve başka çağıran klasörde veri giriş klasöründe; proje kökü dokunulmaz. Worker/işçi grubu/systemd soketi aynı KaynakGirdisi::gercek_io yolundan geçer ama alt klasör girişli gerçek worker yükleme testi yok.
- `spec/08-kalici-dosya.md#532a9e61aa1fe70f` — - Dayanıklılık/kilit primitive'leri bulunmayan platform işlemi reddetmek… — Primitive'siz platform reddi yalnız desteklenen üç platformda derlenen koda bağlı; böyle bir platform CI matrisinde yok.
- `spec/18-paket-yayini.md#2af407978947e1d4` — Üretici dört dosyayı yazmadan önce bütün kümeyi tüketici doğrulayıcıyla … — Yazmadan önce tam doğrulama ve yarım küme reddi kanıtlı; süreç çökmesi ortasında yarım küme senaryosu enjekte edilmiyor (söz de verilmiyor).
- `spec/22-deterministik-io-profili.md#d68db0760744fd45` — Profilin bu gözlemlerinden birini kıran değişiklik yeni profil kimliği, … — Snapshot gözlem değişikliğini yakalar; yeni profil kimliği/RFC zorunluluğu süreç kuralıdır, makinece yalnız snapshot farkıyla görünür.
- `spec/23-yetkinlik-ve-outbound-guvenligi.md#591e270023c9b87c` — ZORUNLU: Bağlanma `min(Zee son tarihi, 30 sn)` ile sınırlıdır. Görev baş… — K-397: görev başına tek bağlantı, dinleyicide ret, eşzamanlı görev kapatma kanıtlı; 30 sn bağlanma tavanı ve 16 bağlantı sınırı kod sabitidir (ag_satiri), ayrı zaman/sınır testi yok.
- `spec/27-uyumluluk-ve-surumleme.md#15d606bdb76e4dc1` — - Kaldırılan biçim sessizce kabul edilmez: derleyici tanı üretir ve öner… — Henüz kaldırılmış bir kullanıcı yüzeyi yok; tanı+öneri kanıtı ilk DEP kaldırmasında regresyon korpusuna eklenecek.

## Açık maddeler

- `spec/03-adlar-ve-kapsam.md#548b78fc716d5ccb` — AÇIK (K-398): iki adımlı geri çevrim (`kaybı → kayıp`, `kaydı → kayıt`),… — K-398: sözlüksüz kural iki adımlı geri çevrimi, l/m sonlu ünlü düşmesini (-lı sıfatlarıyla çakışır) ve iyelik+belirtme çakışmasını ayırt edemez; sözlük ya da zee-tr-2 kararı ister.
- `spec/07-birimler.md#5a223c7685111c87` — AÇIK: Göreli IO kökünü proje klasörüne taşıyan bir bildirim alanı yoktur… — K-382/K-394: göreli IO kökü taşınmaz; projeler arası paylaşılan veri K-394 veri_kökü (mutlak, işletimce verilen kök) ile açıldı.
- `spec/07-birimler.md#ea02c119dae2926d` — AÇIK: Veri kökünde dizin listeleme, görsel küçültme, bağlantı blok dosya… — K-394: listeleme ve görsel küçültme açık ret iletisi verir (veri_kokunde_yazma_tasima_silme_ve_okuma_ailesi_calisir listeleme reddini sınar); Windows/wasm yok.
- `spec/12-web-guvenlik-profili.md#f0003f30d3695cf6` — AÇIK: Parola ve TOTP doğrulaması aşımda rotaya `yanlış` döndürüp rotayı … — K-368 kapsamı dışı: aşımı Zee koduna ayrı değerle göstermek ya da rotayı kesmek GirdiCikti bool sözleşmesini ve IO izi şemasını değiştirir; ZM-F021 hesap kilidi sayacı etkisi bu madde altında izlenir.
- `spec/12-web-guvenlik-profili.md#2e5226f743e35a39` — AÇIK (K-400): Host'u `127.0.0.1` ya da `localhost` olarak yeniden yazan … — K-400: Host yeniden yazan yerel proxy/tünel çalışma zamanınca ayırt edilemez; operatör kuralı ve filo süreç denetimi (25.11.2026: hiçbir üretim sürecinde --deneysel-web yok) ile kapsanır.
- `spec/12-web-guvenlik-profili.md#5d930da7687a2f1e` — AÇIK: Windows'ta paylaşılan dinleyici devri yoktur; `--web-isci-sayisi` … — Windows paylaşılan dinleyici devri, dengeli dağılım, ağ dosya sistemi ve çok hostlu birincillik açık (K-340 `zee.ayar` alanını, K-341 tek makinede elle kurulumda tek birincili bu maddeden çıkardı); Windows'taki çıkış 2 reddi web_cok_surec_testi::isci_grubu_gecersiz_birlesimleri_ve_platform_siniri_reddedilir cfg(windows) vakasıyla Windows CI'da koşar.
- `spec/12-web-guvenlik-profili.md#c0c4aaba019b2440` — AÇIK: Adlı dinleyicilerden bağlantı kabulü yalnız §9'daki bağlantı bloğu… — K-381: çok işçi grubuna adla aktarım ve HTTP dinleyicisinin adlı devirden alınması açık; bloksuz dinleyici K-376 gibi tutulur.
- `spec/12-web-guvenlik-profili.md#1cbfffe0b684d7eb` — AÇIK: Bağlantı görevi içindeki PostgreSQL, dosya ve HTTP çağrıları ile b… — Katman C, bağlantılar arası uyandırma, Unix dışı ret, RSS ölçümü ve ham dosyanın kısmi aktarımı açık (base64 kodlu kısmi getirme K-396 A2d). K-388: birimler arası canlı değer birikimi ve birim içi atılan değer iadesi açık.
- `spec/12-web-guvenlik-profili.md#d0c68c44c7c98c4b` — AÇIK: `Vary`, ETag/koşullu istek, `stale-while-revalidate` ve önbellek t… — K-335 kapsamı dışı; Vary/ETag/purge ve sorgu anahtarı ön kapının sorumluluğundadır.
- `spec/12-web-guvenlik-profili.md#a969792eb3d50575` — AÇIK: Kaynak değişmezindeki bir betiği gövdeye yansıtan istemci o betiği… — K-379 kapsamı dışı: özet CSP'nin genel sınırı (yalnız geliştirici betikleri); öznitelikli script özetlenmez; durum yanıtı satır içi betik izni taşımaz.
- `spec/13-surumlu-morfoloji-profili.md#8a414db8b5ca5b82` — AÇIK: `zee-tr-1` `â`, `î`, `û` harflerini ünlü saymaz. Bu harfleri taşıy… — K-367: düzeltme uyumluluk kaydını değiştirir (53.248 üretimin 10.306'sı, 2.865 çözüm kümesi); zee-tr-1 içinde yapılamaz, zee-tr-2 adayı (RFC + yeni kayıt + kilit geçişi + edition). Çözücü doğru yazımı kabul eder.
- `spec/23-yetkinlik-ve-outbound-guvenligi.md#15b3c0c87713d1ae` — AÇIK: Gerçek systemd servis kurulumu ile canlı kabul, çalışan süreçte ro… — K-339 kapsamı dışı; canlı systemd servis kabulü ve rotasyon ürün/işletim tarafında.
- `spec/23-yetkinlik-ve-outbound-guvenligi.md#6d2ba776755d2562` — AÇIK: Host adı verilirse çözümleme işletim sisteminin engelleyen çözücüs… — K-397: MX → IP yolu ürün kodunda B1a çözücüsüyle (dogfood urun-h/giden-teslim); OS çözücü yolu, TLSA eşlemesi, eşleme türü 0/2 ve gerçek internet teslim kanıtı bu bölümün sözü değil
- `spec/25-postgresql-veri-erisimi.md#f94f82cb0245cf0a` — Managed-provider sertifika rotasyonu, birden çok worker'ın toplam bütçe … — K-169/K-173 release tatbikatı ve çok-worker toplam bütçe provası bekliyor; async driver, PostgreSQL dışı DB ve ORM vaat edilmiyor.
- `spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md#bfe5b0f763cf9612` — AÇIK (K-389): UTF-8 bayt sayısı, metin↔base64/onaltılık, kod noktası ve … — RFC-0043 Dilim 2 (ZM-F023): bayt sayısı, metin↔base64/onaltılık, kod noktası, base64url; çağrılar arası yarım dörtlü durumu bilinçli olarak tutulmaz (K-173 uzun ömürlü durum yok).
- `spec/27-uyumluluk-ve-surumleme.md#285d87941dc0c3ba` — 1.0 sonrasında dondurulmuş yüzeylerde kırıcı değişiklik yalnız yeni ana … — Edition alanının `proje.dil` sözdizimi ayrı RFC ister; 1.0 öncesi bağlayıcı yalnız RFC-0028 çerçevesidir.
- `spec/30-editor-analizi-ve-lint.md#38480ce75d9537ab` — Kapsam ve performans (AÇIK) — K-287: artımlı HIR ve genel gerçek ürün gecikmesi açık; tarihsel S3 66e4149e53451111d153f47283c219079a8d1cab kaynağındaki kabul korunur. S4 4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2 kaynağında ayrı 1800sn RSS ve lint açık/kapalı 1/41 belge stdio gözlemi vardır; 41 belge lint açık p95 223,698ms ile 200ms hedefi açıktır, UI/ürün SLA'sı değildir.
- `spec/31-dns-ve-posta-kimlik-dogrulama.md#f6da609c2bdaca46` — AÇIK: Zee yüzeyinde NXDOMAIN/NODATA ayrımı, olumsuz yanıtın doğrulanmışl… — K-390: NXDOMAIN/NODATA Zee ayrımı ve CNAME adı B4; dil dene bölge dosyası, PTR, SRV ve SVCB, U-label, süreç/oturum bütçesi B2 açık.
- `spec/31-dns-ve-posta-kimlik-dogrulama.md#b62fde1c435ead61` — AÇIK: Başlık ayrıştırıcısı, `Authentication-Results`, SPF, DMARC, ARC (R… — K-390: B2a/B2c–B2e, dil dene listesi, psl/DKIM fuzz hedefleri, IDN ve DMARCbis açık.
- `spec/31-dns-ve-posta-kimlik-dogrulama.md#45c643963a4f22bf` — AÇIK: Açılışta yayımlanmış `p=` ile özel anahtarın eşleşme denetimi, `di… — K-397: DNS p= eşleme, dil dkim-denetle, dkim_imza_girdisi fuzz ve gerçek alıcı dkim=pass açık.

## Sürüklenme

Yok: her normatif madde kayıtlı, her kayıt var olan bir maddeye ve var olan test işlevine bağlı.

Sürüklenme otomatik bir semantik hüküm değildir: kayıtsız madde kanıt ister, bayat kayıt ya silinir ya yeni kimliğe taşınır; seçicinin maddeyi gerçekten kanıtladığı kod incelemesinde değerlendirilir.
