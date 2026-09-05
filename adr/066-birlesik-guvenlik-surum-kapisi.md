# ADR-066 — Birleşik güvenlik sürüm kapısı

- **Durum:** kabul
- **Tarih:** 5 Eylül 2026
- **İlgili kayıt:** K-170, ADR-038/056, V1-P1-23, SECURITY.md

## Bağlam

Güvenlik kanıtı dört yerde dağınıktı: cargo-deny/offline vendor (`tedarik`
workflow'u), günlük ve haftalık RC fuzz (`fuzz`/`fuzz-rc`), web/ağ/yetkinlik
regresyonları (ana CI) ve K-105'ten K-176'ya uzanan inceleme bulguları yalnız
günlükte. "Açık kritik/yüksek bulgu var mı" sorusunun tek makine-okunur cevabı
yoktu; K-163'ün multipart/içerik güvenliği gibi bilinçli sınırları da kayıt
altında değildi. Üçüncü dış inceleme K-170 ile birleşik bir security release
gate ve sıfır açık kritik/yüksek kuralı istedi.

## Karar

1. **Bulgu kaydı.** `docs/guvenlik-bulgulari-v1.tsv` ardışık `GB-NNN`
   kimlikli, kaynak (`inceleme|fuzz|dogfood|advisory|drift|saha`), önem
   (`kritik|yuksek|orta|dusuk`), yüzey, özet, durum (`acik|kapali|kabul`),
   kapanış K-işi ve karar belgesi taşır. Kapalı bulgu gerçek K-işine, kabul
   edilen sınır normatif spec/ADR yoluna bağlanır; kritik sınır kabul edilemez.
2. **Sıfır açık kritik/yüksek.** Açık bulgu yalnız orta/düşük olabilir ve
   hedef K-işi taşır. Kural `guvenlik_kapisi_testi` ile her push'ta, sürüm
   adayı kapısında ve `SECURITY.md` sözleşmesiyle yürütülür.
3. **Tek giriş noktası.** `scripts/guvenlik-kapisi.sh --surekli` sabit
   cargo-deny (compiler+fuzz), fuzz derlemesi, bulgu kaydı ve tedarik kapısı
   testlerini koşar; `tedarik` workflow'u bunu her push/PR/gecelik koşuda
   çalıştırır. `--surum-adayi` ayrıca temiz çalışma ağacı, exact HEAD için dört
   hedefli ≥1800 saniyelik RC fuzz kanıtı (K-157), clippy, tam test paketi ve
   üç yayımlanmış-kanıt koruğunu ister. Sürüm etiketi bu kapı geçmeden kesilmez.
4. **İçerik güvenliği sınırı.** Native yükleme yalnız exact
   `application/octet-stream`dır; multipart, içerik türü doğrulaması ve
   antivirüs taraması vaat edilmez. Runtime yüklenen baytı hiçbir rotadan
   sunmaz; içerik doğrulama ve karantina ürünün sorumluluğudur. Bu sınır
   GB-019 olarak `kabul` kaydındadır; değişmesi RFC-0027/spec-26 revizyonu ister.
5. **Bildirim kanalı.** Depo özel olduğu sürece bulgular GitHub Security
   Advisories ile bildirilir; kabul edilen bulgu üç iş günü içinde kayda girer,
   düzeltme yayımlanmadan ayrıntı paylaşılmaz.

## Reddedilen seçenekler

- **Yalnız cargo-deny:** üçüncü taraf advisory'sini görür, kendi
  bulgularımızı ve bilinçli sınırları görmez.
- **Bulguları yalnız günlükte tutmak:** "açık kritik var mı" sorusu elle
  arama ister; sürüm kapısı makinece kararsız kalır.
- **RC fuzz kanıtını sürekli kapıya koymak:** 30–60 dakikalık kampanya her
  push'ta koşamaz; sürüm adayı kipine ayrıldı.

## Sonuçlar

- İlk taban 22 kayıt: 15 kapalı (K-105…K-176), 4 kabul edilen sınır, 3 açık
  düşük/orta test-kapsam boşluğu (X-Zee-CSRF ve çoklu Content-Type gerçek
  adaptör testi, `__Host-` çerez nitelikleri, gerçek TLS/NULL sütun reddi);
  hepsi K-172/K-173 hedeflidir. Açık kritik/yüksek sıfırdır.
- Yeni güvenlik düzeltmesi aynı committe kayıt satırını `kapali` yapar; yeni
  bulgu önce `acik` girilir. Kritik/yüksek açık bulgu CI'ı ve sürümü durdurur.
- Kapı bakım/işletim aracıdır; dil semantiği ve tanılar değişmez.
