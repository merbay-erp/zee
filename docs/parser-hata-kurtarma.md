# Parser hata kurtarma rehberi

Bu belge ADR-024 ve RFC-0010 §2.1'in uygulama rehberidir. Faz ayrımı için
[derleyici faz modeli](derleyici-faz-modeli.md), kısmi AST sözleşmesi için
[AST/HIR invariant rehberi](ast-hir-invariantleri.md) birlikte okunur.

## Senkronizasyon modeli

```text
hatalı cümle ──> SatirSonu
                    │
                    └─ varsa yalnız ona ait dengeli Girinti…Cikinti gövdesi
sonraki aynı-girintili kardeş ──> ayrıştırmaya devam
```

Parser token tahmini yapmaz ve sonraki yüklemi aramak için birden çok satırı
körlemesine taramaz. Kaynakta güvenilir iki sınır zaten vardır: lexer'ın
ürettiği satır sonu ve dengeli girinti tokenları. Kurtarma yalnız bu sınırları
kullanır.

Bir bloktaki hata yerel tanı havuzuna alınır. Geçerli önceki ebeveyn düğümü ve
sonraki kardeşler korunur; fiziksel blok derinliği çıkışta geri alınır. Hatalı
başlık bir gövde açtıysa gövde bütünüyle atlanır, çünkü hangi alt cümlenin
başlığa ait olduğu ancak dengeli girinti sınırıyla güvenle bilinir.

## Özel bloklar

- Yapı alanında bozuk satır S025 olur; sonraki geçerli alan aynı yapıda kalır.
- Eşzamanlı görevde bozuk satır S038/ifade tanısı olur; sonraki görev korunur.
  Hiç geçerli görev kalmazsa boş `Eszamanli` AST'si üretilmez.
- `göre` içinde bozuk kol ve ona ait gövde atlanır; sonraki geçerli değer kolu
  korunur. Hiç değer kolu kalmazsa boş eşleştirme düğümü üretilmez.
- Bozuk `değilse`/`yetişmezse` devamı önceki geçerli ana bloğu düşürmez.

## Tanı sırası ve bütçesi

`dil denetle`, `dil denetle --json` ve LSP aynı çoklu-tanı hattını kullanır.
Parser, birim ve checker tanıları birlikte kaynak konumuna göre sıralanır;
eşit konumda kod ve mesaj deterministik bağı çözer. Tek belge değişiminde en
çok 20 tanı yayımlanır. Bu sınır, yazım sırasında oluşan tanı selinin editörü
ve kullanıcıyı boğmasını önler.

Normal `dil çalıştır` ve derleme API'si ilk tanıda durur. Kurtarılan kısmi AST
yalnız tanı toplamak içindir; checker başarısı ve typed HIR olmadan çalışmaz.

## Yeni blok türü ekleme kontrol listesi

1. Hatalı satırdan sonra güvenilir yatay ve dikey sınırı belirle.
2. Alt gövde açılmışsa `bekleyen_govdeyi_atla` ile yalnız o gövdeyi tüket.
3. `derinlik` artırıldıysa hiçbir hata yolu onu yüksek bırakamaz.
4. Kısmi düğüm ADR-023 parser invariantlarından geçemiyorsa düğüm üretme;
   ana tanıyı döndür.
5. Hata öncesi ebeveyn, hata sonrası kardeş ve sonraki üst düzey tanım için
   recovery regresyonu ekle.
6. LSP JSON'unda tanı sayısını, kodunu, sırasını ve 0 tabanlı aralığını sınar.

## Kanıt

- `parser_kurtarma_testi.rs`: iç kardeş sahipliği, yapı alanı, derinlik
  sızıntısı, eşzamanlı görev, `göre` kolu ve 20 tanılık bütçe.
- `lsp_testi.rs`: bağımsız S025/S004 tanılarının kaynak sırası ve sahte C000
  oluşmaması.
- Fuzz hedefi hem normal hem kurtarmalı parser yolunu sürekli yürütür.
- Tam test, Clippy, release ve WASM kapıları davranış gerilemesini engeller.
