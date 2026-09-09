# ADR-075 — Site E dogfood'u: parola özeti, TOTP, SMTP ve görsel küçültme yüzeyleri

- **Durum:** kabul
- **Tarih:** 9 Eylül 2026
- **İlgili kayıt:** K-181, `dogfood/urun-e`, K-174 (syntax freeze), spec/11, spec/12, spec/23

## Bağlam

`dogfood/urun-e` zee ile yazılan ilk müşteri sitesidir (Site E, ürün E
Gıda grubu; gerçek ürün `adminpanelzee` çatalı). K-163 legacy panel panelinin
"Desteklenmeyen" listesindeki dört madde burada gerçek müşteri ihtiyacı olarak
tekrar etti ve zee v0.7 ile kapanamadı:

1. Yönetici tarayıcıda parola belirleyemiyor; PHC üretimi `dil parola-özeti`
   CLI köprüsüne bağlı (P0-03 açık farkı).
2. İki adımlı doğrulama yok; panelde "Kullanılamıyor" rozetiyle duruyor.
3. İletişim formu mesajları e-posta bildirimi üretemiyor (SMTP istemcisi yok).
4. Yüklenen görseller küçültülemiyor; müşteri her görseli elle hazırlıyor.

Aynı ihtiyaçlar iki bağımsız üründe (legacy panel, ürün E) tekrar ettiği için K-180'deki
"tek üründeki sürtünme anekdot, iki üründe tekrar eden sürtünme tasarım
problemidir" ölçütünü karşılar. Ürün sahibi 9 Eylül 2026'da bu dört yüzeyin
freeze penceresi içinde açılmasına açıkça karar verdi; K-174 kaydı bu ADR ile
bilinçli olarak güncellendi.

## Karar

Dört yüzey merkezi intrinsic kaydına (ADR-011) yeni kimliklerle eklenir; AST
büyümez, her yüzey mevcut yetkinlik ve etki modelinden geçer:

| Kaynak yüzeyi | Kimlik | Yetkinlik | Etki | Dönüş |
|---|---|---|---|---|
| `<parola>nın argon2 özeti` | `guvenlik.parola_ozeti` | kriptografi | saf | Metin (PHC) |
| `yeni totp anahtarı` | `guvenlik.totp_anahtari` | kriptografi | saf | Metin (Base32, 160 bit) |
| `<kod> <gizli> anahtarıyla doğrulanıyorsa` | `guvenlik.totp_dogrula` | kriptografi | saf | Mantıksal |
| `<alıcı> <konu> <gövde> ile ileti göndermeyi dene` | `iletisim.eposta_gonder` | ağ | **geri alınamaz** | TamSayı Sonucu |
| `<alıcı> <konu> <gövde> ile <bağlantı> üzerinden ileti göndermeyi dene` | `iletisim.eposta_gonder_baglantili` | ağ | **geri alınamaz** | TamSayı Sonucu |
| `<kaynak> görselini <hedef> dosyasına <N> piksele küçültmeyi dene` | `gorsel.kucult` | dosya-yazma | dış yazma | TamSayı Sonucu (yazılan bayt) |

- **Bağlantılı gönderim (9 Eylül 2026 eki):** ürün E paneli SMTP ayarlarını yönetici
  arayüzünden (veritabanı) düzenlemek istedi; ortam değişkeni tek yol olunca
  panel "mail ayarları" sunamıyordu. `... ile <bağlantı> üzerinden ileti
  göndermeyi dene` bağlantı metnini değerden alır; `smtps://`/`smtp://` şema,
  `ağ_hedefleri` allowlist'i, çözülmüş IP denetimi ve oran sınırı ortam
  değişkeni yoluyla birebir aynıdır. IO izinde bağlantı yalnız SHA-256 parmak
  iziyle kaydedilir (parola taşır); `eposta_gonder` olayı 4 alanlıdır, bağlantı
  verilmediğinde ilk alan boştur. Hermetik profil bağlantıyı yalnız ayrıştırır.
- Argon2id üretimi `argon2 0.6` (zaten bağımlılık) ile CSPRNG tuzlu PHC'dir.
  `zee-io-1` hermetik profilinde tuz tohumlu sayaçtan gelir; aynı program aynı
  özeti üretir ve `doğrulanıyorsa` ile doğrulanır. Web isteğinde giriş oran
  sınırı (`Giris`, "parola") uygulanır; playground üretmez (C028).
- TOTP RFC 6238/HMAC-SHA1/Base32 bağımlılıksız gerçeklenir; adaptörün saati
  (native: sistem saati, hermetik: sabit `zaman`) ±1 adım pencere verir.
  Sabit zamanlı karşılaştırma; bozuk anahtar/kod `yanlış`.
- SMTP istemcisi bağımlılıksızdır (RFC 5321/5322/2047): `smtps://` implicit
  TLS, `smtp://` STARTTLS zorunlu, düz metin yalnız loopback. Sır proje
  bildiriminde değil `e_posta_bağlantı_değişkeni` ortam değişkenindedir;
  sunucu `ağ_hedefleri` exact origin allowlist'ine `smtps://host:465` olarak
  girer ve K-127 DNS sonrası IP kapısından geçer. Gönderim `IntrinsicEtkisi::
  GeriAlinamaz`dır: eylem içinde T048, rota/CLI adaptöründe serbest.
- Görsel küçültme `image 0.25` (jpeg/png/webp, cargo-deny temiz) ile süreç
  içindedir: 16 MiB kaynak, 64 MP piksel, 8192 kenar sınırı; hedef biçimi
  uzantıdan (`.jpg` kalite 82, `.png`, `.webp` kayıpsız); büyütme yapılmaz.
  Yol politikası dosya-okuma/dosya-yazma sınırlarıdır.
- `AgSemasi` `smtp`/`smtps` alır; bildirimsiz geliştirici ağında yalnız
  şifreli şemalar (https, smtps) kendiliğinden izinlidir.

## Kanıt ve açık sınır

`compiler/tests/urun_e_yetkinlik_testi.rs` lowering, hermetik davranış,
yetkinlik kapısı olumsuzu, T048 ve tür olumsuzlarını taşır; `guvenlik.rs`
RFC 6238/2202/4648 ve `eposta.rs` RFC 2047 vektörleri birim testtedir;
`dogfood/urun-e` `dil dene` ile hermetik koşar. Gerçek SMTP sunucusuna
gönderim ve gerçek tarayıcı 2FA akışı ürün deposunda doğrulanır.

Açılmayanlar: HTML/ek dosyalı e-posta, SMTP oturum havuzu, TOTP dışı 2FA
(WebAuthn), görsel kırpma/filigran, kayıplı WebP kodlama. Bunlar yeni ürün
kanıtı olmadan CORE FREEZE altında kapalıdır.
