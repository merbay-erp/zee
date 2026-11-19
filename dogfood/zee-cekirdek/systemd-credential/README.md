# K-339 — Proje gizli değerleri için systemd credential kaynağı

Bu anonim kesit aktif `zee-cekirdek` ürününün (bkz. [KOKEN](../KOKEN.md))
ürünleşme talebinin D3 maddesine bağlıdır. Ürün deposu özeldir; müşteri,
alan adı, sunucu yolu ve gerçek sır taşınmaz (ADR-087). D3'ün K-245 host
köprüsü kaydı [credential/](../credential/README.md) altındadır.

Exact harici ürün commit'i: `abaca3aba0e90f57d26435c4befffa903346c425`
(18 Kasım 2026).

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `docs/dil-talepleri/2026-10-14-zee-urunlesme.md` (D3, "systemd credentials") | `6b2fa83bca9e15a1778b0fb6ad3f15b5d7388b73` | `694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7` |
| `cekirdek/proje.dil` (etkilenen SMTP/gizli anahtar/DB ortam bağları) | `f7e939ce513cd78b3a3ebed80b5f0040edd23391` | `310ef82a60af6827e5b828852f178faa0d7962a18210452a329610222722c169` |
| `moduller/icerik-botu/isci/icerik-botu-isci.service` (ürünün bugünkü `LoadCredential=` + `EnvironmentFile=` dağıtımı) | `bb4d885cdee4d71ff1b974a23d1e4229f4f9aaa3` | `d601eca7e76bfef3656ec6058e6eced8cb3e789a90277dcf9d626c56d4aae577` |

## Sürtünme

Ürün siteleri systemd unit'leriyle koşar. Zee native CLI gizli değerleri
yalnız ortamdan okuduğu için sırlar unit dosyasında ya da `EnvironmentFile=`
içinde durur ve süreç ortamıyla alt süreçlere miras kalır. K-245 host
köprüsü dosyayı yine çocuğun ortamına çevirir (GB-068 kabul edilen sınır);
systemd credential'ın "ortamda değil, servis ömürlü özel dosyada" avantajı
kaybolur. Ürünün Zee dışı işçisi aynı sırrı zaten `LoadCredential=` ile
alıyor. K-164 kataloğunda karşılığı K-245'in açık bıraktığı "file-native
sır" sınırıdır.

## Minimal değişiklik

[Reproducer](ana.dil) mevcut `gizli_anahtar_değişkeni` bağını değiştirmeden
kullanır. Native CLI `CREDENTIALS_DIRECTORY` ayarlıysa aynı adı credential
kimliği sayıp `$CREDENTIALS_DIRECTORY/UYGULAMA_GIZLI_ANAHTAR` dosyasını
okur; değişken yoksa davranış birebir eskisidir. Yeni sözdizimi, ayar alanı,
yetki, tanı kodu veya public API yoktur. Sözleşme
[ADR-105 eki](../../../adr/105-credential-host-koprusu.md),
[spec/23 §K-339](../../../spec/23-yetkinlik-ve-outbound-guvenligi.md).

`dil denetle .` bu klasörde hermetik geçer; gerçek dizin/gerçek ikili
kanıtı `compiler/tests/systemd_credential_testi.rs`, dosya sözleşmesi
`compiler/src/guvenlik/sir_dizini/testler.rs`.

## Açık

Ürün unit'lerinin `LoadCredential=`'a geçişi ürün tarafındadır. Gerçek
systemd servis kurulumuyla canlı kabul, Windows karşılığı ve çalışan süreçte
sır rotasyonu bu işin sözü değildir.
