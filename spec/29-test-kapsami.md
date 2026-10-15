# Kaynak karar kapsamı

K-284/ADR-142; grammar genişlemesi değildir. CLI dosya sahipliği profili
Unix'te doğrulanır; diğer platformda açık ret vardır.

- **ZORUNLU:** `--kapsam` yalnız açık opt-in `dene` veya kaynak öncesi `çalıştır` seçeneğidir; kapalı yürütüm kolektör, rapor dosyası veya köken stringi tahsis etmez.
- **ZORUNLU:** `zee-source-coverage-1` paydası çağrılmayan işlem ve rotaları da içerir; ürün ve test gövdeleri ayrıdır, sıfır payda `not-applicable` olarak sunulur.
- **ZORUNLU:** karar sonuçları yalnız gerçekten değerlendirilen yüklem/operand, döngü gövdesi/normal tükenişi ve açık deadline normal/timeout sonucundan sayılır; kısa devre yeni yan etki çalıştıramaz.
- **ZORUNLU:** kaynak SHA256 aynı yükleyici snapshot'ına bağlıdır; ithal rota kökeni opt-in derlemede klonla korunur, başka AST'deki bilinmeyen probe başarılı rapor üretmez.
- **ZORUNLU:** kaynak256, sonuç16384, derinlik256 ve rapor2MiB üst sınırları aşılırsa kapsam reddedilir; yalnız fresh canonical owned normal dosyada `complete:true` bitmiş sonuç yazılabilir.
