# ADR-124 — Exact kaynakta dış RC kanıtı

- **Durum:** kabul; K-265 kaynak/kapı entegrasyonu ve kısa protokol testleri var; gerçek uzun kampanya ve nihai temiz kabul açık
- **Tarih:** 15 Ekim 2026
- **İlgili:** ADR-056/066/064, GB-082

Temiz kaynak S için sonradan oluşan fuzz tarihçe satırını aynı commit'in kendi SHA'sıyla taşıma zorunluluğu self-reference üretir. Var olan default tracked exactHEAD yolu değiştirilmez; bu yolun self-reference sorunu çözülmüş diye gösterilmez. Explicit `--rc-kanit PATH --rc-kanit-sha256 HASH` seçimi, aynı temiz HEAD S için incelenmiş owned-local kanıtı dışarıdan doğrular. Kaynak S ile sonraki belge/kanıt commit'i E eşdeğer sayılmaz.

Kullanıcının AGENTS kuralı uzun RC sonuçlarının aynı toplu committe izlenmesini ister. Bu karar kullanıcı talimatını değiştirmez veya varsayılan bir istisna uydurmaz: kaynak S'nin gerçek kampanyası dış immutable receipt'te tutulur; aynı ürünleşme teslimindeki append-only kayıt commit'i E satırlarda S'yi korur. Gate, etiket ve release artefaktı S için çalışır. E'nin HEAD'i S yerine test edilmiş gösterilemez; bu ayrım teknik düzeltmenin açık inceleme kapsamıdır.

Kanonik `scripts/rc-kampanyasi.py` incelenmiş83807c47... koşucudan gelir; owned WNOWAIT/grup TERM/KILL/reap sırası, iki paralel hedef,1800sn her hedef ve2400sn dış koşu timeout'u korunur. Yeni metadata exactS'deki canonical producer hashini, public command argv ve original/source postcheck'i ekler. Profil macOS-arm64 yereldir; CI veya diğerplatform receipt kabulü icat edilmez.

`rc-kaniti.py export/check` yalnız tam4ASAN +2 seçili Miri grup, fixednightly2026-08-31/cargo-fuzz0.13.2, kaynak başlangıç/son/clean, canonical producer ve rawhash/korpus eşliğini kabul eder. İstenen süre1800..3600, actual>=requested ve<=3600; mevcut producer1800 ister.2400sn dıştimeout değiştirilmez; kayıtlı elapsed üstü2412 yalnız mevcut2snTERM+10snwait cleanup bütçesini kapsar, supervisor_failure varsa daima ret. Yeni30snslack yok. Miri actualnonempty/zeroignored+failed ve exact iki filtre gerektirir, tümRust/Miri kapsamı iddiası yoktur.

Dış manifest explicit SHA256 ile bağlanır; nofollow root/ancestor/leaf, regularfile, exact inventory, bounded256MiB total/64MiB file ve byte snapshot kullanılır. Bu sınırdaki kanonik export büyük kanıtı bölemez/atlanamaz; aşılırsa ret, sessiz budama yok. Hashler integrity sağlar; owned-local origin insan/operatör review'uyla doğrulanır, keyfi selfhashli JSON authenticated execution değildir. Explicit başarısız kanıt trackedfallback yapmaz. Her eski güvenlik kapısı adımı kalır ve sonunda kaynak/bundle yeniden doğrulanır.

Pozitif/negatif protokol fixture'ları ve9 kısa owned-process koşucu testi gerçek1800snASAN/Miri kampanyası değildir. Kaynak S freeze sonrası gerçek dört hedef+Miri yürütülmeden RC kapısı kapanmış sayılamaz.

Bağımsız V1 negatifleri yanlışbuildhedefi/runyerinebuild/bozukinitialmanifest kabulünü ve ilkCargoçağrısındanöncekaynakpini eksikliğini gösterdi. V2 exact build/run argüman yapısını ve target/options tekrarını, initialmanifest count+exactS sourcecorpus blob eşliğini doğrular. RC sourceSHA/tree ilk araç çağrısından önce sabitlenir; evidence öncesi ve sonunda aynı kalır. Bu doğruluk negatifleri GB082 sınırındadır; yeni compiler semantic-bugfix kaydı açılmaz.

V3 kayıtlı corpus yolundan tek mutlak lexical campaignroot çıkarır; dört hedefin root'u aynı, build/run cache root/fuzz-target ve dictionary root/checkout/compiler/fuzz/dictionaries/zee.dict exact olmalıdır. Göreli/dot/dotdot/double-slash yollar reddedilir; kayıtlı original yollara dosya sistemi erişimi yapılmaz. Bu ilişki provenance tutarlılığıdır, execution authentication değildir.

Dizin envanteri de streaming scandir ile ortak giriş bütçesini isim listesi tahsisinden önce uygular; symlinkler izlenmez. MAX_FILES=3 kontrollü iterator negatifinde dördüncü girişte durup devamını taramama gerçek kısa testi vardır.

Etki incelemesi: yeni compiler/src, grammar, facade export veya dil normu yoktur; mevcut RFC/spec/ADR-046 üretim Rust katman kenarları değişmez. Release gate politikası bu ADR ve runbook sahibindedir; yeni spec maddesi/FNV kaydı icat edilmez. Yeni kullanıcı tanı kodu olmadığı için hata kataloğu değişmez. ADR-045/049 sabit benchmark iş yükü ve tarihçesi değişmez; kısa protokol süreleri performans tabanı değildir. Yeni canonical namedformat/profile yüzeyi K-167/ADR-064 envanterindeki altı ek satırla kaydedilir.
