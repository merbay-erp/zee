# K-220 — HTTP POST gerçek ürün kanıtı

Harici talep commit’i `2d570601608592decc3a94d4ad061bdde1d84874`.
Kaynak talep `docs/dil-talepleri/2026-10-14-http-post-baslik.md`; birincil gerçek
ürün dosyası `cekirdek/kaynak/rota_site.dil:20`, POST /iletisim-gonder;
ikincil portal heartbeat kabuk/Python geçişi talepte kayıtlıdır.
[Anonim talep](talep.md), [minimal Zee reproducer](turnstile.dil).
[Bağımsız proje kullanım rehberi](../../../docs/http-post-kullanim.md).

Bu kesit eski proje kimlik/güvenlik gereksinimi ve 100 kartlık yükle aynı ürün
kökündedir. Gerçek rota kaydı ve e-posta davranışı ürün tarafında kalır.
Sahte adaptör testleri geçerli/başarısız/yanlış hostname/eksik alan ve erişim
hatası senaryolarını gerçek Zee helper’ını çalıştırarak sınar.

Sunucu ortamında TURNSTILE_SECRET ve proje bildiriminde yalnız adı gerekir:

```zee
http_gizli_değişkenleri "TURNSTILE_SECRET" listesi olsun
```

Kaynakta gerçek anahtar yoktur. `ağ` yetkinliği ve yalnız
`https://challenges.cloudflare.com` origin’i proje sahibince açılır.
HTTP JSON yardımcısı alanların kanonik JSON metnini taşır: bool true `true`,
metin true `"true"`, hostname ise JSON tırnaklı metindir. Beklenen hostname
`json metni` ile karşılaştırılır; tip taklidi kabul edilmez.
