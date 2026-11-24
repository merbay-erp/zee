# 26 — Binary yükleme ve dosya yaşam döngüsü

Bu bölüm K-163/F030 dogfood gereksiniminin açtığı native web ve kalıcı dosya
sözleşmesini tanımlar (RFC-0027, ADR-061).

## Binary HTTP gövdesi

`Content-Type: application/octet-stream` taşıyan native HTTP isteği en çok
16 MiB olabilir. Adaptör gövdeyi bütünüyle belleğe ALMAZ; 16 KiB parçalarla
göreli IO kökündeki (spec/07 K-382: giriş kaynağının klasörü) `.zee/yuklemeler` dizininde `0600` izinli, benzersiz `.parca`
dosyasına yazar ve SHA-256'yı aynı akışta hesaplar. Başarılı framing sonunda
rota isteği şu alanları görür:

- `yukleme_gecici_yolu`: göreli IO köküne göre güvenli geçici yol (`.zee/yuklemeler/<64 hex>.parca`);
- `yukleme_sha256`: küçük harfli 64 haneli SHA-256;
- `yukleme_bayti`: onluk byte sayısı.

Unsafe rota CSRF belirtecini tek `X-Zee-CSRF` başlığından alır. Birden çok
`Content-Type`, sınırı aşan `Content-Length`, kısa/fazla gövde ve okuma son
tarihi mevcut fail-closed HTTP reddini korur.

K-332 uyarınca `GET`/`HEAD` dışındaki binary yükleme tek ve boş olmayan
`X-Zee-CSRF` başlığı ile boş olmayan oturum çerezi taşımıyorsa, gövde okunmadan
ve diske hiç akmadan `403` ile reddedilmek ZORUNDADIR; yorumlayıcı bu isteği
zaten reddedeceği için ön kapı rota eşleşmesinden önce çalışır. Belirtecin
oturumla sabit zamanlı karşılaştırılması rota denetiminde kalır.

Geçici `.parca` dosyası isteğe aittir ve native adaptör onu silmek ZORUNDADIR:
akış yarıda kalırsa (kısa gövde, okuma son tarihi, yazma/fsync hatası) hemen;
tamamlanmış yükleme ise istek bittiğinde — CSRF/oturum/Origin reddi, oran
sınırı, 404/405, rota tanısı ya da rotanın dosyayı taşımadan yanıt vermesi
dahil. Rota dosyayı aynı istekte taşıdıysa silme etkisizdir; geçici yol
sonraki isteklerde geçerli değildir. Yalnız süreç istek ortasında sonlanırsa
(ölümcül tanı, sinyal, çökme) dosya yetim kalır ve aşağıdaki yaş tabanlı
süpürmeye düşer.

Native web worker, `sunucu başlat` dinleyiciyi kurduktan sonra ve çalışırken
en sık 5 dakikada bir, `.zee/yuklemeler` altında yalnız runtime adlandırmasına
(64 küçük onaltılık karakter + `.parca`) uyan, sembolik bağ olmayan ve son
değişikliği 15 dakikadan eski normal dosyaları silmek ZORUNDADIR. Eşik, canlı
bir yüklemenin son yazımından sonra yaşayabileceği en uzun sürenin (gövde okuma
son tarihi, rota çalıştırma bütçesi ve yanıt yazımı) en az on katıdır; aynı
kökü paylaşan başka worker'ın canlı yüklemesi kilit olmadan korunur ve
eşzamanlı süpürmeler birbirini bozmaz. Başka adlı dosyaya, alt dizine, bağa ve
gelecek tarihli dosyaya dokunmak YASAKTIR; tek tur en çok 10 000 girdi inceler.

## Multipart form yüklemesi — K-378

Normatif kaynak: [RFC-0041](../rfcs/0041-ikili-yanit-ve-ek-akisi.md) katman B,
[ADR-163](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md). JavaScript'siz
HTML formu (`enctype="multipart/form-data"`, gizli `_csrf` alanı dosya
alanından önce) tek dosya yükleyebilir.

