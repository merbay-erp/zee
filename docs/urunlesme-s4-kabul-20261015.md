# S4 — K-297 sonrası yerel teknik kabul

Test kaynağı `4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2`, ağaç `6c4982688f5105a67c025815cbbb72c07fcea7f8`.
Son RC kapı zamanı: `2026-10-15T23:05:59+03:00`. Bu belgeyi ekleyen takip commit'i
kaynak S4'ün yerine geçmez. Yeni compiler özelliği, yayın veya dağıtım yapılmadı.

## Tamamlanan dar kabul

- macOS'ta kanonik iki temiz klondan eş CLI/LSP; kaynak ZIP çifti,
  restore/fsck bundle, SBOM/provenance ve sekiz sahipli kurulum/kaldırma vakası.
  İkili imzasızdır; Linux S4 kabulü ve fiziksel Windows/minimum OS yoktur.
- K-297 ordinary UTF-16 için kanonik native stdio üzerinden on olumlu/olumsuz
  LF/CRLF, tek/çok emoji, geçersiz surrogate ve atomik değişiklik vakası.
- Gerçek IntelliJ IDEA 2025.3.1/LSP4IJ oturumunda lint=true initialize,
  111 completion adayı ve `ols`→`olsun`, definition `[0:0,0:4)`, emoji sonrası
  hover sorgusu `1:12`, rename aralıkları `[0:0,0:4)`/`[1:9,1:13)`, iki emoji
  sonrası A001 `[0:11,0:16)`, format ve shutdown yanıtı/exit gözlendi.
  İki IDE oturumu normal Quit ile exit0 ve özgün sahibi tarafından toplandı.
- 1800,946 saniye soak/360 ham RSS örneği ve dört
  1/41-belge × lint kapalı/açık gerçek stdio gecikme gözlemi.
- Dört explicit AddressSanitizer hedefi, seçili iki Miri grubunda üç test,
  kanonik export/check ve aynı temiz kaynak güvenlik RC kapısı geçti.
  Tam Cargo: **996 başarılı, 0 başarısız, 25 ignored**.

| Hedef | libFuzzer sn | Sahip süreç sn | Yürütme |
|---|---:|---:|---:|
| lexer_parser | 1801 | 1802,754 | 7500101 |
| morfoloji | 1801 | 1803,667 | 8223673 |
| http_istegi | 1801 | 1802,968 | 11374088 |
| wasm_abi | 1801 | 1802,497 | 8883109 |

RC araçları: `nightly-2026-08-31`, `cargo-fuzz 0.13.2`;
release/normal kapılar Rust1.93.1. Makine ve gerçek ham zaman/araç zinciri
makbuzlarda kayıtlıdır. Örnek/ısınma semantiğiyle dört gecikme kohortu:

| Lint | Açık belge | p50 ms | p95 ms | Max ms |
|---|---:|---:|---:|---:|
| kapalı | 1 | 16,074 | 16,978 | 17,176 |
| kapalı | 41 | 99,957 | 101,454 | 101,509 |
| açık | 1 | 38,488 | 40,458 | 40,605 |
| açık | 41 | 210,934 | 223,698 | 225,407 |

Lint açık/41 belge kohortunda p95 **223,698 ms** ölçüldü; P19 için
200 ms hedefi kapanmadı.

Bu stdio ölçümü ilk başarılı yazma→son tam tanı çerçevesinin okunmasıdır;
İstemci zamanlaması ve ara çerçeve ayrıştırması gözlemi etkiler; gönderim
öncesi JSON/framing hazırlığı ve son çerçevenin sonraki doğrulaması aralık dışıdır. UI gecikmesi,
200 ms hard gate, gerçek ürün SLA'sı veya adanmış koşucu tabanı değildir.
[Ayrıntılı ölçüm sınırı](olcumler.md), [soak tarihçesi](soak-gecmisi-v1.tsv)
ve [RC tarihçesi](fuzz-rc-gecmisi-v1.tsv) ayrı tutulur.

## Tarihsel ve kabul dışı kayıtlar

İlk RC denemesi Miri bileşeni eksikken fuzz başlamadan durdu; araç bileşeni
kurulduktan sonraki v2 korunur. RC v2 producer `passed` yazsa da libFuzzer
2019/2019/2746/2217 sn ile sahip süreç 756,086/756,090/103,232/102,570 sn
uyuşmadı; kanonik süre şartını karşılamadığı için kabul edilmedi. Nedenin
uyku veya saat değişimi olduğu kanıtlanmış sayılmaz. Sonuçlar değiştirilmedi;
yukarıdaki geçerli sayılar yeni v3 denemesindendir. Soak monotonic süresi ve
ham örnekleri ayrıca doğrulandı; UTC farkı aktif çalışma yerine kullanılmadı.

