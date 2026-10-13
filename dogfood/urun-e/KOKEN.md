# ürün E ürün kökeni — K-210–K-213

Harici ürün: ürün deposu (özel)

Exact ürün commit'i: `79eb72e00ecddd11535a5a9de9c338db1dd6da70`.
Asıl çekirdek talep kaynağı: `zee-cekirdek` exact
`6de572260daea805fa80e480dac1ba110c37a492` (13 Ekim 13:45 eki).
ürün E compiler kaynak hattı `ci/urun-e-dil` exact
`34fbc0aab7498c53af6fbee0c534966e024a14e4`; compiler tabanı
`bcc17b4406ee9a088777577c0ae366378e6fe691` harici ürün SHA'sı değildir.
İlk ana-hat ürün kaydı bu ayrımı düzeltir.

Reproducer `kimlik-gereksinimi.dil`, ürün panelinin parola üretimi/TOTP,
SMTP ayarları, gizli SMTP parolası, görünüm ve görsel gereksinimlerini
hermetik küçültmeyle taşır. Gerçek kimlik bilgisi/veritabanı taşınmaz.
Ürün kaynakları `kaynak/kimlik.dil`, `kaynak/rota_kimlik.dil`,
`kaynak/eposta_ayarlari.dil`, `kaynak/rota_kullanicilar.dil`;
ürün E `zee/profil.py` normal `zee-catal` üretiminde `aynen=True` kullanır.

Ana hattaki numara eşlemesi ve minimalite kararı
[ADR-086](../../adr/086-site-hatlarinin-birlestirilmesi.md) içindedir.
