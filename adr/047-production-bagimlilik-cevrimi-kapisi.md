# ADR-047 — Production bağımlılık çevrimi kapısı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-150, B-058
- **Revizyon:** 3 Eylül 2026 — K-160/ADR-059 C001'i ve son SCC'yi kaldırdı.

## Bağlam

ADR-046 production modüllerini sahip ve katman yönüyle korudu; fakat aynı
katmanda karşılıklı kenarlar yön matrisini ihlal etmeden strongly connected
component (SCC) oluşturabiliyordu. İlk SCC incelemesi üç çevrim buldu:
`tani ↔ kaynak_sinirlari`, `cozumleyici ↔ hir` ve
`paket → registry → tedarik → paket`.

## Karar

1. Tanı sayısı, bağımlılıksız `tani_politikasi` temel sözleşmesinde tek
   değerdir. Tanı üretimi ve merkezî kaynak profili bu değeri tüketir; tanı
   artık kaynak limitleri uygulamasına geri bağımlanmaz.
2. `Tur`, `VeriTuru` ve `SozlukDegerTuru`, private `semantic_model` sahibine
   taşınır. Checker mevcut public yeniden dışa aktarımı korur; HIR checker'a
   değil ortak modele bağımlanır.
3. Exact production graph'ı üzerinde bütün SCC'ler hesaplanır. Yeni SCC,
   kaybolmuş fakat allowlist'te bırakılmış SCC CI'ı durdurur.
4. Zorunlu geçici istisna; exact ve sıralı üye kümesi, en az 40 karakter
   gerekçe, ISO son tarih ve `K-nnn` kaldırma işi taşır. Son tarih geçtiğinde
   graph değişmese de kapı kırılır.
5. Paket/registry/tedarik SCC'si K-160 sahiplik ayrımıyla kaldırılmıştır.
   C001 kaydı artık yoktur. Allowlist katman yönünü
   veya başka bir SCC'yi meşrulaştırmaz.

## Reddedilen seçenekler

- **Katman yönünü yeterli saymak:** aynı katmandaki çevrimi göremez.
- **Bütün mevcut SCC'leri kalıcı kabul etmek:** mimari borcu görünmez ve
  süresiz yapar.
- **Yalnız SCC sayısını sabitlemek:** farklı üyeli yeni çevrim eski çevrimin
  yerini alarak kapıdan geçebilir.
- **Takvim son tarihi olmadan yalnız backlog maddesi yazmak:** ertelenen
  ayrımın sessizce kalıcılaşmasına izin verir.

## Sonuçlar

- Üç SCC'nin tamamı kaldırılmıştır; production SCC ve izin sayısı sıfırdır.
- K-160 paket modeli, çözümleme, registry protokolü, verification ve yayın
  orkestrasyonu sahipliğini ayrı bağımlılık yönlerine taşımıştır.
- Dil grammar'ı, runtime semantiği, tanılar ve normatif spec değişmedi.
