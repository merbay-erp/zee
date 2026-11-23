# K-370 — Web oran ve kaynak reddi 500 denetimi

Tarih: 23 Kasım 2026. Taban: `master` `966f4e3`. Kapsam: ZM-F020 (K-365) ve
ZM-F021 (K-368) aynı aileden çıktı — oran ya da kaynak reddi ya yakalanamayan
C-tanısıyla 500'e düşüyor ya da yanlış sonuç olarak dönüyordu. Bu belge iki
yüzeyi baştan sona tablolar ve gerçek native sunucuda yeniden üretilen tek
yanlış davranışı kaydeder. Normatif kaynaklar: spec/11 §5 (istek-yerel hata
sınırı), spec/12 (oturum deposu, oran tablosu), spec/24 (kaynak bütçesi).

## 1. `RateLimitTuru` çağrı yerleri

Üç kova türü vardır (`web_guvenligi::RateLimitTuru`): `Ucnokta`, `Csrf`,
`Giris`. E-posta, yükleme, oturum açma/kapama ve oturum yönetimi için ayrı
oran kovası yoktur; bunların sınırları kaynak bütçesi ya da depo kapasitesidir
(bölüm 2). Gerçek adaptör `main.rs`, deterministik test adaptörü
`yorumlayici/toplayan_io.rs` aynı kararı verir.

| Çağrı yeri | Kova / anahtar | Pencere / eşik | Aşımda | Depo hatasında | spec/12 tablo satırı |
|---|---|---|---|---|---|
| İstek başı (`istegi_baslat`, rota öncesi) | Ucnokta / `YÖNTEM yol` | 60 sn / 100 (ayarlı) | 429 `uç nokta oran sınırı aşıldı`, rota çalışmaz | 503 | var |
| `rota_guvenligini_denetle` (POST/PUT/PATCH/DELETE) | Csrf / `csrf` | 60 sn / 60 | 429 `çok fazla CSRF doğrulama isteği` (WebReddi) | 503 | var |
| `parola_dogrula` (web isteği) | Giris / `parola` | 300 sn / 5 | 429 ilk yanıt + rotaya `yanlış` | 503 + `yanlış` | var |
| `parola_ozeti_uret` (web isteği) | Giris / `parola` | 300 sn / 5 | Ç002 → rota kesilir, geri alma, 429 (K-365) | Ç002 → 503 | parola satırı + §5 ZORUNLU |
| `totp_dogrula` (web isteği) | Giris / `totp` | 300 sn / 5 | 429 ilk yanıt + `yanlış` (K-368) | 503 + `yanlış` | var (K-368) |
| Web isteği dışı parola/özet/TOTP | — | — | kova yok | — | §5 ZORUNLU (web dışı) |
| Bellek oran deposu kapasitesi (32.768 canlı anahtar) | hepsi | — | yeni anahtar 503 (fail-closed) | — | TANIMLI madde |

Sonuç: yeni oran açığı yok. Aşımın rotaya `yanlış` olarak görünmesi bilinen
spec/12 §5 AÇIK maddesidir (`#f0003f30d3695cf6`), bu işte değişmedi.

## 2. Web isteğinde yakalanamayan C-tanıları

İstek-yerel sınır (`web_istek::istek_hatasi_yaniti`) tanıyı yalnız koduna
göre yanıta çevirir: C013/C021/C027 ve C023/C024 503, diğer C/D 500, C000 ve
Ç nöbetçileri süreci bitirir (Ç001 504, Ç002 ve K-370 ile Ç003 istek
döngüsünde ayrı eşlenir). `Sonuç` döndüren biçimler (`... dene`, PostgreSQL,
e-posta, yeni HTTP isteği, oturum listeleme/iptal) Zee kodunca yakalanır ve
bu sınıra ulaşmaz.

