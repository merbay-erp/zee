# ADR-048 — GitHub Actions immutable SHA pinleme

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-151, B-059

## Bağlam

Cargo lock/checksum ve offline vendor K-141 ile sabitti; ancak `ci.yml` ve
`fuzz.yml`, `actions/checkout@v4`, `actions/cache@v4`,
`actions/upload-artifact@v4` ve `dtolnay/rust-toolchain@stable` hareketli
referanslarını çalıştırıyordu. Tag veya branch sahibi aynı adı başka commit'e
taşıdığında değişmeyen Zee commit'i farklı CI kodu çalıştırabilirdi.

## Karar

1. Yerel `./` action'ları dışında bütün `uses:` değerleri tam 40 küçük-hex
   commit SHA taşır. Major tag, branch, `main`, `stable` veya kısa SHA yasaktır.
2. İncelenen action, insan-okur sürüm etiketi, commit ve resmî GitHub kaynak
   deposu `docs/github-actions-pinleri-v1.tsv` içinde birebir tutulur. Yeni
   action yalnız workflow ve kayıt aynı incelemede güncellenirse girebilir.
3. `tedarik_kapisi_testi` bütün `.github/workflows/*.yml|yaml` dosyalarını
   keşfeder; SHA biçimini, kayıtla exact eşleşmeyi, workflow'lar arası tek
   pini ve kayıtların kullanımda olmasını fail-closed denetler.
4. Dependabot `github-actions` ekosistemini kökten haftalık izler. Ürettiği PR
   otomatik merge değildir: sürüm notu/kaynak commit incelenir, pin kaydı aynı
   PR'da güncellenir ve bütün supply-chain kapıları yeniden çalışır.
5. Checkout `persist-credentials: false`, workflow token yetkisi yalnız
   `contents: read` olur. Action yükseltmesi yalnız SHA değişimi sayılmaz;
   sürüm etiketi ve bakım belgeleri aynı committe güncellenir.

## İlk doğrulanmış pinler

- `actions/checkout` v4.4.0 → `11d5960a326750d5838078e36cf38b85af677262`
- `actions/cache` v4.3.0 → `0057852bfaa89a56745cba8c7296529d2fc39830`
- `actions/upload-artifact` v4.6.2 → `ea165f8d65b6e75b540449e92b4886f43607fa02`
- `dtolnay/rust-toolchain` stable, 2 Eylül 2026 →
  `4360b52568e2003a75bf9bc1d59f33a8e3fc893c`

SHA ve tag eşleşmeleri 2 Eylül 2026'da action'ların resmî Git uzak
depolarındaki ref'lerden doğrulandı.

## Reddedilen seçenekler

- **Major tag'i yeterli saymak:** okunaklıdır fakat immutable değildir.
- **Yalnız SHA biçimini denetlemek:** incelenmemiş yeni bir action/commit
  sessizce eklenebilir.
- **Dependabot PR'ını otomatik birleştirmek:** yeni third-party kodu insan
  incelemesi ve Zee kapıları olmadan güven sınırına alır.
- **Workflow dosyalarını sabit ad listesiyle taramak:** yeni workflow'un
  denetim dışında kalmasına izin verir.

## Sonuçlar

- Workflow'larda hareketli third-party action ref'i kalmadı.
- Action güncellemeleri görünür, haftalık ve ayrı PR akışındadır; pin kayıt
  değişmeden kapı geçmez.
- Rust dependency güvenliği, action yürütme güveni ve runner image hareketi
  ayrı sınırlardır. Bu karar runner image'ını tekrar üretilebilir saymaz.
- Dil/runtime davranışı, tanılar, grammar, RFC ve normatif spec değişmedi.
