# RFC-0022 — Deterministik IO trace/replay

- **Durum:** geçici kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıtlar:** K-115, B-027, V1-P0-25
- **Gerçekleme:** `compiler/src/yorumlayici/io_izi.rs`, `dil iz`,
  `io_izi_testi.rs`, `io_izi_cli_testi.rs`

## Özet

Zee'nin dış dünya sınırındaki her gözlenebilir çağrı; işlemi, argümanları,
sonucu ve sırasıyla sürümlü bir izde tutulabilir. Aynı program bu izle dosya,
ağ, klavye, saat, rastgelelik, sensör veya web durumuna erişmeden yeniden
oynatılır. Replay bir simülasyon girdisi değil, birebir protokol kanıtıdır:
sıra ya da argüman farkında durur.

## 1. Kapsam (bağlayıcı)

Şema-1 `GirdiCikti` sözleşmesinin bütün yöntemlerini kapsar:

- ekran/istem, program argümanları;
- rastgelelik, takvim zamanı, tekdüze an ve bekleme;
- dosya okuma/yazma;
- HTTP istemcisi, sunucu isteği/yanıtı/yönlendirme/çerez;
- rota güvenliği, CSRF, oturum ve parola doğrulama sonucu;
- eylem transaction başlatma/tamamlama/geri alma;
- sensör okuma ve ışık eyleyicisi.

Birim yükleme derleme fazındadır ve bu yürütme izi kapsamında değildir.
Kaynak program iz içine gömülmez; replay'e ayrıca verilir. Kaynakta IO izini
değiştirmeyen bir düzenleme kabul edilebilir, gözlenebilir çağrı farkı ise
protokol uyuşmazlığıdır.

## 2. Kanonik byte biçimi (bağlayıcı)

İlk satır tam olarak şöyledir:

```text
zee-io-izi<TAB>1<LF>
```

Her sonraki satır:

```text
sıra<TAB>işlem<TAB>argüman_sayısı<TAB>sonuç_sayısı<TAB>alan...<LF>
```

Kurallar:

1. Sıra 1'den başlar, kesintisiz artar ve baştaki sıfırsız onluktur.
2. İşlem şema-1'in kapalı sözlüğündeki küçük ASCII addır.
3. Sayılar baştaki sıfırsız onluk biçimdedir.
4. Her argüman/sonuç alanı UTF-8 metnin küçük harfli hex kodlamasıdır; bu
   kodlama gizleme veya şifreleme değildir.
5. Alan sayıları satırdaki alanlarla birebir eşleşir. Son satır LF ile biter.
6. İşlem-özel tür/etiket şeması dosya okunurken doğrulanır. Örneğin boolean
   `0/1`, seçenek `yok` veya `var,<değer>`, sonuç `ok,...`/`hata,...` taşır.
7. Şema-1 üst sınırı 64 MiB, 100.000 olay ve olay başına 4.096 alandır.

Aynı olay dizisinin tek bir kanonik byte yazımı vardır. İşlem sözlüğü, alan
sırası veya anlamı kırıcı biçimde değişecekse başlık sürümü artırılır; eski
okuyucu bilinmeyen sürümü reddeder.

## 3. Kayıt ve replay (bağlayıcı)

Kayıt sarmalayıcısı önce gerçek IO çağrısını tamamlar, sonra argüman ve
sonucuyla olayı ekler. Yarım kalan host çağrısı tamamlanmış olay sayılmaz.
Dosyaya aktarım atomiktir.

Replay sıradaki olay dışında arama yapmaz. İşlem ve argümanlar eşleşirse kayıtlı
sonuç döner; aksi durumda ilk uyuşmazlık kalıcı hata olur. Dönüş tipi doğrudan
hata taşıyamayan IO yöntemleri güvenli nötr değer döndürür, fakat koşu sonunda
uyuşmazlık yine başarısızdır. Başarılı replay bütün olayları tüketmek
zorundadır.

Replay dış etkileri yeniden uygulamaz. Dosya yazma, HTTP, çerez, oturum,
transaction ve donanım çağrıları yalnız beklenen protokolle karşılaştırılır.
Bu nedenle replay bir yedekten geri yükleme ya da yetki baypası değildir.

## 4. Güvenlik ve gizlilik (bağlayıcı)

İz açıkça istenmeden üretilmez ve `*.zee-io-izi` Git dışında tutulur. Girdi,
dosya/ağ gövdesi, argüman, çerez ve token içerebilir; kullanıcı izi özel veri
gibi saklamalıdır. Hex alanlar düz veridir.

Parola doğrulamasındaki parola ve PHC özeti ham veya geri çevrilebilir biçimde
kaydedilmez. Ayrı SHA-256 parmak izleri yalnız replay çağrısının aynı gizli
girdileri aldığını doğrular. İz paylaşılmadan önce diğer uygulama sırları için
ayrıca temizlenmelidir.

## 5. CLI (tanımlı)

```sh
dil iz kaydet kosu.zee-io-izi program.dil argümanlar...
dil iz oynat kosu.zee-io-izi program.dil
```

Replay program argümanlarını izden alır; komuta yeni argüman eklenemez.
Kullanıcı çıktısı ancak iz eksiksiz eşleştiğinde basılır. İz kaynak programın
üzerine yazılamaz.

## 6. Açık ardıl

B-028; gerçek ve hermetik adaptörlerde rastgele tohum, takvim zamanı,
tekdüze saat ve bekleme ilerlemesinin sürüm semantiğini ayrıca bağlar. Bu RFC
şema-1'de gözlenen sonuç ve sırayı dondurur; görünmeyen tohum algoritmasını
henüz taşınabilir dil ABI'si ilan etmez.

## Dört soru süzgeci

Doğal ✓ (Türkçe CLI) · Deterministik ✓ (tek sıra/tek byte biçimi) ·
Öğrenilebilir ✓ (koşu yeniden görülebilir) · Savunulabilir ✓ (bütçeli,
fail-closed, dış etkisiz replay).
