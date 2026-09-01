# Editör entegrasyonları

## dillsp — LSP sunucusu

Derle: `cargo build --release` → `compiler/target/release/dillsp`.
Sunduğu yetenekler: canlı Türkçe tanılar (çoklu, RFC-0010), kalıp kelime
tamamlama, **hover**, **tanıma git** ve **YENİDEN ADLANDIRMA**. Definition ve
F2, K-120'den beri başarılı checker'ın `SymbolId`/`IslemId`/`YapiId` HIR
bağını kullanır: ayrı kapsamdaki aynı yazıma dokunmaz; yerel sembol eklerini
Türkçe uyumla yeniden giydirir (`sayaç→puan`, `sayacı→puanı`); çok kelimeli
işlem adını tek parça değiştirir. Hatalı/belirsiz belgede metin tahmini yapmaz.
Açık belge dışındaki tanım için yarım WorkspaceEdit üretmez; çok-dosyalı
rename dosya kimlikli span kapısına kadar fail-closed kalır.
Ayrıntılı sözleşme [semantic gezinme rehberindedir](../docs/lsp-semantic-gezinme.md).
Birimleri kaynağın klasöründen, yerel paketleri `proje.dil`/`proje.kilit`
grafiğinden çözer.

K-107 güvenlik sınırı: gelen JSON-RPC çerçevesi en çok 8 KiB başlık ve 8 MiB
gövde taşır; tam bir `Content-Length` zorunludur. JSON en çok 128 iç içelik ve
100.000 değer düğümü kabul eder; geçersiz Unicode vekili ya da kaçışsız kontrol
karakteri reddedilir. Bozuk çerçevede `dillsp` akışı tahmin etmeyip kapanır;
editör process'i yeniden başlatabilir.

### Helix (`~/.config/helix/languages.toml`)

```toml
[language-server.dillsp]
command = "dillsp"

[[language]]
name = "dil"
scope = "source.dil"
file-types = ["dil"]
comment-token = "#"
indent = { tab-width = 4, unit = "    " }
language-servers = ["dillsp"]
```

### Neovim (0.10+, `init.lua`)

```lua
vim.filetype.add({ extension = { dil = "dil" } })
vim.api.nvim_create_autocmd("FileType", {
  pattern = "dil",
  callback = function()
    vim.lsp.start({ name = "dillsp", cmd = { "dillsp" } })
  end,
})
```

### VS Code

[vscode/](vscode/) sözdizimi renklendirmesi verir (symlink kurulumu kendi
README'sinde). LSP istemcisi `vscode-languageclient` paketi ister; npm'siz
sürümde tanılar için şimdilik: `dil denetle --json dosya.dil`.
