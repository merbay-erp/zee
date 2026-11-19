# RFC-0038 — Resmî çok işçi web profili ve açılış birincilliği

- **Durum:** kabul — 18 Kasım 2026, K-336 (W2) ve K-337 (W4) ile gerçeklendi;
  normatif metin spec/12 §7 ve spec/21, karar ADR-161.
- **Tarih:** 18 Kasım 2026
- **İş:** K-336, K-337 (kaynak: zee-cekirdek
  `docs/dil-talepleri/2026-10-20-web-eszamanlilik-ve-kopma.md` W2/W4, exact `ca3870d`)
- **İlgili belgeler:** spec/12, spec/21, ADR-034, ADR-059, ADR-064, ADR-087,
  ADR-118, ADR-157, GB-133
- **Freeze sınıfı:** `dogfood-change` — iki CLI seçeneği ve tek contextual
  sıfır argümanlı ifade; koşul grameri, AST/değer türü, tanı kodu, bağımlılık
  ya da `dil::api::v1` değişikliği yok.

## Sorun

Web işçisi istekleri sıralı işler: istek başına ~250 ms hizmette eşzamanlılık
1/3/10 aynı saniyede ~4 istek verir, gecikme N × hizmet süresidir. Ürün aynı
kökten elle üç süreç (ayrı `--web-worker-port`, nginx `upstream`) açarak
doğrusal ölçek aldı; ama bu kurulum her sitede elle tekrarlanır ve açılış
uzlaştırması (yarım yükleme tamamlama, sahipsiz geçici dosya taraması) her
süreçte koşar. Tek işçi yeniden başlarken ötekinin o an yüklediği, henüz
veritabanı satırı açılmamış geçici dosyayı sahipsiz sayıp silebilir.

## Öneri

```text
dil çalıştır --web-proxy https://site.example --web-worker-port 8096 \
  --web-isci-sayisi 4 --web-saglik-port 8099 .
```

Yönetici süreç `127.0.0.1:8096`'yı bir kez bağlar, aynı açık dinleyiciyi dört
işçiye devreder, işçi çökünce ikincil olarak yeniden başlatır, SIGTERM'i
boşaltarak iletir ve `127.0.0.1:8099/saglik` yanıtlar. Ürün açılışını şöyle
korur:

```zee
açılış_sahibi birincil süreç olsun
açılış_sahibi ise
    uzlastirma yarım yüklemeleri uzlaştır olsun
```

İkincil işçiler birincil `sunucu başlat`a ulaşmadan başlamaz; yeniden
başlatılan işçi ikincildir.

## Alternatifler

- **Süreç içi N istek işçisi (W2-a):** yorumlayıcı değerleri `Rc`/`RefCell`
  paylaşır, IO adaptörü ve oturum/eylem transaction'ı süreç başına tektir;
  paralel yürütme tüm çalışma zamanının yeniden yazımıdır. ADR-161 §Değerlendirilen.
- **SO_REUSEPORT:** Linux dağıtır, macOS dağıtmaz, Windows'ta yoktur; aynı
  porta yabancı sürecin bağlanabilmesi ayrıca güven sınırıdır.
- **N dinleyici + mevcut proxy:** bugünkü elle yöntem; tek komut ve açılış
  sıralaması vermez, sürüm notunda Windows yolu olarak kalır.
- **Açılışta dosya kilidi ya da PG danışma kilidi:** iki sürecin aynı anda
  açılışını sıraya koyar ama tekliğini değil; yeniden başlayan süreç
  uzlaştırmayı çalışan işçilerin ortasında yine koşar.
- **`açılışta bir kez` bloğu / `birincil süreç ise` koşul biçimi:** yeni grammar;
  değişkene bağlanan sıfır argümanlı ifade aynı işi mevcut sözdizimiyle yapar.

## Açık

Windows'ta dinleyici devri yok (çıkış 2). Bağlantı dağılımı çekirdeğe bağlıdır.
`zee.ayar` işçi alanları K-340 ile eklendi (ADR-161 eki). Elle kurulan ayrı
portlu süreçlerde tek makinede birincillik K-341 ile `.zee` süreç kilitlerine
bağlandı (ADR-161 eki). systemd soketiyle kesintisiz yeniden başlatma K-343
ile eklendi (ADR-161 eki); yöneticiden yöneticiye re-exec devri açıktır. Çok
hostlu dağıtım, ağ dosya sistemi ve süreçler arası bellek oran deposu kapsam
dışıdır.
