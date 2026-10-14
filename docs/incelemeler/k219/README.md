# K-219 — Oran ayarı kabul kanıtı

Ürün talebi §3/2; [karar](../../../adr/090-web-oran-ayarlari.md).
Aynı exact ürün kökenli 100 kartlık kaynak kullanılır; müşteri/sunucu bilgisi
ve canlı ürün durumu kopyalanmaz. CLI, kota ve kapasite churn testleri, 1800 sn soak ve Gitsrv kabulü
aşağıdaki gerçek sonuçlarla kapandı.
Kaynak `35955470fdb4fc8474f39326ae0f7aa771b5515b`. Yerel tam faz [786/786, sıfır atlama](macos-faz.md), clippy/fmt, katalog/tazelik, spec drift (183 madde), mimari/SCC/API/uyumluluk ve sürekli güvenlik kapısı geçti. Exact soak, ürün kapasite yükü ve Gitsrv #26 kabulü aşağıdaki gerçek kanıtla tamamlandı; performans tarihçesine yeni taban yazılmaz.

Varsayılan reddet; explicit en-eski yalnız uç nokta pencerelerini tahliye eder.
Tahliye edilen uç nokta tekrar gelirse kota sıfırlanabilir. Login/CSRF canlı
sayacı korunur; tüm anahtarlar korunmuşsa yeni anahtar 503 alır. Ayrı worker
ve restart kotası ortak değildir; proxy ortak saldırı bütçesi gerekir.


## Exact ürün kapasite yükü

[yuk.json](yuk.json) temiz kaynak 35955470fdb4fc8474f39326ae0f7aa771b5515b,
release varsayılan profil, Rust 1.93.1, Apple M4 Pro / 24 GiB / macOS 26.6.2
üzerinde gerçek 100 kartlık ürünü ölçer. Binary ve program SHA-256 JSON’dadır.
Aynı worker’da sırayla iki senaryo, her biri tek örnek; explicit ısınma yok.
1000 isteklik ilk senaryo ikinci senaryodan önce worker’ı ve durumu ısıtır.
Bu gözlem kalıcı benchmark tabanı veya hard eşik değildir.

- 1000 / C10: 2934 istek/sn, 1000×200.
- Ardından 40000 farklı IP/path anahtarı / C10: 2944 istek/sn, 40000×200,
  13,62 saniye; 60 saniyelik pencere dolmadan kapasite aşımı gerçekleşir.
- Sonra yeni aynı IP/path: 250×200, 251’inci 429. Tahliye edilmiş eski
  uç noktanın eşiğinin korunması iddia edilmez; proxy bütçesi koşulu sürer.
- Tüm worker RSS 22464 KiB; anonim yükte oturum dosyası oluşmadı.

Tekrar üretim: kaynak yorum başlığı hariç ürün programı aynıdır; korpus
başlığı semantiği değiştirmez. İzole geçici klasörde programı aşağıdaki
CLI ile başlatıp özgün betiği sırayla çalıştırın; yalnız kendi worker PID’nizi
kapatın, ürün `.zee` durumuna dokunmayın.

```sh
dil çalıştır --web-proxy https://itwise.test --web-worker-port 18097 \
  --web-oran-deposu bellek --web-oran-siniri 250 \
  --web-oran-kapasitesi 32768 --web-oran-doluluk en-eski yuk.dil
python3 yuk2.py 1000 10 18097 100000
python3 yuk2.py 40000 10 18097 400000
```

Kota kontrolü gerçek worker testinde düşük sınırla her CI’da tekrarlanır;
250 sınırıyla exact ölçüm ayrıca JSON’da kayıtlıdır. Geçici ölçüm sürücüsünün
ilk denemesi göreli betik yolu nedeniyle başlayamadı; düzeltme sonrası yeni
worker ve boş durumla alınan tamamlanmış koşu kabul edilir.

## Windows kapsamı

[Exact MSVC kanıtı](windows-capraz.json): cargo-xwin 0.23.1 ile
x86_64-pc-windows-msvc all-targets clippy `-D warnings` ve release dil/dillsp
çapraz derlemesi geçti. Windows üzerinde runtime testleri veya dağıtım
artefaktı üretilmedi. Bu işler Mac soak’ın ilk dakikalarında eşzamanlıydı;
soak RSS’i ve gerçek döngü sayıları raporda korunur.

## Exact dayanıklılık

[1800 sn ham rapor](soak.md), temiz kaynak
35955470fdb4fc8474f39326ae0f7aa771b5515b, Rust 1.93.1 release;
60 sn pencere, 360 örnek, ilk ısınma penceresi dışarıda, 26530 derleme/LSP
döngüsü. Her döngü 32768 kayıt + 4096 tahliye ve sonraki süre dolumunu
çalıştırır. Derleyici medyan RSS 20320→20016 KiB, dillsp 17248→16960 KiB;
ikisinde büyüme %0, kapı geçti. Yeni satır soak tarihçesine eklendi, eski
sonuçlar değiştirilmedi. Kaynak ağacı koşu boyunca temiz kaldı.


## Gitsrv kapanışı

[Exact CI kaydı](gitsrv-ci.json): kaynak 50eb70500f627447b4a25d85228f39450489661d,
özel Forgejo Gitsrv #26 / run 758, 01:44:48–02:26:03 UTC, **başarılı**.
Compiler kaynak commit’i 35955470fdb4fc8474f39326ae0f7aa771b5515b ile aynıdır.
Sonuç kimlikli API’den doğrulandı; artefakt HTTP Basic ile ZIP dönmediği için
Linux alt test sayısı/ham log içeriği ayrıca iddia edilmez. Akışın kalite,
gizlilik, tedarik, WASM, gözlemsel performans ve MSVC adımları aynı başarılı
workflow’dadır; Windows runtime ve release-candidate kapıları ayrı açıktır.

Etki incelemesi: tanı kodu ve hata kataloğu, bağımlılık graph’ı/katman fixture,
api::v1 facade, eski performans tarihçesi ve sürüm artefaktı sözleşmesi değişmez.
Uyumluluk fixture’ında dört host profil girişi aynı kaynak commit’inde kayıtlıdır.
Kaynak, spec/RFC/ADR, olumlu/olumsuz testler ve exact beyan birlikte yayımlandı.
Canlı ürün sunucusu yükseltilmedi. K-219 §3/2 tamamlandı; sonraki iş §3/3 POST’tur.
