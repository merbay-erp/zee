# K-336 — Çok işçi profili işlem hacmi gözlemi

Kabul (zee-cekirdek W2): tek komutla başlayan kurulumda eşzamanlılık 4'te işlem
hacmi ≥ 3,5 × tek istek. Bu sayfa gözlemseldir: ADR-045/049 sabit benchmark
korpusu, `performans-gecmisi-v2.tsv` ve tabanı değişmez; shared CI'da eşik
uygulanmaz.

## Yöntem

[`olcum.py`](olcum.py) her senaryoda [`yuk.dil`](yuk.dil)'i (istek başına
~100 ms sabit CPU işi, durum yok) yeni geçici kopyada
`dil çalıştır --web-proxy https://urun.test --web-worker-port <boş> --web-isci-sayisi N`
ile tek komutla başlatır; 8 istek ısınma (kayda girmez), sonra her turda 40
istek eşzamanlılık 1 ve 80 istek eşzamanlılık 4 gönderir, grubu SIGTERM ile
kapatır. Oran = aynı turdaki c4 istek/sn ÷ c1 istek/sn; tur medyanı raporlanır.
Her istek varsayılan ortak oran deposuna yazar (fsync), yani tek süreçle aynı
oran davranışı ölçüme dahildir.

```sh
cd compiler && cargo build --locked --release --bin dil
python3 -I -B ../docs/incelemeler/k336/olcum.py target/release/dil <çıktı.json> 5
```

## Sonuç

Kaynak `9e51868caedbf1f1c42ab0b7f050b592f835fb1d` temiz exact checkout'ta
(`dirty:false`) `cargo build --locked --release --bin dil`; ikili SHA-256
`53ff9d8ff1c413cfc6524929859a7b4466e95f643abae14ba20cb644a4be369d`. Apple M4
Pro (14 çekirdek), 24 GiB, macOS 26.6.2, Rust 1.93.1, release; 5 tur, 8
istek ısınma. Ham dağılım [`olcum.json`](olcum.json).

| Senaryo | c1 istek/sn (tur medyanı) | c4 istek/sn (tur medyanı) | c4 p50 |
|---|---:|---:|---:|
| 1 işçi | 10,02 | 9,93 | 391,8 ms |
| 4 işçi | 9,89 | 36,80 | 103,5 ms |

4 işçide tur başı c4/c1 oranı 4,04 / 3,73 / 2,76 / 3,72 / 3,72; **medyan 3,72
≥ 3,5** (kabul). Bütün yanıtlar 200, iki grup da SIGTERM ile 0 çıktı. Üçüncü
turdaki 2,76 aynı makinede eşzamanlı başka derleme oturumları (yük ortalaması
13–15) sırasında p95 304,8 ms ile görüldü; tek tur ayrık gözlemdir. 1 işçide
c4 gecikmesi c1'in yaklaşık dört katıdır (sıralı işleme), 4 işçide c1 ile
aynıdır.

Geliştirme sırasında kirli ağaç gözlemleri (kayıt dışı): 3,93 ve 3,52/4,03.

## Sınır

Yük üreteci ve işçiler aynı makinededir; CPU çekirdek sayısı ve eşzamanlı
yerel işler oranı etkiler. Bağlantı dağılımı çekirdeğe bağlıdır (macOS eşit
dağıtmaz). Genel ölçeklenme garantisi ya da SLA değildir.
