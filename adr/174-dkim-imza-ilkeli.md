# ADR-174 — DKIM imza ilkeli ve `dkim_anahtarları` bildirimi

- **Durum:** kabul — K-397 Dilim 2 (RFC-0044 B3) uygulandı
- **Tarih:** 25 Kasım 2026
- **İlgili kayıt:** K-397, `urun-h` (ZeeMail `zeemail/disteslim`), ZM-F042,
  RFC-0044 ("Uygulama kararı — K-397", B3), ZM-F046, ADR-177 (DKIM
  doğrulama ilkeli, ortak `dkim_kripto`; gövde kanonikleştirme istisnası §8), ADR-173 (Dilim 1), ADR-105 eki / K-339 (credential
  sözleşmesi), ADR-059 (core freeze), ADR-064 (dil yüzeyi), spec/31, GB-177

## Bağlam

ZeeMail'in dış teslim hizmeti iletiyi kurar, MX'e bağlanır, STARTTLS yapar
(ADR-173) ve teslim eder. İleti DKIM imzasız çıktığı için alıcı DMARC'ta
hizalama bulamaz ve iletiyi istenmeyene atar. Alanın taklide karşı
`p=reject`'e geçmesi de bu yüzden engellenir (ZM-F042). Kanonikleştirme,
başlık seçimi ve `bh=` hesabı ürünün `dkim` biriminde zaten var; aynı kod
gelen postayı da doğrular (ADR-177). Dilde eksik olan iki şey vardır:
özel anahtarla imza atmak ve anahtarı güvenli biçimde yüklemek.

Özel anahtarla keyfî bayta imza atan bir ilkel **imza kâhini**dir. Hatalı ya
da ele geçirilmiş Zee kodu bu anahtarla başka bir alan adına ya da keyfî
içeriğe imza atabilir. Anahtar baytı Zee değeri, günlük, IO izi ya da tanı
olursa sızar.

## Karar

1. **Bildirim.** Anahtarlar `proje.dil` içinde bildirilir:
   ```text
   dkim_anahtarları "ornek.test z2026a ed25519-sha256 zeemail_dkim_anahtari", "ornek.test r2026a rsa-sha256 zeemail_dkim_rsa" listesi olsun
   ```
   - Her öğe `<alan> <seçici> <algoritma> <credential>` taşır.
   - Bildirim yalnız `kriptografi` yetkinliğiyle geçerlidir.
   - Alan ve seçici küçük harfli DNS adıdır; algoritma `rsa-sha256` ya da
     `ed25519-sha256` olur, `rsa-sha1` adıyla reddedilir.
   - Credential adı K-339 güvenli adıdır.
   - En çok 16 anahtar bildirilir. Aynı alan+seçici ya da aynı credential
     iki kez yazılamaz.
   - Credential, projedeki başka bir gizli değerin adıyla aynı olamaz:
     TLS sertifikası/anahtarı, gizli anahtar, HTTP gizlileri, e-posta ve
     veritabanı bağlantısı, veri kökü. Aksi hâlde imza kâhini TLS anahtarı
     üzerinde çalışırdı.
   - İhlal P015'tir; yeni tanı kodu yoktur.
2. **Açılışta yükleme.** `dil çalıştır` ve `dil iz kaydet` kaynak
   yürütülmeden önce her anahtarı K-339 sözleşmesiyle okur: önce systemd
   credential dizini, yoksa ortam; aynı ad ikisinde birden varsa ret.
   - Biçim: tek blok PKCS#8 PEM (`-----BEGIN PRIVATE KEY-----`). PKCS#1,
     şifreli PKCS#8 ve çok blok kabul edilmez.
   - RSA 2048–4096 bit (`ring::RsaKeyPair::from_pkcs8`). RFC 8301'deki
     SHOULD 2048 burada ZORUNLU'dur. Ed25519 için
     `Ed25519KeyPair::from_pkcs8_maybe_unchecked` kullanılır; OpenSSL'in
     açık anahtarsız v1 PKCS#8'i de kabul edilir.
   - Anahtar türü bildirilen algoritmaya uymalıdır.
   - Hata olursa süreç çıkış 2 ile durur. Tanı yalnız alanı, seçiciyi ve
     sabit nedeni anar; anahtar içeriğini anmaz.
   - Okunan PEM ve DER tamponları sıfırlanır. Bu en iyi çabadır: `zeroize`
     crate'i eklenmedi, `ring` anahtar çifti süreç belleğinde kalır.
   - `dil dene` hermetiktir ve anahtar yüklemez; imza C034
     `anahtar_bilinmiyor` döner.
