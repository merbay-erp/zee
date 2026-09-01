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
`compiler/tests/dokuman_tazelik_testi.rs` RFC/ADR/spec indekslerini ve yerel
Markdown bağlantılarını doğrular. Toplu commit öncesi ikisi de çalıştırılır.