İlk IDE trace Apply hatası (22:17:53.127, LSP4IJ settings EDT exception)
ve kısmi gözlem korunur; hata raporu gönderilmedi. Normal yeniden açılıştan
sonraki kabul ayrı kayıttır; ilk logun 903–904. satırları dış kapsam hashine bağlıdır.
Son kabul küratörlü araç/UI gözlemidir, tam ham konsol dışa aktarımı değildir.
IDE trace özel `zeeBuild` alanını göstermedi; UI BuildID görüldü denmez.
Exact kaynak bağı kanonik teslim/proje pini/launcher ve değişmeyen ikili
hashlerinden gelir; ayrı native stdio BuildJSON'ı doğrular. Completion popup
ayrıca kaydedilmedi; dört IDE spelling uyarısı LSP tanısından ayrıdır.

## Açık sınırlar

[S3](urunlesme-s3-kabul-20261015.md) native/Linux/PG kabulü yalnız
`66e4149e53451111d153f47283c219079a8d1cab` tarihsel kaynağınındır.
S4'te 25 Cargo ignored vaka ayrıca çalıştırılmadı; stdio gecikme ölçümü bu
vakalardan biri sayılmaz. Native alt senaryolar Cargo toplamına eklenmez.

P19 kısmi kalır: artımlı HIR/type, UI/gerçek ürün gecikmesi, genel SLA,
Windows/eklenti yayını açıktır. Core freeze gereği yeni compiler kapasitesi,
kanıtlı gerçek ürün ihtiyacı ve minimal değişiklik zinciri olmadan başlatılmaz.
C1/K-288 ve P33/K-293 otomatik araç incelemesi retleri korunur; yeniden denenmedi. Kullanıcının beş tam anonim projesi
henüz hazır değil ve fiziksel cihazı yok. İnsan çalışması, gerçek ürün
provenance/adoption, takvim, GitHub destek/uzak CI, imza/yayın ve lisans
kapıları [kullanıcı girdileri listesinde](urunlesme-kullanici-girdileri.md)
ayrıdır. Tam P19/V1 veya bütün 39 madde kapandı denmez.

## Belge etki incelemesi

Bu takip kaynak/grammar/API/tanı/bağımlılık davranışını değiştirmez; spec
normları, RFC indeksleri, hata kataloğu, katman/çevrim ve API fixture'ları
bu nedenle değişmez. ADR-136/139/140/145/153 ve ADR indeksi kabul kapsamıyla
güncellenir.
Mevcut spec-madde-kanıtı açıklaması tarihsel bağlam için değiştiğinden
`spec_drift` raporu, README sayıları ve kanıt özeti kanonik araçlarla yenilendi.
ADR-045/049 etkisi incelendi; paylaşımlı makine ölçümü gözlemseldir,
`performans-gecmisi-v2.tsv` yeni sabit taban taşımaz. Altı test grubunda
30 başarılı/0 başarısız/0 atlanan; katalog/tazelik, kanıt özeti ve gerçek
Rust/Go çıktı eşliği, güvenlik/tedarik, fmt, spec drift ve üç koruk geçti.
467 commit ve çalışma ağacı varsayılan sır taramasında temizdir.
[Son belge kapısı makbuzu](incelemeler/urunlesme-20261014/final-4b7dd76/documentation/FINAL-DOC-GATES.json)
ile kaynak kodunun değişmediği ayrıca doğrulandı. Geçici sahipli uyku
engelleyicileri kapatıldı; kalıcı sistem ayarı değiştirilmedi.

[Seçili arşiv ve ham kapsam](incelemeler/urunlesme-20261014/final-4b7dd76/README.md),
[RC kapısı](incelemeler/urunlesme-20261014/final-4b7dd76/rc/GATE-RECEIPT.derived.json),
[IDE kabulü](incelemeler/urunlesme-20261014/final-4b7dd76/jetbrains/ACCEPTANCE.json),
[stdio ölçümü](incelemeler/urunlesme-20261014/final-4b7dd76/lsp/PROOF.json),
[bağımsız ham ölçüm denetimi](incelemeler/urunlesme-20261014/final-4b7dd76/lsp/independent-review.json),
[soak denetimi](incelemeler/urunlesme-20261014/final-4b7dd76/soak/independent-review.json).
