# ADR-036 — Kanonik web proxy origin sınırı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-139, B-052, RFC-0017, ADR-031, ADR-034, V1-P0-03

## Bağlam

Outbound ağ politikası DNS, IPv4/IPv6, port ve şema kurallarını
`AgHedefi` ile kanonikliyordu. Buna karşılık `--web-proxy` CLI'ı kendi
`GuvenliOrigin` ayrıştırıcısını taşıyordu. İki parser aynı güven kavramına
farklı sözdizimi kabul edebiliyor; Host, `Forwarded host` ve `Origin`
karşılaştırmaları kanonik origin kimliği yerine ham metne dayanıyordu.

Production profilinin yalnız loopback reverse proxy'ye güvenmesi belgelenmiş
olsa da kabul edilen bağlantının peer adresi ayrıca doğrulanmıyordu. Bind
değişmezinin ileride gevşemesi proxy tarafından kurulan istemci kimliğini de
güvenilmez kılabilirdi.

## Karar

1. Inbound production web origin'i ile outbound allowlist origin'i aynı
   `yetkinlik::AgHedefi` değer tipini ve aynı özel origin parser'ını kullanır.
   CLI'a ait ikinci bir origin tipi veya parser bulunmaz.
2. `--web-proxy` yalnız HTTPS, yol/sorgu/parça/kullanıcı bilgisi taşımayan,
   geçerli ASCII DNS adı ya da köşeli ayraçlı IPv6 ve 1–65535 portlu origin
   kabul eder.
3. Şema yazımı `https://` olmak zorundadır; DNS adı küçük harfe, IP adresi
   standart metinsel yazımına ve varsayılan HTTPS portu örtük `443`e
   kanoniklenir. `Host`, tek-hop `Forwarded host` ve
   unsafe `Origin` ham metin olarak değil, aynı `AgHedefi` kimliğine
   ayrıştırıldıktan sonra karşılaştırılır.
4. Production listener sabit `127.0.0.1` adresine bind eder. Ayrıca kabul
   edilen socket peer'i loopback değilse istek başlıkları okunmadan 403 ile
   reddedilir. `Forwarded for` ancak bu iki kapıdan sonra oran sınırı kimliği
   olabilir.
5. Parser sahipliği `yetkinlik/origin.rs` dosyasındadır. Mimari test, CLI'da
   ikinci bir `GuvenliOrigin` tipini ve loopback bind/peer kapısının kaybını
   reddeder.

## Reddedilen seçenekler

- **Ham metni case-insensitive karşılaştırmak:** varsayılan port, IPv6 ve
  geçersiz DNS yazımlarında tek kimlik sağlamaz.
- **CLI parser'ını ayrıca sertleştirmek:** bugünkü eşitliği kanıtlasa da iki
  uygulama yeniden drift edebilir.
- **Yalnız bind adresine güvenmek:** güven varsayımını dolaylı bir çağrı
  ayrıntısına bırakır; peer kontrolü ucuz bir defense-in-depth kapısıdır.
- **`X-Forwarded-For` desteği eklemek:** ADR-034'ün tek-hop `Forwarded`
  sözleşmesini gereksiz ve spoof'a açık ikinci kimlik yoluyla genişletir.

## Sonuçlar

- DNS adındaki büyük/küçük harf farkı, açık `:443` ve tek kapanış `/` aynı
  HTTPS origin'idir; farklı port farklı origin olarak kalır.
- DNS etiketi, IPv6 ayraçları ve port aralığı inbound ve outbound yüzeylerde
  aynı fail-closed kurala tabidir.
- Loopback dışı peer, doğru görünen `Host`/`Forwarded`/`Origin` başlıkları
  taşısa bile güvenilir proxy sayılmaz.
- Çok-hostlu harici proxy/backend ve genel internet bind'i V1 sözü değildir.
