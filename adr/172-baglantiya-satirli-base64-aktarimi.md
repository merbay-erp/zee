# ADR-172 — Bağlantıya satırlı base64 aktarımı ve kodlanmış çıktı aralığı

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-396, ZM-F040, [RFC-0039](../rfcs/0039-ag-sunucusu-yetkinligi.md)
  Aşama A2d, ADR-165 (bağlantı yüzeyi, A2c eki), ADR-167 (RFC-0043 Dilim 1
  `kodlama`), ADR-170 (paylaşılan veri kökü), ADR-059 (core freeze),
  ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu), spec/12 §9, GB-171;
  ek: K-408, ZM-F049, ADR-174 §8, spec/26, GB-181

## Bağlam

`urun-h` posta ürünü ekleri çözülmüş hâlde paylaşılan veri kökünde tutar
(`veri:ekler/<kiracı>-<sha256>`, ADR-170). IMAP istemcileri `BODY[n]`
gövdesini RFC 2045 base64 satırları olarak bir literal içinde, kısmi
getirmede (`BODY[n]<o.n>`) kodlanmış çıktının bir dilimi olarak ister.
Var olan parçalarla tek yol 57 baytlık `base64 okumayı dene` döngüsüydü:
her satır dosyayı yeniden açar ve bir Zee `Metin`i üretir. 16 MiB'lık ek
29,6 sn sürüp C024 (kümülatif değer zarfı) ile kesiliyordu; ürün bu yüzden
büyük ekleri sunamıyordu (ZM-F040).

Core freeze yeni yüzeyi varsayılan olarak reddeder. Ürün kaydı etkin,
reproducer depo içinde (`dogfood/urun-h/imap-govde-base64/`); yeni tür,
anahtar kelime, koşul grameri ya da tanı kodu gerekmiyor.

## Karar

1. **Yüzey.** A2c'nin dosya aktarımının kardeşi iki kalıp:
   `<yol> dosyasını base64 satırlarıyla bağlantıya aktarmayı dene` ve
   `<yol> dosyasının base64 satırlarının <konum> baytından <adet> baytını
   bağlantıya aktarmayı dene` → `Sonuç<TamSayı>` (yazılan bayt). Yeni kalıp
   kelimeleri `satırlarıyla`, `satırlarının`; intrinsic'ler
   `ag.baglanti_base64_aktar`, `ag.baglanti_base64_aralik_aktar`
   (`ağ-dinleyici`, geri alınamaz etki).
2. **Biçim ve formül.** RFC 4648 §4 dolgulu alfabe, son satır dışında tam
   76 karakter, her satır CRLF ile biter; e = 4·⌈b/3⌉, T = e + 2·⌈e/76⌉
   normatiftir (spec/12 §9). Ürün literal boyunu veritabanındaki ham
   boydan hesaplar.
3. **Aralık kodlanmış akış üzerindedir.** Dilim [min(K,T), min(K+N,T));
   K ≥ T boş başarı; negatif değer `geçersiz`. Gerçekleme satır başına
   hizalı ham konuma (`⌊K/78⌋·57`) `seek` eder; büyük konumlu kısmi getirme
   dosyanın başını okumaz.
4. **Akış ve sınırlar A2c'den miras.** Çıktı `Metin` olmaz, değer zarfına
   sayılmaz; bellekte en çok bir 57·256 baytlık ham parçanın kodlaması
   durur. 64 KiB giden tampon geri basıncı, 256 KiB tur dilimi, çağrı
   başından mutlak `boşta + yazılan bayt / 64 KiB/sn` bütçesi, karşı taraf
   kapanınca/süre dolunca/dosya kısalınca kırık bağlantı ve `yazılamadı`.
   `bağlantı_blok_sınırı` ham boya uygulanır. Kaynak dosya tavanı ayrıca
   32 MiB'tır (K-395 yazma tavanıyla hizalı; blok sınırı 1 GiB'a
   çıkarılsa da aşılmaz): tam 32 MiB geçer, +1 bayt hiç bayt yazılmadan
   `boyut_aşımı` döner.
5. **Yol ve yetkinlik tek merkezden.** Kaynak `dosya-okuma`/`veri-okuma`
   ile merkezî yol denetiminden geçer (statik ve çalışma anı); `veri:`
   yaprağı bağ izlenmeden açılır (ADR-170). IO izi yalnız yol ve aralığı
   (`aktar_b64`, `tum` ya da `K+N`) taşır.
6. **Tek kodlayıcı.** Satır düzeni ve pencere hesabı saf `kodlama`
   modülündedir (ADR-167); reaktör ve hermetik adaptör aynı işlevleri
   çağırır. `kodlama` CLI adaptörüne `#[doc(hidden)]` iç modül olarak
   açılır (`dil::api::v1` dışı; katman kenarı `dil_cli → kodlama`).

## Neden RFC-0039 ve bu ADR

Güvenlik ve doğruluk sınırı bağlantı akışıdır (geri basınç, süre, yarım
yazımda eşzamanın bozulması, C030). RFC-0043 süreç içi dosya ↔ `Metin`
dönüşümünü ve değer zarfını yönetir; A2d tam olarak o zarfın dışında
kalmayı gerektirir. ADR-165 A2c ekine eklemek yerine ayrı ADR: formül ve
aralık semantiği ürünün SQL'ine sızan kalıcı bir sözleşmedir.

## Sonuçlar

