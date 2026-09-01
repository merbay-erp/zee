# VS Code eklentisi — Türkçe Programlama Dili (.dil)

`.dil` dosyaları için:

- sözdizimi renklendirme, `#` yorum kısayolu (Cmd+/), tırnak tamamlama,
  blok kelimelerinden sonra otomatik girinti;
- **canlı Türkçe tanılar** (yazarken altını çizer, kod + öneriyle),
- **hover** (kalıp kelimesine açıklama + örnek; ada, tanım satırı),
- **tanıma git** (işlem/yapı başlığına ya da `olsun` satırına),
- **yeniden adlandırma (F2)** — ekler Türkçe uyumla yeniden giydirilir
  (`sayaç`→`puan` ⇒ `sayacı`→`puanı`; K-072),
- **tamamlama** (kalıp kelimeleri).

LSP istemcisi elle yazılmıştır ([extension.js](extension.js)) — npm
bağımlılığı YOKTUR; protokol birlikte-çalışması gerçek `dillsp` ikilisine
karşı test edilmiştir.

## Kurulum (marketplace'e çıkmadan)

1. Sunucuyu derle ve PATH'e koy:

```bash
cd compiler && cargo build --release && sudo cp target/release/dillsp /usr/local/bin/
```

(PATH'e koymak istemezsen VS Code ayarlarında `dil.lspYolu`na tam yolu ver.)

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
- Yeniden adlandırma ve biçimleme LSP'ye eklendikçe istemci de genişler.
