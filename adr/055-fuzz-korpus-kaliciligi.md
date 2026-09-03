# ADR-055 — Fuzz korpus öğreniminin kalıcılığı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-156, B-066

## Bağlam

Gecelik dört libFuzzer hedefi yeni coverage üreten girdileri kendi korpus
klasörüne yazıyor, workflow bu klasörü yalnız GitHub cache ile sonraki koşuya
taşıyordu. Cache kalıcı kanıt değildir; tahliye edildiğinde crash üretmeyen ama
yeni yol açan öğrenim kaybolabilirdi. Yalnız başarısız koşudaki crash dosyasını
artefakt yapmak coverage korpusunun soy ağacını korumuyordu.

## Karar

1. Her hedef, fuzz adımı başarılı, başarısız veya önceki bir adım kesilmiş olsa
   da `if: always()` ile koşu sonundaki korpusunu ayrı artefakta yükler.
2. Artefakt 90 gün saklanır ve hedef, korpus, kaynak commit, workflow run/attempt,
   sabit toolchain+cargo-fuzz ile her seed'in göreli yolu ve SHA-256 özetini
   taşıyan `zee-fuzz-corpus-artifact-1` manifesti içerir.
3. Cache yalnız hızlandırmadır. İndirilebilir artefakt cache'ten bağımsız ikinci
   kopyadır; crash girdileri ayrıca kendi 90 günlük artefaktında kalır.
4. `scripts/fuzz-korpus-artefakti-dogrula.sh`, hedef allowlist'ini, şemayı,
   göreli yolları, seed SHA-256 özetlerini ve manifest↔ağaç birebirliğini Linux
   ile macOS'ta doğrular.
5. Coverage korpusu otomatik olarak repoya yazılmaz. Değerli girdiler artefakt
   doğrulaması, `cargo fuzz cmin` küçültmesi, stable replay/regression testi ve
   insan incelemesinden sonra anlamlı adla kaynak korpusuna alınır.

## Reddedilen seçenekler

- **Yalnız cache:** saklama süresi ve tahliye davranışı kanıt zinciri değildir.
- **Yalnız crash artefaktı:** crash üretmeyen coverage artışlarını kaybeder.
- **Her gece otomatik repo commit'i:** review edilmemiş, yinelenen ve anlamsız
  binary girdilerle kaynak geçmişini kirletir.
- **Hashsiz zip:** indirilen girdinin hangi koşudan geldiğini ve değiştirilip
  değiştirilmediğini doğrulatmaz.

## Sonuçlar

- Cache silinse de son 90 günlük hedef-bazlı fuzz öğrenimi indirilebilir ve
  kaynak commit'ine bağlanabilir.
- Kalıcı repo korpusu hâlâ küçük, adlandırılmış ve review'lidir; CI artefaktı
  otomatik olarak dil sözleşmesine dönüşmez.
- Uzun RC fuzz kampanyası, sanitizer ve Miri K-157'nin ayrı açık kapısıdır.