3. **Yüzey.**
   ```text
   b veri verisi için "zeemail_dkim_anahtari" özel anahtarıyla "ed25519-sha256" olarak imzalamayı dene olsun
   ```
   Sonuç `Sonuç<Metin>`'dir. Başarıda değer `b=` değeridir: base64, satır
   kırma yok. Kalıp ADR-177 doğrulama kalıbının aynasıdır; `olarak` kelimesi
   oradan gelir. Anahtar Zee'de yalnız credential adıyla anılır.
4. **İmza kâhini önlemi.** Host kanonikleştirmeyi bilmeden imza girdisinin
   yapısını denetler (RFC 6376 §3.7):
   - Girdi en çok 256 KiB'dır, CR içermez ve satır sonuyla bitmez.
   - Girdi başlık alanlarından oluşur; devam satırları önceki alana aittir.
     Son alan `dkim-signature`'dır.
   - Etiketler tekildir. `v=1` olmalı, `bh=` dolu olmalı, `h=` `from`
     içermeli, `l=` olmamalı. `b=` boş olmalı ve son etiket olmalıdır.
   - Girdide tam olarak bir `from` alanı bulunur. Tırnak ve yorum dışında
     ya tek `<…>` ya da tek çıplak adres yazılır; adreste tek `@` olur.
     Bu adresin alanı `d=` ya da onun alt alanı olmalıdır (gevşek hizalama).
   - `i=` varsa onun alanı da `d=` altında olmalıdır.
   - `d=`, `s=` ve `a=` bildirilmiş anahtarla birebir eşleşmeli, `a=` de
     Zee'nin verdiği algoritmayla aynı olmalıdır.

   Denetim yorumlayıcıda anlamlı tanı için, adaptörde ise savunma
   derinliği için yeniden yapılır.
5. **Kriptografi.** Tek yüzey `ring`'dir (ADR-177 ile aynı kilitli sürüm).
   - İmza girdisi `kanonik_baytlar` ile LF→CRLF'ye çevrilir.
   - `rsa-sha256` RSASSA-PKCS1-v1_5 + SHA-256 ile imzalanır.
   - `ed25519-sha256` RFC 8463'e göre SHA-256(girdi) üzerinde PureEdDSA ile
     imzalanır.
   - İki algoritma da deterministiktir.
6. **Hatalar.** C034 `tür` değerleri ADR-177 ile ortaktır: `algoritma_reddedildi`,
   `algoritma_bilinmiyor`, `veri_bozuk`. Yeni değerler şunlardır:
   - `geçersiz`: kâhin önlemi; mesaj sabit nedeni verir.
   - `anahtar_bilinmiyor`: ad bildirilmemiş ya da anahtar yüklenmemiş.
   - `anahtar_uyuşmuyor`: `d=`, `s=` ya da `a=` bildirime uymuyor.

   Yetkinlik çalışma zamanında reddedilirse `tür=yetkinlik` döner.
7. **Deterministik IO.** IO izine `dkim_imzala [anahtar adı, algoritma,
   girdi SHA-256'sı] → b=` ya da `tür` yazılır. İzde girdinin kendisi,
   anahtar ve açık anahtar yoktur. Replay anahtarsız yapılır ve aynı imzayı
   izden verir.

