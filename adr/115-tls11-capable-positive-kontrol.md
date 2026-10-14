# ADR-115 — Gerçek TLS1.1 capable-positive kontrolü

- **Durum:** kabul, owned fixture kanıtı
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-256/P31/P33/P34, GB-076; ADR-062/111

## Karar ve actual kanıt

Native PostgreSQL TLS minimum1.2 değişmez. Owned loopback cluster yalnız
TLS1.1 sunar; bağımsız Python/OpenSSL istemcisi doğru CA ve localhost SAN
ile TLS1.1 kurar, `pg_stat_ssl` aynı protokolü doğrular. Sonra native Zee
adaptörü aynı legacy sunucuyu reddeder; önce/sonra ayrı TLS1.2 sağlık okuması
sürer. Ulaşılamayan veya TLS1.1 sağlayamayan fixture ret kanıtı sayılmaz.

Sekiz dedicated native test TLS1.2, wrongCA/SAN, NULL/boyut, actual40P01/40001
ve TLS1.1 reddini birlikte yürütür. Strict14 wrapper eksik/duplicate/skip,
sahte sonuç/gözlem, wrong protocol/unverifiedCA veya malformed cipher kabulünü
reddeder. Actual stage8/8, independentTLS1.1 `ECDHE-RSA-AES256-SHA` ve server
TLS1.1 doğrulaması geçti; final exact clean native kabul ayrıca gereklidir.

## Güvenlik ve süre

Legacy cipher `ALL:@SECLEVEL=0` yalnız ikinci fixture ve bağımsız test
context'indedir. Üretim connector/ilk TLS1.2 fixture policy'sine taşınmaz.
Cluster/key/CA ephemeral ve owned loopback'tir; arbitrary müşteri DB URL'si
koşucu seçeneği değildir. Docker immutable image ve local owned runtime
motorları aynı iki-cluster kontrolünü taşır. Minimum platform/provider
rotasyonu ve uzak CI actual kabulü bu yerel çalışmadan türetilmez.

Bağımsız kontrol her connect/handshake/send/receive'den önce kalan mutlak
5s bütçeyi ayarlar; frame64KiB, startup/query en çok64frame, response row
exactsize denetlenir. İncelemede mock handshake6s sonrasında Startup yazısı
başlayabiliyordu; aday helper shared remaining/send ile giderildi. Negative
case başlangıç/handshake sonrası expiry'de hiçbir sonraki tel yazısının
başlamadığını ve socket cleanup'ını doğrular. Mock kontrol actualTLS olumlusu
yerine kullanılmaz. CLI/Rust facade/grammar/dependency graph değişmez; yeni
compiler bug veya regression fixed_by kaydı yoktur.

## Belge etkisi

ADR/kanıt/faz/README sayıları, backlog/günlük/sürüm/plan kapıları aynı dilimde
incelenir. Spec25 mevcut minimum TLS kanıtına native exact test eklenir,
normatif metin değişmez. Tanı kataloğu, RFC, facade/deprecation ve kaynak
mimari fixture değişmez. Sabit benchmark, yeni compiler state ve toolchain
olmadığından benchmark/soak geçmişine bu fixture sonucu yazılmaz.