- 16 MiB + 5 baytlık ikili ek gerçek TCP üzerinden bayt bayt doğru,
  hata ayıklama derlemesinde 0,87 sn (eski yol 29,6 sn + C024).
- Ham dosyanın kısmi aktarımı (`BODY[]<o.n>` ham baytı) açık kalır.
- Quoted-printable kodlama ve satır uzunluğu parametresi reddedildi.
- Yeni ayrıştırıcı yok (sabit konumlu kalıp eşleşmesi); yeni fuzz hedefi
  gerekmez. Satır kodlayıcı ve pencere `kodlama::testler` ile her kenarda
  dilim eşitliğiyle sınanır.

## Ek — dosya hedefli ayna (K-408, ZM-F049, 26 Kasım 2026)

**Bağlam.** `urun-h` giden iletileri DKIM ile imzalıyor (K-397 Dilim 2,
ADR-174). Eksiz ileti bellekte imzalanabilir; ekli iletide ek parçaları
yalnız SMTP'ye akarken (`bağlantıya aktarmayı dene`) oluşur, gövde hiçbir
yerde bütün olarak durmaz. ADR-174 §8 dosyadan akışlı gövde özeti
(`dosyasının 0 baytından "relaxed" kipinde dkim gövde özetini almayı dene`)
bu yüzden kullanılamıyor; ekli dış ileti imzasız gidiyordu. Var olan
`base64 okumayı dene` + `dosyaya ekle` döngüsü ZM-F040'taki 57 baytlık
döngünün aynısıdır (her satır bir `Metin`, C024).

**Karar.**

1. **Yüzey.** `<kaynak> dosyasını base64 satırlarıyla <hedef> dosyasına
   eklemeyi dene` → `Sonuç<TamSayı>` (yazılan bayt). İntrinsic
   `dosya.base64_satirlari_ekle` (`dosya-yazma`, dış yazma etkisi; kaynak
   ayrıca `dosya-okuma`). Yeni kalıp kelimesi, tür ya da tanı kodu yok:
   `satırlarıyla` K-396'dan, `dosyasına eklemeyi dene` K-389'dan gelir.
2. **Bayt eşitliği.** Satır düzeni ve formül §2 ile aynıdır (T = e +
   2·⌈e/76⌉); kodlayıcı tek: `kodlama::base64_satirlari_kodla`. Aynı
   kaynağın bağlantıya ve dosyaya çıktısı bayt bayt aynıdır.
3. **Akış ve atomiklik.** Kaynak 57·256 baytlık parçalarla okunup kodlanır,
   çıktı `Metin` olmaz, değer zarfına sayılmaz. Ekleme spec/08 / K-389
   `eklemeyi dene` modelidir: hedef kilidi altında mevcut içerik bağ
   izlenmeden okunur, geçici dosyaya önek + kodlanmış akış yazılır,
   `fsync` ve tek atomik `rename` ile yayımlanır. Kaynak okuma sırasında
   bildirilen boyundan kısa ya da uzun çıkarsa hiçbir bayt yayımlanmaz.
4. **Tavanlar.** Kaynak 32 MiB (K-395/K-396/K-401 ile aynı), hedefin
   eklemeden sonraki toplamı 32 MiB (K-395 çözerek ekleme hedef tavanı ve
   ADR-174 §8 dosya gövde özeti tavanı). Sonuç olarak bir çağrıyla en çok
   ≈ 23,4 MiB ham ek kodlanabilir; 25 MB iletinin ~18 MB eki (kodlanmış
   ~24,6 MB) sığar. Tam sınır kabul, bir bayt fazlası hedefe dokunmadan
   C013. Eylem içinde C013 (akışlı ekleme bir çalışma dosyasıdır, K-389).
5. **Yetki ve iz.** İki yol ayrı ayrı merkezî denetimden geçer: kaynak
   `dosya-okuma`/`veri-okuma`, hedef `dosya-yazma`/`veri-yazma` (statik +
   çalışma anı; ADR-170). IO izi `dosyaya_base64_satirlari_ekle` olayı
   yalnız `[kaynak, hedef]` ve yazılan bayt sayısını taşır.
6. **Katman.** `kalici_dosya → kodlama` kenarı (ikisi de altyapı; yön
   temel→temel, yeni SCC yok).

**Neden bu ADR'nin eki.** Biçim, formül ve kodlayıcı A2d sözleşmesidir;
değişen yalnız hedef (bağlantı yerine dosya). RFC-0043 dosya ↔ `Metin`
dönüşümünü yönetir; bu ek tam o zarfın dışında kalmayı gerektirir.
ADR-174 §8 tüketicidir, üreticiyi taşımaz.

**Sonuçlar.** Ekli giden iletinin gövdesi tek dosyada kurulur ve `bh=`
aynı dosyadan alınır. Kanıt: `compiler/tests/base64_satirlari_ekle_testi.rs`
(boş, 57/76 kat, tek bayt, önek korunumu, 32 MiB tam sınır/aşım, eylem,
yol kaçışı, dizin, `veri:` okuma/yazma ayrımı, T054, içeriksiz iz ve
replay), `ag_baglantisi_surec_testi` (aynı kaynağın TCP ve dosya çıktısı
bayt bayt eşit), `kalici_dosya::cozulmus` (boy değişiminde yayımlamama,
geçici dosya kalmaz). Quoted-printable dosya hedefi ve aralıklı dosya
hedefi bilinçli dışarıda.
