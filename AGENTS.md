# zee proje çalışma kuralları

## Dokümantasyon tazelik kapısı

Her kod/grammar/araç/güvenlik davranışı işi, ilgili Markdown belgeleriyle aynı
toplu commit içinde tamamlanır. “Kod şimdi, belge sonra” kabul edilmez.

Her işin sonunda etki alanına göre şu dosyalar gözden geçirilir ve gerekenler
güncellenir:

- `spec/` — çalışan normatif davranış;
- `rfcs/` ve `rfcs/README.md` — kararın gerekçesi, kapsamı ve durumu;
- `adr/` ve `adr/README.md` — mimari/güvenlik sınırı;
- `docs/hata-katalogu.md` — yeni/değişen kullanıcı tanıları;
- `docs/surumler.md` — kullanıcıya görünen değişiklik ve göç yolu;
- `docs/v1-surum-kapilari.md` — kapı gerçeği ve eksik kanıt;
- `docs/master-plan.md` — gerçekleşen kapasite ile açık iş ayrımı;
- kök `README.md` ve öğretici rehberler — güncel özellik seviyesi/komutlar.

Bir dosyanın değişmemesi de bilinçli etki incelemesinin sonucu olmalıdır.
Gerçeklenmeyen özellik belgeyle tamamlanmış gösterilemez; kısmi gerçekleme
“açık” sınırıyla yazılır. İlgili karar, spec, olumlu/olumsuz test ve belgeler
birlikte yoksa iş tamamlanmış sayılmaz ve commit alınmaz.

`compiler/tests/katalog_testi.rs` kaynak tanılarıyla hata kataloğunu birebir;
`compiler/tests/dokuman_tazelik_testi.rs` RFC/ADR/spec indekslerini, yerel
Markdown bağlantılarını, `docs/kanit-haritasi-v1.tsv` kapsamını ve README canlı
sayı bloğunu doğrular. Test/golden/tanı/RFC/ADR/spec sayısı değişince kökten
elle arama yapılmaz; `cd compiler && cargo run --bin depo_sayilari -- --yaz`
çalıştırılır. Toplu commit öncesi katalog ve tazelik kapıları çalıştırılır.
Rust kaynakları ayrıca `cd compiler && cargo fmt --all -- --check` kapısından
geçer; kanonik biçim borcu kod değişikliğinden ayrı bırakılmaz.
Test ekleyen ya da taşıyan iş, `cd compiler && cargo run --locked --bin
faz_test_matrisi -- --denetle --rapor target/faz-test-matrisi.md` kapısından
geçer. Her gerçek Cargo/libtest vakası tam bir birincil faz sahibi olmalı;
fuzz/conformance/regresyon ve aşağı akış ilişkisi aynı değişiklikte güncel
kalmalıdır.

Performans davranışı veya sabit iş yükü değişikliği
`docs/performans-gecmisi-v1.tsv`, `docs/olcumler.md` ve ADR-045 etkisini aynı
committe inceler. Shared CI ölçümü gözlemseldir ve `--esik-yuzde` taşıyamaz;
hard eşik yalnız sabitlenmiş adanmış benchmark koşucusunda açıkça
etkinleştirilebilir. Yeni taban, makine/araç zinciri ve 25 turluk p50/p95
dağılımı incelenmeden izlenen tarihçeye yazılmaz.

Production Rust modülü veya iç bağımlılığı değişen iş
`compiler/tests/fixtures/katman-mimarisi-v1.tsv`, ADR-046 ve
`docs/katman-mimarisi.md` etkisini aynı committe inceler. Fixture yalnız
`cargo test --locked --test katman_mimarisi_testi` farkını susturmak için
yenilenmez: sorumluluk sahibi ve temel→adaptör yönü önce değerlendirilir.
Yeni/kayıp üst sahip, eklenen/kaldırılan exact kenar ve ters katman geçişi
committen önce kapıyı geçmelidir.

Her compiler bug düzeltmesi ayrıca `regression/<faz>/` altında tek arızaya
indirgenmiş bir `.dil` kaynağı ve `regression/v1.tsv` içinde K-kimliği, faz,
kip, beklenen tanı+kesin span, exit ve çıktı kaydı bırakır. Bu kayıt olmadan
bug düzeltmesi tamamlanmış ya da commitlenebilir sayılmaz. Korpus sözleşmesi
`docs/semantic-regresyon-korpusu.md` ve ADR-044'tedir. Var olan vaka kimliği,
K-kimliği veya yolu silinmez/yeniden kullanılmaz; CI bunu Git tabanına karşı
`scripts/semantic-regresyon-korugu.sh` ile denetler.

## V1 öncesi iş sırası

Bağlayıcı sıra `docs/oncelikli-backlog.md` içindedir. P0 compiler/dil omurgası
kapanmadan yeni dil özelliği varsayılan olarak öne alınmaz. Başlamış atomik bir
correctness/güvenlik dilimi önce kod+test+belgesiyle kapatılır; ardından sıradaki
iş B-001'den başlayarak backlog bağımlılıklarına göre alınır. Tamamlanan her
madde aynı committe backlog durumunu ve kapanma kanıtını günceller.
