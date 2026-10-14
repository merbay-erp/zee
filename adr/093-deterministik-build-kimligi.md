# ADR-093 — Deterministik compiler build kimliği

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-226, ürünleşme A1

## Karar

`dil sürüm` mevcut SemVer ve profil bilgisinin yanında source commit SHA,
source commit UTC tarihi ve kanalı taşır. `--json`, `zee-build-v1` kaydını
verir: version, source_sha, source_date, channel, dirty. Duvar saati veya
makine yolu ikiliye gömülmez. Git olmayan kaynakta kimlik `unknown` olabilir;
bu ikili exact release kanıtı olarak kabul edilmez.

Git varsa explicit `ZEE_SOURCE_SHA` checkout HEAD'iyle, `SOURCE_DATE_EPOCH`
commit zamanı ile eşleşir; yanlış beyan derlemeyi durdurur. Archive build
bu iki girdiyi explicit sağlayabilir. Kanal varsayılanı `-dev` için
development, diğer seri için beta'dır. Stable ancak explicit seçilir ve
`-dev` seri stable olamaz; kanal etiketi RC kapılarını geçme iddiası değildir.
Kirli compiler kaynak/manifest/lock/build betiği ayrı boolean'dır.

LSP serverInfo.version SemVer kalır; serverInfo.zeeBuild aynı JSON kaydıdır.
Worker startup aynı kaydı yazar. Uygulamanın sağlık rotası dahil tamamlanmış
worker yanıtlarının `X-Zee-Build` başlığı aynı kaydı taşır; yeni rota eklenmez
ve readiness/DB sağlık kabulü uydurulmaz. Başlık yalnız sürüm/SHA/tarih/kanal
ve dirty taşır; ağ hedefi, kaynak yolu, kullanıcı veya sır içermez (GB-055).

## Mimari ve kabul

`surum` temel katmanda yalnız build sabitlerinin sunumunu taşır. CLI/LSP
bu temele bağımlanır; yeni cycle veya temel→adaptör geçişi yoktur. Internal
doc-hidden modül `api::v1` destek sözünü büyütmez. Build betiği compiler
anlamını ve dependency graph'ını değiştirmez; subprocess Git yalnız build
zamanındadır. Yeni grammar/kalıp/koşul/komut yok; JSON biçimi ayrı uyumluluk
kaydıdır. K-174 fingerprint değişmez.

CLI/LSP/SBOM exact version/SHA/tarih testinde karşılaştırılır; SBOM zaten
source commit ve epoch taşır. Worker başlık testi aynı JSON'u karşılaştırır.
İzole build kabulü iki temiz klonda aynı kaynak/aynı hash ve farklı source
commit'te farklı kimliği; SHA/epoch/kanal bozuk beyanlarının reddini sınar.
Asıl dil ikilileri için ADR-068 iki-clone betiği ve exact temiz kaynak kabulü
ayrı gerekir; küçük build fixture gerçek üç platform RC yerine geçmez.

Spec/27, ADR-046 fixture, public API/SCC, sürüm betiği/SBOM/provenance ve
tanı/tazelik kapıları aynı dalgada incelenir. RFC-0028 SemVer/uyumluluk çerçevesi
korunur. Kalıcı cache veya yeni büyüyen durum eklenmez; ADR-069 soak tabanı
ve performans iş yükü bu dilimde değişmez. Hata kataloğuna yeni dil tanı kodu
eklenmez; build beyan hataları compiler kaynak tanısı değildir.
