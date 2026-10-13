# Zee çekirdek gerçek ürün kanıtı (K-208/K-209)

Kaynak depo: ürün deposu (özel)
Exact temiz ürün commit'i: `ebca9a6ddcc9c7535ee3204cfe2af3bbb7fc95d7`. Talep yolu `docs/dil-talepleri/2026-10-13-bcrypt-ve-istemci-adresi.md`;
SHA-256 `f8e258637469d0c9b7fea93cd0f2f724a2b616ffc910012a017ea5df8be658c5`.
Özgün talep özel ürün deposunda korunur; [burada anonim özeti](talep.md) yer alır. Etkilenen gerçek ürün yolları
`cekirdek/kaynak/kimlik.dil` ve `cekirdek/kaynak/rota_kimlik.dil`dir.
Aynalar tam ürün kopyası değil, K-172 gereksinim/dogfood sürtünme kesitidir;
üretim SQL/veri/parola hash'leri kopyalanmaz. legacy panel legacy bcrypt giriş
geçişi ve Site F audit IP talebi bu committe belgelenmiştir.
K-164 kataloğuyla karşılaştırma: F031 DB transport ihtiyacı tekrar
gerçeklenmez; legacy hash biçimi ve mevcut trusted request IP'sinin
handler'a verilmesi yeni dar sınıflardır. Migration/denetim tablosu ve
ikili sunucu yükseltmesi ürün tarafındadır; burada yapılmaz.

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `cekirdek/kaynak/kimlik.dil` | `42d099f6c0d88fdd54c4365142d68334ecd74b0f` | `5335bb01e6d860be1522c2ca34ff43735637345d6f01e31aca0385e354d44539` |
| `cekirdek/kaynak/rota_kimlik.dil` | `28c66395842d4aa9a3650b7fcf5c33f38b7c66e3` | `72345fd3dd15f893d30da470cad62b368701e7ccbbdc754382d4cc5bb795ea35` |
