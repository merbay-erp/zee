# 13 — Sürümlü morfoloji profili

Normatif kaynak: RFC-0018 (geçici kabul), K-089/K-122/K-123. Bu bölüm spec/03'teki ad
çözümünün makinece sabitlenen profilini tanımlar.

## Profil kimliği (TANIMLI)

Zee v1'in etkin profili **`zee-tr-1`**, sayısal sürümü `1`, azami ek katmanı
`2`dir. Tek kaynak-of-truth `compiler/src/morfoloji.rs: EK_TABLOSU`dur.

| Soyut ek | Kabul edilen yüzeyler |
|---|---|
| belirtme | `yı yi yu yü nı ni nu nü ı i u ü` |
| üçüncü tekil iyelik | `sı si su sü ı i u ü` |
| tamlayan | `nın nin nun nün ın in un ün` |
| yönelme | `ya ye na ne a e` |
| ayrılma | `ndan nden dan den tan ten` |
| bulunma | `nda nde da de ta te` |
| araç | `yla yle la le` |
| çoğul+yönelme | `lara lere` |

İyelik tek başına ad kullanım eki sayılmaz; yalnız iki katmanlı zincirin iç
ekidir. Geçerli zincir:

```text
iyelik + (belirtme | tamlayan | yönelme | ayrılma | bulunma | araç)
```

Örnekler: `fiyat+ı+yla`, `zar+ı+ndan`, `elma+sı+ndan`.

## Çözüm algoritması (TANIMLI)

1. Ham kelime kapsamdaki ada byte/Unicode kod noktası olarak doğrudan eşitse
   o ad kazanır; ek çözümü yapılmaz.
2. Değilse profil tablosuyla bütün tek katman çözümleri çıkarılır.
3. Dış yüzey atıldıktan sonra yalnız geçerli zincir biçimi için ikinci katman
   denenir. Üçüncü katman v1'de üretilmez.
4. Her gövde doğrudan adaydır. Ayrıca şu geri çevrimler aday üretir:
   `b→p`, `c→ç`, `d→t`, `ğ/g→k`; son ünsüz ikizini teke indirme ve bunun
   ardından sertleştirme; son iki ünsüz arasına son ünlüye uyumlu dar ünlü.
5. Yinelenen kök adayları tekilleştirilir fakat farklı kökler korunur.
6. Kapsamda 0 eşleşme A001, 1 eşleşme o ad, 2+ eşleşme A002'dir. Alan
   çözümünde aynı ilke T028 sınırında uygulanır. Hiçbir aday öncelik veya
   sözlük puanıyla seçilmez.

## Kanonik üretim (TANIMLI)

`ek_uydur` ve `ek_zinciri_uydur` son ünlüye göre ikili/dörtlü uyumu, `y/n/s`
tamponlarını, `ta/te` sert ünsüz benzeşmesini ve ünlüyle başlayan ekte
çok-heceli `p→b`, `ç→c`, `t→d`, `k→ğ` ile `nk→ng` dönüşümünü uygular.

Üretim sözleşmesi yüzey metninin birebir tersini vermek değildir. İkizleşme
ve ünlü düşmesi sözlüksel olduğu için çözücü kabul edebilir; sözlüksüz üreteç
tek düzenli kanonik biçimi üretir. Normatif property şudur:

```text
çöz(üret(kök, ek-zinciri)) içinde (kök, ek-zinciri) vardır
```

Bu property profil tablosundaki bütün tek ekler ve bütün geçerli iki katmanlı
zincirler için kök korpusu üzerinde test edilir.

## Editör ve araçlar (TANIMLI)

- LSP tanıma-git/hover/rename aynı profil modülünü kullanır. Definition ve
  rename önce başarılı checker HIR'ındaki `SymbolId`yi seçer; morfoloji
  semantic kimliği belirlemek için kullanılmaz.
