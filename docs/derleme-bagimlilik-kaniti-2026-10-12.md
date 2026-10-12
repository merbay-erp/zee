# K-191/K-192 yerel derleme kanıtı — 12 Ekim 2026

Kaynak commit'i: `02c1c0d49d1c6fef907fa9a1bcf75fbb2db44f2c`.
Dal: `codex/zee-derleme-bagimlilik`. Bu kayıt benchmark tarihçesi değildir.

## K-191 gerçek çift-klon üretimi

Temiz kaynak commit'inde `bash scripts/surum-artefakti.sh` çalıştırıldı.
Platform `aarch64-apple-darwin`; derleyici
`rustc 1.93.1 (01f6ddf75 2026-02-11)`;
`SOURCE_DATE_EPOCH=1791798293`. Her iki bağımsız klonda:

| İkili | SHA-256 |
|---|---|
| `dil` | `c73ee537c628def5151ea97206e714e9d3dd01f90dbced2745f14047fdcb603c` |
| `dillsp` | `064f2ab948f5b32a707962a0e8045cd7794136a18d97e40d78e72acd7372abad` |

Provenance `externalParameters.rustc` yukarıdaki sabit sürümü taşır;
`gitCommit` exact kaynak commit'idir; `ikiTemizKlonEsit=true`.
`SHA256SUMS` ve ikinci klonun özeti aynı
`3d4e1830b0467609f46413796af35bad6455fc4d3327b01e35166222367a5d37`
değerindedir. SBOM ve provenance üretildi; anahtar verilmediği için imza yoktur.
Bu yerel doğrulama sürüm etiketi, imzalı dağıtım veya V1 RC değildir.

## K-192 macOS ilk dilimi

Ortam: macOS 26.6.2 arm64, SDK 27.0,
Apple clang 21.0.0 (`clang-2100.3.34.2`). `otool -L` her iki release
ikilisinde aynı doğrudan sistem bağımlılıklarını gösterdi:

- `/System/Library/Frameworks/SystemConfiguration.framework/Versions/A/SystemConfiguration`
- `/System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation`
- `/System/Library/Frameworks/Security.framework/Versions/A/Security`
- `/usr/lib/libiconv.2.dylib`
- `/usr/lib/libSystem.B.dylib`

Rust araçları PATH'ten çıkarıldı; `command -v` ile `rustc`, `cargo`, `rustup`
bulunmadığı doğrulandı. Yukarıdaki release `dil` ikilisi
`golden/01-merhaba-dunya.dil` kaynağını exit 0 ve exact `Dünyaya merhaba\n`
çıktısıyla çalıştırdı. Komut (ikili yolunu yerel artefakta göre ayarla):

```bash
env PATH=/usr/bin:/bin:/usr/sbin:/sbin /bin/sh -c '
  for arac in rustc cargo rustup; do
    if command -v "$arac" >/dev/null 2>&1; then exit 1; fi
  done
  exec "$1" çalıştır golden/01-merhaba-dunya.dil
' sh /mutlak/artefakt/yolu/dil
```

Bu makinede Rust kurulu olmaya devam eder. PATH yalıtımı, Rust hiç kurulmamış
bir makine ya da kurulum/kaldırma provası değildir. `otool -L` doğrudan yükleme
komutlarını gösterir; geçişli sistem kitaplıkları veya minimum macOS sürümü
bu kayıtla kanıtlanmaz. Linux/Windows sonuçları ve `dillsp` stdio provası
bu dilimde ölçülmedi; K-192 **kısmen açık** kalır.

## Provenance'da kalan kaynak kimliği sınırı

Bu kaynak commit'inde üretilen SBOM `software_downloadLocation` ve provenance `resolvedDependencies.uri`
`compiler/src/bin/surum_artefakti.rs` içinde sabit
`git+https://github.com/merbay-erp/zee@<sha>` kullanır. Bu checkout'un origin'i
`https://github.com/itwise-bilisim/zee.git`tir. Eski URL'nin yönlendirmesi veya
bu exact commit'in uzaktan erişilebilirliği bu koşuda doğrulanmadı.
K-192 devam dilimi (ADR-068) SBOM/provenance üreticisini kanonik
`itwise-bilisim/zee` kaynağına bağlayarak GB-030 metadata bulgusunu kapattı.
Bu tarihsel artefaktın metadata'sı yeniden yazılmadı. Exact yeni commit'in
uzak erişimi ayrı yayın kanıtıdır; hash eşitliği bunun yerini tutmaz.

## Açık devam

[Envanter](derleme-ve-bagimliliklar.md) ve
[backlog](oncelikli-backlog.md): K-192 üç platform/sistem araçları/kaynak
kimliği/temiz kurulum kanıtıyla kapanır. K-193 backend işi mevcut ürün ve
V1 kapılarından sonra değerlendirilir; yeni compiler kapasitesi iddia edilmez.
