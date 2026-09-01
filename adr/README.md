# ADR süreci

Güvenlik/mimari sınır değişikliği ADR ister (master plan bölüm 25).
Şablon: [000-sablon.md](000-sablon.md).

## Planlanan ilk ADR'ler (master plan bölüm 38)

| No | Başlık | Durum |
|---|---|---|
| ADR-001 | [Bootstrap dili: Rust + sıfır bağımlılık](001-bootstrap-dili.md) | **kabul** |
| ADR-002 | [Parser: elle yazılmış, yüklem-sonlu dağıtım](002-parser-stratejisi.md) | **kabul** |
| ADR-003 | [İlk yürütme: ağaç-yürüyen yorumlayıcı + IO soyutlaması](003-ilk-yurutme-modeli.md) | **kabul** |
| ADR-009 | [Dilin adı: zee](009-dil-adi.md) | **kabul** (kurucu yetki devriyle) |
| ADR-010 | [Normatif otorite ve değişiklik bütünlüğü](010-normatif-otorite-ve-degisiklik-butunlugu.md) | **kabul** |
| ADR-004 | Bellek yönetimi prototip kararı (GC / ARC benchmark) | bekliyor |
| ADR-005 | Native backend seçimi (Cranelift / LLVM) | bekliyor — Faz 4 |
| ADR-006 | Paket registry trust modeli | bekliyor — Faz 5 |
| ADR-007 | [Telemetri ve gizlilik: araçlar veri toplamaz](007-telemetri-ve-gizlilik.md) | **kabul** |
| ADR-008 | [Self-hosting aşamaları ve geçiş kapıları](008-self-hosting-asamalari.md) | **kabul** |
