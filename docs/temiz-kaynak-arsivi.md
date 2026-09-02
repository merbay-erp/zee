# Temiz kaynak arşivi

Zee'nin paylaşılabilir araştırma ve kaynak arşivi çalışma klasörünün ZIP'i
değildir. Normatif üretim yolu, yalnız Git tarafından izlenen `HEAD`
içeriğini paketleyen şu komuttur:

```sh
scripts/temiz-kaynak-arsivi.sh
```

Varsayılan çıktı `dist/zee-kaynak-<git-kimliği>.zip` olur. İstenirse depo
içinde başka bir çıktı yolu ilk argüman olarak verilebilir. Script `git
archive` kullandığı için çalışma ağacındaki gizli dosyalar, kişisel arşivler,
`target/`, fuzz ikilileri ve başka izlenmeyen dosyalar pakete giremez. Üretim
sonrasında arşiv girdilerini ayrıca tarar; `.git`, `target`, `artifacts`,
`__MACOSX`, `*.profraw` veya `*.profdata` bulursa başarısız olur.

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
