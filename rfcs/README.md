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
| RFC-0006 | [İşlemler ve Parametreler](0006-islemler-ve-parametreler.md) | **geçici kabul** (usability onay kapısı) | K-016, K-032 |
| RFC-0007 | [Temel Tür Sistemi](0007-temel-tur-sistemi.md) | **geçici kabul** | K-009, K-014 |
| RFC-0008 | [Seçenek ve Sonuç](0008-secenek-ve-sonuc.md) | **geçici kabul** (daraltma dahil) | K-017, K-018, K-030 |
| RFC-0009 | [Modül ve Paket Modeli](0009-modul-ve-paket.md) | **geçici kabul** (birim); paket taslak | K-029 |
| RFC-0010 | [Hata ve Tanılama Standardı](0010-hata-ve-tanilama.md) | **kabul** | K-026 |
| RFC-0011 | [Structured Concurrency](0011-structured-concurrency.md) | **geçici kabul — yüzey** (paralellik Faz 5) | K-023 |
| RFC-0012 | [FFI ve Tehlikeli Sınır](0012-ffi-ve-tehlikeli-sinir.md) | **taslak** | A10 |
| RFC-0013 | [Ondalık Sayılar](0013-ondalik-sayilar.md) | **geçici kabul** |
| RFC-0014 | [Standart Kitaplık](0014-standart-kitaplik.md) | **taslak — çalışan prototip ekli** | K-046, K-048 | RFC-0002 §6.3 |

**Durum özeti (31 Ağu 2026 akşamı, K-043):** planlanan 12 RFC + ek 0013 —
2 kabul (0001, 0010), 10 geçici kabul (yüzey gerçeklendi + korpusla/testle
sabitlendi; onay kapısı usability oturumları), 2 taslak: 0012 (FFI — Faz
4/5) ve 0014 (standart kitaplık — çalışan dört-birimlik prototiple).
Kural: bir RFC ancak yüzeyi gerçeklenmiş VE regression testine bağlanmışsa
geçici kabule geçer; tam kabul usability kapısından geçmeyi bekler.
