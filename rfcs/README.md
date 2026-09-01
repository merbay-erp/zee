# RFC süreci

Dil değişikliği RFC ister (master plan bölüm 25). Şablon: [0000-sablon.md](0000-sablon.md).

## Planlanan ilk RFC'ler (master plan bölüm 37)

| No | Başlık | Durum | Beslendiği günlük kayıtları |
|---|---|---|---|
| RFC-0001 | [Dil Manifestosu ve Tasarım İlkeleri](0001-dil-manifestosu.md) | **kabul** | — |
| RFC-0002 | [Lexical ve Unicode Kuralları](0002-lexical-ve-unicode.md) | **geçici kabul** | K-001, K-011, K-014, K-089, A07, A08 |
| RFC-0003 | [Girinti ve Blok Modeli](0003-girinti-ve-blok.md) | **geçici kabul** | A06 |
| RFC-0004 | [Değer Tanımı ve Kapsam](0004-deger-tanimi-ve-kapsam.md) | **geçici kabul** (K-034) | K-003, K-011, K-020 |
| RFC-0005 | [Koşullar ve Mantıksal İfadeler](0005-kosullar-ve-mantiksal.md) | **geçici kabul** | K-005, K-010, K-027, A03 |
| RFC-0006 | [İşlemler ve Parametreler](0006-islemler-ve-parametreler.md) | **geçici kabul** (K-086 tam public imza; K-096 karşılaştırmalı çağrı usability kapısı) | K-016, K-032, K-083, K-086, K-096 |
| RFC-0007 | [Temel Tür Sistemi](0007-temel-tur-sistemi.md) | **geçici kabul** | K-009, K-014 |
| RFC-0008 | [Seçenek ve Sonuç](0008-secenek-ve-sonuc.md) | **geçici kabul** (daraltma + yapılandırılmış Hata) | K-017, K-018, K-030, K-091 |
| RFC-0009 | [Modül ve Paket Modeli](0009-modul-ve-paket.md) | **geçici kabul** (birim + proje + yerel paket/kilit + public kaynak ABI); registry metadata güveni çalışır, uzak bağımlılık entegrasyonu açık | K-029, K-076, K-078, K-086, K-095 |
| RFC-0010 | [Hata ve Tanılama Standardı](0010-hata-ve-tanilama.md) | **kabul** | K-026 |
| RFC-0011 | [Structured Concurrency](0011-structured-concurrency.md) | **geçici kabul** (K-085 deadline; K-090 deterministik scheduler+sahiplik) | K-023, K-085, K-090 |
| RFC-0012 | [FFI ve Tehlikeli Sınır](0012-ffi-ve-tehlikeli-sinir.md) | **taslak** | A10 |
| RFC-0013 | [Ondalık Sayılar](0013-ondalik-sayilar.md) | **geçici kabul** (K-092 keyfî hassasiyet) | K-028, K-092 |
| RFC-0014 | [Standart Kitaplık](0014-standart-kitaplik.md) | **taslak — çalışan prototip ekli** | K-046, K-048, RFC-0002 §6.3 |
| RFC-0015 | [Uygulama Eylemleri ve Web Güvenlik Sınırı](0015-uygulama-eylemleri.md) | **geçici kabul — K-087 eylem, K-088 profil** | K-081, K-087, K-088, ADR-010 |
| RFC-0016 | [Atomik Kalıcı Dosya Sözleşmesi](0016-atomik-kalici-dosya.md) | **geçici kabul** | K-019, K-084, V1-P0-04 |
| RFC-0017 | [Web Oturumu, Yetki, CSRF ve Güvenilir Proxy](0017-web-oturum-ve-csrf.md) | **geçici kabul** | K-082, K-088, V1-P0-03 |
| RFC-0018 | [Sürümlü Morfoloji Profili](0018-surumlu-morfoloji-profili.md) | **geçici kabul** | K-011, K-072, K-089, V1-P1-02 |
| RFC-0019 | [Değer Semantiği ve Gezme İmleci](0019-deger-semantigi-ve-gezme-imleci.md) | **geçici kabul** (makine kanıtı; usability bekliyor) | K-034, K-060, K-074, K-093, V1-P1-05 |
| RFC-0020 | [Paket Yayını ve Registry Güven Zinciri](0020-paket-yayini-ve-registry-guveni.md) | **geçici kabul** (K-094 yayın; K-095 metadata güveni; taşıma/cache/CLI açık) | K-094, K-095, V1-P1-07, ADR-006 |

**Durum özeti (1 Eylül 2026, K-095):** 20 RFC — 2 kabul (0001, 0010),
16 geçici kabul (RFC-0020'nin yayın ve metadata doğrulama katmanı normatiftir;
taşıma/cache/CLI tamamlanmadan uzak paket kullanımı sözü verilmez), 2 taslak: 0012 (FFI — Faz
4/5) ve 0014 (standart kitaplık — çalışan prototiple).
Kural: bir RFC ancak yüzeyi gerçeklenmiş VE regression testine bağlanmışsa
geçici kabule geçer; tam kabul usability kapısından geçmeyi bekler.
