# K-343 — Çok işçi grubunun systemd soketiyle kesintisiz yeniden başlatılması

Bu anonim kesit aktif `zee-cekirdek` ürününün (bkz. [KOKEN](../KOKEN.md))
B6 talebine bağlıdır. Ürün deposu özeldir; müşteri, alan adı, sunucu yolu ve
gerçek veri taşınmaz (ADR-087).

Exact harici ürün commit'i: `abaca3aba0e90f57d26435c4befffa903346c425`
(18 Kasım 2026).

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `docs/dil-talepleri/2026-10-14-zee-urunlesme.md` (B6 "Kesintisiz yeniden başlatma": systemd socket activation; kabul: yük altındaki yeniden başlatmada 5xx/bağlantı reddi yok) | `6b2fa83bca9e15a1778b0fb6ad3f15b5d7388b73` | `694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7` |
| `cekirdek/deploy/` altındaki site worker unit dosyası (soketsiz `systemctl restart`) | `9f1a3a1080eb5aec2862e0c5212c365a7061dc18` | `591d9fcb7ac140db8316dd521a433ff4ef8f872bd89bf71a820bc022881148f1` |

## Sürtünme

Ürün her dağıtımda worker'ı yeniden başlatıyor; birkaç saniyelik bağlantı
reddi oluşuyor, duman betiği 40 sn'ye kadar bekliyor. K-336/K-340 ile tek
komutluk çok işçi grubuna geçen ürünün yöneticisi dinleyiciyi kendisi
bağladığından `systemctl restart` aralığı yine reddedilen bağlantı demekti.
ADR-118 sahipli host devri bu profili kapsamıyordu. K-164 kataloğunda
karşılığı "işletim yüzeyi eksik".

## Minimal değişiklik

[zee.ayar](zee.ayar) `guvenlik.systemd-soketi: true` yazar; [ana.dil](ana.dil)
değişmez. Yönetici dinleyiciyi bağlamak yerine systemd'nin tuttuğu soketi
(LISTEN_PID/LISTEN_FDS, fd 3) devralır; yeniden başlatma aralığındaki
bağlantılar soket kuyruğunda bekler. Birim şablonları ve sözleşme
[docs/cok-isci-systemd-soketi.md](../../../docs/cok-isci-systemd-soketi.md);
karar [ADR-161 eki](../../../adr/161-cok-isci-web-profili-ve-acilis-birincilligi.md),
[spec/12 §7](../../../spec/12-web-guvenlik-profili.md).

`dil denetle ana.dil` bu klasörde hermetik geçer. Gerçek davranış:
`compiler/tests/web_cok_surec_testi.rs::isci_grubu_systemd_soketiyle_yeniden_baslatmada_istek_kaybetmez`
ve CI'daki gerçek systemd ölçümü (`scripts/systemd-soket-olcumu.py`).

## Açık

Ürün unit dosyasının `.socket` birimine geçmesi ürün tarafındadır.
