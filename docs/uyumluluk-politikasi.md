# Uyumluluk ve deprecation rehberi

Bu rehber K-167/RFC-0028/ADR-064'ün işletim sözleşmesidir; normatif metin
[spec/27](../spec/27-uyumluluk-ve-surumleme.md)'dedir.

## Kayıtlar

- [`compiler/tests/fixtures/dil-yuzeyi-v1.tsv`](../compiler/tests/fixtures/dil-yuzeyi-v1.tsv):
  `yuzey, oge, durum, giris, kaynak, kayit` alanlarıyla dondurulmuş envanter.
  Kelime yüzeyleri (`kalip`, `kosul`, `komut`) kaynaktan türetilir ve `kayit`
  `-`dir; `bicim`, `profil`, `abi`, `api`, `tani` satırları kaynak dosyada
  aynen bulunması gereken exact kaydı taşır. Satırlar (yüzey, öğe) sırasında
  ve tekildir.
- [`docs/deprecation-kayitlari-v1.tsv`](deprecation-kayitlari-v1.tsv):
  `kimlik, yuzey, oge, sinif, duyuru, kaldirma, goc, karar, durum` alanlı
  append-only kayıt. Kimlik ardışık `DEP-NNN`, sınıf
  `kaldirma|davranis|yeniden-adlandirma`, durum `duyuruldu|kaldirildi`dir.

## Yeni yüzey eklerken

1. Kelimeyi/komutu kaynağa ekle (K-160A freeze kuralları geçerlidir; K-174
   V1 syntax freeze penceresinde — `docs/v1-syntax-freeze-v1.tsv` — kalıp,
   koşul ve komut yüzeyi değişmez, değişiklik kaydı bilinçli güncellenmeden
   `uyumluluk_testi` geçmez).
2. Fixture'a `aktif` satırı ve mevcut `-dev` giriş sürümünü yaz.
3. `cargo test --locked --test uyumluluk_testi` geçmeli.

## Bir yüzeyi kaldırırken

1. `duyuruldu` durumlu `DEP-NNN` kaydı aç; kaldırma sürümü desteklenen
   yüzeyde bir sonraki alt sürüm serisidir. Sürüm notuna yaz.
2. Kaldırma sürümünde kaynaktan çıkar; fixture satırını `kaldirildi` yap,
   kaydı `kaldirildi`ye çevir.
3. Eski biçimin tanı+öneriyle reddedildiğini `regression/` vakasıyla kanıtla.

İç (`ic`) Rust yolları ve tanı mezar taşları süre şartı taşımaz; yine de kayıt
ister.

## Sürüm kesimi

Etiket atıldıktan hemen sonra `compiler/Cargo.toml`, `fuzz/Cargo.toml`
bağımlılığı ve iki `Cargo.lock` bir sonraki `X.Y.0-dev` serisine çekilir;
`uyumluluk_testi` çalışma ağacının en yüksek `v*` etiketinden büyük olmasını
ister.

## Kapı

```bash
cd compiler
cargo test --locked --test uyumluluk_testi
```

Kapı Ubuntu/macOS/Windows CI'ında tam test paketiyle çalışır; faz matrisinde
`muhe_kapilari` sahibindedir.
