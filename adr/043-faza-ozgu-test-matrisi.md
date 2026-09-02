# ADR-043 — Faza özgü ve tam sahipli test matrisi

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-146, B-038

## Bağlam

Tek `cargo test` sonucu bütün regresyon paketini doğruluyordu fakat bir
değişikliğin lexer, parser, AST, resolver, tür, HIR, runtime ya da aşağı akış
araçlarında hangi yüzeyi etkilediğini görünür kılmıyordu. Dosyadaki `#[test]`
satırlarını saymak da `cfg` koşullarını, makro ile üretilen testleri ve gerçek
Cargo hedeflerini kesin olarak temsil etmez. Test sayısı büyüdükçe sahipsiz
bir test veya yanıltıcı tek toplam, incelemenin blast-radius bilgisini gizler.

## Karar

1. `faz-test-matrisi-v1.tsv`, her entegrasyon hedefini ve lib/bin/doctest
   filtresini tam bir birincil faza sahipletir. Lexer'dan project system'a
   kadar istenen üretim fazlarına ek olarak end-to-end ve mühendislik kapıları
   ayrı görünür; bir testin sıfır veya birden fazla sahibi olamaz.
2. `faz_test_matrisi`, `cargo test --no-run --message-format=json` çıktısındaki
   gerçek çalıştırılabilirleri açar ve libtest'in `--list` envanterini temel
   alır. Kaynak satırı sayısı karar vermez. Sahipsiz yeni hedef/test, boş
   filtre, yinelenen sahiplik ve kayıp ilişki yolu fail-closed hatadır.
3. Her faz ayrı çalıştırılır. Dinamik rapor test count, pass/fail/ignored,
   duvar süresi, kalıcı regresyon kaynağı, fuzz/conformance bağı ve aşağı akış
   yüzeyini gösterir. Süre gözlemseldir; gürültülü genel CI makinesinde
   performans eşiği değildir.
4. Tier-1 Linux, macOS ve Windows işleri eski tek toplam test adımı yerine bu
   matrisi çalıştırır. Her platform kendi `cfg` envanterini yeniden kanıtlar;
   rapor hem job summary'ye hem indirilebilir CI artefaktına yazılır.
5. İzlenen `docs/faz-test-matrisi.md` sayısal platform sonucunu değil,
   platformlar arası kararlı sahiplik ve saldırı yüzeyi sözleşmesini taşır.
   `--dokuman-yaz` kanonik belgeyi üretir, `--denetle` byte tazeliğini sınar.

## Reddedilen seçenekler

- **Test dosyalarını ad önekleriyle yaklaşık gruplamak:** unit/bin/doctest ve
  koşullu testleri kaçırır; yeni testin sessizce sahipsiz kalmasına izin verir.
- **Aynı testi birçok fazın toplamına eklemek:** toplamları şişirir ve birincil
  sorumluluğu belirsizleştirir. Bunun yerine aşağı akış bağı ayrıca raporlanır.
- **Yalnız Linux'ta matris koşmak:** platforma özgü testin sahipsizliğini diğer
  Tier-1 hedeflerde gizler.
- **CI süresini hard performans kapısı yapmak:** shared runner gürültüsünü
  correctness hatasına dönüştürür. B-040 daha sonra K-148/ADR-045 ile ayrı
  sabit iş yükü gözlemi olarak kapandı; shared runner yine hard gate değildir.

## Sonuçlar

- Tek yeşil sayı yerine hangi fazın kaç testi ne sürede geçtiği görülür.
- Mevcut kalıcı fixture, fuzz ve bağımsız conformance bağları aynı raporda
  görünür. B-039/K-147 daha sonra ADR-044 ile sürümlü semantic regression
  korpusunu ayrı 22. rapor fazı olarak eklemiştir.
- CI bütün testleri yine çalıştırır fakat sıralama birincil faz sahipliğine
  göre yapılır. Dil semantiği, tanılar ve normatif spec değişmez.
