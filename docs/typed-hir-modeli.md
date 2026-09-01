# Typed HIR modeli

Bu belge ADR-016'nın uygulama rehberidir. Faz sırası için
[derleyici faz modeli](derleyici-faz-modeli.md), kimlik kuralları için
[semantic kimlik modeli](semantic-kimlik-modeli.md) birlikte okunur.

## K-103/K-104 ile çalışan hat

```text
Parsed AST
   │ checker: çözüm + tür/akış/etki kanıtı
   ▼
Bound AST + HirOlusturmaBilgisi
   │ zorunlu lowering
   ▼
HirProgram
   ├─ HirDugumId → HirIfadeTuru::Deger(Tur) / DegerDondurmez
   ├─ HirDugumId → SymbolId / IslemId / YapiId
   ├─ semantic ID → canonical tanım
   └─ salt-okunur kaynak AST (tanı ve v0 uyumluluğu)
```

`BaglanmisProgram` artık `HirProgram` taşır. `hir().ifade_bilgisi(ifade)` bir
ifadenin düğüm kimliğini, checker'ın kanıtladığı değer/dönüşsüz türünü ve
semantic bağını verir. Değer konumunda olmayan çağrı cümlesi türsüz bırakılmaz;
`HirIfadeTuru::DegerDondurmez` taşır.
`sembol_adi`, `islem` ve `yapi` sorguları depolama konumunu ID'den ayrı tutar.

## Neden AST hemen silinmedi?

Bootstrap yorumlayıcısı büyük bir davranış yüzeyini AST üzerinden yürütür;
formatter, LSP ve Türkçe tanılar da kaynak yazımına ihtiyaç duyar. AST'yi tek
committe kopya bir dev enum'a çevirmek, semantik kazanım olmadan geniş hata
yüzeyi oluşturur. Bunun yerine geçiş iki kanıtlı dilimdir:

1. K-103: typed HIR kaydı zorunlu faz ürünü oldu.
2. K-104: standart runtime ve `dene`, değişken/işlem/yapı kararlarını yalnız
   HIR bağlarından almaya başladı; kaynak adı yalnız tanı/gösterim verisidir.

B-019 iki dilimle kapandı. Raw `Program` alan v0 API'nin ad-temelli davranışı
uyumluluk sınırıdır, yeni iç kod için örnek değildir.

`yorumlayici/hir_gecisi.rs` iki yürütme kolunu açıkça ayırır. HIR kolu bağ
bulamazsa kaynak adına geri düşmez; iç değişmez hatası verir. Scheduler görev
ifadelerini klonlamak yerine özgün düğümü ödünç alır, böylece HIR kimliği
eşzamanlı yürütmede korunur.

## Düğüm kimliği ve ömür

`HirDugumId` program içi semantic düğüm kimliğidir. Checker'ın private AST
adresi yalnız lowering sırasında bu kimliğin kayıt anahtarıdır. Public API
adresi açmaz; `HirProgram` AST'yi kutuda sabit tutar ve bağlı program klonlama
yüzeyi sunmaz. Kalıcı paket/ABI kimliği gerekiyorsa ayrı bir karar gerekir.

## Büyüme kuralları

- Yeni ifade checker'da başarılı tür döndürüyorsa HIR kaydı otomatik oluşur.
- Yeni semantic bağ `String` olarak HIR'a eklenmez; tür güvenli ID ister.
- Runtime'ın HIR tüketicisi, bağ eksikliğini kaynak adından tahmin ederek
  onarmaz; iç değişmez hatası üretir.
- Source span HIR düğümünün zorunlu alanı B-020'de yapılacaktır.
- `hir_modeli_testi.rs` ve mimari sınır testi olmadan HIR sahipliği değişmez.
