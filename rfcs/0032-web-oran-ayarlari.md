# RFC-0032 — Bellek oran sınırı ve doluluk ayarları

- **Durum:** **geçici kabul** (K-219)
- **Tarih:** 14 Ekim 2026
- **İlgili kayıtlar:** ADR-090, spec/12, GB-052

Gerçek ürünün §3/2 talebine yalnız host CLI seçenekleri eklenir. Bellek
backend'i seçilmeden ayar verilmesi, eksik/tekrar/geçersiz değer ve kaynak
bütçesi dışındaki kapasite reddedilir. Varsayılanlar ve dosya backend'i
korunur. Örnek:

```sh
dil çalıştır --web-proxy https://panel.example --web-worker-port 18097 \
  --web-oran-deposu bellek --web-oran-siniri 250 \
  --web-oran-kapasitesi 32768 --web-oran-doluluk en-eski ana.dil
```

Uç nokta eşiği IP+adres başına 60 saniyelik pencerede uygulanır. Login ve
CSRF eşiği sabittir. `en-eski` yalnız uç nokta pencerelerini tahliye eder;
canlı login/CSRF kayıtları korunur. Tahliye edilen uç nokta tekrar geldiğinde
kotası sıfırlanır, bu yüzden ortak proxy bütçesi gerekir. Korunmuş kayıtlarla
dolu depoda yeni kimlik hâlâ 503 alır. Karar ve test zinciri
[ADR-090](../adr/090-web-oran-ayarlari.md) ve [spec/12](../spec/12-web-guvenlik-profili.md).
HTTP POST, HTML temizleyici ve LSP artımlılık bu RFC'nin kapsamı dışındadır.
