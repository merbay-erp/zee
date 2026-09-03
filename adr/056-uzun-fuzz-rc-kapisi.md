# ADR-056 — Uzun fuzz release-candidate kapısı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-157, B-067

## Bağlam

Beş dakikalık gecelik mutation koşusu sürekli erken uyarı sağlar; release
candidate güveni için yeterince uzun soak değildir. Lexer/parser, morfoloji,
ham HTTP ve WASM ABI farklı girdi zarfı ve hata yüzeylerine sahiptir. “Fuzz
var” demek yerine süre, araç, sanitizer, kaynak commit ve sonucu hedef başına
kaydetmek gerekir.

## Karar

1. RC kapısı dört hedefi ayrı matrix job'unda en az 30, en çok 60 dakika
   çalıştırır. Kanonik CI bütçesi hedef başına 3.600 saniyedir; job timeout'u
   75 dakikadır.
2. `cargo-fuzz 0.13.2` ve `nightly-2026-08-31` sabittir. AddressSanitizer
   örtük varsayım bırakılmadan `--sanitizer address` ile seçilir; tek girdi
   timeout'u beş saniyedir.
3. Her hedef final libFuzzer istatistiğini, exit kodunu, kaynak commit/run
   provenance'ını, crash girdisini ve K-156 korpus manifestini 90 günlük tek
   RC artefaktında taşır. Crash, sanitizer bulgusu veya timeout job'ı düşürür.
4. Haftalık pazar koşusu ve elle `workflow_dispatch` aynı 60 dakikalık kapıyı
   çalıştırır. Release candidate yalnız dört hedefin aynı kaynak commit'inde
   yeşil raporu varsa ilerleyebilir.
5. Pointer ağırlıklı fuzz hedeflerinde AddressSanitizer esastır. Saf ondalık ve
   takvim çekirdeği ayrıca sabit nightly Miri testinden geçer; ağ/TLS/süreç
   adaptörlerini Miri'ye zorlayıp sahte kapsam iddia edilmez.
6. Yerel veya CI kampanya sonucu `docs/fuzz-rc-gecmisi-v1.tsv` içinde exact
   çevre, süre, yürütme, crash/timeout ve sanitizer bilgisiyle append-only
   tutulur.

## Reddedilen seçenekler

- **Beş dakikalık nightly'yi RC saymak:** hızlı geri bildirim ile soak kanıtını
  birbirine karıştırır.
- **Sanitizer'ı varsaymak:** cargo-fuzz varsayılanı değişirse kapı sessizce
  zayıflayabilir.
- **Bütün crate'i Miri'de çalıştırmak:** native TLS, socket ve süreç yüzeyleri
  Miri modeli dışında kalır; seçilmiş saf çekirdek daha dürüst kanıttır.
- **Logu yalnız job ekranında bırakmak:** tekrar inceleme ve release provenance
  zinciri kurmaz.

## Sonuçlar

- RC fuzz kapısı nightly smoke'tan ayrı, ölçülebilir ve tekrar edilebilirdir.
- K-156 korpus kalıcılığı uzun kampanyada öğrenilen coverage seed'lerini de
  cache kaybından korur.
- Haftalık sonuç release engelini önceden görünür yapar; RC yine aynı commit
  üzerinde dört yeşil hedef ister.
