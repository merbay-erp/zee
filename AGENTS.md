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
Kalıcı benchmark tarihçesi yalnız temiz exact commit checkout'unda üretilir;
tam Git SHA, ayrı milestone, OS/CPU/RAM/Rust/release profili ve gerçek
örnek/ısınma semantiği olmadan `docs/performans-gecmisi-v2.tsv` değiştirilemez.
Test ekleyen ya da taşıyan iş, `cd compiler && cargo run --locked --bin
faz_test_matrisi -- --denetle --rapor target/faz-test-matrisi.md` kapısından
geçer. Her gerçek Cargo/libtest vakası tam bir birincil faz sahibi olmalı;
fuzz/conformance/regresyon, aşağı akış ve seçici düzeyi `ek_kapsam` ilişkisi
aynı değişiklikte güncel kalmalıdır. Ek kapsam dosya adından tahmin edilmez;
yalnız test grubunun gerçekten yürüttüğü diğer fazlar yazılır.

Performans davranışı veya sabit iş yükü değişikliği
`docs/performans-gecmisi-v2.tsv`, `docs/olcumler.md` ve ADR-045/049 etkisini aynı
committe inceler. Shared CI ölçümü gözlemseldir ve `--esik-yuzde` taşıyamaz;
hard eşik yalnız sabitlenmiş adanmış benchmark koşucusunda açıkça
etkinleştirilebilir. Yeni taban, exact kaynak commit'i, makine/araç zinciri ve
gerçek sample/warmup dağılımı incelenmeden izlenen tarihçeye yazılmaz.

Production Rust modülü veya iç bağımlılığı değişen iş
`compiler/tests/fixtures/katman-mimarisi-v1.tsv`, ADR-046 ve
`docs/katman-mimarisi.md` etkisini aynı committe inceler. Fixture yalnız
`cargo test --locked --test katman_mimarisi_testi` farkını susturmak için
yenilenmez: sorumluluk sahibi ve temel→adaptör yönü önce değerlendirilir.
Yeni/kayıp üst sahip, eklenen/kaldırılan exact kenar ve ters katman geçişi
committen önce kapıyı geçmelidir.

Production bağımlılık graph'ı ayrıca `bagimlilik_cevrimi_testi` kapısından
geçer. Yeni sahipler-arası SCC eklenemez. Zorunlu geçici istisna
`izinli-katman-cevrimleri-v1.tsv` içinde exact üyeler, ayrıntılı gerekçe,
ISO son tarih ve kaldırma K-işi olmadan kabul edilmez; kaybolan ya da süresi
dolan izin aynı committe temizlenir.

GitHub workflow `uses:` satırları yerel action dışında yalnız 40 haneli
immutable commit SHA kullanır. Yeni/yükseltilen action aynı committe
`docs/github-actions-pinleri-v1.tsv` içindeki sürüm, SHA ve resmî kaynak
kaydını günceller; Dependabot PR'ı otomatik merge edilmez. Toplu committen
önce `tedarik_kapisi_testi` bu birebirliği denetler.

Her compiler bug düzeltmesi ayrıca `regression/<faz>/` altında tek arızaya
indirgenmiş bir `.dil` kaynağı ve `regression/v2.tsv` içinde K-kimliği,
`fixed_by`, mümkünse `introduced_by`, `guaranteed_since`, faz, kip, beklenen
tanı+kesin span, exit ve çıktı kaydı bırakır. Düzeltme commit'i önce alınır;
manifestin `fixed_by` alanı bu exact tam SHA'yı izleyen committe kaydeder. Bu kayıt olmadan
bug düzeltmesi tamamlanmış ya da commitlenebilir sayılmaz. Korpus sözleşmesi
`docs/semantic-regresyon-korpusu.md` ve ADR-044'tedir. Var olan vaka kimliği,
K-kimliği veya yolu silinmez/yeniden kullanılmaz; CI bunu Git tabanına karşı
`scripts/semantic-regresyon-korugu.sh` ile denetler.

`1c73298dca6bdbe27fc652daeb940b53709e1dbc` sonrasındaki her
`compiler/src` commit'i, başlığından bağımsız olarak
`docs/compiler-degisiklik-beyanlari-v1.tsv` içinde tam SHA, semantic sınıf ve
en az 40 karakterlik gerekçe taşır. `semantic-bugfix` exact `fixed_by` sahibi
regression vakasına; `semantic-change` var olan `spec/`, `rfcs/` veya `adr/`
kanıtına bağlanır. `maintenance` yalnız `kanıt=-` ve semantic davranışın neden
değişmediğini açıklayan gerekçeyle geçer. Kaynak commit'i önce, beyan ve
gerekiyorsa fixture/provenance commit'i sonra alınır; commit mesajı muafiyet
veya güvenlik sınırı değildir.

Fuzz workflow'u coverage korpusunu yalnız cache'te bırakamaz; her hedefin koşu
sonu korpusu `zee-fuzz-corpus-artifact-1` SHA-256/run/commit manifestiyle 90
günlük artefakta gider. İndirilen seed doğrulanıp küçültülmeden, stable
replay/regresyon kanıtı ve insan review'u olmadan `compiler/fuzz/corpus/`
altına alınmaz.

Release-candidate fuzz kapısı aynı kaynak commit'inde `lexer_parser`,
`morfoloji`, `http_istegi` ve `wasm_abi` hedeflerini 30–60 dakika explicit
AddressSanitizer ile çalıştırır; crash, timeout veya sanitizer bulgusu kapıyı
kırar. Seçili saf çekirdek Miri testleri de geçmeli ve exact çevre/sonuç
`docs/fuzz-rc-gecmisi-v1.tsv` dosyasına aynı toplu committe eklenmelidir.

## V1 öncesi iş sırası

Bağlayıcı sıra `docs/oncelikli-backlog.md` içindedir. P0 compiler/dil omurgası
kapanmadan yeni dil özelliği varsayılan olarak öne alınmaz. Başlamış atomik bir
correctness/güvenlik dilimi önce kod+test+belgesiyle kapatılır; ardından sıradaki
iş B-001'den başlayarak backlog bağımlılıklarına göre alınır. Tamamlanan her
madde aynı committe backlog durumunu ve kapanma kanıtını günceller.
