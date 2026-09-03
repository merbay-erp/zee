# Core freeze değişiklik beyanı

K-160A/ADR-059 sonrasında her `compiler/src` commit'i semantic beyanına ek
olarak [`core-freeze-beyanlari-v1.tsv`](core-freeze-beyanlari-v1.tsv) içinde
tekil bir freeze sınıfı taşır. Kapı, yeni özelliği sırf ADR yazıldığı için
kabul etmez; gerçek dogfood provenance'ı yoksa reddeder.

## Sekiz alanlı satır

`commit`, `sinif`, `urun`, `is`, `reproducer`, `etkilenen_proje`, `minimalite`
ve `karar` sekmeyle ayrılır. Commit tam SHA, minimalite gerekçesi en az 40
karakterdir.

| Sınıf | Zorunlu kanıt |
|---|---|
| `maintenance` | Diğer kanıt alanları `-`; semantic sınıf da maintenance |
| `bugfix` | K-işi ve repoda var olan minimal reproducer; exact semantic bugfix |
| `security` | K-işi, reproducer ve ADR/spec/RFC; semantic bugfix veya change |
| `correctness` | K-işi, reproducer ve ADR/spec/RFC; semantic bugfix veya change |
| `dogfood-change` | Ürün kimliği, K-işi, reproducer, etkilenen gerçek proje dosyası, minimalite gerekçesi ve ADR/spec/RFC; exact semantic change |

`dogfood-change` ürün kimliği küçük harfli kararlı slug'dır. Reproducer ve
etkilenen proje depo içindeki var olan göreli dosyalardır; böylece “ürün istedi”
iddiası incelenebilir bir kanıta bağlanır. Yeni bir semantic feature yalnız
`dogfood-change`, `security` veya `correctness` sınıfıyla geçebilir.

Kaynak commit'i önce; semantic ve core-freeze beyanları exact SHA ile sonraki
committe alınır. Koruğu yerelde şu şekilde çalıştır:

```bash
bash scripts/semantic-regresyon-korugu.sh HEAD~2
bash scripts/core-freeze-korugu.sh HEAD~2
```

Koruğun geçici Git depo testi; beyansız feature'ı, semantic sınıfla uyumsuz
maintenance kaçışını, eksik dogfood kanıtını ve yayımlanmış beyan yeniden
yazımını reddeder.
