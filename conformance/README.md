# Zee conformance korpusları

Bu dizin belirli bir derleyicinin kaynak ağacına ait değildir. Zee'nin ikinci
ve sonraki gerçeklemeleri, yalnız buradaki sürümlü veri ile normatif RFC/spec
metinlerini kullanarak aynı gözlenebilir kararları üretmelidir.

| Alan | Şema | Profil verisi | Test sahibi |
|---|---|---|---|
| Türkçe tanımlayıcı morfolojisi | [`morfoloji/sema-v1.schema.json`](morfoloji/sema-v1.schema.json) | [`morfoloji/zee-tr-1.json`](morfoloji/zee-tr-1.json) | `compiler/tests/morfoloji_conformance_testi.rs` |
| Yapılandırılmış eşzamanlılık | [`eszamanlilik/sema-v1.schema.json`](eszamanlilik/sema-v1.schema.json) | [`eszamanlilik/zee-esz-1.json`](eszamanlilik/zee-esz-1.json) | `compiler/tests/eszamanlilik_conformance_testi.rs` |

Yayımlanmış conformance verisi yerinde değiştirilmez. Yeni anlam yeni profil
kimliği ve yeni veri dosyası ister. Morfoloji ayrıntısı
[`docs/morfoloji-conformance.md`](../docs/morfoloji-conformance.md)'dedir.
Scheduler'ın gözlenebilir uyumluluk sınırı
[`docs/eszamanlilik-conformance.md`](../docs/eszamanlilik-conformance.md)'dedir.
