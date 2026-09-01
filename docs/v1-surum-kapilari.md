# zee v1.0 sürüm kapıları

Bu liste bir dilek listesi değildir: `1.0.0` etiketi bu kapılar kapatılmadan
atılmaz. Amaç yeni özellik sayısını büyütmek değil, çocuktan profesyonele aynı
dilin verdiği sözleri kanıtlamaktır. Bulgular 1 Eylül 2026'da derleyici
kaynakları, spec, RFC'ler ve 343 test üzerinden yeniden doğrulanmıştır.

Durumlar: **KAPALI** = kanıtı var · **AÇIK** = v1 engeli · **KARAR** = önce
normatif seçim gerekir · **KAPSAM DIŞI** = v1'in açıkça vermediği söz.

## P0 — v1 etiketi öncesi zorunlu

| Kapı | Durum | Kaynakta görülen gerçek | Kapanma kanıtı |
|---|---|---|---|
| V1-P0-01 İşlem imzası çağrı sırasından bağımsızdır | **KAPALI (K-083/K-086)** | Yerel başlangıç `<ad> al` çıkarımını korur. Birim/paket işlemlerinde bütün parametre türleri ve `<Tür> döndürür` / `değer döndürmez` zorunludur (T039); gövde dönüşü ve bütün yollar doğrulanır (T040–T042). | spec/10 v1'i bilinçli monomorfik kaynak ABI'si olarak tanımlar; kırıcı semver sınırları belirgindir. Çağrılmayan gövde, paket olumsuzu, liste genişlemesi, özyineleme ve iki çağrı sırası conformance testlidir. |
| V1-P0-02 Web route ile uygulama eylemi ayrıdır | **KAPALI (K-087)** | Açık imzalı `eylem` HTTP etkisi taşıyamaz; aynı çağrı web/CLI/görev/test bağlamında kullanılabilir. GET/HEAD'in çağrı grafiğindeki dolaylı yazması T045, rota içi uygulama yazması T046'dır. Yöntemli rota 404/405, 64 KiB+100 alan 413 ve 30 saniye 504 üretir. Her eylem çalışma hatası/başarısız Sonuç için iç içe dosya savepoint'i taşır. | RFC-0015 geçici kabul + spec/11; doğrudan/dolaylı GET olumsuzları, POST→eylem zorunluluğu, CLI+web ortak eylem, 404/405/413 ve çok-dosyalı/nested rollback regresyonları. Süreç çökmesinde çok-dosyalı tek commit sözü verilmez; dosya başına K-084 geçerlidir. |
| V1-P0-03 Oturum ve çerez üretim güvenliği | **KAPALI (K-088)** | Native runtime 256 bit OS CSPRNG oturum+CSRF üretir, oturum kimliğinin yalnız SHA-256 özetini sunucuda tutar; Argon2id PHC doğrular. Giriş kimlik+CSRF'yi döndürür, 30 dakika sınırlar; logout/süre dolumu iptal eder. Roller sunucudadır; unsafe rota açık politika ve otomatik synchronizer CSRF ister. Production çerezi `__Host-`, Secure, HttpOnly, SameSite=Lax, Path=/ ve Max-Age taşır. | RFC-0017 + spec/12; CSPRNG/Argon2id, rotation/revoke/expiry/rol, eksik-sahte-geçerli CSRF, 400/401/403, çerez nitelikleri, CRLF, Host/proto/Origin olumsuzları. `--web-proxy https://host` yalnız loopback HTTPS proxy zincirini kabul eder. |
| V1-P0-04 Kalıcı durum atomik ve yarış güvenlidir | **KAPALI (K-084)** | Tek-dosya `yaz/ekle`, aynı klasörde temp+sync+atomik replace yapar; Unix/Windows işletim sistemi kilidi thread ve süreç yazarlarını sıralar. Okuyucu yalnız eski/yeni bütün sürümü görür. | RFC-0016 + spec/08; replace hata enjeksiyonu eski veriyi korur, iki thread ve iki bağımsız CLI süreci satır kaybetmez, Drop'suz ani süreç sonu kilidi bırakır. Çok-kaynaklı uygulama transaction'ı V1-P0-02/RFC-0015 sınırındadır. |
| V1-P0-05 Deadline gerçekten iptal eder | **KAPALI (K-085)** | `IcindeBlogu` mutlak son tarihi sahipli Ç001 ile blok/işlem/döngü sınırlarına yayar. `bekle` kalan süreye kırpılır; HTTP aşamaları kalan tek bütçeyi alır. İç içe tarihlerde en erken sahip kazanır. | RFC-0011 + spec/09; geç ağ yanıtı çıktıya dönüşmez, uzun bekleme sonrası cümle çalışmaz, iç/dış `yetişmezse` sahipliği sanal saatle sabittir. Tek kesintisiz ifade/platform syscall sınırı normatif işbirlikli modeldir. |
| V1-P0-06 Normatif otorite tek ve izlenebilirdir | **KAPALI (K-081)** | Spec/RFC drift'i doğrulandı. | ADR-010 belge rollerini ve atomik değişiklik sözleşmesini bağladı; RFC-0006/0011 güncel gerçek ve hedefi ayırdı. |

