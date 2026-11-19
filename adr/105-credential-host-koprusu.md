# ADR-105 — Credential dosyasından mevcut CLI ortamına host köprüsü

- **Durum:** kabul, sınırlı POSIX pilot
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-245/P25, GB-068

## Bağlam ve karar

Exact `zee-cekirdek` talebi D3, mevcut SMTP/DB/anahtar ortam bağlarının
credential dosyasından sağlanmasını ister. Talep SHA'sı ve anonim dosya
kökenleri [ürün kaydındadır](../dogfood/zee-cekirdek/credential/README.md).
Mevcut env API'si yeterlidir; core freeze altında yeni grammar, intrinsic,
CLI komutu, `zee.ayar` alanı, facade export veya dependency eklenmez.

`scripts/credential-baslat.py` açık mutlak root, ENV=basename ve trusted
ikili seçimiyle mevcut `çalıştır` argv'sini başlatır. Bütün dosyalar çocuk
başlamadan doğrulanır: nofollow directory zinciri, sahip/mode, regular tek
hardlink, nonblocking FIFO reddi, pre/open/post identity ve son snapshot
kontrolü. En fazla 16 eşleme ve toplam 64 KiB, tek satır strict UTF-8;
ambient eşleme varsa ret, fallback veya trim yoktur. Ayrıntılı çağrı ve
izin sözleşmesi [host rehberindedir](../docs/credential-baslat.md).

Çocuk stdio'su DEVNULL'dur; dış çıktı sabit schema/source/exit metadata'dır.
Host eşlemesi native CLI proje allowlist'i veya ağ iznini açmaz. Launcher
ikiliyi authenticate etmez; mevcut imzalı seçici ve işletmeci güven sınırı
korunur. Restart-only rotation vardır; çoklu dosyanın atomik credential
snapshot'ı veya canlı reload sözü verilmez.

## Güvenlik ve açık kapsam

GB-068 kabul edilen sınırdır: dosya değeri çocuk env'sine dönüşür ve süreç
belleği/alt süreç inheritance'da bulunabilir. Bu, systemd file credential'ın
noninheritance avantajını koruyan file-native tasarım değildir. Aynı UID,
root, ACL/NFS, genel taint ve OS sandbox garantisi yoktur. Windows DACL,
gerçek systemd servisi, canlı ürün/provider ve tam P25 kabulü açıktır.

## Kanıt ve etki incelemesi

Normal POSIX Cargo wrapper 14 actual Python vakasını atlamasız yürütür:
gerçek dosya/izin/UID/symlink/FIFO/TOCTOU/boyut/UTF8/collision/rotation,
child secret echo bastırması, native env allowlist ret/kabul ve gerçek
loopback SMTP AUTH1/DATA1. Fixture credential'ları public sentineldir.
Temiz exact kaynak kampanyası çalışma ağacı denemesinden ayrıca kaydedilir.

RFC/spec/tanı/ABI/API, production katman graph'ı, benchmark iş yükü ve
compiler cache değişmez; ilgili kayıtların değişmemesi bilinçlidir.
Markdown rehberleri, günlük/backlog, ürünleşme/kapı durumu, güvenlik/kanıt
haritası, faz matrisi, README sayıları ve kanıt özeti aynı dalgada yenilenir.

Host metadata/readiness biçimleri giriş sürümüyle K-167 uyumluluk
fixture'ında aktiftir; bunlar compiler CLI/grammar/facade export'u değildir.

## Ek — 19 Kasım 2026: native systemd credential kaynağı (K-339/GB-135)

Bu ADR'nin açık bıraktığı "gerçek systemd servisi" ve "file-native sır"
sınırının dar dilimidir. Aynı `zee-cekirdek` D3 talebi (exact ürün
`abaca3a`, [anonim kayıt](../dogfood/zee-cekirdek/systemd-credential/README.md))
ve ürünün bugünkü `LoadCredential=` dağıtımı kanıttır.

