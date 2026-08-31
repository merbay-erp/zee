# ADR-008 — Self-hosting aşamaları ve geçiş kapıları

- **Durum:** kabul (31 Ağustos 2026 — plan bağlayıcı; her aşama kendi
  kapısını geçmeden başlayamaz)
- **Bağlam:** master plan bölüm 12; ADR-001 (Rust bootstrap), bölüm 27
  (reproducible build). Nihai hedef: dilin kendini derlemesi — mirasın
  kendi ayakları üstünde durması.

## Karar: altı aşama, her birine ölçülebilir kapı

| Aşama | İçerik | Başlama kapısı |
|---|---|---|
| **Stage 0** | Rust bootstrap derleyici | ✅ bugün buradayız (v0.2.0) |
| **Stage 1** | Standart kütüphanenin parçaları zee'de yazılır | Birim sistemi + K-016 usability onayı; stdlib RFC'si |
| **Stage 2** | Parser/tür denetçisi bölümleri zee'de | Stage 1'de ≥2 gerçek birim; dilde yapı+liste+Sonuç olgunluğu kanıtı |
| **Stage 3** | Derleyici kendi kaynağını derler | Native backend (ADR-005) YA DA yeterli hızda yorumlama; tam spec |
| **Stage 4** | Ardışık derleyici çıktıları bit-eş doğrulanır | Stage 3 + CI'da çift derleme adımı |
| **Stage 5** | Diverse double compiling (bootstrap güveni) | Stage 4 kararlı; ikinci bağımsız gerçekleme ya da eski-sürüm zinciri |

## İlkeler

1. **Bootstrap asla silinmez.** Rust Stage 0, Stage 5'ten sonra bile
   depoda derlenebilir kalır: güven zincirinin kökü ve öğretim aracıdır.
2. **Golden korpus hakemdir.** Her aşamada yeni derleyici, 32 golden +
   proje kitaplığını bit-eş çıktıyla geçmek zorundadır (determinizm sözü
   aşama geçişlerinin test tanımıdır).
3. **Aşama atlanmaz.** Stage 2'ye stdlib deneyimi olmadan girmek, dil
   eksiklerini derleyici koduna gömer — K-044/K-045 dersleri: eksikler
   önce KÜÇÜK programlarda ucuz yakalanır.
4. **Tanılar Türkçe kalır.** Self-hosting hiçbir aşamada tanı kalitesini
   düşüremez (RFC-0010 geriye gidiş kabul etmez).

## Sonuçlar

- Stage 1'in ön işi görünür oldu: stdlib RFC'si (bölüm 13) sıradaki büyük
  tasarım işidir ve golden korpus genişletmesiyle (SQLite, oyun, paket)
  birlikte planlanmalıdır.
- ADR-005 (native backend) Stage 3'ün kapısında zorunlu değil ama güçlü
  adaydır; karar verisi docs/olcumler.md'de birikiyor (çağrı ~0,6 µs).
