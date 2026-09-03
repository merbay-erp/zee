# `zee-io-1` deterministik IO profili

Bu profil, “aynı program + aynı IO dünyası = aynı sonuç” sözündeki tohum,
saat ve sahte dünya davranışını isimlendirir. K-116/RFC-0023/ADR-027 ile
`zee-io-1` etkin profildir.

## Nerede kullanılır?

- Playground'daki görünür **Tohum** alanı aynı tohumla aynı rastgele diziyi
  verir.
- `dil dene` ve Rust entegrasyon testleri `ToplayanIo` ile sabit takvim,
  sanal tekdüze saat, FIFO girdiler ve sahte dosya/ağ kullanır.
- Gerçek CLI aynı sürümlü PRNG çekirdeğini kullanır fakat başlangıç tohumunu
  sistem zamanından alır. Gerçek koşuyu aynen tekrarlamak için
  [`dil iz kaydet/oynat`](io-izi.md) kullanılır.

## Profilin verdiği söz

Tohum 7 ile `[1,100]` aralığındaki ilk altı sayı her platformda şöyledir:

```text
47, 29, 69, 71, 26, 60
```

Aralık iki ucu da kapsar ve tam 64 bit tamsayı uzayında taşmaz. Bu üreteç
oyun, örnek ve simülasyon içindir; parola, oturum veya CSRF belirteci üretmez.
Güvenlik amaçlı rastgelelik işletim sisteminden gelir.

Hermetik tekdüze saat 0'dan başlar, geri gitmez ve `bekle` kadar ilerler.
Takvim saati bundan ayrıdır; varsayılan olarak 31 Ağustos 2026 14:30 UTC'de
sabittir. Eşzamanlı 2 ve 1 saniyelik beklemeler ortak saati 3 değil 2 saniyeye
taşır.

## `ToplayanIo` kısa sözleşmesi

- Girdi ve hazırlanmış rastgele değerler eklenme sırasıyla tüketilir.
- Rastgele kuyruk boşsa aralığın alt ucu kullanılır; dışarıdaki değer aralığa
  kırpılır.
- `sor` istemi çıktıya ekler; dosya yazımı her satıra LF ekler.
- Sahte dosya yolu ve HTTP URL'si birebir eşleşir.
- Atomik taşıma var olan hedefi ezmez; silme bulunmayanda sıfırdır; dizin
  listesi sıralı doğrudan dosyaları ve SHA-256 exact içeriği kullanır.
- Bildirilmemiş sensör kapalıdır; ışık değişimi normal çıktı izine girer.
- Negatif bekleme zamanı geriye götürmez; enjekte edilen eski an yok sayılır.

Algoritma veya bu gözlemler kırıcı biçimde değişecekse `zee-io-1` yerinde
değiştirilmez; yeni profil adı ve geçiş belgesi açılır.
