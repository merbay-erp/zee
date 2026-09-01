# Production panic politikası

Bu belge ADR-021'in uygulama rehberidir. Amaç her bug'ı imkânsız ilan etmek
değil; kullanıcı girdisi veya dış durumla erişilebilen kolay panic yüzeyini
production derlemesinde tekrar açılamaz hale getirmektir.

## Zorunlu kapı

`compiler/src/lib.rs`, `main.rs`, `bin/dillsp.rs` ve `bin/olcum.rs`, test dışı
derlemede aşağıdaki kalıpları reddeder:

```text
unwrap / expect
panic! / unreachable!
todo! / unimplemented!
```

Bu, kök dosyaları okumaya dayalı iki regressyondan biri ve gerçek Clippy
lintleriyle iki kat korunur. Bağlayıcı komut:

```bash
cd compiler
cargo clippy --all-targets -- -D warnings
```

## Hata sınıfları

| Sınıf | Production davranışı |
|---|---|
| Kullanıcı kaynağı / parser girdisi | Kodlu Türkçe `Tani` |
| Elle kurulmuş veya bozuk AST/HIR | Checker'da T016, runtime'da C000 iç tutarlılık tanısı |
| Dosya, socket, thread, arşiv ve proje grafiği | `Result`/açık CLI hatası; process panic'i yok |
| Scheduler iç durumu | C000; kardeş iptali ve savepoint temizliği korunur |
| Matematiksel/toplam dönüşüm | Panikli assertion yerine toplam kurucu veya denetlenmiş `Option`/`Result` |
| Test fixture'ı | `cfg(test)` altında `expect` serbest; production sözü değildir |

## Audit özeti

K-109'da 46 production nokta temizlendi. Öne çıkanlar:

- girinti yığınları ve parser tek-eleman seçimleri boşlukta güvenli davranır,
- kalıcı dosya temp yolu, proje/paket giriş kaydı ve arşiv tamsayı okumaları
  açık hata döndürür,
- checker'ın çözülmüş sembol/yapı/alan/işlem kayıtları eksikse T016 verir,
- raw/malformed runtime durumu ve scheduler invariantları C000 verir,
- CLI thread açma/join ve ölçüm girdisi başarısız süreç koduna iner,
- `SymbolId` yapay 32-bit kapasite assertion'ı taşımaz.

Bu kapı, keyfî UTF-8 parser girdisinin tamamında panic-free olmayı tek başına
kanıtlamaz. B-015/K-110 [fuzz hattı](fuzzing.md) kullanıcı girdisi sınırını;
B-017/K-112 [AST/HIR invariant kapısı](ast-hir-invariantleri.md) malformed iç
yapıların parser/checker sonrası kanıtını kapattı. İhlaller panic yerine faz
ve yapısal yol taşıyan C000'e dönüşür.