| Kod | Web rotasında nasıl oluşur | Yanıt | Sınıf | Karar |
|---|---|---|---|---|
| C001, C008, C009, C011 | iç savunma | 500 | sunucu/iç hata | doğru |
| C002, C003, C006, C007, C010 | program aritmetiği/koleksiyon hatası | 500 | program hatası | doğru |
| C004 | istemci alanını `sayısı` ile korumasız çevirme | 500 | program hatası (girdi doğrulanmamış) | doğru; ürün `alanı gerekli`/denetim kullanır |
| C005 | soru girdisi yok (web'de etkileşim yok) | 500 | program hatası | doğru |
| C012 | `dene`siz dosya okuma | 500 | sunucu yapılandırması | doğru |
| C013 | dosya yazımı | 503 | geçici/kaynak | doğru |
| C015, C016 | `dene`siz CSV/JSON okuma | 500 | sunucu verisi | doğru |
| C017 | sunucu kurulumu (yalnız açılış) | — | rota dışı | uygulanmaz |
| C018 | eski `adresinden getir` GET ağ hatası | 500 | dış bağımlılık | spec/11 §5 kararı; yeni `HTTP isteğini dene` `Sonuç` döndürür |
| C019 | çağrı derinliği 500 | 500 | program hatası | doğru |
| C020 | geçersiz `programı N ile bitir` | 500 | program hatası | doğru (spec/11 §5) |
| C021, C027 | eylem transaction'ı / COMMIT belirsiz | 503 | geçici | doğru |
| C022 — çerez adı/değeri, yönlendirme reddi | program/güvenlik hatası | 500 | program hatası | doğru |
| C022 — `csrf belirteci` / `oturuma al` depo reddi | kapasite dolu, depo IO, kilit sınırı | **500 (hata)** → 503 | **kaynak durumu** | **düzeltildi (K-370, Ç003)** |
| C022 — `oturumu kapat` | yetkinlik politikası | 500 | yapılandırma | doğru |
| C023, C024 | istek adım/çıktı/heap bütçesi | 503 | kaynak | doğru (spec/24) |
| C025 | PostgreSQL | `Sonuç` | yakalanabilir | sınıra ulaşmaz |
| C026 | göç/bağlantı (yalnız CLI) | — | rota dışı | uygulanmaz |
| C028 — oran aşımı | parola özeti kovası | Ç002 → 429/503 | oran | doğru (K-365) |
| C028 — diğer | kriptografi/gizli değer üretimi | 500 | sunucu hatası | doğru |
| C029 | e-posta | `Sonuç` | yakalanabilir | sınıra ulaşmaz |

İstek commit'i (`istek_islemini_tamamla`) sonrasında oturum deposu kaydı
reddederse yanıt zaten 503 `web güvenlik durumu güvenle kaydedilemedi` idi;
istek başında depo okunamazsa 503 `web güvenlik deposuna erişilemedi`. Bu iki
yolla tutarsız kalan tek yer rota içindeki oturum yeri denetimiydi.

## 3. Yeniden üretim (gerçek native worker)

`dil çalıştır --web-proxy https://panel.example --web-worker-port <kapı>`
ile `ağ-sunucusu` + `web-oturumu` yetkinlikli proje; ilk istek ortak
`.zee/web-durumu-v1.json` deposunu kurar, depo 4096 geçerli kimlikli oturumla
yeniden yayımlanır, ardından oturumsuz istemci `GET /form` (`csrf belirteci
yanıtını gönder`) ister.

| İkili | `/form` | Operatör günlüğü | Sonraki `/` |
|---|---|---|---|
| `966f4e3` (düzeltme öncesi) | `500 istek işlenemedi; sunucu hatası` | `durum=500 kod=C022 … web oturumu kapasitesi dolu; yeni giriş reddedildi.` | 200 |
| K-370 | `503 web güvenlik durumu güvenle kaydedilemedi` | `durum=503 kod=Ç003 … web oturumu kapasitesi dolu; yeni giriş reddedildi.` | 200 |

`oturuma al` aynı eşlemeyi kullanır; ancak bu cümle önce isteğin mevcut
oturumunu sildiği ve POST rotası CSRF için oturum gerektirdiği için kapasite
reddi orada pratikte oluşmaz, yalnız depo IO/kilit hatasıyla görülür.
Regression: `regression/security/oturum-deposu-dolu.dil` (kip
`native-web-oturum-deposu`).

## 4. Kapsam dışı

- Rotaya aşımda `yanlış` dönen parola/TOTP doğrulaması: spec/12 §5 AÇIK.
- Eski GET'in C018'ini 503 yapmak: spec/11 §5 sınıf kümesinin değişmesi
  gerekir; ürün `HTTP isteğini dene` ile kendi yanıtını seçebilir.
