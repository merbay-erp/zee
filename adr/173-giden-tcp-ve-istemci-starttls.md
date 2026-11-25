# ADR-173 — Giden TCP, `ağ_hedefleri` `tcp://` politikası ve istemci STARTTLS

- **Durum:** kabul — K-397 Dilim 1 (RFC-0044 B4a) uygulandı
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-397, `urun-h` (ZeeMail `zeemail/disteslim`), RFC-0044
  ("Uygulama kararı — K-397"), RFC-0024 / spec/23 (outbound yetkinlik),
  ADR-165 (bağlantı yüzeyi), ADR-059 (core freeze), ADR-064 (dil yüzeyi),
  ADR-038 (tedarik zinciri), K-387 (tek dosya ağı), GB kaydı (giden TCP)

## Bağlam

ZeeMail başka kurumlara posta teslim etmek zorunda. ZeeOS'ta 25 kapısına
çıkış yalnız `zeemail_disteslim` hizmetine açıktır. Teslim motoru Zee'dedir
(SMTP istemci durum makinesi, MX sırası, yeniden deneme, MTA-STS/DANE/TLS-RPT
kararı, DSN). Dilde eksik olanlar şunlardır: hedefi önceden bilinmeyen public
bir IP'ye giden TCP, o bağlantıda istemci STARTTLS ve DANE için sertifika
özetleri. Bugün dışarıya yalnız HTTP(S) istemcisi ile K-210 aktarıcı SMTP'si
çıkabiliyor, ikisi de tam origin listesiyle sınırlı.

Hedefler saldırganın denetleyebildiği MX kayıtlarından gelir. Keyfî IP'ye
giden TCP yüksek riskli bir yüzeydir: SSRF ve DNS yeniden bağlama bu kararın
çekirdeğidir.

## Karar

1. **Politika `ağ` + `ağ_hedefleri` ile kurulur; yeni yetkinlik açılmaz.**
   Bu, RFC-0044 taslağındaki `smtp-teslim` yetkinliğinden bilinçli bir
   sapmadır (RFC-0044 "Uygulama kararı"). İki yeni bildirim biçimi vardır:
   - `tcp://<konak>:<kapı>` tam hedeftir. Kapı zorunludur, varsayılanı yoktur.
     Çözülen IP spec/23 kuralına uyar: `yerel-ağ` ile yerel adrese de
     gidilebilir, metadata her profilde yasaktır.
   - `tcp://*:25` joker hedeftir ve **yalnız 25 kapısında** yazılabilir.
     Joker yalnız public IP'ye izin verir; `yerel-ağ` açık olsa bile
     genişlemez.
   - Joker yalnız bildirimde tanınır. İstek adresinde `*` reddedilir;
     böylece `https://mta-sts.*/…` isteği joker bildirimiyle birebir
     eşleşemez.
2. **Tek çözümleme, denetlenen adrese sabitleme.**
   `YetkinlikPolitikasi::giden_tcp_hedefini_coz` saf bir fonksiyondur. Gerçek
   host ve hermetik adaptör bu aynı kodu çağırır.
   - Ad çözücüye en çok bir kez gider; en çok 16 adres değerlendirilir.
   - Adreslerden **biri bile** izinsizse hedef reddedilir. Bu fail-closed
     kural HTTP istemcisinin `PolitikaliCozucu` kuralıyla aynıdır.
   - IPv4-mapped (`::ffff:0:0/96`) her profilde reddedilir.
   - Soket yalnız dönen `SocketAddr`'a açılır; host adı ikinci kez çözmez.
   - Public sayılmayan aralıklar spec/23'teki listedir (RFC-0044 tablosu).
