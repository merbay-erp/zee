# K-371 — Dış etki hata eşlemesi denetimi

Tarih: 23 Kasım 2026. Taban: `master` `ef4c8c7` (K-370 merge sonrası).
Kapsam: K-370 web içi oran/kaynak reddini denetledi
([K-370 tablosu](k370-web-oran-500-denetimi.md)); bu belge dış etki tarafını
denetler: SMTP gönderimi, native HTTP istemcisi, PostgreSQL adaptörü ve bunların
DNS/TLS/zaman aşımı/son tarih hataları. Soru her satırda aynıdır: dış hata Zee
kodunda `Sonuç` (`… dene`, `başarısızsa`) ile yakalanabilir mi, yoksa
yakalanamaz C-tanısı olup web'de 500, CLI'da çıkış 1 mi olur; spec ne der;
gerçek davranış uyuşuyor mu.

Normatif kaynaklar: spec/06 (hata sınıfları), spec/09 (son tarih ve iptal
noktaları), spec/11 §5 (istek-yerel hata sınırı: Ç001 504; C013/C021/C027 ve
C023/C024 503; diğer C/D 500), spec/15 (Hata değeri), spec/23 (outbound, K-220
HTTP, K-232/K-236/K-237/K-252 SMTP), spec/25 (PostgreSQL), ADR-017/091/096/099/
100/110, ADR-060/077.

## 0. Ortak kural ve yöntem

- **Yakalanabilir** = intrinsic başarısız `Sonuç` döndürür; Hata kodu sabittir
  (SMTP `C029`, yeni HTTP `C018`, PostgreSQL `C025`). Hata değeri spec/15'tir;
  tanı değildir, istek-yerel sınıra ulaşmaz. Web rotası onu kendisi 4xx/5xx'e
  ya da kullanıcı metnine çevirebilir.
- **Yakalanamaz** = `Tani`; web'de spec/11 §5 tablosuyla yanıt, CLI'da tanı
  raporu ve çıkış 1. Son tarih iptali `Ç001`'dir: blok içinde `yetişmezse`
  koluna, web isteğinin 30 sn son tarihinde 504'e gider.
- Gerçek ölçümler `--release` native `dil` ile loopback'te alındı: sessiz SMTP
  sunucusu (TCP kabul eder, selam vermez), kapalı port (bağlantı reddi), yerel
  PostgreSQL 16 kümesi (`sslmode=disable`, loopback), `--deneysel-web` sunucu +
  `curl`. Ağır yük üretilmedi.

## 1. SMTP gönderimi

Yüzeyler: `alıcı konu gövde ile ileti göndermeyi dene` (ortam bağlantısı),
`… ile <bağlantı> üzerinden ileti göndermeyi dene`, `… ile ekli ileti
göndermeyi dene` (MIME, K-232). Üçü de `TamSayı Sonucu`; eylem içinde T048.

| Hata | Zee'de | Kod / mesaj | Web (rota yakalamazsa) | CLI | Spec beklentisi | Uyuşma |
|---|---|---|---|---|---|---|
| Bağlantı reddi (kapalı port) | yakalanır | C029, sabit maske | rota karar verir (ölçüm: rota `C029` gövdeli 200) | Sonuç | spec/23 K-236 C029 | ✓ |
| DNS çözüm hatası / yasak IP | yakalanır | C029 | aynı | Sonuç | spec/23 origin/IP, C029 | ✓ (aynı `gonder_ic` hata yolu) |
| TLS (SMTPS/STARTTLS sertifika, downgrade) | yakalanır | C029 | aynı | Sonuç | spec/23 K-232 TLS ZORUNLU | ✓ (aynı yol; eposta testleri) |
| Kimlik (AUTH 535) | yakalanır | C029, sunucu yankısı ve AUTH payload'ı sızmaz | aynı | Sonuç | K-236 | ✓ (regression `smtp-auth-echo-sir`) |
| 4xx/5xx SMTP yanıtı (MAIL/RCPT/DATA) | yakalanır | C029; DATA yanıtı kaybı teslim edilmedi kanıtı değildir | aynı | Sonuç | K-232/K-236, otomatik tekrar yok | ✓ |
| Yanıt bayt sınırı (16 KiB) | yakalanır | C029 | aynı | Sonuç | K-237 | ✓ (regression `smtp-sinirsiz-yanit`) |
| Yavaş yanıt, kendi 20 sn bütçesi | yakalanır | C029, bütçe yenilenmez | aynı | Sonuç | K-252 | ✓ (regression `smtp-yavas-yanit`) |
| **Dış `içinde` / web 30 sn son tarihi — ekli (MIME)** | iptal | Ç001 | 504 | `yetişmezse` | spec/23 K-232 "dış son tarihin kalanıyla ZORUNLU sınırlanır" | ✓ ölçüm: `1 saniye içinde` + sessiz sunucu **1,0 sn** |
| **Dış `içinde` / web 30 sn son tarihi — düz metin** | iptal | Ç001 | 504 | `yetişmezse` | spec/09 iptal noktaları (ağ beklemesi kalan bütçeyle), MIME ile aynı | **✗ düzeltme öncesi** ölçüm: **20,0 sn**; düzeltme sonrası **1,04 sn** |
| Ortam/credential değeri yok ya da bozuk | yakalanır | C029 | aynı | Sonuç | K-339 | ✓ |

