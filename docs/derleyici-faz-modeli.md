# Derleyici faz modeli

Bu belge ADR-015'in uygulama rehberidir. Dosya/modül sahipliği için
[derleyici faz sınırları](derleyici-faz-sinirlari.md), checker içi kurallar
için [checker katmanları](checker-katmanlari.md) birlikte okunur.

## Bugün çalışan hat

| Faz türü | Taşıdığı gerçek | Sonraki tek geçerli adım |
|---|---|---|
| `KaynakMetni` | UTF-8 zee kaynak metni | `sozcukle()` |
| `TokenAkisi` | konumlu lexer tokenları | `ayristir(...)` veya kurtarmalı parser |
| `AyristirilmisAst` | hoist/çözüm görmemiş `Cumle` ağacı | birim/paket çözümü + hoist |
| `BaglanmamisProgram` | cümle, işlem, yapı ve test koleksiyonu | checker `denetle()` geçişi |
| `BaglanmisProgram` | ad/ID bağları ve tür denetimi tamam AST | interpreter veya bilinçli eski-API adaptörü |

```text
source ──lexer──> tokens ──parser──> parsed AST ──hoist──> unbound program
                                                           │
                                              resolver + type/effect/flow
                                                           ▼
                                                    bound program ──> runtime
```

## Kullanım

Yeni uçtan uca kod `kaynagi_fazli_derle` veya yükleyicili/kökenli eşini
kullanır. Yürütme `yorumlayici::calistir_baglanmis` ya da
`calistir_baglanmis_io` ile yapılır. `kaynagi_derle` ve raw `Program` alan
runtime fonksiyonları yalnız v0 embedding uyumluluğudur; yeni iç kod bunlara
dayanmaz.

`BaglanmisProgram::program()` immutable görünüm verir. `into_program()` faz
bilgisini bilinçli silen sınırdır; checker öncesi çağrılamaz.

## Bilinçli boşluk: HIR

`BaglanmisProgram` hâlâ kaynak AST'sidir. Değişken düğümü hem kaynak adını hem
`SymbolId`yi, çağrı/yapı düğümü hem adı hem ID'yi taşır. Bu geçiş biçimi
B-019'daki typed HIR değildir. Hedef hat şöyledir:

```text
Parsed AST → Resolution sonucu → Typed HIR → Execution/Lowering
```

B-019 açık tür ve yalnız ID taşıyan HIR'ı; B-020 her semantic düğümde zorunlu
source span'i kurmadan runtime kaynak AST'den kopmuş sayılmaz.

## Kanıt ve büyüme kuralı

- `faz_modeli_testi.rs` parsed/bound farkını ve eski API uyumluluğunu sınar.
- `faz.rs` compile-fail örneği yanlış geçişin derlenmediğini kanıtlar.
- `mimari_sinir_testi.rs` standart hattın faz türlerini gerçekten kullandığını
  ve modül bütçesini korur.
- Yeni compiler aşaması çıplak tuple/type alias ile gizlenmez; veri türü,
  geçiş sahibi, hata biçimi ve hangi önceki fazı tükettiği aynı ADR/rehber
  değişikliğinde yazılır.
