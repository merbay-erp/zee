# ADR-027 — Rastgelelik ve sanal saat için tek sürümlü sahip

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-116, B-028, V1-P0-26

## Bağlam

CLI ve playground aynı xorshift ailesini kopya kodla kullanıyor, fakat durum
hazırlama adımları farklı yerlerde yaşıyordu. `ToplayanIo` sanal saati ve
kuyruk fallback'lerini uyguluyor; bu davranışlar testlerde yoğun kullanıldığı
halde isimli bir uyumluluk profiline bağlı değildi. Bir refactor aynı tohumun
çıktısını veya deadline gözlemini sessizce değiştirebilirdi.

Mevcut aralık hesabı `(üst-alt)` işlemini `i64`te yapıyordu; tam tamsayı
uzayında taşma riski vardı. Doğrudan sahte adaptör çağrısı da kuyruktaki
aralık dışı değeri trait sözleşmesine aykırı döndürebiliyordu.

## Karar

Rastgele durum makinesinin tek sahibi `yorumlayici/io_profili.rs` içindeki
`SurumluRastgele` olur. CLI ve playground aynı sahibi kullanır; algoritma,
tohum karışımı ve yansız uçları-dahil eşleme RFC-0023/spec-22'de `zee-io-1`
adıyla normatiftir. Tam `i64` uzayı ara hesaplarda `i128/u128` ile taşmadan
eşlenir.

`ToplayanIo` mevcut public kurulum yüzeyini korur. Rastgele kuyruğu FIFO'dur,
değeri çağrı aralığına kırpar ve boşlukta alt uca döner. Takvim zamanı sabit,
tekdüze an ayrı ve geriye gitmezdir; negatif bekleme ilerletmez. Scheduler'ın
en-yakın-uyanış modeli spec/14'te aynı kalır.

Profil kimliği public sabittir. Kırıcı davranış değişikliği satır bütçesini ya
da snapshot'ı sessizce yükselterek yapılamaz; yeni profil/RFC ister.

## Değişmezler

1. CLI ve playground ayrı PRNG gerçekleyemez.
2. Aynı profil+tohum+çağrı dizisi aynı sayıları üretir.
3. Rastgele sonuç her zaman geçerli aralıktadır; tam i64 uzayı panic üretmez.
4. Hermetik tekdüze saat geriye gitmez; takvim beklemeyle ilerlemez.
5. Güvenlik belirteçleri bu PRNG'ye bağlanmaz.
6. IO trace/replay gözlenen sonuçları taşır ve profil algoritmasından bağımsızdır.

## Sonuçlar

- B-028 ve V1-P0-26 kapanır.
- Beş conformance testi profil kimliği/vektörü, tam i64, FIFO+fallback,
  sanal saat ve bütünleşik sahte IO/playground davranışını korur.
- Runtime kökü yalnız modül/re-export taşır; profil çekirdeği 80 satırlık
  mimari bütçeye bağlıdır.
- Gerçek CLI başlangıç tohumunu sistem zamanından alır; tekrar üretim için
  RFC-0022 `dil iz kaydet/oynat` kullanılır.
