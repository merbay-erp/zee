# Derleyici faz ve handler sınırları

Bu rehber ADR-012'nin uygulama karşılığıdır. Amaç dosyaları küçük göstermek
değil; bir değişikliğin parser, checker ve runtime'da hangi sorumluluğa ait
olduğunu açık tutmaktır.

## Dizin yapısı

```text
compiler/src/
├── ayristirici.rs            token/blok/tanım orkestrasyonu
│   └── ayristirici/
│       ├── cumle.rs          cümle son-yüklem dağıtımı
│       └── ifade.rs          RFC-0021 ifade katmanları ve çağrı lowering'i
├── cozumleyici.rs            checker geçiş orkestrasyonu ve public API
│   └── cozumleyici/
│       ├── akis.rs           daraltma, gezme ve görev akışı
│       ├── baglam.rs         denetim geçişinin açık durumu
│       ├── cagri.rs          işlem imzası ve çağrı uzlaştırması
│       ├── cumle.rs          cümle denetimi
│       ├── donus.rs          kesin sonlanma ve dönüş birleşimi
│       ├── etki.rs           web/uygulama etkisi ve yetkinlik geçişi
│       ├── ifade.rs          ifade tür denetimi
│       ├── sembol.rs         ad, alan ve sözcüksel kapsam çözümü
│       ├── sozlesme.rs       public parametre/dönüş sözleşmesi
│       └── turler.rs         tür modeli, tür yazımı ve uzlaşma
└── yorumlayici.rs            değer/IO/scheduler ve yürütme orkestrasyonu
    └── yorumlayici/
        ├── cumle.rs          cümle yürütme
        └── ifade.rs          ifade değerlendirme
```

Alt modüller yalnız üst fazına `pub(super)` görünür. Dil kütüphanesinin public
API'si bu iç ayrımla büyümez.

## Değişiklik yönlendirmesi

| Değişiklik | İlk sahibi | Birlikte kontrol edilecek sınır |
|---|---|---|
| yeni cümle son-yüklemi | parser `cumle` | checker/runtime `cumle`, RFC/spec |
| yeni ifade/postfix | parser `ifade` | RFC-0021 çakışma matrisi, checker/runtime `ifade` |
| işlem çağrı uzlaştırması | checker `cagri` | RFC-0006/spec-02/10 ve usability kararı |
| public işlem sözleşmesi | checker `sozlesme` | paket/birim API ve semver sınırı |
| tür yazımı/uzlaşması | checker `turler` | ifade handler'ı ve olumsuz test |
| sembol/alan çözümü | checker `sembol` | morfoloji ve kapsam testleri |
| akış/daraltma | checker `akis` | cümle handler'ı ve flow testleri |
| dönüş/control-flow | checker `donus` | Seçenek/Sonuç ve tüm-yollar kanıtı |
| etki/yetkinlik | checker `etki` | ADR-011, intrinsic kaydı ve web kuralları |
| yürütme semantiği | runtime `cumle` veya `ifade` | ADR-003, değerlendirme sırası testi |
| alan adaptörü | intrinsic kaydı | ADR-011 rehberi, yetkinlik/etki/runtime |

## Büyüme bütçesi

`compiler/tests/mimari_sinir_testi.rs`, köklerin büyük handler'ları geri
yutmadığını ve her faz dosyasının ilan edilmiş satır bütçesini aşmadığını
denetler. Bütçe dolduğunda sayı yükseltilmez; ortak davranış çıkarılır veya
yeni, adı sorumluluğunu anlatan handler modülü açılır. Bütçe değişikliği ancak
ADR-012 gerekçesi ve bu rehber aynı committe güncellenirse kabul edilir.

`katalog_testi.rs` sabit bir kök dosya listesi kullanmaz; `compiler/src`
altındaki bütün Rust dosyalarını özyinelemeli ve sıralı tarar. Yeni handler'da
üretilen bir tanı kodu katalog denetiminden kaçamaz.

Bu test semantik kaliteyi tek başına kanıtlamaz. Tam test paketi, Clippy, WASM
derlemesi ve ilgili RFC/spec conformance'ı her değişiklikte yine zorunludur.

Checker içindeki semantik bağımlılık yönü ve tek sahiplik kuralları ayrıca
ADR-013 ile [checker katman rehberinde](checker-katmanlari.md) bağlayıcıdır.