3. **Yüzey.**
   - `"<ana>:<kapı>" adresine bağlanmayı dene` →
     `Sonuç<Sözlük>{uzak_adres, uzak_kapı, yerel_kapı}`.
   - Ret C030'dur; `tür` şunlardan biridir: `geçersiz`, `hedef_yasak`,
     `çözülemedi`, `bağlanamadı`, `zaman_aşımı`, `zaten_bağlı`, `sınır`.
   - Başarılı bağlantı görevin bağlantısı olur (ADR-165 "bağlantı değer
     değildir"). A2/A2c kalıplarının tamamı onun üzerinde çalışır.
   - Bağlantı kalıpları derlemede `ağ-dinleyici` **ya da** `ağ` ile
     karşılanır. Runtime kapısı aynıdır: giden bağlanma `ağ` ister, açık
     bağlantının işlemleri iki yetkinlikten birini ister.
4. **Görev kapsamı.**
   - Görev başına aynı anda bir giden bağlantı olur; ikincisi `zaten_bağlı`
     döner, `bağlantıyı kapat` yuvayı boşaltır.
   - Dinleyici görevi giden bağlantı açamaz (`geçersiz`), yani gelen
     oturumun bağlantısı sessizce değiştirilemez.
   - Eşzamanlı görevlerin her biri üst görevden başlayan kendi yuvasını
     taşır ve bitişte kendi açtığı giden bağlantıyı kapatır.
   - Her program koşusu kendi kapsamında başlar ve sonunda açık kalan giden
     bağlantıları kapatır, böylece aynı thread'deki `dene`/LSP/test
     koşuları birbirine sızmaz.
5. **Bütçeler.**
   - Bağlanma süresi `min(Zee son tarihi, 30 sn)`.
   - Okuma, yazma ve el sıkışma mevcut bağlantı ayarlarıyla sınırlıdır
     (boşta süresi, satır sınırı, 64 KiB giden ve TLS tamponu).
   - Süreç başına en çok 16 açık giden bağlantı (`sınır`).
   - Giden bağlantılar dinleyicinin kabul sınırlarına sayılmaz.
6. **İstemci STARTTLS.** Yüzey
   `bağlantıda TLS istemcisi başlatmayı dene "<ad>" ile` →
   `Sonuç<Sözlük>{doğrulandı, doğrulama_hatası, sürüm, takım, sunucu_adı,
   sertifika_sha256, spki_sha256, zincir_sha256, zincir_spki_sha256}`.
   - Sunucu tarafıyla aynı `rustls`/`ring` sağlayıcısı ve sürüm/takım
     politikası kullanılır; kökler `webpki-roots 1.0.10` (kilitli ağaçta,
     doğrudan bağımlılığa yükseltildi).
   - PKIX sonucu el sıkışmayı durdurmaz, ayrı alan olarak döner. DANE-EE(3)
     ve fırsatçı TLS el sıkışmanın tamamlanmasını ister; MTA-STS `enforce`
     için ürün `doğrulandı = evet` olmadan teslim etmez.
   - `KayitliDogrulayici` yalnız `verify_server_cert`'in sonucunu kaydeder.
     El sıkışma imzaları `WebPkiServerVerifier`'a bırakılır ve atlanmaz:
     karşı taraf raporlanan SPKI'nin özel anahtarına sahiptir.
   - Hata adları: `ad_uyuşmuyor`, `süresi_dolmuş`, `henüz_geçerli_değil`,
     `bilinmeyen_yayıncı`, `iptal`, `geçersiz_sertifika`. Uç sertifika olarak
     kullanılan CA (`CA:TRUE`'lu kendinden imzalı yaprak) bu sonuncuya düşer.
   - Zincir en çok 10 sertifika olabilir; oturum sürdürme ve 0-RTT kapalıdır.
   - Sunucu adı ASCII DNS adı olmalıdır (IP, sonda nokta, tek etiket →
     `geçersiz`).
   - `220`'den sonra ara bellekte düz metin varsa (CVE-2011-1575 sınıfı)
     yükseltme yapılmaz, bağlantı kapanır, `tür=tls` döner.
   - SPKI, `dil::ag_satiri::x509` adlı sınırlı DER yürüyücüsüyle çıkarılır:
     belirsiz uzunluk ve kanonik olmayan uzunluk yok, en çok 64 KiB, panik
     yok, `unsafe` yok.
7. **MTA-STS ilke indirmesi.** Mevcut HTTP istemcisi yeterlidir: yönlendirme
   kapalı, çözümleme sonrası IP denetimi, PKIX, gövde ve süre sınırı zaten
   var. Tek ek `https://mta-sts.*` jokeridir ve yalnız şu koşullarda eşleşir:
   443 kapısı, `mta-sts.` önekli bir DNS adı, birebir
   `/.well-known/mta-sts.txt` yolu (sorgu yok), yalnız public IP.
8. **Deterministik IO.**
   - Giden bağlanma `baglanti_islemi [0, baglan, adres, son] → giden …`
     olarak kodlanır, istemci TLS'i `tls_istemci` kaydı olarak.
   - Replay ağa gitmez.
   - Hermetik adaptörde betiklenmiş sahte sunucu, sahte çözücü ve sahte TLS
     kaydı vardır. Sahte kayıttaki ad istenen addan farklıysa sonuç
     `ad_uyuşmuyor` olur.
9. **Test kökü yalnız hata ayıklama derlemesinde.** `ZEE_TEST_TLS_KOKLERI`
   (PEM yolu) yalnız `debug_assertions` ile okunur; sürüm ikilisi bu
   değişkeni hiç okumaz.

## Reddedilen seçenekler

- **Yeni `smtp-teslim` yetkinliği.** İkinci bir outbound politika noktası
  açardı (RFC-0024 tek nokta ilkesi). Kapı kısıtı joker biçimiyle aynı
  güvenceyi verir.
- **Joker her kapıda.** Ham TCP'yi keyfî hizmete açardı (6379, 22). Bugünkü
  ürün ihtiyacı yalnız 25'tir.
- **Karışık yanıtta yalnız public adresi seçmek.** Saldırganın denetlediği
  DNS'te karışık yanıt şüphelidir; HTTP istemcisiyle tutarlı fail-closed
  kural seçildi.
- **PKIX başarısızlığında el sıkışmayı kesmek (yalnız katı kip).** DANE-EE
  ve fırsatçı TLS kullanılamaz hâle gelirdi. Açık alan, ürün politikasının
  okuyacağı yerdir.
- **`native-tls` istemcisi.** Özel doğrulayıcı ve SPKI erişimi yok
  (RFC-0044 B4).

## Sonuçlar

- `ağ_hedefleri` K-167 yüzeyi genişler (`tcp://` şeması ve iki joker); iki
  yeni kalıp eklenir. Tanı kodu yeni değildir (C030 `tür` değerleri).
- Güvenlik sınırı GB kaydıyla izlenir. Kalan riskler: public internette
  keyfî bir 25 kapısına bağlanma (hizmetin işi; oran ürün kuyruğunda) ve
  ürünün `doğrulandı` alanını okumaması (ürün testleri).
- Ad çözümlemesi işletim sisteminin engelleyen çözücüsüyle yapılır; süresi
  OS'a bağlıdır. ZeeMail MX'i B1a çözücüsüyle IP'ye çözüp IP verir.
  Bağlantı yolunun B1a çözücüsüne bağlanması #98 birleşince ele alınır
  (açık).
- Fuzz: `x509_spki` hedefi adaydır. Yürüyücü saldırgan baytını yalnız
  rustls/webpki kabul ettikten sonra görür; birim testleri kesik, bozuk,
  kanonik olmayan ve rastgele çevrilmiş DER'i kapsar. RC kapısına alınması
  açık iştir.
- Soak gerekmez: süreç içi önbellek yoktur. Kök deposu süreç başına bir kez
  kurulan sabit `OnceLock`'tur; bağlantı durumu bağlantıyla yaşar.
