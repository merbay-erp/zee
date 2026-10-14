# Güvenlik politikası

Zee'nin güvenlik sözü kaynak koddaki sınırlarla değil, yürütülebilir kapılarla
tutulur (K-170/ADR-066). Bu dosya bildirim kanalını, önem sözlüğünü ve sürüm
öncesi güvenlik kapısını tanımlar.

## Bildirim

- Depo özeldir; GitHub tarafından yönetilen eski PR refs/cache nesnelerinin
  temizliği destek teyidi bekler (GB-046). Depoyu herkese açmak ayrı kullanıcı
  kararıdır. Depo erişimi olan pilot katılımcıları hassas güvenlik bulgularını
  [Security Advisories — özel güvenlik bildirimi](https://github.com/merbay-erp/zee/security/advisories/new)
  üzerinden iletir. Kanal erişimi yoksa depo sahibiyle önceden belirlenen
  özel pilot iletişim yolu kullanılır; gerçek sırlar ve istismar ayrıntıları
  herkese açık issue içine yazılmaz. Yeni bir iletişim adresi bu belgeyle
  tahsis edilmez.
- İlk inceleme hedefi 3 iş günüdür; bu bir SLA değildir. Kabul edilen bulgu önem sınıfıyla
  [`docs/guvenlik-bulgulari-v1.tsv`](docs/guvenlik-bulgulari-v1.tsv)
  kaydına girer; düzeltme K-işi ve regresyon vakasıyla kapanır.
- Düzeltme yayımlanmadan bulgu ayrıntısı paylaşılmaz; yayımdan sonra kayıt,
  sürüm notu ve varsa CVE/RustSec referansı açıktır.

[Sınırlı pilot destek politikası](docs/urun-destek-politikasi.md) desteklenen
kapsamı ve güvenlik/uyumluluk kapılarına bağlı yayın kararını tanımlar.
Lisans seçimi, LTS süresi ve düzenli yayın takvimi henüz verilmiş söz değildir.

## Önem sözlüğü

| Önem | Anlam |
|---|---|
| `kritik` | Uzaktan kod çalıştırma, kimlik doğrulama/CSRF atlatma, request smuggling, kalıcı veri bozulması |
| `yuksek` | Kimliksiz DoS (sınırsız bellek/bekleme), oturum/yetki sızıntısı, tedarik zinciri değişikliği |
| `orta` | Kimlikli DoS, sessiz metadata/veri kaybı, doğrulanmamış giriş yolu, test edilmemiş güvenlik sınırı |
| `dusuk` | Bilgi sızdırmayan sertleştirme boşluğu, doğrudan test edilmeyen nitelik |

## Sürüm kapısı

`bash scripts/guvenlik-kapisi.sh --surekli` her push'ta cargo-deny, fuzz
derlemesi, kayıt şeması ve "açık kritik/yüksek bulgu sıfır" kuralını çalıştırır.
`--surum-adayi` ayrıca temiz çalışma ağacı, exact HEAD için dört hedefli RC
fuzz kanıtı, clippy ve tam test paketini ister. Kritik ya da yüksek önemli
açık bulgu varken sürüm etiketi kesilemez.

Kabul edilen sınırlar (`kabul` durumu) güvenlik açığı değil, belgelenmiş ve
karar bağlı vaat dışı alanlardır; her biri normatif spec/ADR yoluna bağlıdır.

## Sırların yayımlanmasını önleme

GitHub secret scanning, push protection ve özel güvenlik bildirimi etkinleştirildi
(9 Eylül 2026; 12 Eylül yeniden doğrulandı). Bu ayarlar kaynak dosyasından
zorlanmaz; depo yöneticisi görünürlüğü/ayarları değiştirirse yeniden denetler.
CI ayrıca SHA-256 ile sabitlenmiş Gitleaks 8.30.1 ile tüm erişilebilir Git
geçmişini ve güncel dosyaları tarar. Yerelde aynı kontrol:

```sh
bash scripts/sir-taramasi.sh
bash scripts/sir-taramasi.sh --staged
```

İkinci komut commit öncesi staged farkını tarar; otomatik yerel hook kurulmaz.
Tarama çıktıları maskelidir; bulgu CI'ı durdurur. Ignore dosyası veya temiz
bir tarama bütün sırların bulunacağını garanti etmez. `.env.example` gibi
örnekler gerçek sır içeremez. IO izinin hassas veri sınırları için
[IO izi rehberine](docs/io-izi.md) bakın. Commit e-postası tercihi ve Git
geçmişi bu güvenlik çalışmasıyla değiştirilmez.

## İncelenmiş tarihsel yanlış pozitifler

Sır taraması [exact kayıt](docs/sir-yanlis-pozitif-v1.tsv) içindeki dört
immutable belge fingerprintini kaynak satırı SHA-256 ile doğrular (K-204).
Bu kayıt yeni commit, dosya veya gerçek anahtar bulgusunu susturmaz.
Default Gitleaks kuralları ve redaction yürürlüktedir; yeni kayıt aynı
değişiklikte güvenlik bulgusu, gerekçe, hash ve ret testi ister.

### K-214: Anonim açık depo kanıtı

Müşteri ve altyapı kimliği anonimleştirildi; yeni içerik hash politikasıyla
CI içinde denetlenir. Eski geçişler maskeli dökümle ayrı kullanıcı kararını
bekler. Tarihsel ölçümler eski kaynak kimliklerini korur; yeniden açılış ve
GitHub PR önbelleği temizliği ayrıca onay/teyit ister. [Kayıt](docs/incelemeler/k214/README.md).
