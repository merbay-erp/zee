# ADR-099 — SMTP hata maskelemesi (K-236)

- **Durum:** kabul (K-236)
- **Tarih:** 14 Ekim 2026
- **Güvenlik bulgusu:** GB-060, yüksek, K-236 ile kapalı
- **İlgili:** ADR-007, ADR-021, spec/23, C029

## Bulgudan düzeltmeye

Eski SMTP oturumu AUTH reddinin ilk sunucu satırını transport Err metnine
katıyordu; runtime C029 Sonuç bunu ham geçiriyordu. İzinli sunucu kendi aldığı
AUTH PLAIN Base64 satırını `535 denied <payload>` ile echo edince kullanıcı
çıktısı geri çözülebilir sentetik kullanıcı/parolayı içerdi. Native TCP
reproducer bu committed davranışı doğruladı; URL veya müşteri sırrı taşınmadı.
İlk kaynak `cbacc4c09cf0534b7436f33d771a30ce687d510f` olup harici ürün merge’i
`de24878d3b361705d8dd7b6bb3dd165e32f99503` ana hatta bu adaptörü taşımıştır.

Public native `eposta::gonder` bütün transport hatalarını aynı sabit metne
eşler. Runtime eski ortam/explicit bağlantı kalıpları da host/custom IO
hatasını bu metne eşler; URL ayrıştırma veya policy hatası transport’tan
önce gelse dahi kullanıcıya ham ayrıntı gitmez. C029 ve TamSayı Sonucu
değişmez. Maskenin ayar-geçersiz veya belirsiz-gönderim açıklaması sender’ın
teslim edildi/edilmedi garantisi vermediğini korur; otomatik retry eklenmez.

İleti encoder, tel baytları, AUTH yöntemi, TLS/deadline ve failure kontrol
akışı değişmez; yalnız dışarı dönen hata ayrıntısı kaldırılır. Yeni grammar,
intrinsic, Rust facade export, cache veya kalıcı tampon yoktur.
MIME K-232’de zaten aynı güvenli sonuç sınırını taşır. Inherited SMTP yanıt
okuma bütçesi ayrı K-237/ADR-100/GB-061 işidir; bu düzeltme onu kapatmaz.

## Kanıt ve kalan sınır

[Tek arızalı Zee regression](../regression/security/smtp-auth-echo-sir.dil)
`smtp` kipi ile gerçek native göndericiyi ve localhost AUTH 535 echo sunucusunu
yürütür; arıza sahte başarısızlık değeriyle değil gerçek SMTP yanıtıyla üretilir.
`smtp_hata_maskelemesi_testi` public native sender, explicit URL parse hatası
ve native CLI üzerinden aynı Zee reproducer’ında C029/hata maskesi, ham veya
Base64 sentetik credential yokluğu ve retry/mail/data yapılmamasını sınar.
Sözleşme sonucunda kullanıcı kaynakta açık `yaz` ile paylaştığı sır için taint
sistemi eklenmez. Canlı sağlayıcı/özel ürün kabulü bu hermetik test değildir.

Mimari owner graph ve Cargo/toolchain değişmez; kaynak AST/runtime maskesi
mevcut eposta/yorumlayici sahiplerindedir. Benchmark tarihçesi/iş yükü ve
ADR-045/049/069 kalıcı state sınırı değişmez. Kaynak commit’i security bugfix;
exact regression fixed_by ve compiler/freeze beyanı kaynak commit’ini izleyen
provenance commit’inde bu tam SHA’ya bağlanır. Tanı kimliği ve LSP/grammar yüzeyi eklenmez.
