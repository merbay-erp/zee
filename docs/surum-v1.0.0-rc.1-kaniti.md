# v1.0.0-rc.1 — sürüm adayı kanıtı (19 Ekim 2026)

**K-322.** Proje sahibi kararıyla yayımlanan V1 sürüm adayı; GitHub'da
**ön-sürüm** ([yayın](https://github.com/merbay-erp/zee/releases/tag/v1.0.0-rc.1)).
V1 yayını değildir; açık kapılar [sürüm notunda](surumler.md) listelidir.

| Alan | Değer |
|---|---|
| Kaynak S | `aaf9aa883beb8d72cd26e27135f37f852879b8f9` (ağaç `547b3b7cf89a2ae5362b46c0a8fe4b0d82d389f1`) |
| Etiket | `v1.0.0-rc.1` (açıklamalı, nesne `2c042e6789ec86d6b3ad95553e7cd6ddddcab717`) |
| Derleyici kimliği | `dil 1.0.0-dev (aaf9aa883beb, 2026-10-19, development)` |
| CI (`dil`) | koşu 39236514543 — Linux, macOS, Windows yeşil |
| Yayın (`surum-adayi`) | koşu 39245193645 — üç platform iki temiz klon eş hash + imza; `yayinla` ön-sürüm |

## Sürüm adayı güvenlik kapısı

`bash scripts/guvenlik-kapisi.sh --surum-adayi --rc-kanit <paket> --rc-kanit-sha256 <özet>`
temiz S üzerinde geçti ([ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md),
[RC kanıtı](rc-kaniti.md)):

- Dış RC paketi manifesti `bfe454fa11080bb99c726b2a02830e8bd187a267747138b3f8e780297674ca16`
  (`zee-rc-evidence-1`, kaynak sahibinin Mac'inde kanonik `rc-kampanyasi.py`, macOS
  arm64, nightly-2026-08-31, cargo-fuzz 0.13.2): dört hedef (`lexer_parser`,
  `morfoloji`, `http_istegi`, `wasm_abi`) açık ASan ile en az 1800 sn, iki Miri
  grubu (`ondalik`, `zaman`). Durum `validated-integrity-and-policy`; köken
  "owned-local-reviewed-not-authenticated".
- cargo-deny (compiler + fuzz), fuzz derlemesi, clippy `-D warnings`, tam test
  paketi: 1022 geçti, 0 başarısız, 27 yok sayılan.
- Conformance, semantic regresyon ve core freeze korukları.

Paket ve kapı günlüğü depoya kopyalanmaz; proje sahibinin makinesinde
`~/zee-rc-kanit/v1.0.0-rc.1/` altında saklanır.

## Yayımlanan paketler

`ZEE-PAKETLERI.sha256`:

| Paket | SHA-256 |
|---|---|
| `zee-v1.0.0-rc.1-linux-x86_64.tar.gz` | `325ed8a55c96c17d0636557a10825e473e9d760c8508ec30d8a9f8fab3276db5` |
| `zee-v1.0.0-rc.1-macos-arm64.tar.gz` | `08e83d9a36040dd1971032e8105dfb5bea9157051bd80fc4eb79cafb1c1996c0` |
| `zee-v1.0.0-rc.1-windows-x86_64.tar.gz` | `9726e7c3506c088a0b25b2dfbcc677132f45f8a5a19502e0f36ab9115ff26c41` |

İkililer:

| Platform | `dil` | `dillsp` |
|---|---|---|
| linux-x86_64 | `a0325e5dbc854c97edfc149dcd82458763b85bf423c10484f2fbde29b5eb1031` | `88f18b3ef474732b34b884d119f9136c5cfa5e19c9d6464404771d352c1e5dd2` |
| macos-arm64 | `2862cb63013e7f36fac224905836d4db12cc2e5edc8336933623fd58e933b024` | `421a90d0d8d3b36cbf9f2c829ee6fc9ee8cc5bfb856b230ad1d45b0fa9d6658b` |
| windows-x86_64 | `3d90ac0234bb8165282d297bc6f4ff0436f9e00b773cfa57d25b60c9b61ccb6b` | `255c860f0c45b7c5580c2131f65a5490c75cdf592ba741cbbd07a6d68a974bea` |

## Son kullanıcı yolu

Yayından indirilen paketlerle: `ZEE-PAKETLERI.sha256` üç pakette OK; üç
platformun imzası `imza-dogrula.sh` ile resmî yayıncıya karşı geçerli. macOS
paketi tarayıcı karantinasıyla, temiz oturumda (PATH yalnız sistem) kuruldu;
`dil sürüm`, `dil çalıştır merhaba.dil` → `Dünyaya merhaba`, kaldırma geçti.
