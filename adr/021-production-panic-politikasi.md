# ADR-021 — Production panic yüzeyi ve fail-closed hata politikası

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-109, B-014, V1-P0-19

## Bağlam

Bootstrap derleyici; parser, checker, runtime, paketleme ve CLI boyunca
`unwrap`, `expect`, `panic!` ve `unreachable!` kullanan doğrulanmamış iç
varsayımlar taşıyordu. Bir kısmı gerçekten matematiksel olarak kanıtlı olsa da
bir kısmına elle kurulmuş AST, bozuk arşiv, IO hatası veya scheduler iç durumu
ulaşabiliyordu. Kullanıcı girdisinin bir derleyici sürecini tanısız düşürmesi
zee'nin güvenilirlik sözüyle uyuşmaz.

## Karar

Production olarak derlenen dört crate kökü (`lib`, `dil`, `dillsp`, `olcum`)
`not(test)` yapılandırmasında şu Clippy lintlerini `deny` eder:

- `unwrap_used` ve `expect_used`,
- `panic` ve `unreachable`,
- `todo` ve `unimplemented`.

Test kodu bu yasaktan bilinçli olarak muaftır; test fixture'ının başarısız
kurulumda hemen düşmesi production hata sözleşmesi değildir. CI'ın mevcut
`cargo clippy --all-targets -- -D warnings` kapısı bu crate-içi yasağı da
çalıştırır.

Audit'te görülen 46 production noktası şu kuralla ele alındı:

1. Kullanıcı girdisi, bozuk/elle kurulmuş AST, arşiv veya IO/dış durum hatası
   `Result`, Türkçe `Tani`, `C000` ya da açık CLI hata koduna dönüşür.
2. Koleksiyon/sayaç gibi doğrulanmış yapısal invariantlar panik makrosuyla
   ifade edilmez; `Option`/`Result`, doğrudan güvenli kurucu veya toplam
   dönüşüm kullanılır.
3. Scheduler iç tutarsızlığı process'i düşürmez; `C000` üretir.
4. Benchmark girdisi bozulursa ölçüm aracı açıklamalı hata ve başarısız süreç
   koduyla çıkar.

`SymbolId`nin önceki üst/alt 32-bit paketlemesi iki kapasite `assert!`i
gerektiriyordu. Kimlik kalıcı ABI olmadığından kapsam ve sıra iki `usize`
bileşene ayrıldı; yapay kapasite panic'i kaldırıldı.

## Değişmezler

1. Production crate köklerinden biri yasaklı lint listesini düşüremez.
2. Yeni production `unwrap`/`expect` veya açık panic makrosu Clippy'de
   derlemeyi durdurur.
3. Test cfg'si production muafiyeti yaratmak için kullanılamaz.
4. Elle kurulmuş geçersiz public AST, process panic'i yerine kodlu tanı verir.
5. Bu ADR bounds/indexing ve fuzz kaynaklı bütün olası Rust panic'lerini tek
   başına kanıtlamaz; lexer/parser fuzz B-015/K-110/ADR-022 ile tamamlandı,
   AST/HIR invariant doğrulayıcı B-017/K-112/ADR-023 ile tamamlandı.

## Sonuçlar

- B-014 ve V1-P0-19 kapanır.
- İki regresyon testi, elle kurulmuş sıfır konumlu AST'nin T016 üretmesini ve
  dört production crate kökünün lint kapısını taşımasını doğrular.
- Zee kaynak semantiği değişmediğinden yeni normatif dil spec'i gerekmez.
