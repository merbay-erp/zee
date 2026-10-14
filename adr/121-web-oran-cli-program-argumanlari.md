# ADR-121 — Mevcut web oran seçeneklerinde CLI program argümanları

- **Durum:** kabul; K-262 kaynak/provenance bugfix kapanışı. Final exact RC ve birleşik özellik kabulü ayrı ve açıktır.

## Kanıt

Temiz17b29da1b8321fe6dcc4d34bd2027c4c969bf406 checkout ve dirty=false BuildJSON ile mevcut çalıştır --web-proxy https://fixture.example --web-oran-deposu bellek --web-oran-siniri 2 kaynak APP_ARG çağrısı APP_ARG yerine bellek yazdı. program_argumanlari değerli oran seçeneklerini kaynak öncesinde atlamadığından bellek kaynak konumu sayıldı. K257 metrics FD özelliğinden önce var olan bug. Depo seçeneği 4e351d34de861425c8a35b6ad1ddda6cd3e42707/K218; diğer üç değerli seçenek 35955470fdb4fc8474f39326ae0f7aa771b5515b/K219 ile filtre tamamlanmadan eklenmiştir.

## Minimal karar

Yalnız mevcut dört değerli oran seçeneği kaynak öncesinde tek değeriyle atlanır. Kaynak sonrası oran bayrak/değerlerinin program argümanlarında kalması ve eski globalproxy/worker/unary filtre davranışı korunur. Yeni flag, grammar, export, runtime IO/state, dependency veya FD desteği eklenmez. Sınıf semantic-bugfix ve core-freeze correctness/bugfix; dogfood yeni özellik değildir.

## Kanıt zinciri

cli_secenek_testi gerçekCLI seçenek sırası/alias/dört flag ile APP_ARG ve SECOND_ARG exact çıktısını, kaynak sonrası oran argümanları ve eski globalproxy sınırını denetler. Corpus mevcut stderr retlerini korur; explicit # cli-cikti: stdout başarılı CLI çıktısını seçer. Web startup stderr metadata'sı boş olmak zorunda değildir. Fixture alt türü compiler kullanıcı yüzeyi değildir.

Minimal iki satırlı `regression/cli/web-oran-program-argumani.dil` kaynağı, `regression/v2.tsv` içinde exact fixed_by `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179` ve introduced_by `4e351d34de861425c8a35b6ad1ddda6cd3e42707` ile kayıtlıdır. Kaynak commit önce alınmış, bu ayrı takip dilimi mandatory corpus ve semantic-bugfix/core-freeze correctness beyanlarını tamamlamıştır. Mevcut spec27 CLI maddesi değişmez, proof/test bağları yenilenir. Katman/SCC/public API/uyumluluk yüzeyi ve performans/soak davranışı değişmediği incelenmiştir. K-262 dar arıza kapanışı source/provenance ve ilgili kapılarla sınırlıdır; final RC ve diğer ürünleşme dilimlerinin kabulünün yerini almaz.
