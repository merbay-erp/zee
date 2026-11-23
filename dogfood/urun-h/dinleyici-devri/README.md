# K-376 — Adlı dinleyici devri ve hizmet bildirimi (ürün H)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün ve işletim sistemi depoları özeldir; sunucu adı, alan adı, adres ve
yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| Ürün `proje.dil` (posta motoru projesi; ürün commit'i `f405e729b867cff8ad8cb1b08fc8b1c5cf85614c`, 23 Kasım 2026) | `76466ca8326a36b6f844917258d35552cf69c5e9` | `1ba8d992d61d9e9b036c833ab8568c92ca1841ade946bc63e07d7cb4779eddce` |
| İşletim sistemi Faz 2 çalışma zamanı kabul kararı (ADR079, 23 Kasım 2026 kabul) | — (git dışı kopya) | `cc4dc90a06f8ca319b83afdf6835eb8a44220ca2897315fdbfca7c73e969f087` |
| Ürün hizmet yüzeyi kararı (ADR076: on adlı soket, `Type=notify`, sandbox) | — (git dışı kopya) | `534e8279b14904cf343f4302a19106a38abc16a2c825b58dbcbc2bd4b33cd53d` |

## Sürtünme

Faz 2 kararı uygulamayı gerçek Zee çalışma zamanıyla, ayrıcalıksız
kullanıcıda, capability olmadan, `Type=notify` (`NotifyAccess=main`) ve
adla devredilen on soketle (beş kapı × IPv4/IPv6) kabul eder. Zee'de yalnız
K-343 tek soketi vardı: `dinleyiciler` alanı `P001 "dinleyiciler" proje
bildirimi alanı değil` veriyordu, `NOTIFY_SOCKET` okunmuyordu; ürün
sözleşmeyi bir Python yoklamasıyla ölçmek zorunda kaldı. Ürünün iki canlı
ölçümünde (systemd 257.13) fd sırası `Sockets=` sırasından farklıydı. K-164
kataloğunda karşılığı "işletim yüzeyi eksik" (K-343 ile aynı sınıf).

## Minimal değişiklik

[proje.dil](proje.dil) on adlı dinleyiciyi bildirir; [ana.dil](ana.dil)
ürünün bugünkü soketsiz SMTP karşılamasıdır ve değişmez. `dil çalıştır`
soketleri yalnız adla eşler, adres/kapı/V6ONLY'yi doğrular ve tutar; uzun
ömürlü süreç web sunucusuysa (ürünün yönetim paneli) açılınca `READY=1`
gönderilir. Zee kodu bu sürümde adlı soketlere erişmez (RFC-0039 A2).
Karar [ADR-162](../../../adr/162-adli-dinleyici-devri-ve-hizmet-bildirimi.md),
normatif [spec/12 §8](../../../spec/12-web-guvenlik-profili.md), işletim
[docs/adli-dinleyici-devri.md](../../../docs/adli-dinleyici-devri.md).

`dil denetle ana.dil` bu klasörde hermetik geçer (değişiklik öncesi P001).
Gerçek davranış: `compiler/tests/adli_dinleyici_testi.rs` ve CI'daki gerçek
systemd ölçümü (`scripts/systemd-adli-soket-olcumu.py`).

## Açık

Posta motorunun bağlantı kabul etmesi (A2) ve ürünün kendi imajındaki
(AppArmor, dm-verity) uçtan uca kabul ürün tarafındadır.
