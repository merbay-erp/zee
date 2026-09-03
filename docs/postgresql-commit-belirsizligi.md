# PostgreSQL COMMIT belirsizliği provası

Bu runbook K-163/F028'in gerçek wire-level kanıtını yeniden üretir. Araç yalnız
loopback'te tek hedef bağlantıyı bozar; production proxy veya güvenlik katmanı
değildir.

## Araç

```bash
python3 scripts/postgresql-commit-proxy.py \
  --mode before --listen-port 55439
```

`before`, simple-query `COMMIT` çerçevesini PostgreSQL'e iletmeden iki ucu
kapatır. `after`, çerçeveyi iletir; backend `ReadyForQuery` döndürdüğünde COMMIT
yanıtını istemciden yutar ve bağlantıyı kapatır. Son JSON satırı
`commit_seen`, `commit_forwarded`, `commit_ready` ve `commits_seen` kanıtını
taşır. Uygulama öncesinde başka bağlantılar açılıyorsa `--skip-connections N`,
hedef transaction'dan önceki COMMIT'ler için `--skip-commits N` kullanılabilir;
değerler gözleme dayanmadan tahmin edilmez.

Proje bildirimindeki exact loopback portu proxy portuna ve bağlantı ortam
değişkenindeki port aynı değere ayarlanır. Gerçek sır/parola komut satırına
yazılmaz. Çatlı provasında PostgreSQL 16.11 backend'i 5432, proxy 55439 ve
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
