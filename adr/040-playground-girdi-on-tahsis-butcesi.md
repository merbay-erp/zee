# ADR-040 — Playground girdi ön-tahsis bütçesi

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-143, B-056, V1-P1-11

## Bağlam

Playground kaynak metnini derleyicinin ortak 8 MiB sınırına sonunda ulaştırsa
da soru girdisini önce `lines().map(str::to_string).collect()` ile sahipli
satırlar halinde çoğaltıyordu. WASM ABI'nin genel 16 MiB tampon zarfı türün
gerçek kullanım maliyetini ayırmıyor; tarayıcı da UTF-8 byte dizisini JS
heap'inde kurup ardından WASM belleğine ikinci kez kopyalıyordu. Çok sayıda boş
satır byte sınırı altında kalırken binlerce ayrı `String` tahsisine dönüşebilir.

## Karar

1. Merkezî `KaynakSinirlari` profili `PlaygroundSinirlari` görünümünü taşır.
   Kaynak en çok 8 MiB, soru girdisi en çok 1 MiB ve 4.096 satırdır. Program,
   URL, ortam veya host bu değerleri yükseltemez.
2. Native playground çekirdeği kaynak ve soru girdisinin UTF-8 byte boyunu,
   ardından soru satır sayısını `Vec<String>` kurulmadan önce denetler. Sınırda
   değer kabul, bir fazlası `PLAYGROUND SINIR HATASI` ile fail-closed reddir.
3. ABI v3 üç salt-okunur limit dışa aktarımı sunar. `dil_calistir`, kayıtlı
   tamponun exact uzunluğu tür limitini aşıyorsa byte'ları `Vec`e kopyalamadan
   reddeder. UTF-8 doğrulaması ve satır sınırı bundan sonra, derleyiciye
   geçmeden uygulanır.
4. Kanonik tarayıcı hostu limitleri WASM'den okur. UTF-8 byte uzunluğunu tahsis
   yapmadan hesaplar, satırları `split` ile çoğaltmadan sayar, exact WASM
   tamponunu ayırır ve `TextEncoder.encodeInto` ile doğrudan o tampona yazar.
5. Genel ABI tampon sayısı/boyutu K-142/ADR-039 sahiplik zarfı olarak kalır.
   Bu karar daha dar ürün girdisi bütçesidir; aynı sayfadaki JavaScript için
   sandbox veya genel native FFI sözü değildir.

## Reddedilen seçenekler

- **Yalnız ortak kaynak sınırına güvenmek:** soru girdisi derleyici kaynağı
  değildir ve satır koleksiyonu daha önce kurulabilir.
- **Yalnız byte sınırı koymak:** çok sayıda boş/kısa satır, düşük byte sayısına
  rağmen ayrı nesne tahsislerini büyütür.
- **Limitleri HTML içinde yinelemek:** profil değişince host ile compiler'ın
  sessizce ayrışmasına yol açar.
- **Önce `TextEncoder.encode`, sonra ölçmek:** reddedilecek tam byte dizisini JS
  heap'inde zaten tahsis eder ve ikinci kopyayı korur.
- **Sessiz kesmek:** soru yanıtlarının sırasını ve program determinizmini bozar.

## Sonuçlar

- ABI v2'nin kayıtlı pointer/sahiplik güvenliği korunur; zorunlu limit
  dışa aktarımları nedeniyle kanonik host exact ABI v3 ister.
- Kaynak sınırı dil derleyicisinin 8 MiB sözünden geniş değildir. Soru girdisi
  1 MiB/4.096 satırda ayrı ve görünür bir ürün sınırıdır.
- Native sınır/bir-fazlası testleri, mimari allocation-order testi, gerçek
  wasm32 Node hostu ve kaynak+soru kipli `wasm_abi` fuzzer'ı kararı korur. İlk
  K-143 kampanyası 1.709.869 çağrıyı crash/panic olmadan tamamlamıştır.
