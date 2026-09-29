# ADR-078 — Migration SQL seviye ayrıştırması

- **Durum:** kabul
- **Tarih:** 29 Eylül 2026
- **İlgili kayıt:** K-187, spec/25

## Bağlam

Zee migration'ı tek dış transaction içinde uygular ve migration dosyasının
kendi `BEGIN`, `COMMIT`, `ROLLBACK` veya `SAVEPOINT` komutunu taşımasını
yasaklar. Eski denetim SQL'i noktalama işaretlerinden bölüyordu. Bu nedenle
PL/pgSQL işlev gövdesindeki zorunlu `BEGIN`, yorumlar ve metin sabitleri yanlış
biçimde dış transaction komutu sayılıyordu.

## Karar

Migration denetimi yalnız üst seviye SQL tokenlarını inceler. Tek tırnaklı
metinler, çift tırnaklı identifier'lar, satır yorumları, iç içe blok yorumları
ve PostgreSQL dollar-quoted gövdeleri taramadan çıkarılır. Üst seviyedeki dört
yasak transaction komutu aynı C026 davranışıyla reddedilmeyi sürdürür.

## Sonuç

Trigger ve PL/pgSQL işlevleri migration içinde güvenle tanımlanabilir. Zee'nin
tek transaction sahipliği gevşemez; bu bir SQL parser'ı veya genel migration
DSL'i değildir. Kapanmamış quote/body PostgreSQL tarafından ayrıca reddedilir.
