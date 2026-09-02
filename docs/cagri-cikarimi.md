# Yerel çağrı çıkarımı

Bu belge K-121/B-007'nin bakım sözleşmesidir. Public birim/paket işlemleri
açık ve monomorfik imza taşır; buradaki iki faz yalnız ana kaynağın
progressive-disclosure biçimindeki `<ad> al` işlemlerine uygulanır.

## Kapatılan sıra etkisi

Eski tek geçişte ilk çağrının sonucu hemen HIR'a yazılıyordu. Daha sonraki
Ondalık çağrı imzayı genişletse bile önceki TamSayı sonuç yeniden
türlenmiyordu. Bu nedenle aynı iki çağrının yerini değiştirmek sonraki bir
ondalık işlemi kabul veya ret ettirebiliyordu.

K-121 hattı:

```text
parsed AST
  → kopya AST'de erişilebilir ana/test çağrılarını keşfet
  → IslemId + parametre konumu başına kısıtları birleştir
  → asıl AST'yi nihai imzalarla denetle
  → tek ve tutarlı typed HIR üret
```

Keşif geçişi kullanıcı tanısı üretmez. Bir cümlede eski dar sonucun yol
açtığı hata, aynı iç bloktaki veya sonraki bağımsız çağrı kısıtını gizlemez;
keşif modu hatayı her blok derinliğinde yutup kısıt toplamaya devam eder.
Tanı, semantic bağ ve HIR yalnız nihai imzayı kullanan asıl geçişin ürünüdür.

## Birleşim kuralları

Her parametre konumu birbirinden bağımsız birleşir:

- aynı tür + aynı tür → aynı tür;
- `TamSayı` + `Ondalık` → `Ondalık`;
- aynı sayısal genişleme `Liste`, `Sözlük`, `Seçenek` ve `Sonuç`
  kapsayıcılarının içinde de geçerlidir;
- başka bir tür çifti birleşmez ve asıl geçişte T017 olur;
- parametre sayısı farkı T015'tir.

Bu birleşim değişmeli, birleşmeli ve idempotenttir; dolayısıyla kaynak
çağrı sırası sonucu etkileyemez. Açık public imza bu kafese katılmaz ve
çağrı tarafından terfi ettirilemez.

## Özyineleme ve erişilebilirlik

İç içe yerel çağrılarda nihai parametreler çağrı grafiği boyunca yayılır;
bir üst işlemin dönüşü alt işlemin nihai sonucuyla asıl geçişte yeniden
kurulur. Özyineleme için T035 temel-durum-önce kuralı ve T018 dönüş birleşimi
değişmez. Hiç erişilemeyen yerel çıkarımlı işlem progressive-disclosure
sınırında kalır; public işlemler çağrılmasa da denetlenir.

## Mimari sahiplik ve kanıt

- `compiler/src/cozumleyici/cikarim.rs`: kopya-AST keşfi ve tür birleşimi;
- `compiler/src/cozumleyici/cagri.rs`: nihai imzayla gövde/çağrı doğrulaması;
- `compiler/tests/cagri_cikarimi_testi.rs`: dar→geniş ve geniş→dar skaler,
  liste, çok parametre, iç içe çağrı grafiği, iç blok kurtarması ve iki yönlü
  T017 kanıtı;
- `compiler/tests/mimari_sinir_testi.rs`: `cikarim` sahibini ve 120 satırlık
  bütçesini korur.

Bu altı davranış regresyonuyla depo toplamı 481 testtir; B-007 ve V1-P0-01
kapalıdır.
