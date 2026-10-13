# K-206 source identity ve iki-clone kanıtı

Temiz exact ab8e97ef4f42653cb4a2ee0b61b104d604eca98a üzerinde
`scripts/surum-artefakti.sh` iki temiz klonda aynı dil/dillsp SHA-256
üretti. SBOM/provenance/GIT_SHA ve SHA256SUMS ham byte'ları burada;
ikililer ignored target'ta, sürüm etiketi/publish yapılmadı, imza yoktur.
Native ortam/profil ve cc driver kayıtları provenance içindedir.
Bu yerel macOS kabulüdür; yeni source için üç-platform release/kurulum
kanıtı değildir. RC fuzz/Miri kapısı olarak gösterilemez.

`path-rejected.log` ba547f4 path patch ile iki binary'nin eşit olmadığı
başarısız koşudur; bu kaynaktan artefakt verilmedi. `git-accepted.log`
exact Git source pininden sonraki başarılı koşudur. Paket pgo kaynak
URL'si build HEAD'i değil c4e9d03616fefa3d4aa9edcdef5c3ab8af7a8c75
immutable dependency commitini gösterir. Registry checksum gibi
sunulmayan canonical source tree hash'i 30fe54faeea20f6772380c5902389016cef4a7823c5886d5e8e39821764aabe4'tür.

Offline vendor iki üretimde aynı manifest hash'i verdi ve boş Cargo home
compiler/fuzz check geçti: 163 paket, SHA-256
137d1744dc404dba6704764e8c9a9687fb2cbce19a10fb7cb8e6f60c5be31cf9.
Rust protocol src byte'ları upstream ile aynı; 32 MiB wire guard bu kaynakta
etkin değildir. GB-041 ve transaction/erken header sınırı açıktır.

Dependency retention ref `dependency-retention/postgres-protocol-c4e9d03616fe`
clean kabul kaynak ab8e97e'yi tutar; dependency c4e9d03 bunun ancestor'ıdır.
Squash/branch temizliği bağımlılık commitini reachable kümeden çıkaramaz.
Ref silinmez/taşınmaz; Zee `v*` etiketi veya binary publish değildir.
