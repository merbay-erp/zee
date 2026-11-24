# ADR-172 — Bağlantıya satırlı base64 aktarımı ve kodlanmış çıktı aralığı

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-396, ZM-F040, [RFC-0039](../rfcs/0039-ag-sunucusu-yetkinligi.md)
  Aşama A2d, ADR-165 (bağlantı yüzeyi, A2c eki), ADR-167 (RFC-0043 Dilim 1
  `kodlama`), ADR-170 (paylaşılan veri kökü), ADR-059 (core freeze),
  ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu), spec/12 §9, GB-165

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
   `bağlantı_blok_sınırı` ham boya uygulanır.
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
