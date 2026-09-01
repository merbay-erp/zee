# ADR süreci

Güvenlik/mimari sınır değişikliği ADR ister (master plan bölüm 25).
Şablon: [000-sablon.md](000-sablon.md).

## Planlanan ilk ADR'ler (master plan bölüm 38)

| No | Başlık | Durum |
|---|---|---|
| ADR-001 | [Bootstrap dili: Rust + küçük/kilitli bağımlılık yüzeyi](001-bootstrap-dili.md) | **kabul** (K-088/K-092 revizyonu) |
| ADR-002 | [Parser: elle yazılmış, yüklem-sonlu dağıtım](002-parser-stratejisi.md) | **kabul** (K-097 katmanlı ifade bölgeleri revizyonu) |
| ADR-003 | [İlk yürütme: ağaç-yürüyen yorumlayıcı + IO soyutlaması](003-ilk-yurutme-modeli.md) | **kabul** |
| ADR-006 | [Paket registry güven modeli](006-paket-registry-guven-modeli.md) | **kabul** (K-094 yayın; K-095 metadata güveni; taşıma/cache aşamalı) |
| ADR-009 | [Dilin adı: zee](009-dil-adi.md) | **kabul** (kurucu yetki devriyle) |
| ADR-010 | [Normatif otorite ve değişiklik bütünlüğü](010-normatif-otorite-ve-degisiklik-butunlugu.md) | **kabul** |
| ADR-011 | [Core AST intrinsic/yetkinlik sınırı](011-intrinsic-yetkinlik-siniri.md) | **kabul** (K-098/B-004) |
| ADR-012 | [Derleyici fiziksel faz modülleri](012-derleyici-faz-modulleri.md) | **kabul** (K-099/B-005) |
| ADR-013 | [Checker semantik katmanları](013-checker-katmanlari.md) | **kabul** (K-100/B-006) |
| ADR-014 | [Semantic kimlikler](014-semantic-kimlikler.md) | **kabul** (K-101/B-010) |
| ADR-015 | [Derleyici faz tipleri](015-derleyici-faz-tipleri.md) | **kabul** (K-102/B-018) |
| ADR-016 | [Typed HIR çekirdeği](016-typed-hir-cekirdegi.md) | **kabul** (K-103/K-104/B-019) |
| ADR-017 | [Native ağ I/O kaynak sınırları](017-native-ag-kaynak-sinirlari.md) | **kabul** (K-105/B-025) |
| ADR-018 | [Sınırlı web oturum deposu](018-sinirli-web-oturum-deposu.md) | **kabul** (K-106/B-046) |
| ADR-019 | [LSP girdi sınırları](019-lsp-girdi-sinirlari.md) | **kabul** (K-107/B-047) |
| ADR-004 | Bellek yönetimi prototip kararı (GC / ARC benchmark) | bekliyor |
| ADR-005 | Native backend seçimi (Cranelift / LLVM) | bekliyor — Faz 4 |
| ADR-007 | [Telemetri ve gizlilik: araçlar veri toplamaz](007-telemetri-ve-gizlilik.md) | **kabul** |
| ADR-008 | [Self-hosting aşamaları ve geçiş kapıları](008-self-hosting-asamalari.md) | **kabul** |