**Bulunan hata (düzeltildi).** Düz metin yol native adaptöre sabit
`Duration::from_secs(20)` veriyordu ve çağrıdan sonra son tarihi yeniden
denetlemiyordu; MIME yolu ise K-232'den beri 20 sn tavanı ile dış son tarihin
kalanının küçüğünü kullanıyor ve dönüşte denetliyor. Sonuç: `1 saniye içinde`
bloğundaki düz e-posta yanıt vermeyen sunucuda 20 sn bloklanıyor,
`yetişmezse` ancak 20. saniyede çalışıyordu. Web'de isteğin 30 sn son tarihinin
son 20 saniyesinde başlayan gönderim 504'ü 30 sn yerine en geç 50. saniyeye
itiyordu; site çekirdeğinin onay akışı rotası (`ileti göndermeyi dene` ve
`üzerinden` biçimi) bu yolu kullanır. Düzeltme: `GirdiCikti::eposta_gonder`
yürütme bağlamından `zaman_asimi_ms` alır (iz şeması değişmez), intrinsic MIME
ile ortak `eposta_butcesi` yardımcısını kullanır ve çağrıdan sonra
`son_tarihi_denetle` yapar; native adaptör sır/ortam hazırlığında geçen süreyi
aynı bütçeden düşer. Son tarihsiz çağrıda 20 sn tavanı aynen kalır (ölçüm 20,0 sn,
C029). Regression `security/smtp-son-tarih`.

Düzeltilmeyen gözlem: düz metin yol K-130 süreç-geneli 64 bağlantı iznini
almıyor; MIME ve HTTP alıyor. Yorumlayıcı işçisi bloklayan tek SMTP çağrısı
yürüttüğünden işçi başına eşzamanlı düz SMTP bağlantısı en çok 1'dir; spec/23
izin cümlesi K-232 ekli yolu kapsar ve eski yolun "değişmez" olduğunu söyler.
Ölçülen yanlış davranış yok; açık iş olarak backlog K-371 notunda.

## 2. HTTP istemcisi

Yüzeyler: eski `"<url>" adresinden gelen yanıt` / `adresinden getir` (GET,
K-105/K-127) ve yeni `adres yöntem başlıklar gövde süre ile HTTP isteğini dene`
(GET/POST, K-220, yalnız HTTPS).

