# K-381 — Adlı dinleyici bağlantı yüzeyi (ürün H)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün ve işletim sistemi depoları özeldir; sunucu adı, alan adı, adres ve
yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası (ürün commit'i `fe849ca261e3d0990b1d2a89455f530075140083`, 23 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Ürün SMTP oturum durum makinesi (`smtp_oturumu` birimi) | `7d08413f2ca3a0d246c4c4d50f9ccddc4d0207a8` | `a1f70f3accb75e99a9c3e65557e6942720c3802885e17572be305dcf0bcbb411` |
| Ürün stdio deneme aracı (giriş dosyası; ağ yok) | `85dd08e8f79175e6e738345c40c1ab8317063dcf` | `88e5ccbc3e9ede6267fd70cc30b19f30240676819136ee2d51c8dc56bca45ef4` |
| Ürün reproducer'ı ZM-R001 (dinleyici kalıbı, bugün S036) | `e8d50860c423f99fb06a20d1139944e7ed06ca42` | `6748fbb683db5d68b2ce258c0874a5c7c1245bf4483d8ae1765b3aef5d395224` |

## Sürtünme

Ürünün SMTP durum makinesi Zee'de yazılı ve `dil dene` ile sınanıyor, ama
ağa bağlanamıyordu: dinleyici kalıbı S036 veriyordu (F009 / ZM-R001), tek
okuma yolu `"" diye sor` her okumada istem basıyordu (F010). K-376 adlı
soketleri devralıp tutuyor, Zee'ye vermiyordu. Satır sonu (çıplak CR/LF)
ve satır uzunluğu denetimi Zee'de yazılamıyordu (F004, `\r` kaçışı yok).
K-164 kataloğunda karşılığı "işletim yüzeyi eksik" (K-343/K-376 ile aynı
sınıf) ve F004 için K-164/F011 tekrarı.

## Minimal değişiklik

[proje.dil](proje.dil) iki ailenin adlı dinleyicisini, `ağ-dinleyici`
yetkinliğini ve bağlantı sınırlarını bildirir; [ana.dil](ana.dil) ürün
durum makinesinin bağlantı tarafının küçültülmüş hâlidir: karşılama,
satır okuma, NOOP/QUIT, sınıflandırılmış satır reddine protokol yanıtı,
zaman aşımı ve SIGTERM'de `421`. Satır kuralı (CRLF, çıplak CR/LF reddi,
uzunluk, kodlama), mutlak boşta süresi, bağlantı sınırları ve geri basınç
çalışma zamanı değişmezidir; protokol kararı Zee'dedir. Karar
[ADR-165](../../../adr/165-ag-dinleyici-baglanti-yuzeyi.md), normatif
[spec/12 §9](../../../spec/12-web-guvenlik-profili.md), işletim
[docs/adli-dinleyici-devri.md](../../../docs/adli-dinleyici-devri.md).

`dil denetle` bu klasörde hermetik geçer (değişiklik öncesi S036). Gerçek
davranış: `compiler/tests/ag_baglantisi_surec_testi.rs` ve gerçek systemd
ölçümü — `scripts/systemd-adli-soket-olcumu.py` bu programı iki adlı soketle
`Type=notify` sandbox'ında çalıştırır (IPv4/IPv6 oturumu, `systemctl stop`
ile `421 4.3.2 hizmet kapanıyor`).

A2b ile kesit 465 implicit TLS ve 25'te STARTTLS taşır; TLS kimliği
`LoadCredential=` ile verilir (`proje.dil` yalnız adları taşır). Gerçek
systemd ölçümü implicit TLS 1.3, STARTTLS ve STARTTLS enjeksiyonunun
reddini bu programla doğrular.

## Açık

`DATA` gövdesinin dosyaya akıtılması (A2c), ürünün kendi imajında uçtan
uca kabul (ürün tarafı).
