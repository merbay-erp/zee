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
