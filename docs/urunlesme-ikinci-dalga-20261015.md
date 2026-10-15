# İkinci teknik ürünleşme dalgası — 15 Ekim 2026

Bu sayfa K-280–K-296 için güncel dar yerel S3 kabulünü izler. Exact kaynak
`66e4149e53451111d153f47283c219079a8d1cab`; sonuçlar ve açık sınırlar [S3 teknik kabulü](urunlesme-s3-kabul-20261015.md) raporundadır. S1/S2/dirty aday kanıtları kendi tarihsel kimlikleriyle korunur.

| İş | Gerçekleşen S3 kabulü | Açık sınır |
|---|---|---|
| K-280 / P19 | S3 actual completion RPC111aday, biçim/tanı, shutdown/exit ve iki normal Quit exit0 | Windows/yayın kanalı, tam P19 ve C1 özel quickfix açık |
| K-281 / P11 | S3 amaç bağlı imzalı PDF stdio actual9; üç sayfa/font/görsel | Canlı katalog, genel FFI ve hostile PDF sandbox açık |
| K-282 / P09 | S3 gerçek Linux SQL/calendar/unknown-no-replay/adapter4; K-295 düzeltmesi kabul | Genel provider exactly-once ve ürün adoptionı açık |
| K-283 / P13 | S3 gerçek Linux HTTP/PG/user-unit7 | Tam production/platform/adoption ve ingress SLA açık |
| K-284 / P22 | S3 source11 normal vaka+native3; payda14, dene4/14 ve HTTP6/14 | Üretici candidate etiketi preserved; canonical clean binding ayrıca doğrulandı |
| K-285 / P20 | S3 generic SQL yaşam döngüsü actualMacPG9; escaped HTML | Genel extensionpoints ve ürün adoptionı açık |
| K-286 / P22 | S3 dene native köprü4/19HTTP/iki freshPG+admission6 | FreshDB bütün HTTP commitlerini rollback etmez |
| K-287 / P19 | S3 sync/format/cache/lint/stdIO bağımlılık+budget,1/41release ve1800sRSS kabul | Artımlı HIR/type, UI/transport/lint latency ve yayın kanalı açık |
| K-288 / C1 | Gereksinim ve açık durum kaydı | Beş özel tanı/hızlı düzeltme zinciri; araç incelemesi reddetti |
| K-289 / P21 | S3 katalog/bileşen/layout/slot TR–EN10HTTP+4Zee | Yerleşik çevir builtin yok; gerçek ürün adoptionı açık |
| K-290 / P23 | S3 içerik+kimlik+medya+SQL actualPGHTTPS14 | İnsan/üretim/platform ve ürün adoptionı açık |
| K-291 / P14 | S3 mixed revision native9/160istek/dört dalga | Genel SLA ve DBşema migrationı değil |
| K-292 / P34 | S3 native worker-loss/stale-gate2; K-296 oracle kabul | Native DNS iptali ve full fault açık |
| K-293 / P33 | Beş alan/20işlev salt okunur matris | Koordineli reproducer çalışmadı; araç incelemesi reddetti, gözlenmiş istismar yok |
| K-294 / GB-110 | Exact fixed_by382beed+minimalregression+nonemptyreplay+S3canonicalnative26+RC | GB-110 dar kapsamda kapalı; normal CLI tanısı korunur |

## Tarihsel kapanma sırası

İlk kaynakta code/test/ADR/spec/uyumluluk/faz/belge kayıtları birlikte bulunur.
K-294 exact regression ve compiler/core-freeze beyanı kaynak SHA bilindikten
sonra izleyen committe kaydedilir. Ardından temiz exact kaynakta tüm normal
kapılar, dedicated native kampanyalar, dört ASan hedefinin en az30dakikası,
seçili Miri ve1800ssoak yürütülür. İkili/kurulum/taşıyıcı ve gerçek IDE
kanıtı aynı kaynağa bağlanmadan final kabul kapanmaz.

C1/P33 retleri [açık inceleme kaydında](urunlesme-acik-incelemeler.md) ayrıdır.
Beş anonim gerçek site korpusu, insan testleri, fiziksel Windows/minOS,
resmî yayın/lisans/destek kararları ve canlı sağlayıcı/ürün kabulü dış sınırdır.
Bunlara sahte test veya tamamlandı durumu yazılmaz.

K-287 otomatik açık belge yayını sonrasında kirli aday release ölçümü:
20.081 satır,40 birim,5 ısınma/20 örnekte tek açık belge p95 19,754 ms,
en yüksek19,926 ms;41 açık belge p95 126,164 ms,en yüksek130,027 ms.
İlk açılış tek belgede29,627 ms,tüm41 belgeyi sırayla açmada2712,895 ms.
Bu üretilmiş iş yükü gözlemidir; gerçek ürün, taşıma/UI veya lint açık
gecikme garantisi ve temiz exact benchmark tarihçesi değildir.

## Kaynak öncesi ortak kapı kanıtı

