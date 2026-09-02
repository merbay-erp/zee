# WASM C ABI v2 host sözleşmesi

Bu belge K-142/ADR-039 ile gelen elle yazılmış playground köprüsünün işletim
sözleşmesidir. Dilin gelecekteki genel C FFI tasarımı değildir; bootstrap
compiler'ın tarayıcıdaki tek güvenli giriş/çıkış yüzeyidir.

## Dışa aktarımlar

| İşlev | Sonuç | Sözleşme |
|---|---|---|
| `dil_abi_surumu()` | `u32` | Exact değer `2`; host farklı değerde durur. |
| `dil_bellek_ayir(n)` | pointer | `n > 0` kayıtlı girdi tamponu; reddedilirse null. |
| `dil_calistir(kp, kn, gp, gn, tohum)` | pointer | Kayıtlı girdileri kopyalar; uzunluk-önekli sahipli sonuç veya tahsis edilemezse null. |
| `dil_sonuc_tamponu_uzunlugu(p)` | byte sayısı | Yalnız canlı sonuç kaydının başlık dahil exact boyu; diğer her şeyde `0`. |
| `dil_bellek_birak(p, n)` | `u32` | Exact canlı kayıt bırakıldıysa `1`, reddedildiyse `0`; `(null, 0)` başarılı no-op. |

WASM32'de pointer ve `usize` i32, tohum i64'tür. JavaScript i64 için `BigInt`
kullanır ve dönen pointer'ı doğrulamadan önce `>>> 0` ile unsigned linear
memory ofsetine çevirir.

## Zorunlu çağrı sırası

1. Modülü yükle; `memory` export'unu ve `dil_abi_surumu() === 2` sonucunu
   doğrula.
2. Her boş olmayan UTF-8 metin için exact byte boyunda `dil_bellek_ayir`
   çağır. Null ise dur; ancak sonra linear memory'ye yaz.
3. Boş metni `(0, 0)`, dolu metni kayıtlı `(başlangıç pointer'ı, exact boy)`
   olarak `dil_calistir`e ver. Kaynak ve soru girdisi ayrı kayıttır.
4. Sonuç null değilse kayıtlı toplamı sorgula. Toplam en az dört ve linear
   memory içinde olmalıdır. İlk dört little-endian byte `toplam - 4` olmalı;
   kalan byte'lar strict UTF-8 çözülmelidir.
5. Kaynak, girdi ve sonuç tamponlarının her birini kendi exact boyuyla bir kez
   bırak. `0` sonucu host/ABI sözleşme ihlalidir; sessizce geçilmez.

Depodaki `playground/sablon.html` bu sıranın kanonik host uygulamasıdır. Bütün
temizlik `finally` içinde yapılır; derleme veya ABI hatası girdi tamponu
sızdıramaz.

## Red kararları

Şunların hiçbiri Rust dilimine çevrilmez:

- null pointer + sıfır olmayan uzunluk;
- sıfır uzunluk + null olmayan pointer;
- bu ABI'nin ayırmadığı, daha önce bırakılmış veya tamponun içini gösteren
  pointer;
- kayıtlı boyla exact eşleşmeyen uzunluk, `usize::MAX` dahil;
- sonuç tamponunu girdi diye yeniden kullanma;
- kaynak veya soru girdisinde geçersiz UTF-8.

İlk beş sınıf `WASM ABI HATASI: kaynak/girdi: ...`, UTF-8 sınıfı
`WASM ABI HATASI: ... geçerli UTF-8 değil` gövdeli normal sonuç tamponudur.
Hata tamponu da sorgulanıp exact bırakılır. Sonuç tahsisi dahi yapılamıyorsa
tek istisna null sonuçtur.

## Sahiplik ve sınırlar

Tampon pointer'ı yalnız başarılı tahsis ile başarılı exact bırakma arasındaki
sürede canlıdır. Yanlış uzunluk sahipliği düşürmez; ardından doğru bırakma
yapılabilir. İkinci bırakma `0` olur. Sonuç girdiye çevrilemez; girdi de sonuç
boyu sorgusundan geçmez.

Tek kayıt en çok 16 MiB + dört byte, canlı kayıt sayısı sekiz ve kayıtlı toplam
64 MiB'tır. Tahsis `try_reserve_exact` ile denenir; kapasite taşması null olur.
Bu zarf allocator güvenliğini korur. Kaynak ile soru girdisini kendi türünde ve
satır listesi kurulmadan önce daha dar sınırlamak B-056/K-143 işidir.

WASM linear memory hosta açıktır. Bu sözleşme aynı sayfadaki kötü niyetli
JavaScript'i sandbox'lamaz; WebAssembly çağrısı sürerken hostun tamponu eşzamanlı
değiştirmediği, paylaşımlı belleğin açılmadığı senkron kullanım içindir.

## Doğrulama

```bash
cd compiler
cargo test --locked --test playground_testi
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo rustc --locked --release --target wasm32-unknown-unknown \
  --lib --crate-type cdylib -- -C link-arg=-zstack-size=16777216
node ../scripts/wasm-abi-denetle.mjs \
  target/wasm32-unknown-unknown/release/dil.wasm
cargo +nightly-2026-08-31 fuzz run wasm_abi fuzz/corpus/wasm_abi -- \
  -dict=fuzz/dictionaries/zee.dict -max_len=4097 -timeout=5
```

İlk K-142 kampanyası 1.745.134 çağrı dizisini 61 saniyede crash, panic veya
invariant ihlali olmadan tamamladı. Native testler gerçek kayıtlı round-trip'in
yanında null/kayıt dışı/iç pointer, `usize::MAX`, yanlış boy, invalid UTF-8,
yanlış+çift bırakma ve 64 bozuk/geçerli ardışık çağrıyı adlarıyla korur. Gerçek
`wasm32-unknown-unknown` ikilisi ayrıca Node hostundan aynı v2 akışıyla
çalıştırılır.
