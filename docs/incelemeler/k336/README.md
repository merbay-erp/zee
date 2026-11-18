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

Exact temiz kaynak ölçümü kaynak commit'inin beyan commit'inde `olcum.json`
olarak bu klasöre eklenir. Geliştirme sırasında kirli ağaçta
iki gözlem: 4 işçi c4/c1 = 3,93 (35,93 / 9,14 istek/sn) ve 3,52 / 4,03 (iki
tur); 1 işçi c4 = 9,20 istek/sn, medyan gecikme 404,8 ms (sıralı işleme).

## Sınır

Yük üreteci ve işçiler aynı makinededir; CPU çekirdek sayısı ve eşzamanlı
yerel işler oranı etkiler. Bağlantı dağılımı çekirdeğe bağlıdır (macOS eşit
dağıtmaz). Genel ölçeklenme garantisi ya da SLA değildir.
