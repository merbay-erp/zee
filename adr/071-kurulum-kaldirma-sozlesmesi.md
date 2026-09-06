# ADR-071 — Kurulum, kaldırma ve sürüm runbook sözleşmesi

- **Durum:** kabul
- **Tarih:** 6 Eylül 2026
- **İlgili kayıt:** K-169, ADR-066/068, V1-P1-28

## Bağlam

K-168 sürüm ikilisini tekrar üretilebilir ve imzalı yaptı; fakat bir
öğretmenin ya da geliştiricinin artefaktı makinesine nasıl kuracağı, kurulu
dosyaların nasıl doğrulanıp temiz kaldırılacağı ve sürüm gününün adım listesi
yazılı değildi. Üçüncü dış inceleme K-169 ile üç platformda kurulum-kaldırma
ve release runbook tatbikatı istedi.

## Karar

1. **Doğrulanmadan kurulum yok.** `scripts/kur.sh` (Linux/macOS) ve
   `scripts/kur.ps1` (Windows) artefaktın `SHA256SUMS` dosyasındaki `dil`/
   `dillsp` girdilerini yeniden hesaplar; uyuşmayan tek bayt kurulumu iptal
   eder. `SHA256SUMS` yoksa kurulum yoktur.
2. **Manifestli sahiplik.** Kurulum `kurulum-v1.tsv` manifesti yazar: kurulu
   dosya yolu, SHA-256 ve kaynak Git SHA. Var olan dosyanın üzerine yazılmaz;
   manifest varken ikinci kurulum reddedilir.
3. **Yalnız kendi dosyasını kaldırma.** `kaldir.sh`/`kaldir.ps1` yalnız
   manifestteki dosyaları siler; kurulumdan sonra değişmiş dosya `--zorla`
   verilmeden silinmez. Yabancı dosyaya dokunulmaz.
4. **Tatbikat.** `kurulum-tatbikati` workflow'u `v*` etiketinde ve elle
   Ubuntu/macOS/Windows'ta artefakt üretir, kurar, `dil sürüm` çalıştırır,
   kaldırır ve dosyanın gittiğini doğrular. Unix'te Rust entegrasyon testi
   gerçek ikililerle aynı akışı ve oynanmış artefakt reddini her CI koşusunda
   sınar.
5. **Runbook.** `docs/surum-runbook.md` sürüm gününün bağlayıcı sırasıdır:
   güvenlik kapısı sürüm adayı kipi → etiket → tekrar üretilebilir artefakt →
   tatbikat → sürüm notu ve uyumluluk kayıtları → duyuru.

## Reddedilen seçenekler

- **Paket yöneticisi (brew/apt/winget) formülü:** dış depo ve imza altyapısı
  ister; V1 öncesinde artefakt+betik daha denetlenebilir.
- **`curl | sh` kurulumu:** doğrulanmamış uzaktan betik yürütür; tedarik
  zinciri ilkesiyle çelişir.
- **Sistem geneline (`/usr/local`) varsayılan kurulum:** yönetici izni ve
  başka araçların dosyalarına dokunma riski; varsayılan kullanıcı klasörüdür.

## Sonuçlar

- Yerel macOS tatbikatı: kurulum, `dil sürüm`, ikinci kurulum reddi,
  değişmiş dosyada kaldırma reddi, `--zorla` ile kaldırma ve oynanmış artefakt
  reddi geçti; Linux/Windows tatbikatı workflow'da koşar.
- VS Code eklentisi ve `dillsp` PATH bağı `editors/README.md`de kalır;
  betik yalnız ikilileri kurar. Otomatik güncelleme ve imza doğrulaması
  (`surum_artefakti dogrula`) kullanıcı adımı olarak runbook'tadır.
