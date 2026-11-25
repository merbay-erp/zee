# ADR-165 — Adlı dinleyici bağlantı yüzeyi ve bağlantı scheduler'ı

- **Durum:** kabul
- **Tarih:** 23 Kasım 2026
- **İlgili kayıt:** K-381, K-376 (A1), K-355 (`urun-h`), RFC-0039 Aşama A2,
  ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-162 (adlı devir),
  ADR-011 (intrinsic sınırı), spec/12 §9, spec/23, spec/09, spec/14,
  K-372 (yazım bütçesi), K-375 (son tarih), K-339 (credential), ADR-038
  (bağımlılık politikası), GB-157, GB-158; A2c eki K-384, GB-159,
  RFC-0041/spec/26 (`.parca` modeli)

## Bağlam

Etkin dogfood ürünü `urun-h` (e-posta sunucusu) protokol motorunu Zee'de
yazmak istiyor. K-376 (A1) on adlı soketi systemd'den adla devralıp
doğruluyor ve tutuyordu; Zee kodu bağlantı kabul edemiyordu. Ürünün saf SMTP
durum makinesi stdio deneme aracıyla sınırlıydı (`"" diye sor` her okumada
istem basar, F010) ve dinleyici kalıbı S036 veriyordu (F009). 23 Kasım 2026
merkez kararı ("Sırayla aç") RFC-0039 A2'yi ZeeMail'in ikinci adımı yaptı.

Yorumlayıcı zaten future tabanlıdır: `eşzamanlı olarak` görevleri tek iş
parçacıklı, deterministik bir scheduler'da, `bekle` noktasında askıya
alınarak ilerler (spec/14). Engellemeyen ağ beklemesi aynı mekanizmanın
dar bir genişlemesiyle kurulabiliyordu.

## Karar

1. **Yüzey mevcut aileler içinde.** `"<ad>" dinleyicisinde bağlantı
   geldiğinde` rota kalıbının bağlantı karşılığıdır (üst düzey olay kaydı,
   taze ortam, örtük `bağlantı` sözlüğü). Okuma ve yazma `dene` Sonuç
   ailesindedir (`bağlantıdan satır okumayı dene`, `bağlantıya X satırını
   yazmayı dene`); ergonomi için `bağlantıya X satırını yaz` cümlesi ve
   `bağlantıyı kapat` eklendi. Yeni anahtar kelime, değer türü ya da koşul
   grameri yoktur; beş yeni kalıp kelimesi K-167 ve syntax freeze kaydına
   girdi. Bağlantı değer olarak taşınmaz: kalıplar o anki görevin
   bağlantısına uygulanır, böylece işlemlere bölünmüş protokol kodu çalışır.
2. **Hata sınıflandırılmış C030 Sonuç'tur.** `tür` Hata verisi `kapandı`,
   `zaman_aşımı`, `satır_uzun`, `satır_sonu`, `kodlama`, `kapanıyor`,
   `yazılamadı` değerlerinden biridir; protokolün doğru yanıtı (500/421)
   Zee'nin kararıdır. Son tarih Ç001'dir ve `yetişmezse` kolu çalışır
   (K-375 modeli).
3. **Satır kuralı çalışma zamanı değişmezidir.** Tek satır sonu CRLF; çıplak
   CR/LF ve karışık diziler normalleştirilmeden reddedilir (SMTP smuggling),
   uzun satır kırpılmaz, geçersiz UTF-8 Zee'ye sızmaz. Kural tek bir saf
   modülde (`dil::ag_satiri`) durur; gerçek reaktör ve hermetik adaptör
   aynı okuyucuyu kullanır.
4. **Zaman mutlaktır.** Okuma ve tıkanan yazma çağrının başından
   `bağlantı_boşta_süresi` içinde tamamlanmalıdır; damla damla gelen bayt
   süreyi uzatmaz (slowloris; K-372 ilkesi). Görevin tamamı
   `bağlantı_ömrü` son tarihine bağlıdır.
5. **Bağlantı scheduler'ı.** Her bağlantı bir görevdir; host okuma/yazma
   ilerleyemeyince `Bekle` döner, görev askıya alınır, scheduler hostun
   `poll(2)` tabanlı olay beklemesine bekleyen görevleri ve en yakın
   uyanma anını verir. Görev başına son tarih yığını ve çalışma bütçesi
   ayrı tutulur; her başarılı satır bütçeyi yeniler. Görev hatası yalnız
   kendi bağlantısını kapatır (**yalıtılmış bağlantı kapsamı**, spec/14 §3'ten
   bilinçli ayrılış). İç içe `eşzamanlı` görevler bağlantı scheduler
   noktası değildir; ağ beklemesini engelleyerek ve son tarihe bağlı yapar.
