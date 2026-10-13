# K-218 — Gerçek ürün oran yük kanıtı

Ürün: zee-cekirdek. Harici kaynak commit'i:
`1b002e09c3c3fa38d8e88c2d82669618d99cd762`.
Kaynak: ürün deposu `docs/dil-talepleri/2026-10-13-oran-siniri-yuk/`.
Python betikleri değiştirilmeden taşınmıştır; Zee kaynağına yalnız korpus sözleşmesinin istediği başlık yorumu eklenmiştir. `itwise.test` sentetik yerel hosttur;
203.0.113.x ve 198.51.100.x belgeleme IP'leridir. Özel sunucu bilgisi yoktur.

`yuk.dil`: 100 kartlı anonim GET /wiki/ listesi; `yuk.py N C PORT`
p50/p95/p99 + durum kodları; `yuk2.py N C PORT1,PORT2 OFSET` çok worker
ve farklı anahtar doldurma. Loopback'te izole geçici klasör ve yalnız kendi
PID'leriyle çalıştırılır; gerçek ürün .zee durumu temizlenmez.

Eski ürün gözlemi yaklaşık 55/54/56 istek/sn'dir. Yaklaşık 11.000 deneysel
istek/sn ham kanıtı yoktur; 32.768 anahtarda 1,8 MB tahmindi. Bunlar bu
kaynakta ölçülmüş sayılmaz. Yeni kabul exact temiz kaynakta yeniden ölçülür.
[Backend sınırı](../../../adr/089-bellek-oran-deposu.md).
