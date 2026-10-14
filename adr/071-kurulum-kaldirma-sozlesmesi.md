# ADR-071 — Kurulum, kaldırma ve sürüm runbook sözleşmesi

- **Durum:** kabul
- **Tarih:** 6 Eylül 2026; K-194 kurulum revizyonu 12 Ekim 2026
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
   eder. `SHA256SUMS` yoksa kurulum yoktur. K-194 bütün ikilileri önce
   hedef dosya sistemindeki geçici alana kopyalar ve kopyanın checksum'ını
   doğrular; eksik/bozuk/yinelenen ikinci kayıt hiçbir ikili yayımlamaz.
   Yayınlama hatası bu koşunun oluşturduğu ikilileri geri alır; manifest en
   son, no-clobber olarak yayımlanır. Unix hard-link, Windows aynı dosya
   sisteminde File.Move kullanır; desteklenmeyen dosya sistemi hata verir.
   Ani güç kaybı/SIGKILL için crash-recovery veya tüm dizinin tek atomik
   değişimi vaat edilmez; normal hata geri alma bu dilimin sözüdür. Manifest
   iki platformda UTF-8/BOM'suz/LF yazılır; Windows kaldırma açık UTF-8 okur.
   PowerShell kaynakları 5.x Türkçe literal desteği için UTF-8 BOM taşır.
2. **Manifestli sahiplik.** Kurulum `kurulum-v1.tsv` manifesti yazar: kurulu
   dosya yolu, SHA-256 ve kaynak Git SHA. Var olan dosyanın üzerine yazılmaz;
   manifest varken ikinci kurulum reddedilir.
   GB-032/K-194: Artefaktta `GIT_SHA` varsa değer exact 40 küçük hex'tir;
   yoksa yalnız eski yerel artefaktlar için `-` kaydedilir. Hedef/veri yolları
   TAB/CR/LF taşıyamaz; fiziksel kanonik yol da denetlenir. Kaynak metadata
   manifest satırı/alanı enjekte ederek kurulum sahipliğini genişletemez.
3. **Yalnız kendi dosyasını kaldırma.** `kaldir.sh`/`kaldir.ps1` yalnız
   manifestteki dosyaları siler; kurulumdan sonra değişmiş dosya `--zorla`
   verilmeden silinmez. Yabancı dosyaya dokunulmaz.
4. **Tatbikat.** `kurulum-tatbikati` workflow'u `v*` etiketinde ve elle
   Ubuntu/macOS/Windows'ta artefakt üretir, kurar, `dil sürüm` çalıştırır,
   kaldırır ve dosyanın gittiğini doğrular. Unix ve Windows Rust entegrasyon testi
   gerçek ikililerle aynı akışı ve oynanmış artefakt reddini her CI koşusunda
   sınar. K-194 testinde ikinci checksum, eksik ikinci dosya, mevcut yabancı
   hedef ve yinelenen kayıt reddi vardır; Unix'te ikinci yayınlama hatası ve
   kırık sembolik bağ reddi de yürütülür. Windows gerçek yürütme sonucu
   alınmadan Windows kanıtı tamamlanmış gösterilmez.
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

## K-244 — Literal kaldırma yolları

14 Ekim 2026 kaynak incelemesinde kaldırıcı Path parametrelerinin joker
karakter yorumlaması, kurucunun LiteralPath ile kaydettiği receipt'e
uymayan aday olarak bulundu (GB-067). `bin[1]` ve aynı hashli yabancı
`bin1` ayrımı için gerçek PowerShell izole senaryosu hazırlanır. Kaldırıcı
manifest, dosya varlığı, hash okuma ve silme işlemlerinde LiteralPath
kullanır; mevcut manifest/checksum/Zorla sözleşmesi korunur.

Mevcut kurulum Cargo vakası Windows'ta gerçek native PowerShell fixture'ını
yürütür; tatbikat workflow'u aynı fixture'a bağlıdır. macOS'taki resmî
portable PowerShell ile önce/sonra kanıtı, fiziksel Windows veya minimum OS
kabulü değildir. Temiz exact kaynak sonuçları [devam raporunda](../docs/urunlesme-devam-20261014.md)
ayrı kaydedilir; yerel altı senaryo geçti, temiz exact commit tekrarı henüz bekleniyor.
