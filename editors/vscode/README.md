# VS Code eklentisi — Türkçe Programlama Dili (.dil)

`.dil` dosyaları için sözdizimi renklendirme, `#` yorum kısayolu
(Cmd+/), tırnak tamamlama ve blok kelimelerinden sonra otomatik girinti.

## Yerel kurulum (marketplace'e çıkmadan)

```bash
ln -s "$(pwd)" ~/.vscode/extensions/zee-dil.dil-vscode-0.0.1
```

Bu klasörün içinde çalıştır; sonra VS Code'u yeniden başlat. `.dil` uzantılı
her dosya otomatik renklenir.

## Kapsam

- TextMate grammar'ı ayrıştırıcıyla birebir aynı kelime kümesini kullanır;
  yeni kalıp eklenince [syntaxes/dil.tmLanguage.json](syntaxes/dil.tmLanguage.json)
  güncellenmelidir (kaynak: `compiler/src/ayristirici.rs`).
- LSP (`dillsp`: tamamlama, tanıya gitme, yeniden adlandırma) ayrı iştir —
  master plan bölüm 15; bu eklenti onun ilk taşıyıcısı olacak.
