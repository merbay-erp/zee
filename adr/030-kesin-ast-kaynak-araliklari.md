# ADR-030 — Bütün AST ifadelerinde kesin kaynak aralığı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-126, B-050, V1-P0-18

## Bağlam

K-108/ADR-020 her semantic HIR düğümünde kaynak kökenini zorunlu yaptı. Lexer
konumunu doğrudan taşıyan değişkenler kesin satır+sütun+uzunluk alırken eski
AST varyantları yalnız cümlenin satır zarfına düşüyordu. Bu dürüst bir geçiş
modeliydi fakat tanı ve LSP tüketicilerinin bileşik/yaprak ifadeyi tam olarak
işaretlemesine yetmiyordu.

İfade enum'una tek tek isteğe bağlı konum alanları eklemek yeni varyantların
spansiz doğmasına izin verecekti. Ayrıca HIR'ın private AST-adresi eşlemesini
bozacak klonlama veya dış zarfı sonradan değiştirme riski vardı.

## Karar

Parser'ın ürettiği her semantic ifade `Ifade::Kaynakli` zarfında zorunlu bir
`AstKaynakAraligi` taşır. Aralık bileşenleri `NonZeroUsize`dır ve Unicode
karakteri cinsinden tek satırlık `(satır, sütun, uzunluk)` değeridir; byte
ofseti değildir.

- Tekil sabit/değişken kendi token aralığını alır.
- Bileşik ifade, tükettiği ilk tokenın başlangıcından son tokenın bitimine
  kadar uzanır; her alt ifade ayrıca kendi bağımsız zarfını korur.
- Kaynakta açıkça yazılmayan örtük çoğul koleksiyon ifadesi, onu doğuran
  `her <ad> için` başlığındaki `<ad>` tokenına bağlanır; `1:1` uydurmaz.
- Parser kaynaklandırmayı tek `ayristirici/kaynak.rs` kapısından yapar. Boş,
  ters, taşan veya çok satırlı ifade bölgesi `C000` iç sözleşme hatasıdır.
- Checker zarfı yerinde korur; AST→HIR kaydı tam aynı kesin aralığı alır.
  Invariant doğrulayıcı eksik/iç içe zarfı ve AST/HIR aralık uyuşmazlığını
  reddeder.
- Checker'ın eski `(sütun=1, uzunluk=1)` ifade tanıları, daha özel bir alt
  tanı yoksa ilgili AST düğümünün kesin aralığına yükseltilir.
- LSP sembol kullanımında kesin aralığı doğrudan tüketir. İşlem/yapı adı için
  lexical arama bütün satıra değil semantic ifadenin kesin zarfına daralır;
  çağrı/yeni-yapı kuyruğundaki son canonical eşleşme seçilir.

Raw `Program` gömme API'si geriye uyumluluk nedeniyle elle kurulmuş spansiz
ifadeyi temsil edebilir. Bu yol parser AST'si sayılmaz; invariant doğrulaması
böyle bir düğümü reddeder, checker ise panic yerine kodlu iç tanı üretir.

## Değişmezler

1. Başarılı parser çıktısındaki her ifade düğümünün doğrudan ve tek bir kesin
   kaynak zarfı vardır.
2. Alt ifade, ebeveynin zarfını paylaşmaz; kendi tükettiği token bölgesini
   taşır.
3. HIR ifade aralığı AST aralığıyla birebir aynıdır ve hiçbir başarılı
   parser→checker hattında `Satir` zarfına düşmez.
4. Kaynak aralığı semantic kimlik, tür veya AST adresini değiştirmek için
   kullanılmaz.
5. Türkçe/Unicode kaynakta sütun ve uzunluk karakter cinsindedir.
6. Tanı/LSP tüketicisi kesin aralık varken bütün satırda tahmin yapmaz.

## Sonuçlar

- B-050 kapanır; V1-P0-18 K-108/K-126 birleşik kanıtıyla tam kesinliğe çıkar.
- `HirKaynakAraligi::Satir`, AST dışı tanım/gelecek uyumluluk kayıtları için
  kalabilir; başarılı AST ifade kayıtlarında artık kullanılmaz.
- Kaynak dilin sözdizimi veya çalışma semantiği değişmez; yeni normatif spec
  bölümü gerekmez.
- Bileşik+yaprak aralıkları, spansiz AST reddi, tanı işareti, örtük çoğul
  kökeni ve aynı yazımlı argüman/işlem kuyruğu bağımsız regresyonlarla
  korunur. Mimari bütçe, kaynaklandırma ve tanı yükseltmesini ayrı küçük
  modüllerde tutar.
