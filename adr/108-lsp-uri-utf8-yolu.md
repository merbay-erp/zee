# ADR-108 — LSP dosya URI'sinde UTF-8 bayt çözümü

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-248

## Bağlam

Gerçek stdio `dillsp` oturumunda Türkçe klasördeki aynı belge literal UTF-8
URI ile tanısız açılırken yüzde kodlu URI ile A010/S015 üretti. Eski çözüm
`%C3%BC` baytlarını ayrı Unicode karakterlerine çevirerek `ü` yerine `Ã¼`
dosya yolunu arıyordu. İlk LSP kaynağı
`9a1f0b7e683bf778d3226aecbe8bd7b5f273740e` aynı arızalı çözümü eklemiştir.
[RFC 8089 bölüm 4](https://www.rfc-editor.org/rfc/rfc8089.html#section-4)
dosya URI'si için UTF-8 kodlamasını açıklar.

## Karar

Mevcut `file://` öneki korunur. Yüzde kaçışları tek bayta çevrilir; literal
UTF-8 baytları aynı tamponda korunur. Tamamlanan tampon bir kez UTF-8 olarak
doğrulanır. Eksik/nonhex kaçış, kesik UTF-8, overlong ve surrogate dizileri
path üretmez. `+` boşluğa çevrilmez; `%25` yalnız bir kez çözülür.

Bu doğruluk düzeltmesi yeni URI authority, UNC, Windows sürücü, query veya
fragment sözleşmesi açmaz. Yeni grammar, CLI, facade export, dependency,
önbellek veya kalıcı durum yoktur. Mevcut LSP belge/birim gezinme sınırları
korunur; fiziksel platform kabulü çıkarılmaz.

## Kanıt ve provenance

Olumlu/olumsuz private helper testleri yanında gerçek `dillsp` stdio testi
Türkçe, emoji, boşluk ve yüzde içeren geçici klasörden disk birimini yükler;
yanıtın URI'si, framing, tanı kodları, shutdown ve exit birlikte denetlenir.
`regression/lsp/yuzde-utf8-birim-yolu.dil` iki satırlık tek arızadır;
`yardimci.birim` hermetik yardımcıdır. `lsp-uri` kipi aynı native sınırı
manifestten yürütür. Kaynak commit'i önce, exact `fixed_by`, semantic-bugfix
ve core-freeze bugfix beyanı izleyen provenance commit'inde kaydedilir.

## Etki incelemesi

RFC/spec davranışı genişlemez; yeni tanı veya uyumluluk yüzeyi yoktur.
Hata kataloğu, RFC/spec indeksleri ve deprecation kaydı bu nedenle değişmez.
ADR-046 katman sorumluluğu/kenarları ile SCC değişmez ve kapıları çalışır.
ADR-045/049 sabit iş yükü ve performans tarihçesi değişmez; benchmark veya
soak kabulü bu küçük yol çözümünden türetilmez. README, sürüm notu, LSP
rehberi, corpus/ADR-044, günlük, backlog, faz ve kanıt kayıtları güncellenir.

Private UTF-8 helper testleri `compiler/src/lsp/uri_testleri.rs` içindedir;
LSP kök dosyasının 1500 satır sınırı yükseltilmez. Test taşıması production
modül kenarı veya semantic davranış değişikliği değildir.
