# ADR-158 — Proje başına oturum ömrü ve kayar yenileme

- **Durum:** kabul
- **Tarih:** 28 Ekim 2026
- **İlgili kayıt:** K-330, ADR-059 (core freeze), ADR-064 (dil yüzeyi),
  ADR-067 (dogfood korpusu), ADR-018 (mutlak ömür), spec/12 §2–§4, GB-002,
  GB-127

## Bağlam

K-088'den beri kimlikli web oturumu, oluşturma anından başlayan **mutlak 30
dakika** ile sınırlıydı; erişim yalnız tahliye sırasını güncelliyor, son
geçerlilik anını taşımıyordu. Çerez `Max-Age=1800` ile yazılıyordu ve proje bu
değeri değiştiremiyordu. Gerçek mini ERP ürünü (ürün G, `dogfood/urun-g`)
gün boyu açık kalan formlarda kullanıcıyı iş ortasında girişe atıyordu; ürün
sahibinin talebi "oturum kullanıcı çıkış yapana kadar etkin kalmalı" idi.

Core freeze (ADR-059) yeni kalıp açmayı reddeder. İhtiyaç bir dil kalıbı
değil, proje sahibinin dağıtım politikasıdır; `proje.dil` zaten yetkinlik,
ağ hedefi ve sır değişkeni adı gibi politika kararlarını taşır.

## Karar

1. `proje.dil` iki isteğe bağlı alan kabul eder:
   - `oturum_ömrü_saniye N olsun` — kimlikli oturum ömrü, sabit Sayı,
     **300 – 2 592 000 sn (5 dakika – 30 gün)**. Sınır dışı değer P015,
     Sayı olmayan değer P001 ile reddedilir. Verilmezse 1800 sn.
   - `oturum_kayar doğru olsun` — Mantıksal; verilmezse `yanlış`.
   İkisi de `web-oturumu` yetkinliği ister; yetkinlik yoksa P015.
2. Kayar kapalıyken davranış bugünküyle aynıdır, yalnız ömür ve ilk çerez
   `Max-Age` değeri seçilen N olur.
3. Kayar açıkken her **kimlikli** istekte, istek başındaki kalıcı depo CAS
   güncellemesi içinde `son_geçerlilik = min(şimdi + N, oluşturma + 30 gün)`
   yazılır (var olan geçerlilik asla kısalmaz) ve yanıt aynı belirteçle
   `Max-Age = son_geçerlilik − şimdi` taşıyan oturum çerezini yeniden yazar.
   Böylece N, bir **boşta kalma** sınırına dönüşür.
4. Anonim (giriş öncesi) form oturumu ayardan etkilenmez: mutlak 10 dakika,
   yenileme yok.
5. Giriş, `oturumu kapat` ve kendi oturumunun kimlikle iptali aynı istekte
   kayar çerez yenilemesini düşürür; çıkış yanıtı yalnız `Max-Age=0` silme
   çerezini taşır ve sunucu kaydı commit ile hemen silinir.
6. Hermetik `dil dene` adaptörü proje ayarını uygulamaz; varsayılan 30
   dakika mutlak kalır (açık sınır, testler zamanı açıkça sürer).

## Güvenlik gerekçesi

ADR-018 (GB-002) erişimin ömrü sınırsızca kaydırmasını bir bulgu olarak
kapatmıştı: "mutlak ömür kullanıcının sürekli trafiğiyle sonsuza uzamaz".
Bu karar o sözü korur; değiştirdiği tek şey, proje sahibinin açıkça seçtiği
kipte boşta kalma süresinin kaymasıdır. Varsayılan ve ayarsız projeler
ADR-018 davranışında kalır.

- **Sonsuz oturum yok.** Kayar yenileme, oluşturma anından sayılan 30 günlük
  mutlak tavanı aşamaz; sürekli kullanılan bir belirteç bile en geç 30 günde
  geçersiz olur ve kullanıcı yeniden kimlik doğrular. Üst sınır aynı zamanda
  N'nin üst sınırıdır; çalışma zamanı `OturumAyari::yeni` aralığı ikinci kez
  fail-closed denetler.
