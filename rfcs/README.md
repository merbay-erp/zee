# RFC süreci

Dil değişikliği RFC ister (master plan bölüm 25). Şablon: [0000-sablon.md](0000-sablon.md).

## Planlanan ilk RFC'ler (master plan bölüm 37)

| No | Başlık | Durum | Beslendiği günlük kayıtları |
|---|---|---|---|
| RFC-0001 | [Dil Manifestosu ve Tasarım İlkeleri](0001-dil-manifestosu.md) | **kabul** | — |
| RFC-0002 | [Lexical ve Unicode Kuralları](0002-lexical-ve-unicode.md) | **geçici kabul** | K-001, K-011, K-014, A07, A08 |
| RFC-0003 | [Girinti ve Blok Modeli](0003-girinti-ve-blok.md) | **geçici kabul** | A06 |
| RFC-0004 | [Değer Tanımı ve Kapsam](0004-deger-tanimi-ve-kapsam.md) | **geçici kabul** (K-034) | K-003, K-011, K-020 |
| RFC-0005 | [Koşullar ve Mantıksal İfadeler](0005-kosullar-ve-mantiksal.md) | **geçici kabul** | K-005, K-010, K-027, A03 |
| RFC-0006 | [İşlemler ve Parametreler](0006-islemler-ve-parametreler.md) | **geçici kabul** (K-086 tam public imza; çağrı yüzeyi usability kapısı) | K-016, K-032, K-083, K-086 |
| RFC-0007 | [Temel Tür Sistemi](0007-temel-tur-sistemi.md) | **geçici kabul** | K-009, K-014 |
| RFC-0008 | [Seçenek ve Sonuç](0008-secenek-ve-sonuc.md) | **geçici kabul** (daraltma dahil) | K-017, K-018, K-030 |
| RFC-0009 | [Modül ve Paket Modeli](0009-modul-ve-paket.md) | **geçici kabul** (birim + proje + yerel paket/kilit + public kaynak ABI); uzak registry taslak | K-029, K-076, K-078, K-086 |
| RFC-0010 | [Hata ve Tanılama Standardı](0010-hata-ve-tanilama.md) | **kabul** | K-026 |
| RFC-0011 | [Structured Concurrency](0011-structured-concurrency.md) | **geçici kabul** (K-085 deadline; paralellik Faz 5) | K-023, K-085 |
| RFC-0012 | [FFI ve Tehlikeli Sınır](0012-ffi-ve-tehlikeli-sinir.md) | **taslak** | A10 |
| RFC-0013 | [Ondalık Sayılar](0013-ondalik-sayilar.md) | **geçici kabul** | K-028 |
| RFC-0014 | [Standart Kitaplık](0014-standart-kitaplik.md) | **taslak — çalışan prototip ekli** | K-046, K-048, RFC-0002 §6.3 |
| RFC-0015 | [Uygulama Eylemleri ve Web Güvenlik Sınırı](0015-uygulama-eylemleri.md) | **geçici kabul — K-087 çekirdeği, K-088 açık** | K-081, K-087, ADR-010 |
| RFC-0016 | [Atomik Kalıcı Dosya Sözleşmesi](0016-atomik-kalici-dosya.md) | **geçici kabul** | K-019, K-084, V1-P0-04 |

**Durum özeti (1 Eyl 2026, K-084):** 16 RFC — 2 kabul (0001, 0010),
11 geçici kabul (yüzey gerçeklendi + korpusla/testle sabitlendi; onay kapısı
usability oturumları), 3 taslak: 0012 (FFI — Faz 4/5), 0014 (standart
kitaplık — çalışan dört-birimlik prototiple), 0015 (uygulama eylemi ve
production web sınırı).
Kural: bir RFC ancak yüzeyi gerçeklenmiş VE regression testine bağlanmışsa
geçici kabule geçer; tam kabul usability kapısından geçmeyi bekler.