| Hata | Eski GET | Yeni `HTTP isteğini dene` | Web | CLI | Spec | Uyuşma |
|---|---|---|---|---|---|---|
| DNS hatası, yasak/karışık IP cevabı | C018 tanısı | C018 Sonuç (sabit maske) | eski 500 / yeni rota karar verir | eski çıkış 1 | spec/23: "ihlali runtime'da C018"; K-220: "ağ/süre/boyut/istek hatası C018 kodlu başarısız Sonuç" | ✓ |
| Bağlanma reddi | C018 tanısı | C018 Sonuç | ölçüm: eski **500** `istek işlenemedi; sunucu hatası` (operatör günlüğü `kod=C018`), yeni rota `C018` gövdeli 200 | ölçüm: eski `HATA C018`, çıkış 1 | spec/11 §5 diğer C → 500 | ✓ |
| TLS doğrulama | C018 tanısı | C018 Sonuç | aynı | aynı | rustls, spec/23 | ✓ |
| 30 sn varsayılan bütçe / `süre` (1..30000 ms) | C018 tanısı | C018 Sonuç; süre dış son tarihin kalanına kırpılır | aynı | aynı | spec/09, ADR-017, K-220 | ✓ |
| Dış son tarih dolar | Ç001 (`yetişmezse`) | Ç001 (dönüşte `son_tarihi_denetle`) | 504 | `yetişmezse` | spec/09 "süre dolmuş ağ yanıtı değere dönüşemez" | ✓ |
| 8 MiB zarf / 64 KiB başlık | C018 tanısı | C018 Sonuç | aynı | aynı | spec/23, ADR-017 | ✓ |
| Yönlendirme (3xx) | izlenmez; 3xx başarı değeridir | izlenmez; `durum` 3xx başarı | rota görür | değer | spec/23 "Redirect izlenmez", K-220 3xx/4xx/5xx başarı taşıması | ✓ |
| `ağ_hedefleri` dışı hedef | sabit URL T054 (derleme); dinamik URL C018 tanısı | T054 / C018 Sonuç | dinamik eski 500 | çıkış 1 | spec/23 compile+runtime | ✓ |
| 64 bağlantı bütçesi dolu | C018 tanısı | C018 Sonuç | aynı | aynı | K-130 | ✓ |
| Yeni kalıpta düz `http://` | — | C018 Sonuç | rota karar verir | Sonuç | K-220 "native HTTPS GET/POST" | ✓ |

Eski GET'in yakalanamaz C018 tanısı web'de 500'dür; spec/11 §5 bunu açıkça
tanımlar ve tanının önerisi yeni `HTTP isteğini dene` kalıbını gösterir
(RFC-0033). Geçici dış hatanın 502/503 sayılması ancak ürün kodunda
`Sonuç` ile yapılabilir; eski yüzeyi 503'e çevirmek spec/11 kararını değiştiren
tasarım işidir, düzeltme değildir. Site çekirdeğinde eski GET kullanılmıyor;
Google yorumları modülü yeni kalıbı 4000 ms süre ve başarısızlıkta boş sonuçla
kullanıyor.

## 3. PostgreSQL adaptörü

Yüzeyler: `"SQL" sorgusunu parametreler ile okumayı dene` (List Sonucu),
`… değiştirmeyi dene` (TamSayı Sonucu, yalnız eylemde), eylem transaction'ı,
request-bound tenant bağlamı, `dil göçür`.

| Hata | Zee'de | Kod | Web | CLI | Spec | Uyuşma |
|---|---|---|---|---|---|---|
| Sunucu kapalı / bağlantı kurulamaz | yakalanır | C025 Sonuç (ölçüm: checkout 2 sn sonra `C025`) | rota karar verir; worker yaşar (ölçüm) | Sonuç | spec/25: "Havuz tükenmesi ve TLS/bağlantı kurma hatası C026" | **belge sapması** (aşağı) |
| Kimlik / TLS / CA / hostname | yakalanır | C025 Sonuç | aynı | Sonuç | aynı cümle | belge sapması |
| Sorgu reddi (sözdizimi, yok tablo, kısıt) | yakalanır | C025; `sqlstate`/`kısıt`/`tablo` verisi (ölçüm: `42P01`) | rota karar verir | Sonuç | spec/25 "Veritabanı reddi C025" | ✓ |
| Kilit zaman aşımı / deadlock (`55P03`, `40P01`) | yakalanır | C025 + `sqlstate` | rota karar verir | Sonuç | aynı | ✓ (aynı SQL reddi yolu) |
| Havuz tükenmesi (4 bağlantı, 2 sn checkout) | yakalanır | C025 | aynı | Sonuç | spec/25 C026 cümlesi | belge sapması; tek iş parçacıklı işçide pratikte oluşmaz |
| Yanıt çerçevesi 32 MiB | yakalanır | C025 `db.response_frame_limit` | aynı | Sonuç | spec/25 TANIMLI | ✓ |
| Eylem BEGIN/COMMIT/ROLLBACK hatası | yakalanamaz | C021 | 503 `işlem tamamlanamadı; otomatik tekrar yok` | çıkış 1 | spec/11 §6, spec/25 | ✓ |
| COMMIT sonucu belirsiz | yakalanamaz | C027 | 503 `işlem sonucu belirsiz…` | çıkış 1 | spec/25 | ✓ (F028 kanıtı) |
| Tenant bağlamı kurulamaz / çözücü reddi | yakalanamaz (rota çalışmaz) | WebReddi | 503 | — | spec/25 "fail-closed 503" | ✓ |
| İstek transaction'ı commit edilemez | yakalanamaz | — | 503 | — | ADR-077 | ✓ |
| Göç planı / uygulama hatası | — | C026 (`dil göçür`) | — | çıkış 1 | spec/25 Migration | ✓ |
| **Sorgu dış `içinde` / web 30 sn son tarihini aşar** | iptal yalnız sonraki cümle sınırında | Ç001 | 504 ama sorgu bitene kadar bekler (ölçüm: `pg_sleep(33)` → **504, 33,0 sn**) | `yetişmezse` geç (ölçüm: `1 saniye içinde` + `pg_sleep(4)` → **4,1 sn**) | spec/09 iptal noktaları; spec/25 sorgu son tarihi söylemez | **açık sınır** → RFC-0042 taslağı; **K-375'te kapandı** (1,00 sn / 504 30,0 sn) |

