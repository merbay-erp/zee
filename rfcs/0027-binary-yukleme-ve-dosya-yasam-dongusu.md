# RFC-0027 — Binary yükleme ve dosya yaşam döngüsü

- **Durum:** geçici kabul
- **Tarih:** 3 Eylül 2026
- **İş:** K-163/F030
- **Karar:** ADR-061
- **Normatif yüzey:** spec/26

## Sorun

Çatlı dogfood'u metin dosyası + PostgreSQL metadata uzlaştırmasını F029'da
kanıtladı. F030 gerçek binary gövde, akışlı özet, temp→final yayın, tombstone,
fiziksel temizlik ve orphan taraması istedi. Eski adaptör bütün gövdeyi `Vec`
içinde topluyor ve UTF-8'e zorluyordu; Zee kodu binary dosyayı doğrulayamıyor,
atomik yayımlayamıyor veya beklenen silme hatasını `Sonuç` olarak alamıyordu.

## Karar

Native adaptör yalnız exact `application/octet-stream` için 16 MiB'lık ayrı
zarf açar; diske ve SHA-256'ya tek geçişte akar. Rotaya ham byte değil güvenli
temp yol/hash/boyut verilir. Genel runtime'a dört dar, capability kontrollü
dosya intrinsic'i eklenir: no-clobber atomik taşıma, idempotent silme, tek
dizin listeleme ve akışlı SHA-256.

Dil DB+dosya atomikliği iddia etmez. F030 ürünü ayrı eylemler, kalıcı durum
makinesi ve restart reconciliation kullanır. Read/list/hash tekrar edilebilir;
write otomatik retry edilmez.

## Kanıt ve sonuç

`dosya_yasam_dongusu_testi.rs`, `kalici_dosya` birim testleri, binary akış
birim testi, faz matrisi ve gerçek PG16.11/TCP dogfood deneyi sözleşmeyi taşır.
Process temp yazımında, metadata hazırlığından sonra, rename'den sonra ve
tombstone'dan sonra öldürüldüğünde restart tek kanonik sonuca yakınsamalıdır.

Multipart, büyük nesne deposu, virüs tarama ve production pool/TLS ayrı açık
kanıttır; bu RFC onları tamamlanmış göstermez.
