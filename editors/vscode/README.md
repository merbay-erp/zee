# VS Code eklentisi — Türkçe Programlama Dili (.dil)

`.dil` dosyaları için:

- sözdizimi renklendirme, `#` yorum kısayolu (Cmd+/), tırnak tamamlama,
  blok kelimelerinden sonra otomatik girinti;
- **canlı Türkçe tanılar** (yazarken altını çizer, kod + öneriyle),
- **hover** (kalıp kelimesine açıklama + örnek; ada, tanım satırı),
- **tanıma git** (`SymbolId`/`IslemId`/`YapiId` HIR bağıyla doğru kapsama),
- **yeniden adlandırma (F2)** — yalnız seçilen semantic kimlik; ekler Türkçe
  uyumla yeniden giydirilir, çok kelimeli işlem adı birlikte değişir
  (`sayaç`→`puan` ⇒ `sayacı`→`puanı`; K-072); başka dosyadaki tanım için
  eksik tek-dosya düzenlemesi üretmez,
- **tamamlama** (kalıp kelimeleri).

LSP istemcisi elle yazılmıştır ([extension.js](extension.js)) — npm
bağımlılığı YOKTUR; protokol birlikte-çalışması gerçek `dillsp` ikilisine
karşı test edilmiştir.

## Kurulum (marketplace'e çıkmadan)

1. [Kanonik teslim/kurulum](../../docs/surum-runbook.md) ile sahipli prefix'e
   kurulan `dillsp` ikilisinin mutlak yolunu VS Code ayarlarına açıkça ver:

```json
{
  "dil.lspYolu": "/absolute/owned/prefix/bin/dillsp"
}
```

Örnekteki yolu kendi doğrulanmış kurulumuna göre değiştir. Geliştirme için ayrı
owned checkout'ta `cd compiler && cargo build --locked --release --bin dillsp`
sonrası `compiler/target/release/dillsp` mutlak yolu da seçilebilir; bu kanonik
iki temiz klon teslim/kurulum kabulü yerine geçmez. Global PATH veya ayrıcalıklı
kopyalama gerekmez. Eksik/bozuk seçimi başka bir compiler ile değiştirme.

2. Eklentiyi bağla (bu klasörün içinde):

```bash
ln -s "$(pwd)" ~/.vscode/extensions/zee-dil.dil-vscode-0.1.0
```

VS Code'u yeniden başlat; `.dil` dosyası aç.

## Kapsam notları

- TextMate grammar'ı ayrıştırıcıyla birebir aynı kelime kümesini kullanır;
  yeni kalıp eklenince [syntaxes/dil.tmLanguage.json](syntaxes/dil.tmLanguage.json)
  güncellenmelidir (kaynak: `compiler/src/ayristirici.rs`).
- Belge eşitleme tam metindir (textDocumentSync: 1) — dosyalar küçükken
  (eğitim ölçeği) doğru ve basit olan bu.
- Yeniden adlandırma mevcut semantic sözleşmeyle sunulur. LSP biçimleme/lint ve
  incremental cache bu istemciye eklenmiş sayılmaz.