**Belge sapması (davranış değişmedi).** spec/25 "Havuz tükenmesi ve
TLS/bağlantı kurma hatası C026'dır" diyor; uygulama okuma/değişiklik
intrinsic'i bütün adaptör hatalarını `C025` Hata değerine çevirir, `C026`
yalnız `dil göçür` CLI yolunda üretilir (hata kataloğu C026 satırı ve K-370
notu da böyle). Davranış daha güvenlidir (yakalanabilir; tanı yerine `Sonuç`);
spec cümlesi K-371'de gerçek davranışa göre düzeltildi.

**Açık sınır (düzeltilmedi).** PostgreSQL sorgusu son tarihe bağlanmaz:
sürücü çağrısı sürerken iptal edilmez, adaptöre kalan süre verilmez,
`statement_timeout` ya da iptal isteği gönderilmez. Kilit bekleyen bir sorgu
web işçisini 30 sn son tarihin ötesinde tutar ve 504 ancak sorgu döndükten
sonra gelir. Düzeltmek server-side iptalin açık transaction'ı ve savepoint'i
abort etmesi, COMMIT sırasında iptalin C027 sınıfıyla ilişkisi, otomatik
tekrar yasağı ve iz şeması kararlarını ister; bu bir minimal düzeltme değildir.
[RFC-0042 taslağı](../rfcs/0042-postgresql-sorgu-son-tarihi.md) seçenekleri ve
kabul kanıtını yazar; spec/25'e AÇIK maddesi eklendi.

## 4. Ürün kullanımı

| Ürün | Dış etki yüzeyi | Hata işleyişi | K-371 etkisi |
|---|---|---|---|
| Site çekirdeği, onay akışı rotası | düz `ileti göndermeyi dene` ve `… üzerinden` | `Sonuç`; başarısızlıkta yönlendirme kodu | yavaş SMTP artık isteğin 30 sn son tarihini aşamaz (504 zamanında) |
| Site çekirdeği, e-posta ayarları | ekli ileti (logo), başarısızsa eksiz ikinci deneme | `Sonuç`; ikinci deneme kalan bütçeyle | değişmez (MIME zaten bağlıydı) |
| Site çekirdeği, Google yorumları | `HTTP isteğini dene` 4000 ms | başarısızlıkta boş sonuç | değişmez |
| Site çekirdeği, panel/modüller (286 sorgu satırı) | `sorgusunu … okumayı/değiştirmeyi dene` | `Sonuç` + eylem C021/C027 503 | sorgu son tarihi açık sınırı (RFC-0042) |
| ZeeMail (`urun-h`) | depo içi aynada dış etki yüzeyi yok; ürün SMTP **sunucusu**dur (RFC-0039), Zee'nin SMTP istemcisini kullanmaz; web tarafı ZM-F020/F021 ile parola/TOTP kullanır | — | yok; ürün kaynağı özel depoda, bu denetim depo içi aynayla sınırlıdır |

## 5. Sonuç

- Bütün dış hatalar yeni yüzeylerde yakalanabilir `Sonuç`'tur; yakalanamaz
  dış hata yalnız eski GET (C018 → 500, spec/11 kararı) ve transaction sınırı
  (C021/C027 → 503) ile son tarih (Ç001 → 504) yollarıdır. Yanlış HTTP kodu
  bulunmadı.
- Tek gerçek hata: düz metin SMTP dış son tarihi gözetmiyordu (K-371 düzeltmesi).
- Belge sapması: spec/25 C026 cümlesi (düzeltildi).
- Açık sınır: PostgreSQL sorgusu son tarihe bağlanmıyor (RFC-0042 taslağı,
  spec/25 AÇIK).
