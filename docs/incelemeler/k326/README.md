# K-326 — 14–17 Ekim belge geçmişinin anonimleştirilmesi

20 Ekim 2026'da açık depo kapısı (`scripts/acik-depo-denetle.py`, ADR-087)
güncel ağaçta temizdi; ancak tüm erişilebilir tarihçede 11 konum buldu.
`legacy_fingerprints` istisnası K-215 gereği kabul edilmez. Kullanıcı
geçmişin yeniden yazılmasını onayladı; force-push ayrıca son onaya bağlandı.

## Bulgular

Bütün bulgular 14–17 Ekim belge sürümlerinde, commit mesajlarında değil:

- sunucu adı kuralı: `docs/surumler.md`, `docs/urunlesme-plan-v1.tsv`,
  `docs/v1-surum-kapilari.md`, `docs/v1-teknik-kapanis-20261017.md`;
- sunucu kök yolu kuralı: `docs/db-hazirlik.md`, `docs/worker-supervisor.md`
  örnek komutları.

Güncel ağaç bu metinleri 16c83da1 ve 8209b7aa (eski 054046df, b5f460c5) commit'lerinde
zaten “ürün sunucusu” ve `/var/lib/...` biçimine çevirmişti. Yeniden yazım
geçmiş sürümlere yalnız aynı sekiz kesin metin değişimini uygular.

## Yöntem

Bağımsız `--mirror` klonunda tek geçişli `git fast-export | fast-import`
akışı: (1) blob metinlerinde sekiz kesin değişim; (2) yalnız
`regression/v2.tsv`, `docs/compiler-degisiklik-beyanlari-v1.tsv` ve
`docs/core-freeze-beyanlari-v1.tsv` içindeki 40 haneli kaynak sahibi SHA'ları
o ana kadar yazılmış yeni ata SHA'sına bağlanır; (3) commit mesajlarındaki
tam/kısa SHA atıfları eşlenir. Kayıtlar yalnız atalara atıf yaptığından tek
geçiş yeterlidir; tarihçenin her noktasında beyan ve regresyon sahipleri
kendi soyunda var olan commit'i gösterir. Merge topolojisi, yazar/tarih,
K/vaka kimlikleri ve harici ürün SHA'ları korunur. GitHub'ın imzaladığı
merge commit'lerinin imzası yeniden yazımda düşer.

Sonuç: 540 ana dal commit'inin 98'i yeni SHA aldı (ilk değişen eski 6a85a3ab → f1a6dae4).
Yeniden yazılmış ana dal ucu `ab2e0e752185f52352bcf098b91204b3330cdc8a`;
eski uç `e196b6ede4109e59a4202e7503bd1a2387eb9498` ile arasındaki ağaç farkı
yalnız yukarıdaki üç kayıt dosyasındaki 38 SHA satırıdır.

## SHA ve tarihi kanıt

[Eşleme](sha-esleme-v1.tsv) eski→yeni tam SHA'dır; [ref eşlemesi](ref-esleme-v1.tsv)
ana dal, `v1.0.0-rc.1` etiketi ve Enterprise/Git2 ayna ucunu kaydeder.
K-215 kuralı aynen geçerlidir: benchmark, soak, fuzz-RC, dogfood
karşılaştırma, platform, ikili, SBOM/provenance ve inceleme makbuzları gerçek
ölçüldükleri özgün SHA'yı korur; soy, bu eşleme ve HEAD atası şartıyla
doğrulanır (`dogfood_karsilastirma_testi`). Bu kayıtlar yeni SHA'da yeniden
koşulmuş kanıt sayılmaz. `v1.0.0-rc.1` etiket mesajı özgün RC kaynağı
`aaf9aa88…`'i korur; etiket nesnesi yeni karşılığı
`87598d4d5523ce0e953277db87fa9153379743dc`'yi gösterir. Yayın paketleri
değişmez; imzaları özgün kaynak SHA'sına bağlıdır.

CI push tabanı `ci-taban-kimligi.py` ile yalnız iki kayıtlı eski uç için
(GitHub `e196b6ed…`, Enterprise/Git2 `bbc9ba4b…`) exact yeni ataya çevrilir.

## Dış sınır

GitHub'ın yönetilen `refs/pull/*` ve önbellek nesneleri kullanıcı push'u ile
silinmez; açık PR'lardaki eski nesneler GB-046 destek teyidi kapsamındadır.
Enterprise ve Git2 aynalarının dal ve etiketleri 20 Ekim'de origin ile birebir
eşitlendi.

## Etiket koşuları

Yeniden yazılan `v1.0.0-rc.1` etiketinin push koşuları kırmızıdır ve öyle
bırakılır: `surum-adayi` var olan Release'i ezmeyi reddeder; `dil` testleri
etiket ağacında (`87598d4d`) henüz bulunmayan K-326 eşlemesi ile silinmiş
eski etiket tabanı yüzünden kırılır. Aynı kod master `d72a949`'da yeşildir.
Etikete özel CI istisnası eklenmez, etiket yeniden çekilmez.
