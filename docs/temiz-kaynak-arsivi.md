# Temiz kaynak arşivi

Zee'nin paylaşılabilir araştırma ve kaynak arşivi çalışma klasörünün ZIP'i
değildir. Normatif üretim yolu, yalnız Git tarafından izlenen `HEAD`
içeriğini paketleyen şu komuttur:

```sh
scripts/temiz-kaynak-arsivi.sh
```

Varsayılan çıktı `dist/zee-kaynak-<git-kimliği>.zip` olur. Aynı `HEAD` aynı
ZIP byte'larını üretir; çalışma ağacının kirli olması içeriği değiştirmez.
İstenirse depo
içinde başka bir çıktı yolu ilk argüman olarak verilebilir. Script `git
archive` kullandığı için çalışma ağacındaki gizli dosyalar, kişisel arşivler,
`target/`, fuzz ikilileri ve başka izlenmeyen dosyalar pakete giremez. Üretim
sonrasında arşiv girdilerini ayrıca tarar; `.git`, `target`, `artifacts`,
`__MACOSX`, `*.profraw` veya `*.profdata` bulursa başarısız olur.

Arşivin kökündeki `KAYNAK-SHA256.txt`, her izlenen dosyanın kaynak byte'ı için
sıralı SHA-256 kaydıdır. Arşivin yanındaki `.zip.sha256` dosyası da ZIP'in
tamamını doğrular. Böylece inceleyen kişi hem taşıma bütünlüğünü hem açılmış
kaynağın dosya bazında bütünlüğünü bağımsız denetleyebilir.

## Saklanan ve atılan içerik

- `compiler/fuzz/corpus/`, fuzz hedefleri ve sözlükleri kaynak kanıtıdır;
  Git'te ve kaynak arşivinde kalır.
- `compiler/fuzz/target/`, `compiler/target/`, crash `artifacts/`, coverage
  çıktıları ve profiler dosyaları yeniden üretilebilir build verisidir;
  Git'e ve kaynak arşivine alınmaz.
- Finder'ın `.DS_Store`/`__MACOSX` metadatası ve yerel `*.zip` dosyaları depo
  girdisi değildir.
- Yerel mutlak build yolları derlenmiş fuzz ikililerinde bulunabileceği için
  ikililer araştırma/kaynak arşivi olarak paylaşılmaz.

Script bilerek yalnız commit edilmiş `HEAD` durumunu arşivler. Henüz commit
edilmemiş düzeltmelerin paylaşılması gerekiyorsa önce ilgili kod, test ve
belgeler tek bir tutarlı commit olarak tamamlanır; çalışma klasörü doğrudan
sıkıştırılmaz.
