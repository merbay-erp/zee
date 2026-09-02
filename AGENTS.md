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

## V1 öncesi iş sırası

Bağlayıcı sıra `docs/oncelikli-backlog.md` içindedir. P0 compiler/dil omurgası
kapanmadan yeni dil özelliği varsayılan olarak öne alınmaz. Başlamış atomik bir
correctness/güvenlik dilimi önce kod+test+belgesiyle kapatılır; ardından sıradaki
iş B-001'den başlayarak backlog bağımlılıklarına göre alınır. Tamamlanan her
madde aynı committe backlog durumunu ve kapanma kanıtını günceller.
