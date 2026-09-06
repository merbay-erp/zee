# Sürüm runbook'u

K-169/ADR-071'in bağlayıcı sürüm günü sırası. Her adım makine kapısıdır;
atlanmaz.

1. **Temiz ağaç ve kapılar.** `bash scripts/guvenlik-kapisi.sh --surum-adayi`
   (exact HEAD için RC fuzz kanıtı, clippy, tam test, koruklar). Ayrıca
   `tani_kalitesi`, `spec_drift`, `islev_egilimi` ve faz matrisi yeşil.
2. **Uyumluluk kaydı.** `docs/deprecation-kayitlari-v1.tsv` ve
   `dil-yuzeyi-v1.tsv` bu sürümde giren/kaldırılan yüzeyi taşır; sürüm notu
   `docs/surumler.md` içinde “Yolda” bölümünden sürüm başlığına taşınır.
3. **Etiket.** `git tag vX.Y.Z` → `surum-adayi`, `kurulum-tatbikati` ve
   `soak` workflow'ları tetiklenir. Etiket sonrası Cargo sürümü
   `X.(Y+1).0-dev` serisine çekilir (`uyumluluk_testi` bunu ister).
4. **Artefakt.** `surum-adayi` artefaktı (`dil`, `dillsp`, `SHA256SUMS`,
   `sbom.spdx.json`, `provenance.intoto.json`, varsa `SHA256SUMS.zee-imza`)
   indirilir; yerel iki-klon özetiyle karşılaştırılır.
5. **Tatbikat.** Üç platformda `kurulum-tatbikati` yeşil; yerelde
   `bash scripts/kur.sh --artefakt <klasör>` → `dil sürüm` →
   `bash scripts/kaldir.sh`.
6. **Duyuru.** Sürüm notu, yayıncı anahtar kimliği ve `SHA256SUMS` içeriği
   yayımlanır; README canlı sayıları `depo_sayilari --yaz` ile yenilenir.

## Kullanıcı kurulumu

```bash
bash scripts/kur.sh --artefakt ~/İndirilenler/zee-surum      # ~/.local/bin
dil sürüm
bash scripts/kaldir.sh
```

Windows: `powershell -File scripts/kur.ps1 -Artefakt <klasör>` ve
`scripts/kaldir.ps1`. Betikler yalnız `SHA256SUMS` ile doğrulanan `dil`/`dillsp`
ikililerini kurar, var olan dosyanın üzerine yazmaz ve yalnız kendi
manifestindeki dosyaları kaldırır. Editör bağı için `editors/README.md`.
