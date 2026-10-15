# K-289: bileşenlerden TR/EN liste ve detay

`golden/composed-tr-en` anonim, uygulamaya ait altı rotalı bir sayfadır.
K272'nin büyük, iki içeriği birlikte üretip `hidden` ile seçen ağacı yerine
`components.dil` bağlantı, gezinme, liste, detay ve altbilgiyi ayrı işlemlerde
üretir. `layout.dil` gezinme/ana içerik/altbilgi adlı slotları bu işlemlerden
oluşturur; yalnız seçilen liste veya detay ağacı çıktıya girer. Aynı bağlantı
bileşeni hem gezinmede hem listede kullanılır. HTML birleştiren `ile` ifadesi
yoktur; katalog anahtarı birleştirmesi yalnız sözlük erişimidir.

`catalog.dil` başlık, gövde, gezinme, altbilgi dahil bütün sayfa etiketlerinin
TR/EN kaynağıdır. Eksik anahtar ve dil açık Sonuç hatasıdır. Sabit altı rota
`routes.dil` içinde dil/seçim kimliğine çevrilir; bilinmeyen rota 404 olur.
Bunlar uygulama işlemleridir; yeni `çevir` sözcüğü veya rota API'si değildir.

Slot güven sınırı kapalıdır: layout dışarıdan HTML Metin parametresi almaz.
`ham` yalnız kendi görünüm bloklarının ürettiği bileşen çıktısını ve sabit
DOCTYPE bildirimini yerleştirir (spec/28 mevcut sözleşmesi). Kullanıcı/katalog
verisi bileşenlerde `metin` veya kaçırılan özniteliktir. Ara HTML değerleri
Metin'dir; dilin ayrı bir güvenli HTML türü, sandbox veya ham HTML doğrulayıcısı
varmış gibi sunulmaz. Kaynak kodunu değiştirebilen uygulama yazarı güvenilirdir.
Genel üçüncü taraf slot/HTML eklentisi bu örneğin kapsamı değildir.

```sh
python3 -I -B scripts/tests/composed-tr-en-test.py \
  --framework-root /absolute/clean/source \
  --dil /absolute/same-source/dil --output /absolute/fresh/evidence
```

Runner ikiliyi ve yardımcıları kendi geçici alanına snapshot eder; başlangıç ve
son hashlerini karşılaştırır. Gerçek Zee `denetle` ve dört `dene` testi, altı
HTTP200 sayfası, iki 404 ve katalogdaki düşman metinle iki ek HTTP200 yürütür.
Bağımsız Python byte oracle'ı tüm HTML'i karşılaştırır; ağaçta yalnız seçilen
slot bulunur, script/img/event özniteliği doğmaz ve metin çift kaçırılmaz.
Kendi process-group'ları temizlenir, başka owner'ın süreci canlı kalır.
K272/K249 kaynakları değiştirilmez; eski testlerin kabulü ayrıca korunur.

Bu anonim uygulama bileşimi C4'ün mevcut grammar ile uygulanabilen somut
liste/detay dilimidir. Özel ürünün gerçek sayfasına aktarım, genel yerleşim
kütüphanesi ve dilde yerleşik çeviri/safeHTML yüzeyi ayrı açık sınırlardır.
Geliştirme ağacındaki ikiliyle prova, yeni temiz SHA kabulünün yerine geçmez.
