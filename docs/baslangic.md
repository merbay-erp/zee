# 5 dakikada başla

## 1. Kur (bir kez)

```bash
cd compiler && cargo build --release
```

Çıkan tek dosya her şeydir: `compiler/target/release/dil` — kopyala, taşı,
internetsiz kullan. İstersen PATH'e ekle:

```bash
sudo cp compiler/target/release/dil /usr/local/bin/dil
```

## 2. İlk projen

```bash
dil yeni ilk-projem
```

```bash
dil çalıştır ilk-projem/program.dil
```

Program adını sorar, seni selamlar — ve içinde hazır bir test vardır:

```bash
dil dene ilk-projem/program.dil
```

## 3. Kendi programını yaz

`merhaba.dil` diye bir dosya aç:

```
"Adın ne?" diye sor
"Merhaba " ile yanıt yaz

yaş 10 olsun
yaş 8 veya daha büyükse
    "Programlamaya başlayabilirsin!" yaz
değilse
    "Biraz daha oyun zamanı" yaz
```

```bash
dil çalıştır merhaba.dil
```

## 4. Bir hata al — korkma

Hatalar Türkçedir, yol gösterir:

```
HATA A001

"toplem" adı bu kapsamda tanımlı değil.

3 | toplem yaz
    ^^^^^^

Öneri:
Bu ad tanımlı değil. Tanımlı adlar: toplam. Önce "<ad> <değer> olsun" ile tanımla.

Ayrıntı için: dil hata A001
```

Son satırdaki komutu çalıştırırsan hatanın tam açıklamasını görürsün —
internet gerekmez, katalog `dil`in içindedir.

## 5. Araç kutusu

| Komut | Ne yapar |
|---|---|
| `dil yeni <ad>` | testli başlangıç projesi kurar |
| `dil çalıştır <dosya>` | programı çalıştırır (`dil çalıştır oyun.dil Ali Ayşe` → argümanlar programa gider) |
| `dil dene <dosya>` | `test "..."` bloklarını koşar |
| `dil biçimle <dosya>` | kodu resmi biçime getirir (girinti, boşluklar) |
| `dil denetle <dosya>` | çalıştırmadan hata arar (`--json`: makine çıktısı) |
| `dil hata <kod>` | bir hata kodunu açıklar |
| `dil sürüm` | sürümü gösterir |

## 6. Daha fazlası

- Dilin bütün örnekleri: [golden/](../golden/) — 30 program, kolaydan zora
  numaralı, hepsi çalışır belge niteliğinde.
- Dilin OLMADIĞI şeyler: [anti-ornekler/](../anti-ornekler/)
- Hata sözlüğü: [hata-katalogu.md](hata-katalogu.md)
- Tasarım kararlarının tamamı: [../rfcs/](../rfcs/) ve [../kararlar/gunluk.md](../kararlar/gunluk.md)
- VS Code renklendirme: [../editors/vscode/](../editors/vscode/)
