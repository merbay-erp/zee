# urun-e — dördüncü dogfood ürünü (K-181)

Site E (ürün E) kurumsal sitesi ve yönetim paneli; zee ile
yazılan ilk müşteri sitesidir. Gerçek ürün `[ürün deposu (özel)]`
deposundadır (`adminpanelzee` / K-163 legacy panel çatalı, 9 Eylül 2026). Bu klasör,
ürünün derleyiciyi genişletmesini gerektiren dört ihtiyacın depo içi
reproducer aynasıdır:

| İhtiyaç | Üründeki yer | Yüzey |
|---|---|---|
| Tarayıcıda parola belirleme (CLI `dil parola-özeti` köprüsü yerine) | Kullanıcılar / Güvenlik | `parolanın argon2 özeti` |
| İki adımlı doğrulama | Giriş / Güvenlik | `yeni totp anahtarı`, `kod gizli anahtarıyla doğrulanıyorsa` |
| İletişim formu e-posta bildirimi | `/iletisim-gonder` rotası | `alıcı konu gövde ile ileti göndermeyi dene` |
| Panelden SMTP ayarı (E-posta sayfası) | `/yonet/eposta` rotası | `alıcı konu gövde ile bağlantı üzerinden ileti göndermeyi dene` |
| Yüklenen görselin küçültülmesi | Medya yükleme | `kaynak görselini hedef dosyasına N piksele küçültmeyi dene` |

`dil denetle .` ve `dil dene .` bu klasörde hermetik koşar. Kanıt: ADR-075,
`compiler/tests/urun_e_yetkinlik_testi.rs`, K-181 günlük kaydı.
