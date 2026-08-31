# Editör entegrasyonları

## dillsp — LSP sunucusu

Derle: `cargo build --release` → `compiler/target/release/dillsp`.
Sunduğu yetenekler: canlı Türkçe tanılar (çoklu, RFC-0010), kalıp kelime
tamamlama, **hover** (kalıp kelimesine Türkçe açıklama + örnek; ada, tanım
satırı) ve **tanıma git** (işlem/yapı başlığına ya da `olsun`/`al` satırına —
ek almış kullanımlar morfolojiyle çözülür: `sayacı` → `sayaç`). Birimleri
dosyanın klasöründen çözer.

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
