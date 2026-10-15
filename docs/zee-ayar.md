# Sürümlü proje runtime ayarı — K-234

`dil çalıştır <proje-klasörü>` verilen kökte `zee.ayar` varsa okur. Bu
biçimin ilk destek tabanı **0.8.0-dev + `zee-ayar-1` yeteneği**dir. Config
okuyucusu olmayan gerçek 0.7 compiler dosyayı sessizce atlayabilir; 0.7'ye
veya bir önceki küçük sürüme güvenli rollback kabulü verilmez.

```json
{
  "sema": "zee-ayar-1",
  "surum": 1,
  "ayarlar": {},
  "guvenlik": {
    "web-modu": "proxy",
    "proxy-origin": "https://panel.example.invalid",
    "worker-port": 18123,
    "oran-deposu": "bellek",
    "oran-siniri": 25,
    "oran-kapasitesi": 120,
    "oran-doluluk": "reddet"
  }
}
```

Zarfın dört alanı zorunludur. `sema` exact `zee-ayar-1`, `surum` JSON tam
sayısı `1`; `ayarlar` ve `guvenlik` nesnedir. Eksik/fazla üst alan, gelecek
şema/sürüm, boş veya geçersiz JSON, her derinlikte yinelenen anahtar
(Unicode kaçışıyla yazılan eş ad dahil) exit 2 ile kaynak çalıştırılmadan
reddedilir. Config hata mesajları kullanıcı anahtarını/değerini, JSON
alıntısını veya dosya yolunu basmaz.

`ayarlar` yalnız **güvenlik etkisi olmayan olağan uzantı** alanıdır. Şema 1
henüz bu daldan davranış ayarı uygulamaz. Her alan/değer varsayılan davranışı
koruyarak atlanır ve tek sabit uyarı çıkar: `zee.ayar: bilinmeyen olağan
ayarlar atlandı; varsayılanlar korundu.` Anahtar/değer adından güvenlik
niteliği tahmin edilmez. Güvenlik gereksinimi bu dala yerleştirilerek
etkinleştirilemez; üretici yalnız `guvenlik` dalını kullanır.

`guvenlik` dalının tek izin listesi aşağıdadır. Bilinmeyen bütün anahtarlar
adı ne olursa olsun reddedilir; tip, aralık ve anahtarlar arası bağımlılıklar
CLI override'dan önce doğrulanır.

| Alan | Tip / izinli değer | Mevcut CLI eşlemesi / varsayılan |
|---|---|---|
| `cocuk-modu` | boolean | true: `--güvenli`; false/yok: kapalı |
| `web-modu` | `kapali`, `deneysel`, `proxy` | yok: kapalı; deneysel: `--deneysel-web`; proxy: `--web-proxy` |
| `proxy-origin` | HTTPS origin, kullanıcı bilgisi/yol/query/fragment yok | proxy modunda zorunlu |
| `worker-port` | tam sayı 1..65535 | `--web-worker-port`, yalnız proxy |
| `oran-deposu` | `bellek` | `--web-oran-deposu`, yalnız proxy; yok: mevcut kalıcı depo |
| `oran-siniri` | tam sayı 1..1000000 | `--web-oran-siniri`, bellek deposunda 60 sn penceresi |
| `oran-kapasitesi` | tam sayı 1..32768 | `--web-oran-kapasitesi`, bellek deposunda |
| `oran-doluluk` | `reddet`, `en-eski` | `--web-oran-doluluk`, bellek deposunda |

Varsayılan oran/k kapasite mevcut kaynak sınırlarından gelir; config yeni
web yetkinliği açmaz. Proje bildiriminin yetkinlik/hedef listesi aynı kalır.
Deneysel mod mevcut localhost prototip uyarısını, proxy mevcut güvenlik/
restart/tahliye uyarılarını basar. Config kullanımı proxy'yi üretim kabulü
veya birden çok worker ortak kotası haline getirmez.

Öncelik: mevcut güvenli varsayılanlar → geçerli proje config'i → kaynak
yolundan **önceki** açık CLI bayrakları. Aynı alanın CLI karşılığı config
bayrağını bastırır; Türkçe/ASCII çocuk ve proxy alias'ları aynı alandır.
Açık deneysel/proxy modu config'in mod/origin çiftini bastırır. Diğer alanlar
bağımsız kalır: proxy worker/bellek alanlarıyla birleşen deneysel override
çelişkisi reddedilir. Geçersiz explicit değer, tekrar bayrak veya çocuk+web
çakışması sessizce varsayılana dönmez. Mevcut CLI'da kapatma bayrağı olmayan
bir true/mod ayarını kapatmak için config düzenlenir; yeni bayrak eklenmez.