**Karar.** Native CLI, ana projenin mevcut gizli değer bağlarını
(`gizli_anahtar_değişkeni`, `e_posta_bağlantı_değişkeni`, PostgreSQL
bağlantı değişkeni ve `_TLS_CA_PEM` eki, `http_gizli_değişkenleri`)
`CREDENTIALS_DIRECTORY` ayarlıysa aynı adlı credential dosyasından da okur.
Ad credential kimliğidir: `LoadCredential=UYGULAMA_SMTP_URL:/etc/...` ya da
`SetCredential=`. Tek okuyucu `dil::guvenlik::sir_dizini`'dir; yeni grammar,
`zee.ayar` alanı, yetki, CLI komutu, tanı kodu, facade export veya bağımlılık
yoktur. Değişken ayarlı değilse davranış birebir eskisidir. Değer artık
çocuk ortamına dönüşmez; GB-068'in env mirası sınırı bu yolda kalkar.

**Güvenlik sözleşmesi** ([spec/23 §K-339](../spec/23-yetkinlik-ve-outbound-guvenligi.md)):
mutlak dizin, `/`'den her bileşen `openat(O_DIRECTORY|O_NOFOLLOW)`; dizin
sahibi euid/root, grup/diğer yazamaz. Ad yalnız `[A-Za-z0-9_][A-Za-z0-9_.-]*`
(≤255 bayt); uymayan ad dizinde aranmaz. Dosya `openat(O_NOFOLLOW|O_NONBLOCK)`,
descriptor üzerinde fstat: düzenli, tek link, sahibi euid/root, grupça
yazılamaz, herkesçe okunamaz (0440 ACL maskesi kabul). 1..=sınır bayt (anahtar
4 KiB, URL 64 KiB, CA PEM 256 KiB), katı UTF-8, BOM/NUL yok, PEM dışı CR/LF
yok, kırpma yok; hata yolunda tampon sıfırlanır. Aynı ad hem ortamda hem
dizinde varsa ret (K-245 launcher'ının "öncelik yok" ilkesi). Dizin yoksa açık
hata, ortama sessiz dönüş yok. Ret iletisi değişken adı + sabit neden; değer,
yol, dizin adı yok. HTTP/SMTP sonuç maskeleri ve IO izinin gizli ortam reddi
değişmez.

**Neden ortam önceliği değil ret?** İki kaynak varken hangisinin kazandığı
işletmecinin bilmediği bir rotasyon/sızıntı durumu yaratır; mevcut
launcher sözleşmesi de ambient eşlemeyi reddeder.

**Neden yeni ayar değil?** systemd credential kimliği ile proje bildirimindeki
değişken adı zaten tek ad alanıdır; ayrı eşleme tablosu yeni yüzey ve
uyuşmazlık sınıfı açardı.

**Platformlar.** Linux hedeftir; dosya sözleşmesi POSIX olduğu için macOS'ta
da aynı testler koşar (systemd yoktur; dizin elle hazırlanabilir). Windows'ta
systemd ve POSIX mode/uid karşılığı yoktur; DACL/reparse sözleşmesi
kanıtlanmadan açmak yerine `CREDENTIALS_DIRECTORY` ayarlıysa okuma
"bu platformda desteklenmez" ile reddedilir (fail-closed), ayarlı değilse
ortam davranışı sürer. Windows credential kaynağı ayrı karar ister.

**Kanıt.** `compiler/src/guvenlik/sir_dizini/testler.rs` gerçek geçici dizinde
11 vaka (symlink, hard-link, writersız FIFO, dizin, mod, boyut, UTF-8/BOM/NUL,
satır sonu, ad, çakışma); `compiler/tests/systemd_credential_testi.rs` gerçek
`dil` ikilisiyle 3 vaka (HTTP gizli ortam ve gizli anahtar dosyadan, maskeli
HTTP retleri, C028 sabit nedenler, stdout/stderr'de değer/yol yok). Linux ve
macOS CI'da yürür; Windows'ta platform reddi birim testi yürür. Gerçek systemd
unit'iyle canlı kabul, çalışan süreçte rotasyon ve ürün geçişi açıktır.
K-245 host köprüsü değişmeden kalır; systemd altında artık gerekli değildir.
