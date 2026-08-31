# RFC-0012 — FFI ve Tehlikeli Sınır

- **Durum:** taslak (tasarım — gerçekleme Faz 4/5)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** master plan bölüm 19; anti-örnek A10
- **Gerçekleme:** yok

## Özet

Dış dünya köprüsü C ABI'dir; güvensiz sınır görünür Türkçe kelimeyle
işaretlenir: **tehlikeli**. Sıradan kullanıcı FFI'yı hiç görmez — paketler
sarar (A10 ilkesi: İngilizce API yüzeyi Türkçe sarmalayıcıyla sunulur).

## 1. Katman modeli

1. **Kullanıcı katmanı:** yalnız Türkçe stdlib/paket API'leri. FFI yok.
2. **Paket yazarı katmanı:** `tehlikeli` bloklarında dış bildirimler; paket
   bunları güvenli Türkçe işlemlerle sarar ve YALNIZ sarılmış yüzeyi dışa açar.
3. **Derleyici katmanı:** C ABI çağrı üretimi (Faz 4 native backend'le gelir;
   yorumlayıcı döneminde FFI yoktur — bilinçli erteleme).

## 2. Yüzey taslağı

```
tehlikeli
    dış işlem c_toplam
        adı "add" olsun
        kütüphanesi "libhesap" olsun
        girdileri TamSayı ve TamSayı olsun
        çıktısı TamSayı olsun

işlem güvenli toplam
    a al
    b al
    sonucu a ve b ile c_toplam olsun    # tehlikeli çağrı yalnız burada
    sonucu döndür
```

Kurallar (öneri):

- `tehlikeli` blok dışında dış bildirim ve çağrı derleme hatasıdır; blok,
  koddaki görünür "burada dikkat" işaretidir (bölüm 18: unsafe açık sınır).
- Dış adlar (`"add"`) metin sabitidir — RFC-0002 tanımlayıcı kuralları dış
  ada uygulanMAZ (İngilizce ad kaynakta yalnız tırnak içinde yaşar).
- Tür köprüsü v1: TamSayı↔int64, GerçekSayı↔double (RFC-0013'e bağlı),
  Metin↔UTF-8 salt-okunur görünüm. Sahiplik GEÇMEZ: dilin değerleri C'ye
  ödünç verilir; C'den dönen bellek sahipliği paket yazarının sorumluluğunda
  ve `tehlikeli` içinde kalır.
- Çökme sınırı: dış çağrıdaki çökme dilin güvence alanı dışındadır; tanı
  üretilemez — bu yüzden 1. katman kullanıcısına asla doğrudan FFI verilmez.

## 3. Güvenlik bağları (bölüm 18)

- `tehlikeli` içeren paket, registry'de görünür şekilde işaretlenir; kurulum
  onayı ister (çocuk/öğretmen modunda varsayılan: reddet).
- Native ikili taşıyan paketlerde OS/mimari metadata + checksum zorunlu.

## 4. Açık sorular

1. `tehlikeli` mi `güvensiz` mi — kelime usability'ye gider ("tehlikeli"
   çocuk için daha caydırıcı, tercih bu yönde).
2. Callback'ler (C→dil çağrısı) v1'de yok; olacaksa yeniden giriş kuralları.
3. WASM hedefinde FFI'nın karşılığı (host fonksiyon ithali) — playground
   sandbox'ıyla ilişkisi (bölüm 15: client-side WASM).
4. Rust/C++ üreteç-adaptörleri (bölüm 19) — ayrı araç RFC'si.

## Dört soru süzgeci

Doğal ✓ ("tehlikeli" kendini açıklar) · Deterministik ✓ (sınır sözdizimsel,
kaçış yok) · Öğrenilebilir ✓ (kullanıcı katmanı FFI'yı hiç görmez) ·
Savunulabilir ✓ (C ABI evrensel köprü; işaretli paket + onay zinciri).

## Korpus etkisi

Yok (korpus 1. katmandadır). Paket yazarı belgelerine örnek eklenir.
