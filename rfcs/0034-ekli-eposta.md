# RFC-0034 — Ekli SMTP e-posta (K-232)

- **Durum:** geçici kabul (K-232 teknik dilimi; canlı ürün kabulü açık)
- **Tarih:** 14 Ekim 2026
- **İlgili:** ADR-096, spec/23, spec/24, GB-058
- **Ürün:** zee-cekirdek B2/P10, exact harici talep `cd5d12f0008620819e6167af5146e83503fd2b38`

Ürün yerel katalog PDF’ini TR/EN düz metin+HTML ileti ve Reply-To ile mevcut
SMTP panel ayarından göndermek ister. [Exact anonim ürün kanıtı](../dogfood/zee-cekirdek/mime/README.md)
ve [Zee reproducer](../dogfood/zee-cekirdek/mime/katalog-talep.dil) bu ihtiyaçtır.
Mevcut HTTP transport tek başına binary PDF/Base64 dosya surface’i sağlamaz.
Tek additive kalıp seçilir; yeni temel değer türü ya da queue grammar’ı açılmaz.

```zee
sonuc alici konu duz html yanit ekler baglanti ile ekli ileti göndermeyi dene olsun
```

İmza `[Metin, Metin, Metin, Metin, Metin, MetinListesi, Metin]` →
`TamSayı Sonucu`, intrinsic `iletisim.eposta_mime_gonder`, etki
`GeriAlinamaz`, yetkinlik `ağ`dır. Ekli gönderim DosyaOkuma’yı runtime’da
ayrıca ister; DosyaYazma istemez. Boş HTML/yanıt/ekler ilgili opsiyonu
kaldırır; boş bağlantı mevcut declared SMTP ortam ayarını kullanır.
Başarı 1, hata ayrıntısı maskelenmiş C029’dur. Eylem içinde T048; kayıtta
çağrı hiç yürütülmez, export ve replay kapalıdır. WASM/playground desteklenmez.

MIME profili `zee-smtp-mime-1`: text/plain; HTML varsa multipart/alternative,
ek varsa dış multipart/mixed; Base64 76 sütun/CRLF; Unicode başlık RFC2047,
dosya adı RFC2231 UTF-8 extended parameter. Kontrollü ek türleri
PDF/PNG/JPEG/WebP; yol/uzantı türü belirler, içerik taraması yapmaz.
Kaynak sınırları merkezi `KaynakSinirlari.eposta()` görünümündedir.
Eski plain kalıp, tel biçimi, hata/süre davranışı K-232’de değişmez.

Karar ve olumlu/olumsuz kanıt [ADR-096](../adr/096-ekli-eposta-ve-mime.md).
Normatif protokol kaynakları [RFC2045](https://www.rfc-editor.org/rfc/rfc2045),
[RFC2046](https://www.rfc-editor.org/rfc/rfc2046),
[RFC2047](https://www.rfc-editor.org/rfc/rfc2047),
[RFC2231](https://www.rfc-editor.org/rfc/rfc2231),
[RFC5321](https://www.rfc-editor.org/rfc/rfc5321).
K-174 syntax gözlem penceresi 14.10.2026–11.11.2026 olarak aynı gün bilinçli
yeniden hesaplanır; RC/insan review ve canlı ürün kapıları açık kalır.