## P1 — profesyonel kapasite kapıları

| Kapı | Durum | Kaynakta görülen gerçek | Kapanma kanıtı |
|---|---|---|---|
| V1-P1-01 Ondalık hassasiyeti dil semantiği mi profil sınırı mı? | **KARAR** | RFC-0013/spec `k ≤ 9`u dil sözleşmesi yapıyor; runtime i128 ara değer kullanıyor. Bu bug değil, genel amaçlı kapsam kararıdır. | Para/ERP, bilim ve kur davranışlarıyla RFC kararı; limit kalırsa açık tür adı/profil, kalkarsa coefficient+scale semantiği ve taşma testleri. |
| V1-P1-02 Morfoloji deterministik ve sürümlenebilirdir | **KAPALI (K-089)** | `zee-tr-1` profili soyut ekleri, yüzeyleri, iki katman sınırını ve kanonik üretimi tek modülde sabitler. Doğrudan eşleşme önce; sonra 0=A001, 1=çözüm, 2+=A002, heuristik yoktur. İyelik ayrı kimlikle iki katmanlı üretilir; LSP aynı profili kullanır. | RFC-0018 + spec/13; tablo snapshot'ı, düzenli kök×bütün tek/iki katman `üret→çöz` property'leri, ters ses değişimi ve A002 belirsizlik korpusu. `proje.dil` profili pinler (P011); `proje.kilit` v2 paket profillerini taşır. Kırıcı tablo değişikliği yeni profil+ana sürüm/edition ister. |
| V1-P1-03 Structured concurrency adı runtime gerçeğini aşmaz | **KAPALI (K-090)** | `Eszamanli` görevleri dış ortam snapshot'ıyla kaydeder; `HepsiniBekle` kaynak sıralı tek-thread scheduler'da `bekle` noktalarında gerçekten dönüşümlü ilerletir. İç görev ağacının beklemesi dış kardeşe kadar yayılır. Aynı anda tek görev çalışır; data race yoktur. | RFC-0011 + spec/14; 2 sn+1 sn görevlerin 2 sn'de biten sabit izi, iç ağaç↔dış kardeş ilerlemesi, aynı-anda kaynak sırası, T033/T051 sahiplik, ilk hata→kardeş iptali, dış deadline→bütün ağaç ve görevde atomik eylem rollback kanıtları. Çok çekirdekli paralellik v1 sözü değildir. |
| V1-P1-04 Sonuç hata tarafı yapılandırılmıştır | **AÇIK** | `Sonuç<T>` hata tarafı bugün Metin; RFC-0008 bunu açık soru sayar. Tanılar ise zaten kod/mesaj/konum/öneri taşır. | Basit etiketli `Hata` değeri (kod, mesaj, neden/veri); eşleme, kaynak zinciri ve geriye uyum RFC'si. |
| V1-P1-05 Gezmede yazma kullanıcı zihniyle doğrulanmıştır | **AÇIK** | Spec K-074 kopya bağlayıp tur sonunda listeye geri yazmayı normatif yapıyor; davranış testli ama aliasing modeli henüz yok. | Çocuk/profesyonel usability sonucu; değer/reference semantiği RFC'si; yeniden bağlama ve alan yazma ayrımının conformance testleri. |
| V1-P1-06 Yerel modül/paket paylaşımı | **KAPALI (K-076–K-080)** | Kökenli birim/paket yükleme, doğrudan sınır, SHA-256 kilit, güvenli ekle/çıkar ve grafik görünümü çalışıyor. | P001–P010/A011 ve proje entegrasyon testleri; deterministik kilit. |
| V1-P1-07 Dağıtım/registry güven zinciri | **AÇIK** | Uzak registry, imza/provenance, SBOM ve yanked/güvenlik duyurusu henüz taslak. | Ayrı güvenlik RFC/ADR'leri, imzalı metadata, reproducible paket, offline cache/mirror ve saldırı testleri. |

## Uygulama sırası

1. Güvenli olmayan çalışan web/eşzamanlılık yüzeylerinin opt-in sınırını koru
   (K-082); son tarih iptali artık K-085 ile ayrı ve tanımlıdır.
2. K-083/K-086 public işlem sözleşmesini kitaplık, paket ve gelecekteki eylem
   API'lerinin değişmez tabanı olarak koru.
3. K-087 eylem sınırını ve K-088 oturum/CSRF/proxy profilini koru.
   Çok-dosyalı çökme atomikliği ve çok süreçli ortak oturum deposu verilmiş
   söz değildir.
4. K-090, K-085 deadline çekirdeğinin üstüne deterministik scheduler,
   sözcüksel sahiplik ve kardeş iptalini koydu. Sıradaki kapı yapılandırılmış
   hata değeridir.
5. K-089 `zee-tr-1` morfolojisini property kanıtıyla dondurdu. Ardından
   ondalık ve gezme kararlarını usability + property kanıtıyla tamamla.

Her kapının kapanışı: karar + spec + olumlu/olumsuz test + sürüm notu. Yalnız
“kod çalışıyor” işareti v1 kanıtı değildir (ADR-010).
