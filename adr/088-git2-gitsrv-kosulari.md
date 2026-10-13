# ADR-088 — Git2 üzerinde Gitsrv CI

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıt:** K-216, ADR-048, ADR-066, ADR-087, GB-048

## Bağlam

Ana GitHub hesabının ödeme/limit engeli K-215'in altı uzak işini test
başlamadan durdurdu. Kullanıcı koşuların özel Git2 üzerindeki Gitsrv
koşucularında yapılmasını istedi. GitHub ana kaynak, Enterprise özel ayna
olarak kalır; GitHub ve Enterprise Actions kapalıdır.

## Karar

`.forgejo/workflows/zee-ci.yml` yalnız ana dal/ayrılmış CI dalı push'u ve
elle başlatma ile çalışır. Fork veya PR kodu kurumsal koşucuda kendiliğinden
yürütülmez. Linux işi açık konteyner sınırında araçlarını kurar; üretim
sunucularını veya özel ürün pinini değiştirmez.

Checkout yalnız seçili ref'in tam geçmişini alır, exact olay SHA'sını
doğrular; diğer özel Git2 dallarını/etiketlerini ve eski ürün soy ağacını
çekmez. İş tokeni Git config'e, URI'ye veya diske yazılmaz; yalnız fetch
işleminin ortam başlığıdır. Başlangıç dizini doluysa kapı kapanır.
Action'lar 40 haneli SHA ve ayrı resmî kaynak envanteriyle sabittir.

Faz matrisi, clippy/fmt, normatif belge/kanıt/tanı drift'i, kaynak korukları,
tam kaynak gizlilik+sır taraması, cargo-deny/compiler+fuzz, offline vendor
ve gerçek WASM yürütmesi Linux'ta aynı exact kaynakta çalışır. Faz raporu
ile p50/p95 shared-CI gözlemi 90 günlük özel artefakttır. Performans hard
eşiği yoktur ve izlenen tarihçeye yeni baseline yazılmaz. Başarısız veya
başlamamış iş geçiş sayılmaz.

## Sınırlar

Gitsrv Linux kanıtı Windows çalışma zamanı testi değildir. Önceki macOS
kanıtı actual kaynak/ortamıyla kalır; Windows exact çalışma zamanı kanıtı
açık kalır. Bu geçiş GitHub ödeme engelini çözmüş veya GitHub checks'i
yeşile çevirmiş sayılmaz. GitHub PR refs/cache temizliği ve yeniden açılış
onayı ayrıca beklenir.

Forgejo `permissions` anahtarının GitHub ile eşdeğer uygulandığı varsayılmaz;
bu yüzden PR yürütmesi yoktur. Kurumsal koşucu ve push yetkisi olan kaynak
hesapları güven sınırındadır. Gerçek özel erişim adresleri ve tokenler
açık depodaki karara/rapora taşınmaz.

## Kanıt

Tedarik testi Forgejo pinlerini ve PR'siz iş kapsamını denetler. Unix
hermetik test gerçek workflow fetch adımını iki farklı dal ve özel etiketli
Git fixture'ında çalıştırır; tam seçili geçmiş, exact SHA, kalıcı kimlik
yokluğu ve ilgisiz nesnelerin alınmaması olumlu/olumsuz sonuçlarla korunur.
Uzak exact kaynak sonuçları ayrı takip kaydına eklenir.

## Etki incelemesi

Spec/RFC, tanı/grammar/API/ABI ve production Rust modülleri değişmez.
ADR-046/katman fixture/SCC incelendi: yeni compiler sahibi veya kenarı yok.
Paket grafiği, release tarifi/SBOM/provenance ve ikili kümesi değişmez;
CI araçları konteynerde kurulur. ADR-045/049 incelendi: shared gözlem
eşiksizdir, geçmiş ölçümler ve sabit iş yükü değişmez. ADR-048 pin ilkesi
Forgejo envanterine de uygulanır; test fazı ve canlı sayılar aynı committe
güncellenir. Kullanıcı tanısı veya compiler bug olmadığı için yeni dil
regression kaydı yoktur.