- Yol yalnız proje bildirimiyle açılır: `çok_parçalı_yükleme doğru olsun`
  (`web-oturumu` yetkinliği ZORUNLU, aksi P015; mantıksal olmayan değer P001).
  Bildirimsiz projede ve `GET`/`HEAD` isteğinde multipart gövde bugünkü gibi
  64 KiB metin/form yolunda kalır. `application/octet-stream` + `X-Zee-CSRF`
  yolu her iki durumda da değişmez. Bu seçim TANIMLIDIR.
- `Content-Type` tek ve `multipart/form-data` olmak; tek `boundary`
  parametresi (RFC 2046 karakterleri, 1–70) taşımak ZORUNDADIR; başka ya da
  yinelenen parametre 400'dür. `Content-Length` 16 MiB + 64 KiB'ı aşarsa gövde
  okunmadan 413 döner. Oturum çerezi yoksa gövde okunmadan 403 döner.
- Gövde tek geçişte, belleğe alınmadan ayrıştırılır. En çok 8 parça, tam bir
  dosya parçası; parça başlık bloğu ≤ 8 KiB ve ≤ 8 satır; yalnız
  `Content-Disposition: form-data; name=…[; filename=…|filename*=UTF-8''…]` ve
  isteğe bağlı `Content-Type` tanınır. `Content-Transfer-Encoding`, bilinmeyen
  başlık ya da parametre, başlık katlaması, yinelenen alan adı, `yukleme_` ile
  başlayan alan adı, UTF-8 olmayan metin alanı, 1 KiB'ı aşan
  önsöz/sonsöz, sınırdan sonra CRLF/`--` dışı bayt, iki dosya ve
  kapanış sınırı olmadan biten gövde YASAKTIR (400). 16 MiB'ı aşan dosya 413.
- K-391 ([ADR-168](../adr/168-istek-boyut-sinirlari-ve-govde-asimi.md)):
  metin alanları sıradan form alanıdır. Dosya parçası isteğe bağlıdır;
  yalnız metin alanlı form kabul edilir ve rotaya urlencoded formla aynı
  `istek` alanlarını verir. Alan başına tavan yoktur; bütün metin alanı
  değerlerinin ham UTF-8 bayt toplamı isteğin form bütçesini (64 KiB) aşarsa
  akış 413 ile durur. Türkçe metin bu yolda yüzde kaçışıyla şişmez. Dosyasız
  formda CSRF yorumlayıcının olağan `_csrf` form denetimindedir. Bu sözleşme
  ZORUNLUDUR.
- `_csrf` metin alanı dosya parçasından önce gelmek ve oturumun belirteciyle
  sabit zamanlı eşleşmek ZORUNDADIR; aksi hâlde dosya baytı diske hiç
  yazılmadan 403 döner (oturum gövdeden önce yüklenir; başarısız deneme CSRF
  oran sınırına sayılır). Bu yolda `X-Zee-CSRF` başlığı yok sayılır.
- Dosya parçası yukarıdaki binary gövdeyle aynı yükleyiciye akar (`.parca`,
  `0600`, SHA-256, istek sonu silme, yaş süpürmesi); red yanıtından ve
  spec/11 §5 gövde aşımı rotasından önce kısmi dosya silinir. Dosya
  parçası varsa rota bugünkü üç alana ek olarak `yukleme_dosya_adi`
  (doğrulanmış basename; geçersizse boş — CR/LF, tırnak, yön karakteri, `..`)
  ve `yukleme_bildirilen_tur` (beyan edilen tür; güvenilmez, ASCII ≤ 127 bayt
  ya da boş) alanlarını ve diğer metin alanlarını `istek` sözlüğünde görür.
  Dosya adı hiçbir koşulda disk yoluna girmez. Bu sözleşme ZORUNLUDUR.
