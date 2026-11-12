# ADR-061 — Binary yükleme ve dosya yaşam döngüsü sahipliği

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İş:** K-163/F030

## Karar

HTTP framing/content-length/timeout ve temp spool native web adaptörünün;
atomik publish/delete kalıcı dosya çekirdeğinin; tür/etki/yetkinlik intrinsic
kaydının; metadata durum makinesi ve uzlaştırma ise ürünün sahibidir.

Yorumlayıcı IO trait'i binary buffer taşımaz. Rota yalnız proje-kökü göreli
temp yolunu, hash'i ve boyutu görür. Böylece dil heap zarfı yükleme boyutuna
bağlanmaz, binary payload string'e dönüştürülmez ve ürün DB dışı yan etkiyi
açık saga olarak yönetir.

`ToplayanIo` ayrı modüle çıkarılarak kök yorumlayıcı bütçesi korunur. Yeni
işlemler deterministik adaptör, capability sargısı ve IO trace/replay boyunca
aynı sözleşmeyle taşınır. Genel bir dosya sistemi API'si veya gizli DB+dosya
transaction'ı açılmaz.

## Ek — K-329: aygıtlar arası taşıma (21 Ekim 2026)

Atomik taşıma yalnız `rename(2)` kullanıyordu. Üretim unit'leri
`ProtectSystem=strict` ile `ReadWritePaths=…/kaynak/.zee …/kaynak/medya …`
yollarını ayrı bind mount ettiği için aynı disk üzerinde bile rename `EXDEV`
(errno 18) veriyor, panel yüklemeleri ve 4.534 eski medyanın taşınması
`dosya atomik taşınamadı: Invalid cross-device link` ile başarısız oluyordu
(zee-cekirdek talebi W5).

- **Karar.** Talebin (a) seçeneği: `kalici_dosya::atomik_tasi` rename
  `ErrorKind::CrossesDevices` (Unix `EXDEV`, Windows `ERROR_NOT_SAME_DEVICE`)
  döndürürse aynı hedef klasörü kilidi altında hedef klasöründe geçici dosyaya
  akışla kopyalar, replace yolunun metadata sözleşmesiyle (mod, sahip,
  ACL/xattr) kaynağın bilgisini uygular, fsync eder, aynı klasörde atomik
  adlandırır, klasörü fsync eder ve en son kaynağı siler. Görünürlük atomik
  kalır, no-clobber sözü ve EXDEV dışındaki hata yolları değişmez; tanı
  kodu, grammar ve `dil::api::v1` aynıdır.
- **Hata yolu.** Yayından önce her arıza geçiciyi siler, kaynağa dokunmaz.
  Hedef klasörü eşzamanlanamaz ya da kaynak silinemezse yayımlanan hedef geri
  çekilir; iki kopya ya da yarım dosya kalmaz. Kaynak silindikten sonra
  kaynak klasörünün fsync arızası rename yolundaki gibi hata döner, veri
  hedefte bütündür.
- **Açılış öz-sınaması reddedildi.** Geri düşüş doğruluğu sağladığı için
  “aynı dosya sistemi” önkoşulu yalnız performans konusudur (yükleme başına
  ≤ 16 MiB kopya). Açılışta `.zee/yuklemeler` → hedef sınaması üretim
  dizinlerine yazar, yeni bir CLI uyarı yüzeyi (K-167 uyumluluk yüzeyi) açar
  ve rotaların seçtiği keyfi hedef klasörlerini kapsayamaz. Ürünün kendi
  `ExecStartPre` öz-sınaması ürün sahipliğinde kalır.
- **Reddedilen:** (b) ayrı hata türü/`nedeni` alanı (kullanıcıya çözüm yerine
  dağıtım bilgisi yükler), hard link ile yayın (bazı dosya sistemlerinde yok),
  Linux'a özel `copy_file_range`/`renameat2` yolu (`io::copy` zaten uygun
  çekirdek kopyasını seçer).
- **Kanıt:** `kalici_dosya::yasam_dongusu` birim testleri (EXDEV enjeksiyonu,
  yayın arızasında temizlik, EXDEV dışı hata değişmez, Linux `/dev/shm` gerçek
  aygıt sınırı), regression `runtime/aygitlar-arasi-tasima`
  (`native-dosya-exdev` kipi: gerçek `dil` süreci, `medya` tmpfs'e bağ;
  düzeltme öncesi `taşıma reddedildi`).

## Ek — K-332: geçici yükleme dosyası isteğe aittir (12 Kasım 2026)

zee-cekirdek 1.18.0 ziyaretçiye açık CV yükleme rotası (`/basvuru-yukle`)
ekledi; uçtan uca sınamada CSRF'siz octet-stream isteğinin 403 almasına
rağmen tam gövdeli `.parca` bıraktığı görüldü. Doğrulamada beş yol da dosya
bırakıyordu: CSRF/oturum yok, sahte oturum, 404, kısa gövde, rotanın dosyayı
taşımadan 200 vermesi. Gövde, rota ve güvenlik denetimi yorumlayıcıda
koşmadan önce diske aktığı için oturumsuz istemci istek başına 16 MiB yetim
dosya bırakarak diski doldurabiliyordu; temizlik yalnız ürünün açılış
süpürgesine kalmıştı (GB-129, yüksek).

- **Karar.** Temp spool'un sahibi native adaptör olduğu için silme sözü de
  onundur. `AkanYukleme` düşerken kısmi dosyayı siler; `tamamla` bir
  `IstekYuklemesi` koruması döndürür, adaptör onu istek boyunca tutar ve
  `istek_islemini_tamamla`/`istek_islemini_geri_al` (yedek: sonraki
  `istek_al`) onu düşürür. Rota dosyayı aynı istekte taşıdıysa yol boştur ve
  silme etkisizdir; eylem geri alması dosyayı geri getirdiyse silinir.
- **Ön kapı.** Yorumlayıcı güvenli olmayan her yöntemde CSRF ve oturum ister
  (`!guvenli()`), bu yüzden `GET`/`HEAD` dışındaki binary yükleme tek ve boş
  olmayan `X-Zee-CSRF` ile oturum çerezi taşımıyorsa adaptör gövdeyi okumadan
  403 döner. Belirtecin oturumla karşılaştırılması rota denetiminde kalır;
  sahte belirteçli istek akar ve istek sonunda silinir. Gözlenebilir fark:
  böyle bir istek var olmayan adreste 404 yerine 403 alır.
- **Reddedilen:** rota tablosunu ve oturum deposunu gövdeden önce native
  katmanda çözmek (yorumlayıcı rota yolu ifadelerini istek anında
  değerlendirir; güvenlik transaction'ını gövde okumasından önce açmak her
  okuma hatasında ayrıca geri alma ister), geçici yolu istekler arası
  tutmak (bilinen tüketici yok; disk sınırı ölçülemez kalır), yalnız açılış
  süpürgesine güvenmek.
- **Açık sınır.** Süreç istek ortasında ölürse (ölümcül tanı, sinyal) kalan
  dosya restart uzlaştırıcısına kalır. Eşzamanlı akan yükleme sayısı bağlantı
  izni sınırıyla zaten sınırlıdır.
- **Kanıt:** `web_yukleme` ve `main.rs` birim testleri, regression
  `security/reddedilen-yukleme-parcasi` (`native-web-yukleme` kipi: gerçek
  `dil` worker'ı, altı senaryo; düzeltme öncesi `csrf-yok` 400 ve `parça 5`).
