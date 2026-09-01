# zee v1.0 sürüm kapıları

Bu liste bir dilek listesi değildir: `1.0.0` etiketi bu kapılar kapatılmadan
atılmaz. Amaç yeni özellik sayısını büyütmek değil, çocuktan profesyonele aynı
dilin verdiği sözleri kanıtlamaktır. Bulgular 1 Eylül 2026'da derleyici
kaynakları, spec, RFC'ler ve 279 test üzerinden yeniden doğrulanmıştır.

Durumlar: **KAPALI** = kanıtı var · **AÇIK** = v1 engeli · **KARAR** = önce
normatif seçim gerekir · **KAPSAM DIŞI** = v1'in açıkça vermediği söz.

## P0 — v1 etiketi öncesi zorunlu

| Kapı | Durum | Kaynakta görülen gerçek | Kapanma kanıtı |
|---|---|---|---|
| V1-P0-01 İşlem imzası çağrı sırasından bağımsızdır | **AÇIK (K-083 temeli var)** | Başlangıç `<ad> al` çağrı-güdümlüdür. `<ad> <Tür> olarak al` açık imzası çağrı beklemeden denetlenir, çağrıyla terfi etmez; TamSayı→Ondalık runtime dönüşümü ve çağrı sırası permütasyonu testlidir. | Paket/public sınırında açık imza zorunluluğu ve ABI politikası; generic ya da bilinçli monomorfik model; açık imzalı bütün kapsayıcı/özyineleme permütasyon conformance paketi. |
| V1-P0-02 Web route ile uygulama eylemi ayrıdır | **AÇIK (K-082 korkuluğu var)** | `IstekGeldiginde` rota yolunu eşler; yöntem `istek` sözlüğünde sıradan metindir. Yetki, CSRF, idempotency, gövde sınırı ve transaction sözleşmesi yoktur. Gerçek TCP yalnız `--deneysel-web` opt-in'iyle açılır; örneklerde GET mutasyonu kaldırılmıştır. | RFC-0015'in kabulü; yöntemli route; form/API/CLI/job tarafından çağrılabilen eylem; GET'in durum değiştirmediği dil düzeyi olumsuz testler. |
| V1-P0-03 Oturum ve çerez üretim güvenliği | **AÇIK** | Gerçek IO çereze yalnız `HttpOnly` ekler; Secure/SameSite/ömür politikası yoktur. `girisli-panel` rastgele sayı + düz parola kullanan eğitim demosudur. | CSPRNG token, hash'li kimlik bilgisi, süre/rotation/revoke, güvenli çerez politikası, CSRF ve HTTPS/proxy sınırı; saldırı regresyonları. |
| V1-P0-04 Kalıcı durum atomik ve yarış güvenlidir | **KAPALI (K-084)** | Tek-dosya `yaz/ekle`, aynı klasörde temp+sync+atomik replace yapar; Unix/Windows işletim sistemi kilidi thread ve süreç yazarlarını sıralar. Okuyucu yalnız eski/yeni bütün sürümü görür. | RFC-0016 + spec/08; replace hata enjeksiyonu eski veriyi korur, iki thread ve iki bağımsız CLI süreci satır kaybetmez, Drop'suz ani süreç sonu kilidi bırakır. Çok-kaynaklı uygulama transaction'ı V1-P0-02/RFC-0015 sınırındadır. |
| V1-P0-05 Deadline gerçekten iptal eder | **KAPALI (K-085)** | `IcindeBlogu` mutlak son tarihi sahipli Ç001 ile blok/işlem/döngü sınırlarına yayar. `bekle` kalan süreye kırpılır; HTTP aşamaları kalan tek bütçeyi alır. İç içe tarihlerde en erken sahip kazanır. | RFC-0011 + spec/09; geç ağ yanıtı çıktıya dönüşmez, uzun bekleme sonrası cümle çalışmaz, iç/dış `yetişmezse` sahipliği sanal saatle sabittir. Tek kesintisiz ifade/platform syscall sınırı normatif işbirlikli modeldir. |
| V1-P0-06 Normatif otorite tek ve izlenebilirdir | **KAPALI (K-081)** | Spec/RFC drift'i doğrulandı. | ADR-010 belge rollerini ve atomik değişiklik sözleşmesini bağladı; RFC-0006/0011 güncel gerçek ve hedefi ayırdı. |

