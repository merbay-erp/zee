# zee — tek merkez (master) toparlaması ve inceleme bulguları (13.10.2026)

Talep eden: ürün tarafı (zee-cekirdek). Uygulayacak: zee dil geliştirme (Codex). Kullanıcı kararı: dallardaki değişiklikler
tek merkezde (master) toplanacak; ana hatta olmayan benzersiz işleri Codex dondurma kurallarıyla alacak.
(Bu belge açık depoya kopyalanabilir: müşteri/alan/sunucu adı içermez.)

## 1) Ürün tarafının yaptıkları (bilgi)
- Ürün siteleri (5 üretim sitesi) master `6cdf6e0` ile yerelde sınandı: tüm `dil dene` paketleri geçti (54/54/55/47 + çekirdek kökü 34/34).
- Site CI sabitleri ve sunucu ikilileri master'a taşınıyor; geçici CI dalları (`ci/site-dil`, `ci/urun-e-dil`) taşıma bitince silinecek.
- Ana hatta zaten olan dallar silinecek: `codex/zee-derleme-bagimlilik`, `codex/zee-site-tek-hat`, `fix/elmas-ice-alim-tohumu` (K-190),
  `codex/public-depo-guvenligi`, eski `dependabot/…/rust-toolchain-02cb101…`. Silmeden önce tüm referansların yerel git bundle yedeği alındı.

## 2) Codex'e: ana hatta alınacak benzersiz işler (sonra dallar silinsin)
| Dal | İçerik | Not |
|---|---|---|
| `site/zeedili-com` (açık PR #6) | resmî dil sitesi (8 commit) | birleştir ya da yeniden temellendir |
| `dgx-issue-formlari` | 4 issue formu (`.github/ISSUE_TEMPLATE`) | eski geçmiş; içeriği master'a yeniden uygula |
| `ci/tier1-gitsrv` | Windows çapraz derleme CI akışı (self-hosted) | kurum kuralı: özel depo CI'ı kendi runner'ında; açık depoda self-hosted YASAK |
| `wip/spec27-snapshot` | spec/27 ek dosyası | değerlendir; gereksizse kapat |
| `dependabot` (upload-artifact 7.0.2, rust-toolchain e2a55d2) | eylem pin güncellemeleri | pin listesi/beyanlarıyla birlikte |

## 3) Bağımsız inceleme bulguları (eski `c1ab630` üzerinde yapıldı; master'a göre güncellendi)
Master'da ÇÖZÜLMÜŞ: TOTP/2FA, SMTP (bağlantı değişkeni), rota birimleri, `görünüm` bloğu (tipli HTML, otomatik kaçış) — inceleme
bunları birleştirilmemiş dalda görmüştü.

Hâlâ GEÇERLİ (ürün kanıtı: 5 üretim sitesi üretim web kipinde, paylaşımlı sunucuda):
1. **Oran sınırı ortak durumu her istekte dosyaya yazılıp fsync ediliyor.** Ölçüm: 100 kartlık liste sayfası deneysel kipte ~11.000
   istek/sn, üretim kipinde ~55 istek/sn (tek worker); 2 worker toplam ~56 istek/sn (dosya kilidi sıralıyor). Anahtar sayısıyla dosya
   büyüyor (32.768 anahtarda ~1,8 MB). Öneri: bellek içi (süreç başına) ya da PostgreSQL tabanlı durum. **Öncelik 1.**
2. **Oran sınırı sabit:** IP+adres başına dakikada 100; 32.768 canlı anahtar dolunca site geneli 503. Ayarlanabilir sınır ve
   doluluk davranışı (en eskiyi düşür) gerekli; ürünler önünde ayrıca proxy sınırı kullanılıyor.
3. **Dışarı HTTP istemcisi yalnız GET** (başlık/gövde yok): POST + başlık (API anahtarı) gerektiren servisler yazılamıyor.
4. **HTML kaçışı:** `görünüm` bloğu varken metin birleştirme hâlâ yaygın; izin listeli HTML temizleyici yok (kullanıcı/AI üretimi zengin metin).
5. **LSP her değişiklikte tüm belgeyi yeniden analiz ediyor:** 2 bin satır 0,16 sn · 10 bin 4,6 sn · 20 bin 20 sn.

Ölçüm betiği ve tekrar üretici inceleme oturumunda mevcut; 1. madde için ürün tarafı yük senaryosu sağlayabilir.
