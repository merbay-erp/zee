# RFC-0046 — Makine uç noktası: CSRF'siz, kimliksiz POST rotası

- **Durum:** kabul — 26 Kasım 2026, K-405'te gerçeklendi; normatif metin
  spec/12 §K-405, karar ADR-179.
- **Tarih:** 26 Kasım 2026
- **İş:** K-405 (kaynak: ürün H web arayüzü sürtünmesi `ZM-F047`; exact ürün
  provenance'ı [dogfood kesiti](../dogfood/urun-h/outlook-autodiscover/README.md))
- **İlgili belgeler:** spec/11, spec/12 §1 ve §K-335, ADR-034, ADR-059,
  ADR-064, ADR-160, ADR-163 eki (K-385), ADR-176 (K-400), GB-180
- **Freeze sınıfı:** `dogfood-change` — mevcut `herkese açık` politikasının
  tek contextual biçimi ve yalnız o rotada okunan sıfır argümanlı bir
  ifade; yeni değer türü, bağımlılık veya `dil::api::v1` facade değişikliği
  yok. Yeni tanılar S050/T060/T061/T062.

## Sorun

Klasik masaüstü Outlook posta hesabını otomatik kurmak için
`POST /autodiscover/autodiscover.xml` gönderir. Gövde yaklaşık 500 baytlık
`text/xml` belgesidir ve `<EMailAddress>…</EMailAddress>` taşır; yanıt
`application/xml` IMAP/SMTP ayarlarıdır. Ürün H Thunderbird autoconfig'ini
(GET, K-385), Apple profilini ve DNS SRV kayıtlarını zaten sunuyor; yalnız
Outlook POX yolu eksik ve Outlook elle kuruluma düşüyor.

Bugünkü dil bunu iki yerden engeller:

1. Her POST rotası `_csrf` form alanını oturuma karşı doğrular (spec/12 §3);
   `herkese açık` CSRF'yi kapatmaz. Outlook ne çerez ne `_csrf` ne de
   `Origin` gönderir → 403.
2. Gövde her zaman `application/x-www-form-urlencoded` olarak ayrıştırılır;
   ham gövde `Metin` olarak okunamaz. XML'deki `&`/`%` 400 ya da 413'e
   (alan sınırı) yol açar.

CSRF kapısını bir rotada kaldırmak, tarayıcının ortam kimlik bilgisini
(oturum çerezini) taşıyan istekleri saldırgan sitenin tetiklemesine kapı
açar. Bu yüzden asıl tasarım sorusu, CSRF'siz bir rotanın **hiçbir koşulda**
tarayıcı kimliğine ulaşamamasını derleyicinin garanti etmesidir.

## Öneri

```zee
POST "/autodiscover/autodiscover.xml" adresine istek geldiğinde
    herkese açık makine uç noktası
    gövde isteğin ham gövdesi olsun
    …
```

- `herkese açık makine uç noktası` mevcut açık politikanın tek biçimidir
  (K-335 `önbelleklenebilir` gibi). Oturumlu/yetkili biçim ya da tek başına
  `makine uç noktası` S050'dir; kimlikli rota yapısal olarak makine ucu
  olamaz.
- Yalnız POST, sabit ve öneksiz yol (T060). Native adaptör yolu gövdeyi
  okumadan derlenen kümeden tanır; önekli/hesaplanan yol bu kararı
  belirsizleştirirdi.
- Rota gövdesi `oturum`/`çerezler` okuyamaz; gövde ya da çağırdığı işlemler
  `csrf belirteci`, çerez yazma/silme, `oturuma al`, `oturumu kapat`, web
  oturumu listeleme/iptal kullanamaz (T061, derleme zamanı).
- `isteğin ham gövdesi` yalnız makine rotasının kendi gövdesinde okunur
  (T062): en çok 8 KiB, katı UTF-8 `Metin`.
- Çalışma zamanı: CSRF ve Origin denetimi yok; istek çerezi hiç okunmaz,
  oturum anonim başlar, kayar yenileme yapılmaz ve yanıt hiçbir koşulda
  `Set-Cookie` taşımaz. İçerik türü yalnız `text/xml`, `application/xml`,
  `application/json` (varsa `charset=utf-8`) — diğerleri 415; 8 KiB aşımı
  gövde okunmadan 413; geçersiz UTF-8 400.
- Değişmeyenler: Host/Forwarded/HTTPS proxy kapısı, CSP ve güvenlik
  başlıkları (`no-store` dahil), başka rotaların gövde sınırları, uç nokta
  oran sınırı (rota kendi `POST <yol>` kovasındadır) ve K-400 üretim
  sözleşmesi.

## Tehdit modeli

| Tehdit | Değerlendirme | Sınır |
|---|---|---|
| CSRF / CORS kötüye kullanımı | Saldırgan site kurbanın tarayıcısından makine ucuna istek gönderebilir. Etkisi yoktur: rota ortam kimlik bilgisi görmez (çerez okunmaz, oturum anonim, T061 derlemede), yanıt Set-Cookie taşımaz, durum değişikliği kimliğe bağlanamaz. Tarayıcının basit (preflight'sız) istek türleri — form-urlencoded, multipart, text/plain — 415 ile reddedilir; XML/JSON türü CORS preflight gerektirir ve sunucu CORS başlığı yazmadığından tarayıcı yanıtı okumaz. Saldırganın elde ettiği yanıt, kendi sunucusundan doğrudan isteyebileceği yanıtla aynıdır. | T061, çerezsiz zarf, Set-Cookie bastırma, 415 |
| Önbellek zehirleme | Yanıt `Cache-Control: no-store` taşır (K-335 önbellek izni POST'ta zaten T056); POST yanıtı paylaşılan önbellekte varsayılan olarak tutulmaz. Yanıt yalnız istek gövdesinden türetilir; başlık yansıtmaz. Ürün yanıtı adres yerine sabit ayarlardan kurmalıdır. | `no-store`, POST |
| XML varlık genişletmesi (XXE, "billion laughs") | Dil XML ayrıştırmaz; gövde 8 KiB'lık düz `Metin`dir. Ürün H `<EMailAddress>` değerini metin bölmeyle çıkarır; DTD/varlık hiç yorumlanmaz. Gelecekte bir XML ayrıştırıcı eklenecekse DTD ve dış varlık kapalı olmak zorundadır (bu RFC'nin sözü değil). | ayrıştırıcı yok, 8 KiB |
| Bellek / işçi tüketimi | Gövde tavanı 8 KiB, Content-Length başlıktan okunur ve aşım gövde okunmadan 413'tür; okuma K-372 istek okuma süresine bağlıdır. Uç nokta oran sınırı rotanın kendi kovasında sürer. | 413, oran sınırı |
| Günlük enjeksiyonu / kişisel veri | Çalışma zamanı istek gövdesini ve sorgu dizgisini günlüğe yazmaz; K-324 operatör satırı durum/kod/satır, sorgusuz yol ve denetim karakterleri kaçışlanmış tanı mesajı taşır. Ürün e-posta adresini günlüğe ya da tanı mesajına taşımamalıdır; adres CR/LF içerebilir ve kişisel veridir. Kesitte adres yalnız varlık sınaması için bölünür, hiçbir yere yazılmaz. | çalışma zamanı günlüğü gövdesiz ve kaçışlı; ürün kuralı |
| Alan adı / hesap sayımı | Yanıt adres ya da alan adına göre değişirse (tanınmayan alana 404, tanınana XML) saldırgan hangi alan adlarının barındırıldığını sayar. Öneri: her geçerli biçimli istek için aynı yanıt (ayarlar sunucu düzeyindedir), biçimsiz istek için tek 400; hesap varlığını yanıtla belli etme. Oran sınırı sayımı yavaşlatır, engellemez. | ürün kuralı, oran sınırı |
| İstek kaçakçılığı / zarf karışması | Adaptör makine isteğini açık boş çerez satırıyla iletir; gövde `çerez ` ile başlasa bile çerez sayılmaz. İstemcinin `__zee_ic_oturum_*` alanları zarfa hiç girmez. | açık zarf, çerezsiz zarf |
| Muafiyetin yayılması | Makine sınıfı istek (POST + makine yolu) makine olmayan bir rotaya düşerse (yalnız öneksiz/sabit olmayan bir önceki rota) o rota ham gövdeyi ve `_csrf`'yi görmez ve 403 ile kapanır. Sıradan POST rotaları Origin ve CSRF ister; testle gösterilir. | fail-closed |

## Alternatifler

- **`herkese açık` CSRF'yi kapatsın** — reddedildi; bugünkü açık form
  rotalarının (giriş, kayıt) CSRF korumasını kaldırırdı.
- **Önsözde ayrı `makine uç noktası` cümlesi** — reddedildi; `oturum gerekli`
  ile birleşebilirdi ve ayrıca yasaklanması gerekirdi. Politika biçimi
  kimlikli rotayı yapısal olarak dışarıda bırakır (ADR-160 ile aynı gerekçe).
- **Proje düzeyi CSRF muaf yol listesi** — reddedildi; karar rotadan uzakta
  kalır, derleyici rotanın oturuma dokunmadığını kanıtlayamaz.
- **Genel ham gövde (her rotada)** — reddedildi; form rotalarında gövde iki
  kez yorumlanır, CSRF'li rotada ham gövde gereksiz yüzeydir.
- **Dilde XML ayrıştırıcı** — reddedildi; varlık genişletmesi ve bağımlılık
  yüzeyi açar, ürün yalnız tek alan istiyor.
- **Uyarı (lint)** — reddedildi; CSRF'siz rotada oturum kullanımı her zaman
  güvenlik hatasıdır.

## İlişkiler

- **ADR-163 eki §5 / RFC-0040 #5(c).** ADR-163 Outlook POX'u açık sınır
  bırakıp dar CSRF muafiyetini DAV işiyle tek modele bağlamıştı. Bu RFC o
  modelin **kimliksiz** yarısıdır: ortam kimlik bilgisi hiç okunmadığı için
  CSRF anlamsızdır. DAV istemcileri ise `Authorization: Basic` ile kimlikli
  ve PROPFIND/REPORT gibi yöntemlerle gelir; kimlikli, çerezsiz muafiyet bu
  RFC'nin dışında kalır ve kendi tehdit modelini (parola oran sınırı,
  uygulama parolası) ister.
- **Yanıt.** MS-OXDSCLI yanıtı istekteki kullanıcıyı anlatır; durağan dosya
  yanıtı (K-385, `attachment`) ara çözümdür. Kullanıcıya göre üretilen
  `application/xml` metin yanıtı ayrı iştir; ürün o zaman da adresi XML'e
  kaçışlayarak yazmak ve alan adına göre farklı hata vermemek zorundadır.

## Açık

- Metin yanıtının içerik türü: bugün `application/xml` yanıtı yalnız dosya
  yanıtıyla (K-385, `attachment`) yazılır; türlü metin yanıtı ayrı iştir.
- Kimlikli makine istemcileri (DAV, Basic) — RFC-0040 #5(c).
- Yol eşleşmesi büyük/küçük harfe duyarlıdır; `/Autodiscover/Autodiscover.xml`
  gibi biçimler için ürün ayrı rota tanımlar.
- `PUT`/`PATCH`, önekli yol, webhook imza doğrulaması, CORS başlıkları ve
  JSON ayrıştırma bu RFC'nin sözü değildir.
