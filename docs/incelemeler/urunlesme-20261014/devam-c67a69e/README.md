# K-245–K-249 exact yerel kabulü

Kaynak `c67a69e6ed9c24c4a540a55eeaaa986a348a2de3`.
[Toplu rapor](../../../urunlesme-pilot-k245-k249.md) kapsamı ve açık
ürün/insan/platform/lisans/uzak CI/RC sınırlarını korur.

- [Faz](faz-test-matrisi.md), [sürekli kapılar](gate-summary.json).
- [28 PostgreSQL](native-proof.json), [parola/C027](parola-goc-cases.json),
  [SQL queue](is-kuyrugu-cases.json).
- [Credential14/supervisor25](host-proof.json), [worker kampanyası](worker-campaign.json).
- [TR–EN7+4](tr-en-proof.json), [K249 aday incelemesi](k249-independent-review.json).
- [İki temiz klon](build-proof.json), [exact kaynak ZIP](source-archive.json),
  [çapraz inceleme](independent-review.json).

Her rapor kendi kaynak kimliğini korur. Çapraz inceleme sırasında son faz
tekrarı henüz bekliyordu; nihai kabul ayrı gate-summary dosyasındadır.
İlk CARGO_TARGET_DIR faz ve aggregate KeyError kayıtları kaynak dışı
kalıcı özel ham kanıt indeksinde korunur. Python/host/native senaryoları
Cargo sayısına ayrıca eklenmez. İmza, yayın veya canlı ürün kabulü yoktur.