8. **Dosyadan akışlı kanonik gövde özeti (ZM-F046; merkez kararı B, 25
   Kasım 2026).** K-390 B2 ürün sonucu (ZeeMail `cb047c5`, kayıt `d6705f1`):
   256 KiB üstü gövdenin `bh=`'si Metin ilkesiyle (`X gövdesinin dkim
   gövde özeti`) hesaplanamıyor, çünkü Metin zarfı aşılıyor. 400 KB ekli
   geçerli imzalı ileti bu yüzden `dkim=belirsiz` kalıp İstenmeyen'e düştü;
   giden tarafta da ekli ileti imzalanamıyordu. Gelen ileti K-384 blok
   dosyasında başlıklarıyla birlikte ham ve CRLF'lidir.
   ```text
   bh blok dosyasının gövde_başı baytından "relaxed" kipinde dkim gövde özetini almayı dene olsun
   ```
   Kalıp `Sonuç<Metin>` verir (base64 SHA-256).
   - Gövde `baş` baytından (0 ≤ baş ≤ dosya boyu) dosya sonuna kadardır.
   - `kip` yalnız `simple` (RFC 6376 §3.4.3) ya da `relaxed` (§3.4.4)
     olabilir.
   - Satır sonu LF ya da CRLF olabilir; LF'siz CR `veri_bozuk`tur. Özet
     kanonik CRLF baytları üzerinde alınır.
   - Dosya 16 KiB parçalarla akar; Metin oluşmaz, zarf harcanmaz.
   - İşlenen gövde en çok 32 MiB'dır (dosya SHA-256'sı ve K-395 ile aynı).
   - `veri:` yolu `veri-okuma` ister, diğer yollar `dosya-okuma`. Ayrıca
     `kriptografi` gerekir.
   - Ret C034 Sonucudur; dosya dış veri olduğu için duruş değildir. `tür`
     değerleri: `geçersiz` (kip, baş), `veri_bozuk`, `sınır`, `dosya`.
   - IO izine `dosya_dkim_govde_ozeti [yol, baş, kip] → bh=` yazılır;
     içerik izde yoktur.
   - `l=` desteklenmez: ilke gövdenin tamamını özetler. `l=` taşıyan gelen
     imzayı ürün reddeder; imza tarafında `l=` zaten yasaktır (§4).
   - **Gövde kanonikleştirmesi host'ta, başlık kanonikleştirmesi Zee'de.**
     Bu, ADR-177'nin "kanonikleştirme Zee'dedir" kararına dar bir
     istisnadır. Gövde kuralı RFC 6376'da iki satırlık ve iyi tanımlıdır;
     başlık seçimi ve kanonikleştirmesi ise ürün politikasıdır.
     Alternatif, büyük gövdeyi Zee'de satır satır ikinci bir dosyaya
     çevirmekti; bu yavaş, disk ister ve C024 zarf riskini açar. Host'ta
     tek ve test edilmiş bir uygulama daha güvenlidir.
   - Giden taraf aynı kalıbı `baş 0` ile kullanır. Ürün kanonik gövdeyi LF
     ya da CRLF satır sonuyla yazabilir; iki biçim de aynı özeti verir.
     ZeeMail giden gövdeyi CRLF'li yazar, çünkü aynı dosya SMTP'ye
     aynen akıtılır.
   - Metin ilkesi (K-390) değişmez; simple kip ve LF kanonik gövdede onunla
     aynı değeri verir.

9. **Açılışta yayımlanmış anahtar denetimi (K-413, 26 Kasım 2026).** Proje
   `dkim_anahtarları` ile birlikte `dns` yetkinliği ve çözücü bildiriyorsa,
   `dil çalıştır` ve `dil iz kaydet` anahtarları yükledikten sonra her
   anahtar için `<s>._domainkey.<d>` TXT kaydını bildirilmiş loopback
   çözücüye bir kez sorar (K-390 B1a istemcisi, 5 sn).
   - Kayıt RFC 6376 §3.6.1 anahtar kaydı olarak okunur: `v=` varsa ilk
     etiket ve `DKIM1`, `k=` varsayılanı `rsa`, `p=` zorunlu. Diğer TXT
     kayıtları yok sayılır.
   - `p=` (SPKI ya da PKCS#1; Ed25519'da ham 32 bayt) ADR-177'nin
     `AcikAnahtar` ayrıştırıcısıyla çözülür. Özel anahtarın açık parçasıyla
     birebir eşleşen bir kayıt varsa denetim geçer.
   - **Anahtar başına sonuç, süreç durmaz.** Eşleşen kayıt yoksa ve iptal
     edilmiş kayıt (boş `p=`) ya da başka anahtar yayımlanmışsa yalnız o
     anahtar devre dışı kalır. O anahtarla imza denemesi C034 tipli hata
     döner: `anahtar_iptal` ya da `anahtar_dns_uyuşmuyor`. Ürün o alanı
     imzasız gönderip yöneticiyi uyarabilir.
   - Açılışta stderr'e (systemd günlüğü) `HATA:` satırı yazılır: alan,
     seçici, kayıt adı, neden ve C034 türü. Anahtar baytı yazılmaz.
   - Kayıt yoksa ya da DNS geçici hata verirse `UYARI:` yazılır, anahtar
     etkin kalır: doğrulanamadı, uyuşmuyor demek değildir.
   - Zorunlu kip için proje alanı açılmadı (core freeze; gerçek ürün ihtiyacı
     çıkarsa ayrı işte).
   - `dns` bildirimi yoksa denetim yapılmaz; bu bilinçlidir, çünkü dil ancak
     bildirilmiş çözücüye sorgu atabilir. `dil dene` anahtar ve DNS
     yüklemez.
   - Gerekçe (core freeze `correctness`; ZeeMail'de ayrı ZM kaydı yok):
     - Bildirilen anahtarla yayımlanan anahtarın uyuşmaması, teslimde sessiz
       DKIM `fail` ve DMARC reddi demektir; hata en erken açılışta görünür
       olmalıdır (ADR-174 açık maddesi).
     - ZeeMail ürün geri bildirimi: çok kiracılı kurulumda
       `<s>._domainkey.<müşteri-alanı>` müşterinin DNS'indedir. Uyuşmazlıkta
       süreci durdurmak, tek bir kiracının `p=`'yi değiştirmesiyle bütün
       kiracıların dış postasını keserdi; müşteri denetimindeki veri hizmet
       reddine dönüşürdü. Bu yüzden sonuç anahtar başınadır.
   - Açılış denetimi süreç içi büyüyen durum bırakmaz (anahtar başına tek
     işaret; soak gerekmez). Sonuç IO izine girmez, çünkü Zee yürütmesinin
     parçası değildir; replay'de imza sonucu zaten izden gelir.

## Reddedilen seçenekler

- **İmzada da doğrulama sınırı (RSA 1024–4096).** Doğrulama tarafı (ADR-177)
  1024 bitlik anahtarı kabul etmek zorundadır (RFC 8301 §3.2: doğrulayıcı
  1024–4096 bitlik anahtarı doğrulayabilmeli). Ancak aynı bölüm
  imzalayanın en az 1024 bit (SHOULD 2048) kullanmasını ister; 1024 bit
  bugün kırılabilir kabul edilir ve birçok alıcı bu imzayı zayıf sayar.
  Algoritma kümesi ve 4096 üst sınırı iki tarafta aynıdır (`DkimAlgoritmasi`,
  `RSA_EN_COK_BIT` ortak). İmzada alt sınır 2048'dir (`RSA_IMZA_EN_AZ_BIT`).
  2048 bitten kısa anahtar sessizce kabul edilmez; açılışta "RSA anahtarı
  2048 bitten kısa (RFC 8301)" tanısıyla çıkış 2 verir (merkez önerisi,
  25 Kasım 2026).

- **Kanonikleştirmenin host'ta yapılması** (RFC-0044 taslağındaki "ileti
  dosyasını … alanı adına dkim ile imzalamayı dene"). Bu ürünün doğrulayıcı
  koduyla ikinci bir kanonikleştirme yazmak demekti; iki uygulama ayrışınca
  kendi imzamızı doğrulayamayız. Kâhin önlemi girdi yapısı üzerinden de
  kurulabildi.
- **Anahtar adı yerine alan+seçici ile çağrı.** Credential adı zaten
  bildirimdeki tekil anahtardır. Alan ve seçici girdideki `d=`/`s=`
  etiketlerinden okunup karşılaştırılır.
- **Yalnız dosya tutamağına imza.** Ürün ileti dosyasını değil, başlık
  bloğunu imzalar; gövde zaten `bh=` ile bağlıdır.
- **RustCrypto `rsa`.** RUSTSEC-2023-0071 (Marvin) nedeniyle aday değil.
  `ring` sabit zamanlıdır.
- **Tarifteki "kanonik LF dosya" biçimi (A).** Ürün gelen ham CRLF
  iletiyi önce Zee'de ayrı bir LF kanonik dosyaya çevirmek zorunda kalırdı
  (bkz. §8 gerekçe).
- **Metin zarfını büyütmek ya da Metin ilkesini parçalı yapmak.** 256 KiB
  sınırı bilinçli bir bellek zarfıdır; parçalı Metin API'si yeni bir durum
  nesnesi ister. Dosya tutamağı ürünün zaten diske yazdığı kanonik gövdeyi
  akışla okur.
- **Uyuşmazlıkta ya da iptalde çıkış 2** (K-413'ün ilk tasarımı). Çok
  kiracılı kurulumda bir müşterinin DNS kaydı bütün hizmeti durdururdu (§9).
- **Kayıt yokken ya da DNS hatasında anahtarı devre dışı bırakmak.** Geçici
  bir DNS kesintisi ya da henüz yayılmamış yeni bir seçici imzayı keserdi;
  doğrulanamamak uyuşmazlık değildir.

## Sonuçlar

- K-167 yüzeyi büyür: yeni bir contextual Sonuç kalıbı ve
  `dkim_anahtarları` proje alanı eklenir. Yeni yetkinlik, tür ve tanı kodu
  yoktur.
- Güvenlik sınırı GB-177 ile izlenir. Kalan riskler:
  - Ürün kendi alanı adına yanlış içeriği imzalayabilir. Gönderim
    yetkilendirmesi ürünündür.
  - Süreç belleği ve çekirdek dökümü risk taşır. ZeeOS birimine
    `LimitCORE=0` önerilir.
- Fuzz: `dkim_imza_girdisi` hedefi (K-411, 26 Kasım 2026) RC fuzz kapısındadır:
  kanonik gövde özetleyicisi ve imza kâhini denetimi her RC adayında 30–60
  dakika AddressSanitizer altında çalışır (docs/fuzz-rc.md).
  Birim testleri kenar durumları ve kâhin girdilerini kapsar.
- Soak gerekmez. Anahtar deposu süreç başına bir kez kurulan sabit
  `OnceLock`'tur; önbellek ya da büyüyen durum yoktur.
- Açık işler:
  - Gerçek alıcıda `dkim=pass` ve DMARC `pass` (insan adımı, RFC-0044 B3
    kabul 2).
  - `dil dkim-denetle` adayı.