- Rename, yalnız seçilen `SymbolId`ye ait tek çözümlü ek zincirini yeni köke
  kanonik olarak giydirir. Kaynak kök+zincir çözümü belirsizse veya belge
  derlenemiyorsa tahmin ederek düzenleme üretmez.
- `dil morfoloji` profil snapshot'ını; `dil morfoloji <kelime>` bütün yapısal
  kök+ek çözümlerini gösterir.
- `dil sürüm` etkin profil kimliğini gösterir.

## Proje ve paket sabitlemesi (ZORUNLU)

Yeni `proje.dil` dosyası şunu taşır:

```text
morfoloji "zee-tr-1" olsun
```

Alanı olmayan eski bildirim `zee-tr-1` kabul edilir. Yazılmış ama bu
derleyicide desteklenmeyen profil P011'dir; sessiz fallback YASAKTIR.
`proje.kilit` sürüm 3, ana proje ve her yerel/uzak paket satırında profil
kimliğini taşır. Sürüm 2 tarihsel yerel-paket biçimidir.

Yüzey tablosu, zincirler, ses dönüşümü, kanonik üretim veya çözüm önceliği
değişirse `zee-tr-1` yerinde değiştirilemez. Yeni profil kimliği ve ana dil
sürümü/edition geçiş kararı gerekir.

`zee-tr-1`in yayımlanmış uyumluluk kaydı **ZORUNLU** olarak şudur:

```text
profil=zee-tr-1
şema=zee-morfoloji-uyumluluk-v1
sha256=e6034e7359e5d5d1bf5f3f06b6a7767616b2b6daf8ab916226b48d522f99f220
```

Kayıt; profil dökümünü, 11 sabit sınır yüzeyinin bütün çözümlerini ve 4.096
deterministik kökün yedi tek+altı iki katmanlı zincirindeki 53.248 kanonik
üretim ile bütün çözüm kümelerini özetler. Aynı profil kimliği altında başka
kayıt **YASAK**tır. Yayımlanmış fixture silinemez/değiştirilemez; yeni semantic
davranış yeni `zee-tr-N` kimliği, yeni fixture ve sürüm/edition geçişi ister.

## Conformance kanıtı

- `compiler/tests/morfoloji_v1.snapshot`
- `compiler/tests/fixtures/morfoloji-zee-tr-1.sha256`: immutable semantic
  parmak izi; Git tabanına girdikten sonra CI'da değiştirilemez
- `conformance/morfoloji/sema-v1.schema.json` ve `zee-tr-1.json`: Rust'tan
  bağımsız profil, 27 çözüm/karar ve 21 kanonik/geçersiz üretim vakası
- `compiler/tests/morfoloji_testi.rs`: tablo, tek/iki katman property,
  kanonik üretim golden'ları, ters ses değişimi, belirsizlik korpusu, CLI;
  ayrıca 4.096 deterministik kök, 2.048 bütün-aday kararı ve NFC/NFD sınırı
- `compiler/fuzz/fuzz_targets/morfoloji.rs`: byte girdiden geçerli kök üreten
  gecelik üret→çöz ve fail-closed belirsizlik mutation hedefi
- `compiler/tests/lsp_testi.rs`: tek/iki katmanlı rename, kapsam ayrımı ve
  belirsiz belgede fail-closed semantic seçim
- `compiler/tests/proje_testi.rs`: profil sabitleme, P011, kilit v2 ve sürüm CLI

K-122/B-008'in iki kapılı bakım akışı
[profil uyumluluk rehberindedir](../docs/morfoloji-profil-uyumlulugu.md).
K-123/B-009'un veri şeması ve ikinci compiler tüketici protokolü
[morfoloji conformance rehberindedir](../docs/morfoloji-conformance.md).

K-111/B-016 kanıtının çalıştırma, crash küçültme ve dürüst sınır ayrıntıları
[morfoloji doğrulama rehberindedir](../docs/morfoloji-dogrulama.md).
