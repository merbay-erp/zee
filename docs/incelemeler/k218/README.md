# K-218 — Oran backend kabul kanıtı

Kaynak `4e351d34de861425c8a35b6ad1ddda6cd3e42707`; exact compiler/core-freeze
beyanı `5a3cb270a10e39567243c3b6d10b7302f60cb17c` içindedir. Önceki kaynak
`fef5e989c0a72db220f4641e45fcf5ac52dbd8ca`. Her ikili temiz exact checkout'ta
`cargo build --locked --release --bin dil` ile üretildi; hash ve çevre her
JSON dosyasındadır. Bu yerel gözlem yeni kalıcı benchmark tabanı değildir;
`performans-gecmisi-v2.tsv` ve eski ölçüm SHA/süreleri değiştirilmedi.

## Ürün yükü

[Özgün betikler ve köken](../../../dogfood/zee-cekirdek/oran-yuk/README.md).
Program SHA-256: b69c2b205dce532351b7928bb0e4cbba853003253359e3cfff326585fa40fe09.
Depo korpus başlık yorumu ölçüm kopyasından çıkarıldı; kalan program harici
ürün 1b002e0 kaynağıyla aynıdır. Deneysel kipte yalnız listener portu geçici
boş porta değiştirilir. Her kip izole geçici klasörde çalışır; ürün .zee
verisi silinmez. Worker'lar yalnız kendi PID'leriyle kapatılır.

Apple M4 Pro, 24 GiB RAM, macOS 26.6.2, Rust 1.93.1, release varsayılan
profil; her senaryo bir tur, sıfır ısınma. JSON `elapsed_seconds` Python
süreci kurulumunu da kapsar; betiğin iç istek/sn/p50 ölçümü stdout'tadır.
İlk önce-tam turunda başka yerel kalite işleri de çalışıyordu; temel iki
senaryo boşta yeniden ölçüldü (`once-bosta.json`).

| Senaryo | Önce, ortak dosya | Bellek |
|---|---:|---:|
| Tek worker, 1000 istek / C10 | 54 istek/sn, p50 182,0 ms | 2638 istek/sn, p50 3,6 ms |
| İki worker, 600 istek / C10 | 54 istek/sn | 5079 istek/sn |
| 4000 yeni anahtar, tek worker | 48 istek/sn (ilk tur) | 2780 istek/sn |
| Doldurma sonrası 300 / C1 | 41 istek/sn (ilk tur) | 1993 istek/sn |

Her normal senaryoda bütün yanıtlar 200. Bellek kipinde anonim yük hiçbir
web durum dosyası üretmedi. Aynı betikle deneysel taban 2876 istek/sn;
eski 11000 iddiası bu koşuda doğrulanmadı. İki/tek worker oranı yalnız bu
kurulumda yaklaşık 1,93'tür; genel ölçeklenme garantisi değildir.

32.768 anahtar tam kapasite ayrı yeniden başlatılmış ikinci worker'da
12,20 sn içinde dolduruldu: 32768×200, yeni anahtar 503. Mevcut anahtara
99 ek istek 200, bir sonraki 429 oldu; canlı kota düşürülmedi. RSS o anda
17968 KiB (bütün worker), durum dosyası 0 bayt/yok. Eski 1,8 MB dosya
rakamı tahmindi; bellek backend'i için dosya boyutu iddiası değildir.

## Kod ve makine kapıları

Yerel tam faz 780/780, sıfır atlanan; clippy/fmt, katalog/tazelik, 180 maddelik
spec drift, 57 kritik işlev eğilimi, katman/SCC/public API/uyumluluk,
beyan/semantic koruklar, 23 kurallı gizlilik ve Gitleaks geçti. Birleşik
`guvenlik-kapisi --surekli` (cargo-deny compiler+fuzz, fuzz derlemesi ve
tedarik) geçti. İlk denemelerin başarısızlıkları kabul sayılmaz: korpus
başlığı, test dosyası bütçesi, yüzey sırası ve taşınan kanıt yolu düzeltildi.

1800 sn exact kaynak soak geçti; Gitsrv #24 sonucu beklenir. Soak her dönemde
32.768 anahtar doldurur; zaman `dönem × 60000 ms` ile deterministik olarak
ilerler, önceki pencere temizlenir. RSS aynı derleyici/soak sürecindeki bu
uzun ömürlü tamponu ve ayrı gerçek dillsp sürecini kapsar. Uzak tedarik/Windows CI bitmeden tam platform kabulü veya sürüm hazır sayılmaz. Canlı yükseltme ürün tarafındadır.

Gitsrv #23 / 5a3cb270a10e39567243c3b6d10b7302f60cb17c: 774/778.
Dört kanıt özeti/Rust/Go eşdeğerlik testi, SHA beyanları sonrası bayat
türetilen sayfayı yakaladı. Oran ve compiler testleri geçti; sonraki
tedarik/gizlilik/WASM/Windows adımları koşmadı. Sayfa Zee üreticisiyle
yenilenir; başarısız koşu geçiş sayılmaz.

## Soak kapanışı

[Ham 360 örnek ve karar](soak.md): 4e351d34de861425c8a35b6ad1ddda6cd3e42707
temiz checkout, 1800 sn, 60 sn pencere, bir ısınma penceresi dışlandı.
32148 derleme + LSP dönemi ve her dönemde 32768 oran anahtarı/expiry
çevrimi çalıştı. Derleyici RSS medyanı 16480→16192 KiB, dillsp
17120→16896 KiB; ikisinde büyüme %0 ve sonuç GEÇTİ. UTC tarih
2026-10-13, İstanbul bitişi 2026-10-14'tür. Eski soak satırları değişmez;
yeni workload eski satırlarla aynı iş yükü olarak karşılaştırılmaz.

Tam 23 kurallı HEAD tarihçesi gizlilik taraması sıfır bulguyla geçti.
Gitsrv #24 Linux faz/kalite kapısı geçti; sonraki uzak adımlar sürmektedir.
