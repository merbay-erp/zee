# ADR-003 — İlk yürütme modeli: ağaç-yürüyen yorumlayıcı + IO soyutlaması

- **Durum:** kabul
- **Tarih:** 31 Ağustos 2026
- **Revizyon:** 1 Eylül 2026 — K-099/ADR-012 ile cümle yürütme ve ifade
  değerlendirme handler'ları fiziksel modüllere ayrıldı

## Bağlam

Semantiği hızla doğrulamak için ilk yürütme modeli (master plan bölüm 11:
"önce interpreter ile semantiği doğrula, native backend'i erteleme" — risk
kaydındaki uyarı).

## Karar

**Ağaç-yürüyen yorumlayıcı**, tek belirleyici mimari eklentiyle:
**GirdiCikti soyutlaması** — ekran, klavye, rastgelelik, saat, dosya sistemi
ve komut satırı argümanlarının TAMAMI tek trait arkasında.

## Gerekçe

- Bytecode/VM, semantik henüz akışkanken erken optimizasyondu; ağaç yürüyüşü
  golden korpusu günler değil saatler içinde çalışır hale getirdi.
- IO soyutlaması determinizm ilkesini test altyapısına taşıdı: 63 testin
  hepsi sabit zaman (31 Ağu 2026 14:30), kuyruklu "rastgele", sahte dosya
  sistemiyle koşuyor; `dil dene` kullanıcı testlerini de hermetik koşuyor.
- "Programı bitir" gibi akış istisnaları iç nöbetçi tanıyla (Ç000) çözüldü —
  Akis enum'unu şişirmeden.

## Sonuçlar

- Performans hedefi yok (bölüm 22: önce doğruluk); benchmark bütçeleri
  bytecode/native geçişinde (ADR-005) tanımlanacak.
- Yorumlayıcı, gelecekteki backend'ler için referans semantik kaynağıdır:
  differential test (bölüm 21) yorumlayıcıya karşı koşacak.
- İşlem çağrısında gövde klonlanmıyor; derin özyineleme Rust yığınını
  kullanırdı — v0'da özyineleme zaten yasak (T016), kaldırılırken yığın
  derinliği sınırı eklenecek.
