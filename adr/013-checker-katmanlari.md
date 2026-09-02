# ADR-013 — Checker semantik katmanları ve tek sahiplik

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-100, K-121, B-006, B-007, V1-P0-01/11
- **Revizyon:** 2 Eylül 2026 — K-121 sıra-bağımsız çağrı kısıtı katmanı

## Bağlam

ADR-012 checker'ın cümle, ifade ve çağrı handler'larını fiziksel olarak
ayırdı; ancak `cozumleyici.rs` hâlâ tür modelini, bağlam durumunu, sembol
çözümünü, akış daraltmasını, açık sözleşmeleri, etki geçişini ve dönüş
birleştirmeyi aynı kök modülde sahipleniyordu. Bu durum bir tanının hangi
semantik faza ait olduğunu belirsizleştiriyor ve B-007/B-010 gibi sıra
bağımsız çıkarım ile semantic ID işlerini gereksizce birbirine bağlıyordu.

## Karar

Checker aşağıdaki tek-sahipli katmanlara ayrılır:

| Katman | Sorumluluk |
|---|---|
| `turler` | `Tur`/kapsayıcı türler, tür yazımı çözümü ve temel uzlaşma |
| `baglam` | işlem imzaları ve bir denetim geçişinin açık durumu |
| `sembol` | ad/alan çözümü ve sözcüksel kapsam yaşamı |
| `akis` | Seçenek/Sonuç daraltması, gezme ve görev akış değişmezleri |
| `cagri` | işlem çağrısı, çıkarımlı imza ve özyineleme uzlaştırması |
| `cikarim` | yerel çağrı kısıtı keşfi ve sıra-bağımsız tür birleşimi |
| `sozlesme` | public parametre/dönüş sözleşmesi doğrulaması |
| `etki` | uygulama/web etkisi ve intrinsic yetkinlik metadatası geçişi |
| `donus` | kesin sonlanma, dönüş dalları ve Seçenek/Sonuç birleşimi |
| `cumle` / `ifade` | AST handler'ları; yukarıdaki hizmetleri orkestre eder |

Kök `cozumleyici.rs` yalnız geçiş sırasını ve public API yeniden dışa
aktarımını taşır. Denetim sırası:

```text
etki/yetkinlik
  → tür yazımları
  → kopya AST'de yerel çağrı kısıtı keşfi
  → açık işlem sözleşmeleri
  → nihai imzayla cümle/ifade + sembol/akış/çağrı + HIR
  → dönüş/control-flow kanıtı
```

İç katmanlar `pub(super)` görünürdür. Var olan
`dil::cozumleyici::{Tur, VeriTuru, SozlukDegerTuru, ad_cozumle}` yüzeyi
yeniden dışa aktarılır ve kırılmaz.

## Değişmezler

1. Aynı kaynak aynı tanı kodu, mesaj, konum ve öneriyi üretir.
2. Etki/yetkinlik denetimi tür geçişinden önce fail-closed kalır.
3. Public sözleşme çağrı beklemeden; çıkarımlı yerel işlem bütün
   erişilebilir çağrı kısıtları birleştirildikten sonra denetlenir.
4. Akış daraltması cümle katmanının açık bağlamıdır; tür enum'una gizlenmez.
5. Dönüş birleşimi çağrı katmanından bağımsız tek control-flow sahibidir.
6. Katman dosya bütçesini aşarsa köke geri taşınmaz; yeni sorumluluk sınırı
   ADR-012/013 ve faz rehberiyle birlikte açılır.

## Sonuçlar

- Checker kökü 963 satırdan 143 satıra indi.
- Eski bağımsız `eylem.rs`, checker'ın `etki` katmanına taşındı; davranışı ve
  tanıları değişmedi.
- Beş kaynak-mimari test katman sahipliğini, bütçeleri ve public API'yi korur.
- B-010 semantic ID, K-101/ADR-014 ile bu katmanlar üstünde tamamlandı.
  B-007, K-121'de ayrı `cikarim` katmanı ve mimari bütçesiyle kapandı.
- K-121'in sıra-bağımsız yerel çıkarım semantiği RFC-0006 ile spec/04 ve
  spec/10'da normatifleşti; diğer katman ayrımları kaynak yüzeyini değiştirmez.
