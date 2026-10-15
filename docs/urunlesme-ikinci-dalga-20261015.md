# İkinci teknik ürünleşme dalgası — 15 Ekim 2026

Bu sayfa K-280–K-294 aday kaynak durumudur. Tarihsel S1
`8b49113c413f62b35d4b251ed31068524d2fe26c` ve E1
`9214a3e676bc1940ae9c3b82af4fafadb5f9f0fd` kanıtı yeni compiler kaynağının
kabulü değildir. Aşağıdaki aday testler temiz exact kampanya yerine geçmez.

| İş | Gerçekleşen aday | Son kabulde açık |
|---|---|---|
| K-280 / P19 | Gerçek JetBrains keyword completion ve normal Quit exit0; restart shutdown/exit ayrıca gözlendi | Yeni LSP kaynağıyla UI tekrarı, final Quit konsolu ve Windows |
| K-281 / P11 | Amaç bağlı imzalı PDF stdio host: model7, native9; gerçek Zee istek/yanıtı | Yeni clean exact kaynak; canlı katalog, genel FFI ve hostile PDF sandbox |
| K-282 / P09 | Sürekli dispatcher hazırlık21 ve sahipli adaptör | Gerçek Linux SQL/calendar/adapter4 |
| K-283 / P13 | Sürekli HTTP service hazırlık18 ve eski watcher10 | Gerçek Linux HTTP/PG/user-unit7 |
| K-284 / P22 | Kaynak karar kapsamı11 Rust vaka; çağrılmayan dal paydası, köken, bounded rapor, gerçek native HTTP | Yeni temiz kaynak ortak kapıları |
| K-285 / P20 | Modül SQL yaşam döngüsü v3 model10 ve gerçek MacPG9; kaçışlı JSON düzeltmesi bağımsız incelendi | Yeni temiz exact kaynak |
| K-286 / P22 | Dene birim testinden sonra pinli native kit; dört vaka,19HTTP, iki freshPG ve admission negatif6 | Yeni clean exact; HTTP commitlerinin tümünü rollback etme vaadi yok |
| K-287 / P19 | UTF16 sync2, biçim, token/yaprak AST, taze grafik/kilit, açık tüketici yenileme, opt-in lint | Clean exact1/41açıkbelge ölçümü ve1800sRSS; artımlı HIR ve yayın kanalı |
| K-288 / C1 | Gereksinim ve açık durum kaydı | Beş özel tanı/hızlı düzeltme zinciri; araç incelemesi reddetti |
| K-289 / P21 | Katalog/bileşen/layout/slot ile metin birleştirmesiz TR/EN liste-detay10HTTP+4Zee | Yeni clean exact ve gerçek ürün adoptionı; yeni çevir builtin yok |
| K-290 / P23 | İçerik+kimlik+medya+parametreli SQL başlangıcı; model8 ve gerçek PostgreSQL/HTTPS14 | Donmuş son paket hazır; clean exact ve rehberle gerçek insan kabulü |
| K-291 / P14 | Farklı kod sürümlerinin devir hazırlığı8 | Native9,160istek/dört dalga; genel SLA/DBşema geçişi değil |
| K-292 / P34 | Worker fault ve kontrollü Python resolver hazırlığı7 | Native2; native DNS iptali kanıtı yok |
| K-293 / P33 | Beş alan/20işlev salt okunur matris | Koordineli reproducer çalışmadı; araç incelemesi reddetti, gözlenmiş istismar yok |
| K-294 / GB-110 | Kontrol FD alias tanı sızıntısı düzeltildi; actual26 geçti | Exact fixed_by ve minimal regression kaydedildi; temiz kaynak kabulü bekleniyor |

## Kapanma sırası

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
İki düzeltme uygulanmış adaydır; yeni temiz kaynakta ortak/native/RC/Miri/soak
ve JetBrains kabulü beklenir. Compiler/grammar/tanı/yüzey/API/katman/bağımlılık
değişmedi; C1/P33 araç retleri ve insan/platform/ürün kabulü açık kalır.

K-295/K-296 kaynak öncesi kapılar: dispatcher22 ve fault7 hazırlık,
Clippy all-target -D warnings, biçim, katalog/tazelik/güvenlik odak grupları
ve tam faz **989/1014, 0 hata, 25 atlanan** geçti (527,915 sn).
Semantic/core-freeze korukları temizdir. Compiler/src değişmediği için yeni
compiler/core-freeze beyan satırı eklenmez; S3 temiz kabulü bundan ayrıdır.