## P1 — profesyonel kapasite kapıları

| Kapı | Durum | Kaynakta görülen gerçek | Kapanma kanıtı |
|---|---|---|---|
| V1-P1-01 Ondalık hassasiyeti dil semantiği mi profil sınırı mı? | **KARAR** | RFC-0013/spec `k ≤ 9`u dil sözleşmesi yapıyor; runtime i128 ara değer kullanıyor. Bu bug değil, genel amaçlı kapsam kararıdır. | Para/ERP, bilim ve kur davranışlarıyla RFC kararı; limit kalırsa açık tür adı/profil, kalkarsa coefficient+scale semantiği ve taşma testleri. |
| V1-P1-02 Morfoloji deterministik ve sürümlenebilirdir | **AÇIK** | 0 aday=A001, 1 aday=çözüm, 2+=A002 kuralı doğru; `kok_adaylari` iki katman, ses değişimleri ve rename üretimini taşır. Heuristik seçim yoktur. | Sürümlemeli ek tablosu; çözüm↔üretim dönüşümlü kapsamlı/property testleri; her yeni ek için belirsizlik korpusu ve edition etkisi. |
| V1-P1-03 Structured concurrency adı runtime gerçeğini aşmaz | **AÇIK** | `Eszamanli` bugün görevleri kaynak sırasıyla değerlendirir; `HepsiniBekle` no-op'tur. Data race yok ama eşzamanlı ilerleme de yoktur. | Deterministik tek-thread scheduler, sahiplik ağacı, hata yayılımı/iptal ve sahipsiz görev olmadığını kanıtlayan testler; o zamana dek yüzey deneysel. |
| V1-P1-04 Sonuç hata tarafı yapılandırılmıştır | **AÇIK** | `Sonuç<T>` hata tarafı bugün Metin; RFC-0008 bunu açık soru sayar. Tanılar ise zaten kod/mesaj/konum/öneri taşır. | Basit etiketli `Hata` değeri (kod, mesaj, neden/veri); eşleme, kaynak zinciri ve geriye uyum RFC'si. |
| V1-P1-05 Gezmede yazma kullanıcı zihniyle doğrulanmıştır | **AÇIK** | Spec K-074 kopya bağlayıp tur sonunda listeye geri yazmayı normatif yapıyor; davranış testli ama aliasing modeli henüz yok. | Çocuk/profesyonel usability sonucu; değer/reference semantiği RFC'si; yeniden bağlama ve alan yazma ayrımının conformance testleri. |
| V1-P1-06 Yerel modül/paket paylaşımı | **KAPALI (K-076–K-080)** | Kökenli birim/paket yükleme, doğrudan sınır, SHA-256 kilit, güvenli ekle/çıkar ve grafik görünümü çalışıyor. | P001–P010/A011 ve proje entegrasyon testleri; deterministik kilit. |
| V1-P1-07 Dağıtım/registry güven zinciri | **AÇIK** | Uzak registry, imza/provenance, SBOM ve yanked/güvenlik duyurusu henüz taslak. | Ayrı güvenlik RFC/ADR'leri, imzalı metadata, reproducible paket, offline cache/mirror ve saldırı testleri. |

## Uygulama sırası

1. Güvenli olmayan çalışan web/eşzamanlılık yüzeylerinin opt-in sınırını koru
   (K-082); son tarih iptali artık K-085 ile ayrı ve tanımlıdır.
2. Public işlem tür sözleşmesini çöz; çünkü kitaplık, paket ve eylem API'lerinin
   hepsi bunun üstüne oturur.
3. K-084 tek-dosya atomikliğinin üstüne uygulama eylemi + çok-kaynaklı
   transaction modelini kur; yalnız web'e özel olmasın.
4. K-085 deadline çekirdeğinin üstüne gerçek scheduler ve yapılandırılmış hata
   değerini tamamla.
5. Morfoloji, ondalık ve gezme kararlarını usability + property kanıtıyla
   dondur.

Her kapının kapanışı: karar + spec + olumlu/olumsuz test + sürüm notu. Yalnız
“kod çalışıyor” işareti v1 kanıtı değildir (ADR-010).
