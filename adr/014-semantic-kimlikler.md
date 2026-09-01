# ADR-014 — Yapı, işlem ve sembol için açık semantic identity

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-101, B-010, V1-P0-12

## Bağlam

Checker kullanıcı yapılarını `Tur::Yapi(usize)` ile doğrudan
`Program.yapilar` vektörüne bağlıyordu. İşlem imzaları ve özyineleme kaydı
kaynak adını, yerel değişken çözümü ise yalnız `String`i identity olarak
kullanıyordu. Bu temsil çalışan bootstrap için yeterli olsa da modül
birleştirme, incremental derleme, cache/serialization ve ilerideki typed HIR
için üç farklı kavramı karıştırıyordu:

1. kullanıcının yazdığı ve tanıda gösterilen ad;
2. koleksiyondaki fiziksel depolama konumu;
3. derleyicinin bağladığı semantik varlık.

## Karar

`compiler/src/kimlik.rs` üç tür güvenli newtype tanımlar:

- `YapiId`: yapı türü ve yapı oluşturma bağının kimliği;
- `IslemId`: çağrı, imza ve özyineleme kaydının kimliği;
- `SymbolId`: kapsam + tanım sırasından oluşan yerel sembol kimliği.

Checker, yapı ve işlem adlarını sıralı katalog üzerinden kimliklendirir.
`YapiId` ayrıca kimlik→vektör-konumu dizininden çözülür; kimliğin sayısal
değeri vektör indeksi olarak kullanılamaz. Böylece yapı tanımlarının fiziksel
sırası değişse de aynı ad aynı derleme birimi kimliğini alır. İşlem kimliği de
`HashMap` iterasyonundan ve çağrı sırasından bağımsızdır.

Sembol tablosu `ad → (SymbolId, Tur)` kaydı taşır. Yeniden atama aynı sembol
kimliğini korur; blokta doğan yeni tanım yeni kimlik alır. Parser'ın ürettiği
`Degisken`, `YeniYapi` ve `IslemCagrisi` düğümlerindeki kimlik alanları
başlangıçta boştur; checker başarılı bağlamada bunları doldurur. Kaynak adı
tanı ve mevcut runtime uyumluluğu için yanında korunur.

## Değişmezler

1. `Tur::Yapi` yalnız `YapiId` taşır; checker handler'ı bu değeri vektör
   indeksi olarak kullanamaz.
2. İmza ve özyineleme kayıtları adla değil `IslemId` ile anahtarlanır.
3. Çözülmüş her değişken kaynak adıyla birlikte `SymbolId` taşır.
4. Kimlik ataması `HashMap` rastgeleliğinden ve yapı depolama sırasından
   bağımsızdır.
5. Kimlikler bir derleme birimi içindeki semantic identity'dir; kalıcı paket,
   ABI ya da ağ protokolü kimliği değildir.
6. Faz türleri B-018/K-102 ile görünürdür. Runtime'ın kaynak adını bırakıp
   yalnız ID/HIR tüketmesi B-019/K-104 ile sonradan tamamlandı; bu ADR'nin
   kendi kapanış kanıtı semantic kimlik temelidir.

## Sonuçlar

- Çıplak `usize`, yapı semantic identity'si olmaktan çıktı.
- AST checker sonrası yapı, işlem ve sembol bağlarını açıkça taşır.
- Üç davranış ve bir kaynak-mimari testi sıralama bağımsızlığını, bağlamayı ve
  indeks gerilemesini korur.
- Rust embedding yüzeyindeki `Tur::Yapi` payload'ı `usize` yerine `YapiId`
  olur; zee kaynak dili ve runtime çıktısı değişmez.
- Yeni normatif dil spec'i gerekmez; karar derleyici iç temsilidir.