15 Ekim ikinci tam faz koşusu **988/1013 başarılı, 0 başarısız,
25 atlanan** vaka ile 516,804 saniyede geçti. Atlanan dedicated native
vakalar temiz exact kampanyada ayrıca yürütülecektir. İşlev eğilimi
raporu 74 kritik kayıtla yenilendi ve denetimi geçti; bağımsız incelenmiş
on dar taban eski 64 tabanı ve global büyüme eşiklerini değiştirmez.
Rust biçim, tüm hedeflerde Clippy (`-D warnings`), katalog, altı belge
tazelik testi ve sürekli güvenlik kapısı da geçti. Sürekli kapı compiler
ve fuzz bağımlılıkları için advisory/lisans/ban/kaynak denetimini, stable
fuzz derlemesini ve güvenlik/tedarik testlerini kapsar. Bu sonuçlar kirli kaynak kapısıdır;
clean exact RC, soak, native veya IDE kabulü sayılmaz.

İlk kaynak commit: `382beed1d01440f6f9df28b5ac7a111fbe1f75ee`. K-294 exact
regression/fixed_by ve compiler/core-freeze beyanları izleyen dilimdedir.

Takip diliminin tam faz kapısı **989/1014 başarılı, 0 başarısız,
25 atlanan** vaka ile 540,935 saniyede geçti. İlk takip koşusu 984/1014,
beş hata ile korunur: üretilmiş kanıt özeti yenilenmemişti (dört vaka) ve
başka Cargo işi test sırasında ikiliyi değiştirdiği için native listener
hash koruması reddetti (bir vaka). Özet kanonik Zee ürünüyle yenilendi;
üç ilgili test grubu, ardından başka Cargo işi olmadan tam matris geçti.
Son Clippy, biçim, katalog/tazelik ve semantic/core-freeze korukları temizdir.

## K-295/K-296 — S2 kabulünün ardından yeni kaynak gereği

S2 `e35b617095e6a9d00915794d6adee90842238a5f` final kabul değildir.
macOS iki-klon paketi, canonical stdio probe'ları, TR/EN, signed-PDF9, SQL9,
starter14, kapsam, bridge4/19HTTP+negatif6 ve mixed-revision9 kanıtı korunur.
Linux dispatcher0/4 ve fault stale-gate oracle başarısızlığı K-295/K-296 olarak
ayrıldı. İlk worker-loss geçti; her iki başarısız native koşunun cleanup'ı temizdir.
S2 ASan/soak kampanyaları sahipli cancel-file ile erken kapatılmış, outer
handle'lar reaped olarak kaydedilmiştir; 30 dakika/1800 saniye kabulü sayılmaz.
Bu bölüm S3 öncesi aday kaydını korur; iki düzeltmenin actual temiz
ortak/native/RC/Miri/soak ve JetBrains kabulü [S3 teknik kabulü](urunlesme-s3-kabul-20261015.md) raporundadır. Compiler/grammar/tanı/yüzey/API/katman/bağımlılık
değişmedi; C1/P33 araç retleri ve insan/platform/ürün kabulü açık kalır.

K-295/K-296 kaynak öncesi kapılar: dispatcher22 ve fault7 hazırlık,
Clippy all-target -D warnings, biçim, katalog/tazelik/güvenlik odak grupları
ve tam faz **989/1014, 0 hata, 25 atlanan** geçti (527,915 sn).
Semantic/core-freeze korukları temizdir. Compiler/src değişmediği için yeni
compiler/core-freeze beyan satırı eklenmez; S3 temiz kabulü bundan ayrıdır.

## S3 — ikinci dalga temiz kaynak kabulü

Kaynak `66e4149e53451111d153f47283c219079a8d1cab` üzerinde ikinci dalganın native senaryoları, macOS/Linux için iki temiz klondan teslim ve sekizer kurulum testi, gerçek JetBrains tamamlama/biçimlendirme/tanı/kapanış akışı, 1.800 saniyelik soak, 1 ve 41 açık belge ölçümü, dört ASan hedefi ve seçili Miri testleri geçti. Kanonik kanıt dışa aktarımı ve denetimi ile aynı kaynakta güvenlik kapısı geçti: tam Cargo koşusunda 989 başarılı, 0 başarısız, 25 varsayılan dışı vaka. Bu 25 vaka ayrıca çalıştırılan kampanyalara eşlendi; alt senaryolar Cargo toplamına eklenmez. GB-110 exact regression/native26/RC zinciriyle kapalıdır.

[S3 teknik kabulü](urunlesme-s3-kabul-20261015.md), sonuçları ve önceki başarısız denemeleri ayrı kaydeder. E commitleri belge ve kanıtları kaydeder. C1/K-288 ve P33/K-293 araç retleri; artımlı HIR/type, gerçek ürün/provenance/adoption, insan/takvim, fiziksel Windows/minOS ve yayın/lisans/destek kapıları açıktır. Bu kayıt tam V1 ya da public signed release kabulü değildir.