6. **Reaktör `libc` `poll(2)`.** Yeni bağımlılık yok (mio reddedildi:
   `bağlantı_sınırı` ≤ 4096 için poll yeterli, tedarik yüzeyi büyümez).
   Sınırlar reaktörde zorlanır: süreç geneli ve uzak IP başına bağlantı
   sayısı (aşımda bildirilen ret satırı tek engellemeyen yazımla, Zee görevi
   kurulmadan), satır uzunluğu, bağlantı başına 64 KiB giden tampon (geri
   basınç). Ret günlüğü oran sınırlıdır.
7. **Yetkinlik `ağ-dinleyici`.** Statik denetim (T054) ve çalışma zamanı
   politikası; çocuk modu kapalı. `ağ-sunucusu` anlamı değişmez. Bağlantı
   işlemleri geri alınamaz etkidir: eylem içinde T048.
8. **Hizmet yaşam döngüsü.** Scheduler başlayınca `READY=1`; SIGTERM'de
   yeni kabul durur, `STOPPING=1`, okumalar `kapanıyor` döner, en çok 10
   saniyelik boşaltmadan sonra kalan görevler iptal edilir ve süreç 0 ile
   çıkar.
9. **Kanıt altyapısı.** Bağlantı olay beklemesi ve her işlem IO izine girer
   ve yeniden oynatılır. Hermetik adaptör betiklenmiş istemcileri parça
   parça teslim eder; eşzamanlı oturum sırası deterministik sınanır.
10. **Birleşmeyen kipler.** Aynı programda web sunucusu ve bağlantı bloğu
    (C030), çok işçi grubu (K-376 sınırı), `--güvenli` ve Unix dışı platform.
    Bildirimsiz projede ve bağlantı bloğu olmayan programda hiçbir davranış
    değişmez; mevcut web sunucusu yolu dokunulmadan kalır.

## Ek karar — A2b sunucu TLS'i (24 Kasım 2026)

11. **Kimlik adla, K-339 modeliyle.** `tls_sertifika_değişkeni` ve
    `tls_anahtar_değişkeni` (P021) credential/ortam adlarıdır; zincir ve
    anahtar açılışta `CREDENTIALS_DIRECTORY` (nofollow, izin/sahip
    denetimi) ya da ortamdan okunur, kaynak yürütülmeden doğrulanır; hata
    çıkış 2 ve anahtar içeriği iletiye girmez. Proje köküne göreli PEM yolu
    reddedildi (sır proje ağacında durur, yedeklere/depoya sızar).
