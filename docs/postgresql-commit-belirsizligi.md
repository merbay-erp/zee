# PostgreSQL COMMIT belirsizliği provası

Bu runbook K-163/F028'in gerçek wire-level kanıtını yeniden üretir. Araç yalnız
loopback'te tek hedef bağlantıyı bozar; production proxy veya güvenlik katmanı
değildir.

K-242 / [ADR-103](../adr/103-native-commit-belirsizligi-kaniti.md) aynı
anonim parola helper'ında dört yeni kontrol/arızayı [şema 2 kampanyasına](parola-goc-ci.md)
bağlar. Final temiz exact native kabulü bu aşamada bekleniyor; eski F028
ve 7/7 TLS kayıtları kendi kaynak ve kapsamlarında tarihsel kalır.

## Araç

```bash
python3 scripts/postgresql-commit-proxy.py \
  --mode before --listen-port 55439
```

`before`, simple-query `COMMIT` çerçevesini PostgreSQL'e iletmeden iki ucu
kapatır. `after`, çerçeveyi iletir; backend exact `CommandComplete(COMMIT)`,
ErrorResponse yokluğu ve `ReadyForQuery(I)` ile tamamlayınca bütün COMMIT
yanıtını istemciden yutar ve bağlantıyı kapatır. Eksik/uyumsuz command tag,
ERROR veya idle olmayan durum başarılı after kanıtı değildir. `observe`
yanıtları değiştirmeden iletir; ERROR+idle gözlemi COMMIT başarısı sayılmaz.
Son JSON satırı `commit_seen`, `commit_forwarded`, `commit_ready`,
`commit_complete`, `commits_seen`, `completion_delivered`, `error_delivered`
ve yalnız severity/SQLSTATE taşıyan `commit_error` kanıtını
taşır. Uygulama öncesinde başka bağlantılar açılıyorsa `--skip-connections N`,
hedef transaction'dan önceki COMMIT'ler için `--skip-commits N` kullanılabilir;
değerler gözleme dayanmadan tahmin edilmez.

`--application` startup'taki exact test application kimliğini isteğe bağlı
olarak doğrular. Başlangıç protokolü yalnız plaintext PostgreSQL 3'tür;
SSLRequest reddedilir. Startup en fazla 64 KiB, frame en fazla 32 MiB;
sınır gövde okunmadan zorlanır. `--deadline-seconds` toplam absolute
accept/okuma/yazma süresidir; parça geldikçe yenilenmez. Owned listener,
bağlantılar ve thread'ler kapanmadan kabul yoktur. [Sahte wire kapısı](../scripts/tests/postgresql-commit-proxy-test.py)
başarı, ack reddi, boyut/deadline ve cleanup yollarını gerçek yerel
soketlerle sınar. Araç TLS interception veya public plaintext desteği değildir.

Proje bildirimindeki exact loopback portu proxy portuna ve bağlantı ortam
değişkenindeki port aynı değere ayarlanır. Gerçek sır/parola komut satırına
yazılmaz. legacy panel provasında PostgreSQL 16.11 backend'i 5432, proxy 55439 ve
uygulama 8093 kullanmıştır.

## Değişmez kabul matrisi

| Varyant | Proxy kanıtı | DB sonucu | İstemci sonucu |
|---|---|---|---|
| A — COMMIT öncesi kesinti | forwarded=false, ready=false | iş anahtarı yok | C027 / HTTP 503 |
| B — COMMIT sonrası yanıt kaybı | forwarded=true, ready=true | iş anahtarı var | C027 / HTTP 503 |

İki varyantta da worker sonraki DB'siz sağlık isteğini 200 yanıtlamalı ve
runtime write'ı otomatik tekrarlamamalıdır. Sonraki bağlantıda ürün iş
anahtarıyla kaydı sorgular. Kayıt varsa başarılı sonucu benimser; yoksa ürün
politikası yeni girişime izin verebilir. Aynı UNIQUE anahtarla bilinçli tekrar
ikinci satır üretmemelidir. Deney sonunda sentinel kayıtlar ve uygulama
bağlantıları sıfır olmalıdır.

İlk gerçek ölçümde A `503 / 2,615 ms / 0 satır`, B
`503 / 3,914 ms / 1 satır` vermiştir. Her iki worker sağlık isteğine 200
döndürmüştür. B'de uzlaştırma GET'i kaydı görmüş, aynı slug'lı bilinçli girişim
`Location: /sayfalar?hata=slug` üretmiş ve satır sayısı 1 kalmıştır.
Exact ürün günlüğü `8a12848` commit'indedir.

## K-242: ayrı helper kampanyası

Güncel koşucu yedi TLS vaka + dört yalnız owned loopback plaintext vaka
ile toplam on bir native kabul ister. Olumlu kontrol exact COMMIT+idle
cevabını iletir; yeni özet/tek oturum/audit, başarı stdout'u ve tam bir
sentetik SMTP bildirimi vardır. `before` ve `after` C027 yollarında stdout
ve SMTP sıfırdır; bağımsız DB gözlemcisi sırasıyla eski/sıfır ve
modern/tek kalıcı sonucu görür. Deferred trigger `ERROR/23514`+idle'ı
istemciye ileten `observe` vakasında bu fixture rollback olur; C027 yine
korunur. Dört vakada da bir UPDATE attempt, bir INSERT attempt ve bir
COMMIT gözlenir; otomatik tekrar yoktur.

Mevcut native dış COMMIT hatasının konservatif C027 sınıfı değişmez.
ErrorResponse, severity, SQLSTATE veya idle durumu evrensel rollback
kanıtı değildir: PostgreSQL durable commit sonrasındaki cleanup/callback
de hata üretebilir. Native wire cevabı hermetik modelin backend-known
`Commit::Red`/C021 reddine eşitlenmez. Yeni 11/11 final exact kayıt henüz
bekleniyor; particular fixture gözlemi provider/extension, canlı ürün,
uzak CI, fiziksel platform veya RC kabulüne genişletilmez.
