# RFC-0031 — Açık seçilen bellek oran backend'i

- **Durum:** **geçici kabul** (K-218)
- **Tarih:** 13 Ekim 2026
- **İlgili kayıtlar:** ADR-089, spec/12, GB-051

zee-cekirdek gerçek yük senaryosu ortak dosya/fsync sıcak yolunu gösterdi.
Yeni dil grammar'ı yerine host CLI seçimi `--web-oran-deposu bellek`
yeterlidir. Yalnız güvenli proxy kipinde tek kez ve tek değerle kabul edilir.
Eksik/geçersiz/tekrar değer veya deneysel/kapalı web kipinde kullanım hata
ile çıkar. Varsayılan davranış aynı kalır. Kullanım:

```sh
dil çalıştır --web-proxy https://panel.example --web-worker-port 18097 --web-oran-deposu bellek ana.dil
```

Oran sayacı süreç başına, oturum dosyası süreçler arasında ortaktır. Ortak
proxy kotasıyla birlikte seçilmelidir; restart oranı sıfırlar. PostgreSQL bu RFC’de açık kalır. Kota ayarı ve explicit uç nokta tahliyesi
[RFC-0032](0032-web-oran-ayarlari.md) / K-219 dilimindedir. Karar ve
olumlu/olumsuz test zinciri [ADR-089](../adr/089-bellek-oran-deposu.md) ve
[spec/12](../spec/12-web-guvenlik-profili.md) içindedir.