- **Oturum sabitleme açılmaz.** Kayar yenileme yeni belirteç üretmez ve
  belirteç kabul etmez; yalnız depoda zaten var olan, kimliği doğrulanmış bir
  kaydın süresini uzatır. Giriş eski/anonim oturumu silip yeni 256 bit
  belirteç + CSRF üretmeye (K-088 döndürmesi) devam eder; anonim oturum asla
  kaymaz, dolayısıyla saldırganın tarayıcıya yerleştirdiği giriş öncesi
  belirteç uzatılamaz ve girişte zaten geçersiz kılınır.
- **Çıkış ve iptal sunucu tarafıdır.** Belirteç çerezin değil depodaki SHA-256
  özetinin varlığıyla geçerlidir; `oturumu kapat`, kullanıcı oturumlarının
  iptali ve parola değişiminde ürünün yaptığı toplu iptal kaydı siler. Çalınmış
  çerez çıkıştan sonra kayar ile canlanamaz.
- **Dağıtım herkesi düşürmez.** `--web-proxy` profili oturumları proje
  kökündeki `.zee/web-durumu-v1.json` kalıcı CAS deposunda tutar (K-137);
  bütün worker'lar aynı dosyayı paylaşır, yeniden başlatma kaydı silmez.
  Kayar güncellemesi aynı CAS turunda yazıldığından eşzamanlı worker'lar
  birbirinin uzatmasını ezmez; geçerlilik yalnız ileri gider.
- **Ek yazma yükü yok.** Çerezli her istek K-088'den beri `son_erisim_ms`
  için aynı depo turunu zaten yazıyordu; kayar yalnız aynı kayda bir alan ekler.

## Bilinen sınırlar (açık)

- Uzun ömür, terk edilmiş oturumların depoda daha uzun kalması demektir.
  Depo 4096 kayıtla sınırlıdır ve yalnız kimlikli kayıtlarla doluysa yeni giriş
  fail-closed reddedilir (mevcut K-088 davranışı). Ürünlere 30 gün mutlak
  yerine makul bir boşta kalma ömrü (ör. 8 saat) + kayar önerilir.
- Tarayıcı kapanınca çerez silinmez (Max-Age taşır); paylaşılan bilgisayar
  riski ürünün ömür seçimine aittir.
- Hermetik `dil dene` adaptörü ayarı uygulamaz (madde 6).

## Reddedilen seçenekler

- **Sabit 30 dakikayı yükseltmek:** her ürün için uzun mutlak ömür dayatır,
  kısa ömür isteyen ürünlerin güvenlik duruşunu zayıflatır.
- **Sınırsız kayar:** çalınmış ama sürekli kullanılan belirteç hiç bitmez.
- **Yeni dil kalıbı (`oturumu yenile`):** core freeze'e aykırı; politika kaynak
  koda değil proje bildirimine aittir.
- **Ayrı mutlak tavan anahtarı:** ihtiyaç kanıtı yok; tek sabit tavan (30 gün)
  minimal yüzeydir.

## Kanıt

- `compiler/src/web_guvenligi/tests.rs` — sınırlar, mutlak kipte uzamama,
  kayar uzama + 30 gün tavanı, boşta bitiş, anonim kaymama, çıkış/iptal,
  kalıcı depoda iki süreç ve yeniden başlatma.
- `compiler/tests/proje_testi.rs` — bildirim okuma, sınır dışı/tür/yetkinlik
  tanıları, proje ve çalışma zamanı sınırlarının eşitliği.
- `compiler/tests/web_cok_surec_testi.rs::kayar_oturum_iki_worker_restart_ve_cikista_dogru_davranir`
  — gerçek iki `dil` worker'ı, çerez yenilemesi, restart, çıkışta tek silme
  çerezi ve diğer worker'da hemen 401.
- `dogfood/urun-g` — ürün bildirimi ve küçültülmüş akış.