- Okuma 10 sn mutlak istek son tarihine (spec/11 §5) tabidir; orantılı okuma
  bütçesi bu sürümde AÇIK sınırdır.

## Beklenen dosya işlemleri

```text
kaynak dosyasını hedef yoluna atomik taşımayı dene
yol dosyasını silmeyi dene
dizin yolundaki dosyaları listelemeyi dene
yol dosyasının sha256 özetini almayı dene
yol dosyasının ilk N baytını onaltılık okumayı dene
```

İlk iki ifade `TamSayı sonucu`, listeleme `Metin listesi sonucu`, özetleme
`Metin sonucu` döndürür. Beklenen IO arızaları C013 kodlu `Hata` değeridir;
worker'ı düşürmez. Taşıma var olan hedefi ASLA ezmez ve aynı dosya sistemi
içindeki rename görünürlük sınırını kullanır. Silme bulunmayan hedefte başarılı
`0`, sildiğinde `1` döndürür. Listeleme yalnız dizinin doğrudan normal
dosyalarını, göreli ve sıralı verir; symlink/dizin döndürmez. Özetleme en çok
16 MiB'ı akışla okur.

K-329 uyarınca kaynak ile hedef ayrı bağlama noktalarındaysa (`rename`
`EXDEV` verir; ör. systemd `ReadWritePaths=` her yolu ayrı bind mount eder)
runtime taşımayı hedef klasöründeki geçici dosyaya akışlı kopya, kaynağın
izin/sahip/ACL/xattr bilgisi ve fsync, aynı klasörde atomik adlandırma, hedef
klasörü fsync ve en son kaynağın silinmesiyle tamamlamak ZORUNDADIR. Hedef
yine tek adımda bütün olarak görünür; yayından önceki arızada geçici dosya
silinir ve kaynak korunur, kaynak silinemezse yayımlanan hedef geri çekilir.
EXDEV dışındaki arızalar ve no-clobber reddi bugünkü C013 sonucunu verir.
Geri düşüş kopya maliyetini taşır; yükleme dizini ile hedefi aynı bağlama
altında tutmak hızlı yoldur.

K-333 (RFC-0036, ADR-159) uyarınca imza okuması dosyanın ilk N baytını
küçük harf, ayraçsız onaltılık `Metin sonucu` olarak verir; dosya daha
kısaysa olan kadarını, boş dosyada `""` döndürür. N TamSayı olmak
ZORUNDADIR (aksi T034 tür hatası) ve 1 ile 64 arasında olmak ZORUNDADIR;
dışındaki değer dosyaya dokunmadan C013 başarısız Sonuç verir. Yol kuralı
özetlemeyle aynıdır; normal dosya olmayan hedef C013 ile reddedilir. Okuma
`dosya-okuma` yetkinliğine bağlıdır ve IO izinde `dosya_ilk_baytlari` olayı
exact yol + N argümanıyla yer alır. İmza eşleşmesi içeriğin zararsız olduğunu
kanıtlamaz; tür tablosu ve karşılaştırma ürünün sorumluluğundadır.

Bu işlemler sırasıyla `dosya-yazma`/`dosya-okuma` yetkinliklerine bağlıdır,
IO trace/replay'de exact argüman ve sonuçla yer alır ve eylem savepoint'ini
korur. Aynı eylem içinde PostgreSQL ve dosya yazımı yine YASAKTIR.

## Açık sınır

Dosya sistemi ile PostgreSQL arasında dağıtık ACID sözü yoktur. Ürün
`hazırlanıyor → hazır/hatalı` ile `siliniyor` durumlarını, iş anahtarını ve
restart uzlaştırmasını kurmak zorundadır. Birden çok dosya alanı, iç içe
`multipart/mixed`, antivirüs/içerik türü doğrulaması, production TLS/proxy yük limiti ve nesne deposu bu bölümün
sözü değildir.