12. **`rustls` + `ring` sabit politika.** Ağaçtaki `ureq` yığını doğrudan
    bağımlılık yapıldı (Cargo.lock'ta yeni paket yok); TLS 1.3/1.2, yalnız
    ECDHE + AEAD takımları, yeniden müzakere/sıkıştırma/istemci
    sertifikası yok. `native-tls` sunucu tarafında kullanılmaz (platforma
    göre değişen politika).
13. **Implicit TLS görev öncesi.** El sıkışma reaktörde biter; başarısız ya
    da süresi dolan el sıkışma Zee görevi kurmaz (kimliksiz DoS'ta
    yorumlayıcı maliyeti yok). `bağlantı` Sözlüğü `tls` taşır.
14. **STARTTLS tek kalıp, enjeksiyona karşı kapanış.** `bağlantıda TLS
    başlatmayı dene` → `Sonuç<Metin>` (sürüm + takım). Tamponda tüketilmemiş
    düz metin varsa yükseltme yapılmaz, tampon atılır ve bağlantı kapanır
    (CVE-2011-0411); RFC 3207'nin "sunucu düz metni atmalı" kuralının
    sıkı biçimi. Zee'ye bağlantıyı açık bırakma seçeneği verilmedi:
    istemci zaten protokolü ihlal etmiştir. Giden düz tampon önce
    boşaltılır, satır okuyucu sıfırlanır.

Reddedilen (A2b): ayrı `bağlantının TLS durumu` sorgu kalıbı (Sonuç ve
sözlük yeterli; yüzey küçük tutuldu), SIGHUP ile sertifika yeniden
yükleme (süreç yeniden başlatması `Accept=no` ile kesintisiz), istemci
sertifikası doğrulaması (SMTP alıcısında gereksiz; IMAP/submission için
ayrı karar), ALPN.

Kanıt (A2b): `compiler/tests/ag_baglantisi_tls_testi.rs` — implicit TLS
1.3 dinleyicisi ve görevin el sıkışmadan sonra kurulması; STARTTLS TLS 1.3
ve TLS 1.2 (ECDHE + GCM/CHACHA20) ile, ikinci yükseltme `454 tls`;
STARTTLS ile aynı pakette gönderilen `RCPT` sonrası `220` ardından EOF
(enjeksiyon TLS'e taşınmaz, süreç hizmete devam eder); TLS 1.1 ve CBC
(0x002f, 0xc013) ClientHello'ları TLS uyarısıyla reddedilir ve karşılama
gönderilmez; bozuk anahtar/zincir ve eksik kimlik çıkış 2 (iletide anahtar
yok), credential dizini 0600 okunur, 0644 reddedilir. Hermetik:
`ag_baglantisi_testi` STARTTLS enjeksiyonu ve kimliksiz süreç;
`proje_testi` P021. Gerçek systemd 255 (Docker; `scripts/systemd-adli-soket-
olcumu.py` bağlantı koşusu, dört adlı soket): Python/OpenSSL istemcisiyle
implicit TLS 1.3 (`220`/`221`), STARTTLS sonrası TLS 1.3 `NOOP`, aynı
pakette `STARTTLS` + `NOOP` → `220` ardından EOF; `~@privileged
@resources` + `MemoryDenyWriteExecute` altında `rustls`/`ring` çalışır.
Bu konteyner systemd'si credential dizini bağlayamadığı için ölçüm kimliği
ortamdan verdi (betik `LoadCredential=` desteğini sınayıp seçer; CI
ubuntu-24.04'te credential yolu kullanılır).

## Ek karar — A2c blok okuma ve bayt-kesin akış (24 Kasım 2026, K-384)

ZeeMail SMTP (19/19) ve IMAP (imaplib 36/36) gerçek TCP'de çalıştıktan
sonra kalan engeller: `DATA` gövdesi ve IMAP literal'i satır akışına
sığmıyor (satır sınırı, ikili içerik, bellek), `+ ` devam yanıtı ve `{n}`
literal'i satır sonusuz yazma ve bayt-kesin okuma istiyor (ZM-F031),
`IDLE` boşta süresi dolmadan posta kutusunu yoklayamıyor (ZM-F032).

15. **Blok dosyaya, Zee'ye yalnız künye.** `bağlantıdan nokta bloğunu
    dosyaya almayı dene` ve `bağlantıdan <n> baytı dosyaya almayı dene`
    içeriği belleğe almadan `0600` `.parca` dosyasına akıtır, SHA-256'yı
    aynı akışta hesaplar, `fsync` sonrası `Sonuç<Sözlük>` (`yol`, `sha256`,
    `bayt`) verir. Model RFC-0041/spec/26 web yüklemesiyle aynıdır; dizin
    ayrıdır (`.zee/baglanti-bloklari`) çünkü uzun bağlantı oturumunda
    taşınmamış dosya web süpürmesinin 15 dakikalık eşiğini aşabilir.
    Bağlantı süpürmesinin eşiği `bağlantı_ömrü` + 15 dk; tamamlanan dosya
    bağlantıya aittir, sonraki blokta ya da bağlantı bitince silinir.
16. **Smuggling'e karşı tek sonlandırıcı.** Nokta bloğu yalnız birebir
    `CRLF . CRLF` biter; çıplak CR/LF satır başı sayılmaz, blok
    `satır_sonu` ile atılır ve akış gerçek sonlandırıcıya kadar tüketilir.
    Sınır aşımı da tüketilerek reddedilir; iki retten sonra oturum eşzamanlı
    sürer. Saf çözücü `dil::ag_satiri::BlokOkuyucu` gerçek reaktörle
    hermetik adaptörün ortak kuralıdır.
17. **Mutlak süre ve okuma yönünün kapanması.** Blok, literal ve aktarım
    çağrı başından `boşta + bayt / 64 KiB/sn` bütçesine bağlıdır (K-372).
    Yarıda kalan blok/literal okuma yönünü kapatır: eşzamanı bozuk akışın
    kalanı komut olarak okunamaz; yazma sürer (`421`).
18. **Bayt-kesin akış kaçış genişletmeden.** `bağlantıya X parçasını yaz`
    (ve `… yazmayı dene`) satır kuralına uyan Metni satır sonu eklemeden
    yazar; `bağlantıdan <n> bayt okumayı dene` en çok satır sınırı kadar
    UTF-8 baytı Metin verir; `bağlantıya <yol> dosyasını aktarmayı dene`
    dosyayı geri basınçla akıtır (`dosya-okuma` da gerekir). Zee metnine
    `\r` kaçışı eklenmedi: CRLF'yi yalnız çalışma zamanı üretir.
19. **Kısa süreli okuma; zaman aşımı oturumu bitirmez.** `bağlantıdan
    satır okumayı <n> saniye içinde dene` boşta süresini aşamaz; zaman
    aşımı (bütün okumalarda) bağlantıyı kapatmaz, yarım satırı korur.
20. **Bağlam ipuçları, davranış aynı.** Örtük `bağlantı` Sözlüğü
    sözcüksel kapsamda kalır; işlemde A001 parametre önerir. Yerleşik
    `yaz`/`gönder` ile biten işlem adının cümle çağrısında A001/S037
    çakışmayı adıyla anar. Kodlar, ret ve ifade çağrısı değişmedi.
21. **Sınır proje bildirimi.** `bağlantı_blok_sınırı` (P020, 1 KiB–1 GiB,
    varsayılan 32 MiB) blok ve aktarım tavanıdır.

Reddedilen (A2c): blok içeriğini Metin vermek (ikili ileti, bellek), satır
akışından blok kurmak, `\r` kaçışı ya da ham bayt değeri türü (grammar ve
değer modeli; core freeze), bloğun içinde LF normalleştirmesi, okuma zaman
aşımını Ç001 iptali yapmak (IDLE oturumu kaybolur), `bağlantı`yı işleme
örtük geçirmek (gizli dinamik kapsam), yerleşik fiille biten işlem adını
tanımda reddetmek (ifade çağrısı geçerli; uyumluluk kırılır), bağlantılar
arası uyandırma (Katman C/E, ayrı adım).

Kanıt (A2c): `compiler/tests/ag_blok_testi.rs` (hermetik: 3 MiB nokta
bloğu 64 KiB teslimle, nokta doldurma, sınır aşımında tüketme, üç
smuggling dizisi, kesilen bağlantı, susan istemci ve okuma yönünün
kapanması, literal tam/eksik/fazla bayt, sınır ve `geçersiz`, parça ve
NUL `kodlama`, kısa süreli okuma ve sanal saat, dosya aktarımı/sınır/yok,
IO izi kaydı ve yeniden oynatma, T034, tanı ipuçları);
`compiler/tests/ag_baglantisi_surec_testi.rs` (gerçek TCP: 900 KiB DATA
7 KiB parçalarla, `0600` dosya ve SHA-256, pipelining, smuggling,
1.1 MB sınır aşımı, taşınmamış dosyanın sonraki blokta ve bağlantı
sonunda silinmesi; damla damla DATA'nın mutlak süreyle kesilip okuma
yönünün kapanması ve yarım dosyanın silinmesi, blok ortasında kopan
istemci; `+ ` iki bayt, 70000 baytlık ikili literal, 6 bayt parola,
literal'in bağlantıya geri aktarımı; IDLE benzeri 1 sn yoklama ve yarım
satırın korunması); `ag_satiri` birim testleri; `proje_testi` P020;
gerçek systemd ölçümüne kısa DATA + IDLE yoklama adımı.

## Ek karar — Protokol birimi sınırında bütçe yenileme (24 Kasım 2026, K-388)

Ürün ölçümü (`urun-h` ZM-F035): uzun IMAP IDLE oturumu ve kötü satırlara
`500` yazan SMTP döngüsü, canlı değer birkaç KiB iken C024 ile kapanıyordu.
A2a "her başarılı satır okuması bütçeyi yeniler" diyordu; süreli okumanın
`zaman_aşımı` dönüşü (A2c IDLE yoklaması) ve `satır_uzun`/`satır_sonu`/
`kodlama` retleri yenilemiyordu, iade-yok zarfı (ADR-033) oturum boyunca
doluyordu.

Karar: protokol birimi sınırı, girdi tüketen ya da süresini dolduran her
okuma dönüşüdür — satır, süreli satır, bayt okuması, nokta bloğu ve bayt
bloğu çağrısının başarısı ya da `zaman_aşımı`, `satır_uzun`, `satır_sonu`,
`kodlama` dönüşü; nokta bloğunda sonlandırıcıya kadar tüketilen
`boyut_aşımı`. Her sınırda adım/çıktı/heap bütçesi taze kurulur. Anında
dönen `kapandı`, `kapanıyor`, `geçersiz`, `tls`, okumadan dönen
`boyut_aşımı`, yazma, aktarım, TLS başlatma ve `bekle` sınır değildir.

DoS değerlendirmesi (GB-162): yenileme hızı ya istemcinin gönderdiği bayta
(satır sınırıyla; K-381'den beri başarılı satırda aynı) ya da duvar
saatine (süreli okuma en az 1 sn, normal okuma `bağlantı_boşta_süresi`)
bağlıdır; istemci bunu anında dönen sonuçla hızlandıramaz. Kapalı
bağlantıyı yok sayan döngü yenilenmez ve C023/C024 ile biter. Görev
toplamı `bağlantı_ömrü`, eşzamanlılık bağlantı sınırlarıyla sınırlıdır;
tek birimde 64 MiB/10 M adım/16 MiB çıktı aynen geçerlidir. Açık: birimler
arası yaşayan değerler yeni bütçeye yeniden sayılmaz (K-381'den beri);
birim içi atılan değer iadesi yoktur, büyük gövdeyi tek birimde Zee metni
olarak işlemek C024'tür (RFC-0043 aralıklı okuma). K-393 (ADR-173) ile birim
içinde düşen değer artık iade edilir; tek birimde **canlı** 64 MiB aşımı
C024 olarak kalır.

Kanıt: `compiler/tests/baglanti_butce_yenileme_testi.rs` (IDLE zaman aşımı,
girdi tüketen ret ve çok komut olumlu; tek birim zarf aşımı, `kapandı`
döngüsü ve okumadan `boyut_aşımı` olumsuz), regression
`regression/runtime/baglanti-idle-butce-yenileme.dil` (`baglanti` kipi).
Web istek bütçesi değişmedi (`web_testi` 503 vakaları).

## Reddedilen seçenekler

- **Bağlantı başına süreç (inetd modeli):** yalıtım güçlü ama üst düzey kod
  her bağlantıda yeniden koşar, başlangıç maliyeti ve bellek binlerce
  oturumda kabul edilemez.
- **Bağlantı başına OS iş parçacığı + ayrı yorumlayıcı:** süreç içi dosya
  kilitleri (fcntl) iş parçacıkları arasında yalıtım vermez; `zee-esz-1`
  determinizmi kaybolur (RFC-0039 alternatifleri).
- **Yalnız `dene` biçimli yazma:** her yanıt satırı için Sonuç denetimi
  protokol kodunu şişiriyordu; cümle biçimi hata durumunda yalnız o
  bağlantıyı kapatır.
- **Satır sonunu normalleştirme (LF'yi kabul):** SMTP smuggling'i açar.

## Kanıt

- `compiler/tests/ag_baglantisi_testi.rs` (hermetik): satır akışı ve QUIT,
  çıplak LF/CR, `<CR>.<CR>`, uzun satır ve kodlama sınıfları, iki oturumun
  parça parça iç içe ilerlemesi, bağlantı/uzak adres sınırı ve kibar ret,
  susan istemci zaman aşımı, `içinde`/`yetişmezse` okuma noktasında, görev
  hatasının yalıtımı, yazma enjeksiyonu Sonuç'u, kayıt hataları, çocuk modu,
  `bekle` ile kardeşin ilerlemesi ve IO izi kaydı/yeniden oynatması.
- `compiler/tests/ag_baglantisi_surec_testi.rs` (gerçek süreç + TCP,
  systemd soket devri taklidi): uçtan uca SMTP benzeri oturum ve pipelining,
  eşzamanlı oturumların birbirini beklememesi ve dördüncü bağlantının
  `421` ile kapanması, damla damla bayt gönderen istemcinin mutlak süreyle
  kesilmesi, SIGTERM boşaltması (`421 kapanıyor`, READY/STOPPING), 3000
  satırlık yanıtta geri basınç ve kayıpsız teslim, hiç okumayan istemcinin
  yazma süresinde kesilmesi, yetkinliksiz T054 ve devirsiz çıkış 2.
- `dil::ag_satiri` birim testleri, `proje_testi` P020, io izi kodlama
  birim testi, gerçek systemd ölçümü `scripts/systemd-adli-soket-olcumu.py`
  (bağlantı bloğu ve 465/993 dışı kapılar dahil).

## Sonuçlar

- `urun-h` SMTP alıcısının bağlantı tarafı Zee'de yazılabilir; F009 ve
  F010 sürtünmeleri kapanır.
- Bağlantı içindeki PostgreSQL/dosya/HTTP çağrıları senkron dilimdir;
  uzun DB çağrısı kardeş bağlantıları bekletir (Katman C açık).

## Açık

- Sertifika yenilemede süreç içi yükleme; bağlantılar arası uyandırma
  (IDLE'ı yeni iletiyle uyandırma); kısmi literal ve dosya aralığı aktarımı;
  blok dosyasına yazımın senkron dilimi (Katman C).
- Çok işçi grubuna adla aktarım; binlerce boşta oturumun RSS ölçümü (H4).
- `dil dene` içinden betiklenmiş bağlantı cümlesi.
