# Zee conformance korpusları

Bu dizin belirli bir derleyicinin kaynak ağacına ait değildir. Zee'nin ikinci
ve sonraki gerçeklemeleri, yalnız buradaki sürümlü veri ile normatif RFC/spec
metinlerini kullanarak aynı gözlenebilir kararları üretmelidir.

| Alan | Şema | Profil verisi | Test sahibi |
|---|---|---|---|
| Türkçe tanımlayıcı morfolojisi | [`morfoloji/sema-v1.schema.json`](morfoloji/sema-v1.schema.json) | [`morfoloji/zee-tr-1.json`](morfoloji/zee-tr-1.json) | `compiler/tests/morfoloji_conformance_testi.rs` |

Yayımlanmış profil verisi yerinde değiştirilmez. Yeni anlam yeni profil
kimliği ve yeni veri dosyası ister; ayrıntı
[`docs/morfoloji-conformance.md`](../docs/morfoloji-conformance.md)'dedir.
