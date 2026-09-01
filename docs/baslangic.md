# 5 dakikada başla

## 0. Kurulumsuz yol: playground

Hiçbir şey kurmadan denemek istersen: `playground/zee-playground.html`
dosyasını çift tıkla — derleyici sayfanın içinde, tarayıcıda yaz-çalıştır.
(Dosya depoda yoksa `playground/olustur.sh` ile bir kez üret.)

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

Yeni klasörde `proje.dil` bulunur; giriş dosyası ve sürüm burada tanımlıdır.
Bu nedenle dosya adını ezberlemeden projeyi çalıştırabilirsin:

```bash
dil çalıştır ilk-projem
```

Program adını sorar, seni selamlar — ve içinde hazır bir test vardır:

```bash
dil dene ilk-projem
```

Proje klasöründeysen daha da kısadır: `dil çalıştır .`, `dil dene .`,
`dil denetle .`.

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
| `dil çalıştır <dosya\|proje>` | dosyayı ya da `proje.dil` taşıyan klasörü çalıştırır; sonraki argümanlar programa gider |
| `dil çalıştır --güvenli <dosya\|proje>` | çocuk modu: ağ kapalı, dosyalar programın klasörüyle sınırlı |
| `dil dene <dosya\|proje>` | `test "..."` bloklarını koşar |
| `dil biçimle <dosya\|proje>` | dosyayı veya projedeki bütün `.dil` kaynaklarını resmi biçime getirir |
| `dil denetle <dosya\|proje>` | çalıştırmadan hata arar (`--json`: makine çıktısı) |
| `dil hata <kod>` | bir hata kodunu açıklar |
| `dil belge <birim>` | birimin işlemlerini listeler (örn. `dil belge matematik`) |
| `dil sürüm` | sürümü gösterir |

## 6. Daha fazlası

- Dilin bütün yüzeyi tek yazıda: [dil-turu.md](dil-turu.md)
- Oynayarak öğren: [projeler/](../projeler/) — çocuklar için proje kitaplığı
- Dilin bütün örnekleri: [golden/](../golden/) — 32 program, kolaydan zora
  numaralı, hepsi çalışır belge niteliğinde.
- Dilin OLMADIĞI şeyler: [anti-ornekler/](../anti-ornekler/)
- Hata sözlüğü: [hata-katalogu.md](hata-katalogu.md)
- Normatif tanım: [../spec/](../spec/) — dilin resmi spesifikasyonu
- Gömülü kitaplık: [../kitaplik/](../kitaplik/) — `matematik birimini kullan` her yerde çalışır
- Tasarım kararlarının tamamı: [../rfcs/](../rfcs/) ve [../kararlar/gunluk.md](../kararlar/gunluk.md)
- VS Code renklendirme: [../editors/vscode/](../editors/vscode/)
