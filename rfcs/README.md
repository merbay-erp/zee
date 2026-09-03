# RFC süreci

Dil değişikliği RFC ister (master plan bölüm 25). Şablon: [0000-sablon.md](0000-sablon.md).

## Planlanan ilk RFC'ler (master plan bölüm 37)

| No | Başlık | Durum | Beslendiği günlük kayıtları |
|---|---|---|---|
| RFC-0001 | [Dil Manifestosu ve Tasarım İlkeleri](0001-dil-manifestosu.md) | **kabul** | — |
| RFC-0002 | [Lexical ve Unicode Kuralları](0002-lexical-ve-unicode.md) | **geçici kabul** | K-001, K-011, K-014, K-089, A07, A08 |
| RFC-0003 | [Girinti ve Blok Modeli](0003-girinti-ve-blok.md) | **geçici kabul** | A06 |
| RFC-0004 | [Değer Tanımı ve Kapsam](0004-deger-tanimi-ve-kapsam.md) | **geçici kabul** (K-034) | K-003, K-011, K-020 |
| RFC-0005 | [Koşullar ve Mantıksal İfadeler](0005-kosullar-ve-mantiksal.md) | **geçici kabul** (K-097 katman sırası) | K-005, K-010, K-027, K-097, A03 |
| RFC-0006 | [İşlemler ve Parametreler](0006-islemler-ve-parametreler.md) | **geçici kabul** (K-086 tam public imza; K-121 sıra-bağımsız yerel çıkarım; K-096 usability kapısı) | K-016, K-032, K-083, K-086, K-096, K-121 |
| RFC-0007 | [Temel Tür Sistemi](0007-temel-tur-sistemi.md) | **geçici kabul** | K-009, K-014 |
| RFC-0008 | [Seçenek ve Sonuç](0008-secenek-ve-sonuc.md) | **geçici kabul** (daraltma + yapılandırılmış Hata) | K-017, K-018, K-030, K-091 |
| RFC-0009 | [Modül ve Paket Modeli](0009-modul-ve-paket.md) | **geçici kabul** (birim + proje + yerel/exact registry paket, kilit v3 + public kaynak ABI) | K-029, K-076, K-078, K-086, K-095, K-135, K-136 |
| RFC-0010 | [Hata ve Tanılama Standardı](0010-hata-ve-tanilama.md) | **kabul** (K-113 recovery, K-114 kimlik fixture'ı) | K-026, K-113, K-114 |
| RFC-0011 | [Structured Concurrency](0011-structured-concurrency.md) | **geçici kabul** (K-085 deadline; K-090 scheduler+sahiplik; K-124 bağımsız conformance; K-133/K-134 iptal güvenliği) | K-023, K-085, K-090, K-124, K-133, K-134 |
| RFC-0012 | [FFI ve Tehlikeli Sınır](0012-ffi-ve-tehlikeli-sinir.md) | **taslak** (K-125 örtük binary float yasağı bağlı; FFI gerçeklenmedi) | A10, K-125, ADR-029, B-012 |
| RFC-0013 | [Ondalık Sayılar](0013-ondalik-sayilar.md) | **geçici kabul** (K-092 keyfî hassasiyet; K-125 binary sınırı) | K-028, K-092, K-125, ADR-029 |
| RFC-0014 | [Standart Kitaplık](0014-standart-kitaplik.md) | **taslak — çalışan prototip ekli** | K-046, K-048, RFC-0002 §6.3 |
| RFC-0015 | [Uygulama Eylemleri ve Web Güvenlik Sınırı](0015-uygulama-eylemleri.md) | **geçici kabul — K-087 eylem, K-088 profil, K-176 strict form** | K-081, K-087, K-088, K-176, ADR-010/054 |
| RFC-0016 | [Atomik Kalıcı Dosya Sözleşmesi](0016-atomik-kalici-dosya.md) | **geçici kabul** (K-128 metadata koruması; güç-kesintisi/disk-dolu kapısı açık) | K-019, K-084, K-128, B-048, V1-P0-04/30 |
| RFC-0017 | [Web Oturumu, Yetki, CSRF ve Güvenilir Proxy](0017-web-oturum-ve-csrf.md) | **geçici kabul** (K-134 request transaction; K-137 ortak depo/rate-limit) | K-082, K-088, K-134, K-137, V1-P0-03 |
| RFC-0018 | [Sürümlü Morfoloji Profili](0018-surumlu-morfoloji-profili.md) | **geçici kabul** (K-120 semantic LSP, K-122 immutable kayıt, K-123 bağımsız conformance) | K-011, K-072, K-089, K-111, K-120, K-122, K-123, V1-P1-02 |
| RFC-0019 | [Değer Semantiği ve Gezme İmleci](0019-deger-semantigi-ve-gezme-imleci.md) | **geçici kabul** (makine kanıtı; usability bekliyor) | K-034, K-060, K-074, K-093, V1-P1-05 |
| RFC-0020 | [Paket Yayını ve Registry Güven Zinciri](0020-paket-yayini-ve-registry-guveni.md) | **geçici kabul** (K-094 yayın; K-095 metadata; K-117 kanonik yol; K-135 taşıma/cache/offline; K-136 exact proje/kilit/CLI) | K-094, K-095, K-117, K-135, K-136, V1-P1-07/08, ADR-006/028 |
| RFC-0021 | [İfade Grameri Büyüme Mimarisi](0021-ifade-grameri-mimarisi.md) | **geçici kabul** (K-097 katmanlar, K-119 formatter eşdeğerliği) | K-004, K-008, K-010, K-016, K-027, K-038, K-097, K-119 |
| RFC-0022 | [Deterministik IO Trace/Replay](0022-deterministik-io-izi.md) | **geçici kabul** (K-115 şema-1 + CLI; `zee-io-1` ilişkisi K-116) | K-115, K-116, B-027, V1-P0-25 |
| RFC-0023 | [Sürümlü Deterministik IO Profili](0023-deterministik-io-profili.md) | **geçici kabul** (`zee-io-1`, K-116) | K-116, B-028, V1-P0-26 |
| RFC-0024 | [Merkezî Yetkinlik ve Outbound Ağ Güvenliği](0024-merkezi-yetkinlik-ve-outbound-guvenligi.md) | **geçici kabul** (K-127) | K-127, B-023/B-024/B-049, V1-P0-29 |
| RFC-0025 | [Merkezî Kaynak Bütçesi](0025-merkezi-kaynak-butcesi.md) | **geçici kabul** (K-129/K-130/K-131/K-132/K-143; B-025/B-056 kapalı) | K-105, K-107, K-129/K-130/K-131/K-132/K-143, B-025/B-056, V1-P0-31/V1-P1-11 |

**Durum özeti (2 Eylül 2026, K-143):** 25 RFC — 2 kabul (0001, 0010),
21 geçici kabul (RFC-0020'nin yayın, metadata doğrulama, taşıma/cache/offline ve
exact proje/kilit/CLI katmanları çalışır), 2 taslak: 0012 (FFI — Faz
4/5) ve 0014 (standart kitaplık — çalışan prototiple).
Kural: bir RFC ancak yüzeyi gerçeklenmiş VE regression testine bağlanmışsa
geçici kabule geçer; tam kabul usability kapısından geçmeyi bekler.
