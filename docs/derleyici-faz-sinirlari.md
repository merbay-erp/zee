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
├── cozumleyici.rs            türler, bağlam ve denetim orkestrasyonu
│   └── cozumleyici/
│       ├── cumle.rs          cümle/akış denetimi
│       ├── ifade.rs          ifade tür denetimi
│       └── cagri.rs          işlem imzası ve çağrı uzlaştırması
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
| işlem çağrı sözleşmesi | checker `cagri` | RFC-0006/spec-02/10 ve usability kararı |
| tür/akış kuralı | checker `cumle` veya `ifade` | tanı kataloğu ve olumsuz test |
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
