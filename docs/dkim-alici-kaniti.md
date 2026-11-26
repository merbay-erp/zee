# Gerçek alıcıda DKIM ve DMARC kanıtı

K-416, [RFC-0044](../rfcs/0044-dns-posta-kimlik-dogrulama-ve-teslim-guvenligi.md)
B3 kabul ölçütü 2: Zee'nin imzaladığı ileti gerçek bir alıcıda `dkim=pass`
ve DMARC hizalaması `pass` almalı. Bu kanıt bir **insan / operatör adımıdır**.
DNS kaydını yayımlamak ve gerçek bir alıcıya ileti göndermek otomatik
oturumların ya da CI'ın işi değildir. Bu depo yalnız yöntemi, denetleyiciyi
([`scripts/dkim-alici-kaniti.py`](../scripts/dkim-alici-kaniti.py)) ve anonim
kayıt defterini ([`dkim-alici-kanitlari-v1.tsv`](dkim-alici-kanitlari-v1.tsv))
taşır. İlgili kararlar: [ADR-174](../adr/174-dkim-imza-ilkeli.md) (imza
ilkeli, §9 açılış denetimi) ve [spec/31](../spec/31-dns-ve-posta-kimlik-dogrulama.md) §4.

## Gizlilik sınırı

- Açık depoya gerçek alan adı, seçici, alıcı adresi, IP, `Received` zinciri
  ya da ileti gövdesi **girmez** (açık depo kapısı). Kayıt satırında alan
  `<kurum-alanı>`, seçici `<seçici>` yazılır.
- Gerçek `.eml` ve alan adı ürünün özel deposunda (ZeeMail ya da sürüm kanıt
  deposu) saklanır. Açık satır ona yalnız `.eml`'in SHA-256'sıyla bağlanır.
- Denetleyici yalnız bu anonim alanları basar; hata iletileri de ad taşımaz.

## Adımlar

1. **Anahtar ve TXT kaydı.** Özel anahtarı üretin. ZeeMail'de
   `araclar/dkim-anahtar-uret.sh` PKCS#8 PEM, 0600 izin ve RSA-2048 ya da
   Ed25519 üretir. Anahtarı `dkim_anahtarları` bildirimindeki credential
   adıyla systemd credential olarak verin. `dil dkim-denetle` henüz yalnız
   adaydır (K-415), bu yüzden TXT kaydını elle hazırlayın:
   ```sh
   # Ed25519 (RFC 8463): p= ham 32 bayt
   openssl pkey -in anahtar.pem -pubout -outform DER | tail -c 32 | base64
   # RSA: p= SubjectPublicKeyInfo
   openssl pkey -in anahtar.pem -pubout -outform DER | base64 | tr -d '\n'
   ```
   Kayıt: `<seçici>._domainkey.<alan>. IN TXT "v=DKIM1; k=ed25519; p=…"`.
   RSA-2048 kaydı tek dizgeye sığmaz; 255 baytlık parçalara bölün.
2. **Yayım (operatör).** Kaydı alanın DNS'ine koyun, TTL kadar bekleyin.
   Alanın DMARC kaydı (`_dmarc.<alan>`) en az `p=none` olmalıdır.
3. **Açılış denetimi.** Proje `dns` de bildiriyorsa `dil çalıştır` açılışta
   kaydı denetler (K-413). `HATA:` satırı görürseniz kayıt anahtara uymuyor
   demektir; gönderimden önce düzeltin.
4. **Gönderim (operatör).** ZeeMail'in dış teslim yoluyla (`disteslim`)
   kendi alanınızdan en az bir büyük sağlayıcıdaki (Gmail, Outlook) test
   kutusuna düz ve ekli birer ileti gönderin.
5. **Ham iletiyi alın.** Gmail'de "Orijinali göster" → "Orijinali indir";
   Outlook'ta "İleti kaynağını görüntüle". Dosyayı `.eml` olarak **özel**
   depoya koyun.
6. **Denetleyin ve kaydedin.**
   ```sh
   python3 -I scripts/dkim-alici-kaniti.py ileti.eml --saglayici Gmail \
       --zee-sha "$(git rev-parse HEAD)"
   ```
   - Çıkış 0 ise satırı `docs/dkim-alici-kanitlari-v1.tsv` sonuna ekleyin.
   - Çıkış 1 ise ileti kanıt değildir; neden stderr'dedir (`dkim=fail`,
     `dmarc=fail`, hizasız `header.d`).

## Denetleyicinin kuralları

- Yalnız **en üstteki** `Authentication-Results` başlığı okunur. Bu
  başlığı alıcı ekler (RFC 8601 §5). Alttaki başlıklar göndericiden ya da
  aradan gelmiş olabilir ve yok sayılır.
- Yorumlar (`(…)`) ve tırnaklı dizgeler atılır. Birden çok `dkim=` sonucu
  varsa `pass` olan seçilir.
- Hizalama, From alanıyla `header.d`'nin aynı ya da birbirinin alt alanı
  olmasıdır (PSL'siz yaklaşık). Asıl hüküm alıcının kendi `dmarc=pass`
  sonucudur.
- Kanıt için `dkim=pass`, `dmarc=pass` ve hizalama birlikte gerekir.
- Betik yalnız Python standart kitaplığıyla, `-I` ile çalışır ve ağa
  çıkmaz. İmzayı yeniden doğrulamaz: bağımsız doğrulama ZeeMail'in
  `disteslim-deneme` paketindedir, buradaki kanıt alıcının kendi hükmüdür.

## Kayıt defteri

[`dkim-alici-kanitlari-v1.tsv`](dkim-alici-kanitlari-v1.tsv) append-only'dir.
Sütunlar: `tarih`, `saglayici`, `alan` (`<kurum-alanı>`), `secici`
(`<seçici>`), `dkim`, `spf`, `dmarc`, `hizalama`, `eml_sha256`, `zee_sha`.
Kayıt yokken B3 kabul 2 açıktır; ilk satır sürüm kanıtına bağlanır.
