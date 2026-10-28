# ürün G ürün kökeni — K-330

Harici ürün: ürün deposu (özel), zee ile yazılmış mini ERP.

Exact ürün commit'i: `8e8101cee72b7b3db1b17db97eb49ab5c6fecd8f`
(28 Ekim 2026). Talep aynı gün ürün sahibinden geldi: 30 dakikalık mutlak
oturum ömrü kullanıcıyı iş ortasında girişe atıyor; oturum kullanıcı çıkış
yapana kadar etkin kalmalı.

Reproducer `oturum-gereksinimi.dil`, ürünün giriş → kimlikli sayfa → çıkış
akışını hermetik küçültmeyle taşır; `proje.dil` ürünün seçeceği
`oturum_ömrü_saniye`/`oturum_kayar` bildirimidir. Gerçek kimlik bilgisi,
veritabanı, alan adı ve sunucu yolu taşınmaz (ADR-087). Ürünün etkilenen
kaynakları giriş/çıkış rotaları ve ürün bildirimidir.

Karar ve güvenlik gerekçesi [ADR-158](../../adr/158-proje-basina-oturum-omru-ve-kayar-yenileme.md).
