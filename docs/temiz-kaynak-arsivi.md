# Temiz kaynak arşivi

Zee'nin paylaşılabilir araştırma ve kaynak arşivi, çalışma klasörünün ZIP'i
değildir. Üretici başlangıçtaki `HEAD^{commit}` değerini bir kez tam SHA'ya
çözer ve yalnız o snapshot'ın Git içeriğini paketler:

```sh
scripts/temiz-kaynak-arsivi.sh
```

Varsayılan çıktı `dist/zee-kaynak-<git-kimliği>.zip` olur. İsim, kaynak
listesi, dosya byte'ları, commit zamanı ve `git archive` aynı exact SHA'yı
kullanır. Eşzamanlı commit veya checkout bu koşunun snapshot'ını değiştirmez;
kirli çalışma ağacı ve staging alanındaki değişiklikler pakete alınmaz.
Aynı kaynak ve araç ortamında çıktı byte'ları aynıdır. Kaynak listesi NUL
ayırıcıyla okunur; Git'in varsayılan pathname alıntılaması UTF-8/Türkçe ve
köşeli parantezli adları bozmaz. Satır sonu taşıyan kaynak adı manifestte
desteklenmez ve üretim reddedilir.

Araç Bash, Git, `unzip`, `sha256sum` veya `shasum` ve **Python 3** ister.
Python yalnız standart kütüphaneyi ve depodaki
`scripts/kaynak-ci-esleme.py` doğrulayıcısının yerel saf işlevlerini kullanır;
ek Python paketi, ağ erişimi veya CI kaydı gerekmez.

İlk argümanla farklı bir çıktı dosyası verilebilir. Göreli yol depo köküne
bağlanır; mutlak yol da fiziksel olarak depo içinde kalmalıdır. Üretici
ebeveyn yolu `mkdir` öncesinde kanonikleştirir: `..` veya sembolik bağ ile
depo dışına çıkan yol reddedilir. ZIP ve `.sha256` hedefleri normal dosya
olmalı veya henüz bulunmamalıdır; son bileşendeki sembolik bağ ve dizin,
`.git` yolu, Git indeksinde veya snapshot'ta izlenen dosya üzerine yazma
reddedilir. Metadata dizini fiziksel kimliğiyle denetlenir; büyük/küçük harf
eşdeğerleri ve izlenen dosyayla aynı kimliği taşıyan hard-link çıktısı da
reddedilir. İzlenen dosya çalışma ağacında silinmişse çıktı dosya sisteminin
harf/Unicode normalizasyon eşdeğerleri de denetlenir; Linux'ta fiziksel
olarak ayrı büyük/küçük harfli izlenmeyen ad normal çıktı olarak kalabilir.
Bu denetim sidecar hedefini de kapsar. İzlenmeyen normal eski
çıktı doğrulama tamamlandıktan sonra yenilenebilir.

`git archive` kullanıldığı için çalışma ağacındaki gizli dosyalar, kişisel
arşivler, `target/`, fuzz ikilileri ve başka izlenmeyen dosyalar pakete
giremez. Arşiv girdileri ayrıca taranır; `.git`, `target`, `artifacts`,
`__MACOSX`, `*.profraw` veya `*.profdata` bulunursa üretim başarısız olur.

`zee/` kökündeki `KAYNAK-SHA256.txt`, snapshot'ın kaynak dosyaları için
sıralı SHA-256 kaydıdır. ZIP yorumu exact kaynak SHA'sını taşır. Üretici
çıktı ebeveynindeki özel geçici dizinde ZIP ve özetini hazırlar. Yayınlamadan
önce gerçek ZIP yorumunu, dosya kümesini ve her dosyanın byte hash'ini aynı
SHA'nın Git arşivi ve manifestiyle birebir karşılaştırır; tek metadata fazlası
`KAYNAK-SHA256.txt` olabilir. Bozuk ZIP/manifest bu aşamada reddedilir ve
var olan çıktılar değiştirilmez. Arşivin yanındaki `.zip.sha256` dosyası
ZIP'in tamamını doğrular.

Doğrulanmış ZIP ve sidecar aynı dosya sisteminde ayrı yeniden adlandırmalarla
yayımlanır; **ikisi tek atomik işlem değildir**. Aynı kullanıcı yetkisindeki
eşzamanlı dosya sistemi değişikliklerine karşı OS sandbox veya atomik FS
snapshot garantisi verilmez. Yerel kaynak bütünlüğü, yayıncı imzası, sürüm
ikilisi kabulü, primary kaynak yetkisi veya uzak CI başarısı değildir.
Karar ve gerçek olumlu/olumsuz test kapsamı
[ADR-104](../adr/104-exact-kaynak-arsivi-siniri.md) içindedir.

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

Script bilerek yalnız başlangıçta commit edilmiş `HEAD` durumunu arşivler. Henüz commit
edilmemiş düzeltmelerin paylaşılması gerekiyorsa önce ilgili kod, test ve
belgeler tek bir tutarlı commit olarak tamamlanır; çalışma klasörü doğrudan
sıkıştırılmaz.
