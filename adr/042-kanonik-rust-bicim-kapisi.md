# ADR-042 — Kanonik Rust biçim kapısı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-145, B-037

## Bağlam

Depo `rustfmt` ile parça parça biçimlenmişti fakat bütün kaynak ağacı kanonik
değildi. `cargo fmt --check` 53 dosyada 349 fark bloğu üretiyor; CI yalnız
derleme, test ve Clippy ile bunu göremiyordu. Bu borcu özellik değişikliklerine
karıştırmak, incelemeyi gürültülü ve tarihsel sorumluluğu belirsiz bırakır.

## Karar

1. K-145 yalnız sabit Rust 1.93.1 `rustfmt` çıktısını uygulayan kontrollü,
   davranış değiştirmeyen toplu biçim dilimidir. 53 dosyanın tümü aynı komutla
   kanoniklenir; elle seçilmiş stil istisnası yoktur.
2. `rustfmt` bileşeni `rust-toolchain.toml` ve üç platformlu CI kurulumunda
   açıktır. `cargo fmt --all -- --check` her push ve pull request'te çalışır.
3. Fiziksel faz satır bütçeleri kanonik biçimin gerçek satırlarına bir kez
   yeniden kalibre edilir. Artış yeni davranış alanı değildir; eski sıkışık
   yazımın biçimleyici tarafından açılmasıdır. Sonraki bütçe değişiklikleri
   ADR-012'nin sorumluluk kuralına tabidir.
4. K-144 işlev eğilim tabanı aynı semantik kaynakların kanonik biçimdeki satır
   ölçüsüne `K-145/ADR-041 kanonik rustfmt tabanı` kaydıyla yeniden alınır.
   Karmaşıklık kararı değişmez; biçimin ürettiği tek yeni 80+ satırlık işlevle
   güncel taban 49 kayıttır.

## Reddedilen seçenekler

- **Yalnız değiştirilen dosyaları biçimlemek:** yıllarca sürebilen karma stil
  bırakır ve CI hard gate'ini açamaz.
- **Biçim farkını otomatik commit etmek:** geliştiricinin çalışma ağacını
  habersiz değiştirir; CI yalnız denetler.
- **Eski işlev tabanını korumak:** anlamsal olmayan satır açılımlarını tasarım
  gerilemesi gibi raporlar ve gerçek sonraki trendi ölçemez.
- **Satır bütçelerini kaldırmak:** biçim değişikliğini gerekçe göstererek
  fiziksel sahiplik koruğunu gereksiz yere zayıflatır.

## Sonuçlar

- Yeni Rust değişikliği kanonik biçimi aynı committe taşımak zorundadır.
- Tam test paketi, Clippy, release, rustdoc, wasm32/Node ve tedarik kapıları
  biçim öncesiyle aynı davranışı kanıtlar.
- Dil semantiği, kullanıcı tanısı, RFC ve normatif spec değişmez.
