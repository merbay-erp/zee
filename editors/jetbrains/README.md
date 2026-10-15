# JetBrains — mevcut dillsp için yerel POSIX şablonu

Bu dizin, LSP4IJ ile içe alınan bir kullanıcı şablonudur. Yeni compiler
özelliği veya dağıtılmış JetBrains eklentisi değildir. Şablonun IDE içinde
kurulum ve etkileşim kabulü henüz açık; gerçek stdio sunucusu mevcut LSP
ve K-270 launcher testleriyle ayrıca korunur.

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
   Sunucu tam metin eşitlemesi kullanır. Incremental cache, LSP format/lint
   sağlayıcısı ve <200 ms bütün-proje garantisi bu şablonla eklenmez.

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