Kök CWD/ev/üst dizinlerden aranmaz. Standalone `.dil` çalıştırması config
okumaz; `denetle`/`dene` gibi yürütme dışı komutlar da okumaz. Kaynak sonrası
program argümanları config yolu değildir. Config taşıyan kökte normal
`proje.dil` gerekir; root yolunda `..` veya sembolik bağ, config/bildirim
symlink'i, dizin/FIFO gibi config girdisi reddedilir. Yol 4096 bayt/64
bileşen, dosya 65536 baytla sınırlıdır; dosya sınırlı okuma ve son bileşende
nofollow/reparse denetimiyle açılır. JSON'un standart recursion sınırı
korunur. Yerel aynı kullanıcı tarafından eşzamanlı bütün dosya ağacı
mutasyonu için transaction/sandbox güvencesi verilmez; proje dosyalarını
çalıştırma sırasında değiştirmek desteklenmez. Windows fiziksel reparse/
minOS kabulü ayrı açık kapıdır.

## Seçici ve geri dönüş sınırı

Seçici `zee.ayar` taşıyan bir proje dizinini eski binary'ye başlatmadan önce
authenticated binary'de şu **ayrı** sorguyu başarıyla çalıştırmalıdır:

```sh
dil sürüm --ayar-semasi
```

```json
{"schema":"zee-config-capability-1","config_schemas":["zee-ayar-1"],"unknown_security":"reject","version":"0.8.0-dev"}
```

Exit 0, tek JSON nesnesi, exact capability şeması, `zee-ayar-1` desteği,
`unknown_security=reject` ve authenticated build sürümüyle eş `version`
zorunlu guard girdileridir. Sorgu yok/başarısız/bozuk veya destek yoksa
başlatılmaz. Bu sorgu `dil sürüm --json` / LSP / SBOM `zee-build-v1`
kimliğini değiştirmez. Birlikte `--json`/extra argüman reddedilir. Şema 1
okuyan compiler'da gelecekte eklenen olağan alanların atlanması test edilir;
bu test gerçek eski 0.7 okuyucusu, ürün staging veya önceki küçük sürüm
release rollback kanıtı değildir.

## Kanıt ve etki

`compiler/tests/cli_ayar_testi.rs` gerçek CLI'da varsayılan, bilinmeyen olağan
alan, exact güvenlik reddi, bozuk/gelecek/duplicate JSON, sınır, web/çocuk
mapping, explicit precedence, kök/symlink ve capability sınırlarını yürütür.
`compiler/src/cli/ayar.rs::tests` effective `WebModu` ve exact oran/worker
alanlarını, tek allowlist, aralık/tip ve yol sınırını ayrıca doğrular.
[ADR-097](../adr/097-surumlu-runtime-ayarlari.md) güvenlik ve uyumluluk
kararını açıklar. GB-057 kabul sınırıdır; tam önceki küçük sürüm rollback,
insan/product/minOS/RC kabulü açık kalır.

Yeni gramer, tanı kodu, dependency, public API, cache, owner veya sahipler
arası kenar eklenmez: `src/cli/ayar.rs` var olan `dil_cli` adaptör sahibine
aittir. Yeni kısa ömürlü bounded config okuması, uzun ömürlü tampon/soak veya
sabit benchmark iş yükü değildir. Faz sahibi CLI; gerçek kaynak çalıştıran
vakalar proje/web/IO/runtime aşağı kapsamını taşır. Spec/27, uyumluluk biçim/
profil kayıtları, faz fixture, canlı sayılar, sürümler/README/backlog/kapılar
root tarafından aynı toplu değişiklikte güncellenir; temiz exact kaynak
kabulü bu adayın kirli test koşusuyla tamamlanmış gösterilmez.

## K-274 kontrollü config yenileme sınırı

Mevcut `zee.ayar` başlangıç okuması, geçerli dosyanın atomic replace edilip HUP ile
yeni worker nesli açılması halinde yeniden kullanılır. CLI override önceliği
korunur. Bu yalnız config istisnasıdır: çalışan `.dil`/modül kaynaklarını genel
sıcak mutation/reload sözleşmesine açmaz. Bellek oran sayaçları restart'ta sıfırlanır;
state koruma/zero downtime sözü yoktur. İki owned worker ile config100→101 HUP
kısa native5 cohort'unda gözlenir; platform/systemd/gerçek ürün ve finalclean
kabulü ayrıca açıktır. [ADR-132](../adr/132-sureli-hazirlik-ve-kontrollu-config-yenileme.md).
