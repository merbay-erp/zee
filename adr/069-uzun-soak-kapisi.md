# ADR-069 — Uzun soak ve kaynak sızıntısı kapısı

- **Durum:** kabul
- **Tarih:** 6 Eylül 2026
- **İlgili kayıt:** K-173, ADR-045/050/051/056, V1-P1-26

## Bağlam

Performans koşucusu tek süreçte 25 turluk p50/p95 ve tek `ru_maxrss` tepe
değerini ölçer; LSP cold-start ve ölçek eğrisi ayrıdır. Hiçbiri "saatlerce
açık kalan editör oturumu ya da sürekli derleyen CI süreci bellek sızdırıyor
mu" sorusuna cevap vermez. K-163 dogfood'u gerçek bir workspace'te 41 ms p95
ölçtü ama uzun oturum davranışı ölçülmedi. Üçüncü dış inceleme K-173 ile uzun
compiler/LSP workspace soak ve kaynak sızıntısı kanıtı istedi.

## Karar

1. **Soak koşucusu.** `compiler/src/bin/soak.rs` süre bütçesi boyunca iki
   döngüyü kesintisiz koşar: derleyici döngüsü (bütün golden korpusunu derleme
   + sabit yürütme) ve gerçek `dillsp` süreci (tek `initialize`/`didOpen`,
   ardından her turda içeriği değişen 2.000 satırlık tam metin `didChange` ve
   `publishDiagnostics` beklentisi). Her 5 saniyede kendi ve `dillsp`
   sürecinin RSS'i `ps -o rss=` ile örneklenir.
2. **Sızıntı ölçütü.** Isınma penceresi (ilk pencere) atılır; ısınma sonrası
   ilk pencere medyanı ile son pencere medyanı karşılaştırılır. Bir sürecin
   büyümesi hem %10'u hem 32 MiB'ı aşarsa kapı kalır. Yüzde tek başına küçük
   süreçlerde, mutlak değer tek başına büyük süreçlerde yanıltıcı olduğundan
   ikisi birlikte aranır.
3. **Provenance.** `--gecmis` yalnız temiz çalışma ağacından yazar; satır tam
   Git SHA, tarih, platform, rustc, süre, pencere, dönem sayıları, ilk/son
   medyanlar, büyüme yüzdeleri, sınırlar ve sonucu taşır
   (`zee-soak-gecmisi-1`). Deterministik Markdown raporu her koşuda üretilir.
4. **CI.** `soak` workflow'u haftalık ve elle Ubuntu'da 30 dakika koşar,
   rapor ve tarihçe satırını 90 günlük artefakt olarak saklar. Shared runner
   gürültüsü RSS medyanlarını değil süreleri etkiler; kapı yalnız RSS
   büyümesine bakar.

## Reddedilen seçenekler

- **Yalnız `ru_maxrss`:** tepe değeri sızıntıyı ısınmadan ayıramaz.
- **Valgrind/ASan sızıntı dedektörü:** Rust'ta doğrudan sızıntıdan çok
  büyüyen önbellek/klon sınıfını yakalamak gerekir; RSS eğilimi bunu görür.
- **Tek uzun süreçte yalnız LSP:** derleyici döngüsü (birim yükleyici,
  morfoloji önbelleği, tanı bütçesi) ayrı sızıntı yüzeyidir.

## Sonuçlar

- İlk taban bu ADR'nin commit'inden sonra `docs/soak-gecmisi-v1.tsv`
  içinde exact SHA ile kaydedilir; kapı ısınma sonrası büyümeyi ölçer, mutlak
  bellek bütçesi koymaz (o RFC-0025/spec-24'ün işidir).
- Yeni önbellek ya da uzun ömürlü durum ekleyen iş soak koşusunu tekrar
  ister; büyüme sınır içindeyse taban güncellenmez, tarihçe satır kazanır.
