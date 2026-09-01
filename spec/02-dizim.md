# 02 — Dizim

Normatif kaynak: RFC-0003 (girinti/blok), RFC-0006 (işlem/çağrı). Tanı
kodları: S bölümü.

## Yüklem-sonlu dağıtım (TANIMLI)

zee'de gramer birimi **cümledir** ve cümlenin türünü **son kelimesi**
belirler: `... yaz`, `... olsun`, `... tekrarla`, `... döndür`, `... sor`,
`... ekle`, `... artır/azalt/böl`, `... bitir`, `... bekle`, `... başlat`,
`... gönder`, `... yak/söndür`, `... kullan`, `... olmalı`, `... sil`,
`... yönlendir` (yaz'ın hedefli biçimleri: `dosyasına`, `çerezine`). Türkçenin
yüklem-sonlu doğası ayrıştırma kuralının kendisidir; sembolik işleç ve
parantez **YASAK**tır (S001) — anlam kelimelerle kurulur.

Son kelimesi hiçbir bilinen kalıba uymayan satır S004 ile reddedilir ve
tanı desteklenen kalıpları önerir. Belirsizlik dile giremez: bir satır ya
tek biçimde ayrışır ya hatadır (RFC-0001).

## Blok yapısı (TANIMLI — RFC-0003)

- Girinti birimi **4 boşluktur**; sekme **YASAK** (S003).
- Blok açan satırlar: `... ise` / `değilse` (+zinciri), döngüler, `işlem`,
  `yapı`, `test`, `... göre`, `eşzamanlı olarak`, `... geldiğinde`,
  `... içinde` / `yetişmezse`.
- Blok açan satırdan sonra girintili en az bir satır **ZORUNLU** (S007).
- Bir satırın girintisi, açık bloklardan birinin hizasına inmiyorsa
  **YASAK** (S005).
- `değilse` yalnız bir `... ise` bloğunun hemen ardından ve aynı hizada
  gelebilir (S031).

## Üst düzey kuralı

`işlem`, `yapı` ve `test` tanımları yalnız en dış düzeyde **ZORUNLU**dur
(S021). İşlem adları dosya başından ön-taranır: işlem, kendisini çağıran
satırdan sonra da tanımlanabilir; karşılıklı özyineleme geçerlidir.

İşlem gövdesinin başındaki parametre satırı başlangıç yüzeyinde `<ad> al`,
açık sözleşmede `<ad> <tür yazımı> olarak al` biçimindedir (K-083):
`sayıyı Ondalık olarak al`, `fiyatları Ondalık listesi olarak al`. Bir işlemde
açık ve çıkarımlı satırlar karıştırılamaz (T037). Tam sözleşmede bunların
hemen ardından `<Tür> döndürür` ya da `değer döndürmez` gelir (K-086/spec-10).

## İşlem çağrısı (geçici kabul — RFC-0006, K-016)

Çağrı, argümanların ardından `için` / `ile` ayracı ve işlem adıyla kurulur:

```
kare 4 için karesini hesapla olsun
"Ayşe" ve 10 ile selamla
```

Birden çok argüman `ve` ile ayrılır; her `ve`/`ile` dilimi tam bir ifade
bölgesidir (K-038). Bu yüzey usability onay kapısını bekler; alternatifi
RFC-0006'da karşılaştırılmıştır. V1 kararı, gerçek kullanıcı verisi öncesinde
önden bağlanan [K-016 karar paketindeki](../docs/k016-cagri-karar-paketi.md)
tek-genel-sözdizimi ve alt grup eşiklerini geçmeden bu normatif geçici yüzeyi
değiştirmez veya tam kabul saymaz.
