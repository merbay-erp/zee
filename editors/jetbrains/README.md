# JetBrains — mevcut dillsp için yerel POSIX şablonu

Bu dizin, LSP4IJ ile içe alınan bir kullanıcı şablonudur. Yeni compiler
özelliği veya dağıtılmış JetBrains eklentisi değildir. Şablonun macOS IntelliJ/LSP4IJ etkileşimi K-280 ile tarihsel S1 kaynağında
doğrulandı. Yeni kaynak ve diğer platformlar ayrıca sınanır; proje pininin
seçtiği compiler sürümü gerçek sunucu yeteneklerini belirler.

1. [Yerel araç rehberindeki](../../docs/derleyici-yerel.md) signed bundle
   import ve proje launcher kurulumunu tamamla. Proje kökünde
   `.zee-araclar/zee` ve `derleyici.kilit.json` bulunmalı. `proje.kilit`
   bağımlılık kilididir; compiler pininin yerine geçmez.
2. IDE içinde LSP4IJ kullanılabiliyorsa Language Servers ayarından yeni
   sunucu oluştur, `Import from custom template...` ile bu dizini seç.
   Açık projenin kökünü ve Command önizlemesini denetle:
   `"$PROJECT_DIR$/.zee-araclar/zee" lsp`. Boşluk içeren yol da tek komut
   yolu olarak kalmalı. `$PROJECT_DIR$` genişlemesini IDE yapar.
3. Mappings içinde `*.dil` ve language ID `dil` seçili olmalı.
   Bir `.dil` belgesi aç, initialize sonrasında tanılar, completion,
   hover, tanıma git ve rename işlevlerini dene; shutdown/exit ile kapat.
   Tarihsel S1 pininde tam metin eşitlemesi vardır. K-287 adayı UTF-16
   aralıklı eşitleme, biçim ve opt-in lint ekler; güncel sınırlar
   [editör rehberindedir](../../docs/editor-analizi.md). Şablon bu compiler
   güncellemesini kendiliğinden kurmaz.

Sözdizimi rengi için IDE'nin TextMate Bundles ayarına mevcut
[VS Code dizinini](../vscode/) ekle. JetBrains, `package.json` taşıyan VS Code
bundle dizinlerini desteklediğini belgeliyor. Grammar kopyası tutulmaz;
`.dil` eşlemesini özel file type oluşturarak ezme. VS Code JavaScript
istemcisi JetBrains içinde çalıştırılmaz.

Şablon POSIX K-270 launcher profilidir. Windows native launcher ve IDE
kabulü açık kalır. LSP4IJ'nin indirme/installer alanları bu şablonda yoktur;
IDE eklentisini veya executable'ı kendisi kurmaz. Local proje launcher'ı
ve trust policy operatörün bilinçli yürütme sınırıdır. Eksik/bozuk pin için
başka PATH compiler'ına düşme yoktur. Alternatif olarak Command alanına
owned kurulumundaki `dillsp` ikilisinin tam yolunu elle verebilirsin; bu,
proje pinli launcher kabulü sayılmaz.

Kaynak şema ve import/macro davranışı [sabit upstream
belgesinden](https://github.com/redhat-developer/lsp4ij/blob/12703a3301557acd6d57bc903fed07cd826f632f/docs/UserDefinedLanguageServerTemplate.md)
ve [kullanıcı şablonu
rehberinden](https://github.com/redhat-developer/lsp4ij/blob/12703a3301557acd6d57bc903fed07cd826f632f/docs/UserDefinedLanguageServer.md)
alındı. [JetBrains TextMate
rehberi](https://www.jetbrains.com/help/idea/textmate.html) bundle import'unu
anlatır. `upstream.json` belge hashlerini kaydeder; kurulmuş bir IDE/plugin
sürümü veya güvenli plugin indirme kanıtı değildir.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

IntelliJ IDEA2025.3.1/LSP4IJ0.21.0 izole profilinde `Import template...` ile bu klasör, TextMate Bundles’a mevcut VS Code klasörü alındı; yeniden açılış sonrasında gerçek sunucu başlatıldı. Boşluk içeren `$PROJECT_DIR$` önizlemesi ve tanı/düzeltme, hover, tanıma git, rename görüldü. Tamamlama ve düzenli UI/LSP kapanışı doğrulanamadı; OS kısayol çakışması ve CUA timeout retleri korunur. [Son rapor](../../docs/urunlesme-final-kabul-20261015.md). Bu dar kısmi kabul tam P19 veya Windows kabulü değildir.


## K-280 — açık kalan etkileşimin tamamlanması

15 Ekim ayrı owned profilde mevcut S8b49113 launcher ile Basic tamamlama
`sayi 7 ols` → `sayi 7 olsun` uygulandı ve LSP tanısı boşaldı. macOS Ctrl+Space
çakışmasında Find Action → Basic eylemi kullanılabilir; benzersiz aday doğrudan
uygulanır. Debug → verbose trace gerçek completion trafiğini gösterdi. Trace
uygulamasında shutdown/yanıt/exit, normal Quit/Confirm Exit sonrasında exit0 ve
owned süreç grubu kapanışı doğrulandı. Final Quit konsolu dışa aktarılmadı;
önceki başarısız denemeler silinmedi. [ADR-138](../../adr/138-jetbrains-tamamlama-ve-duzenli-kapanis.md)
ve [kanıt](../../docs/incelemeler/urunlesme-20261014/k280-jetbrains/ACCEPTANCE.json).
Bu mevcut keyword desteğinin S1 kanıtıdır. K-287 yeni eşitleme/cache/lint
kaynağının temiz IDE kabulü ayrıca yapılır; Windows/marketplace yayını açıktır.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](../../docs/urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.
