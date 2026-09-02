# 24 — Kaynak güvenlik profili

Normatif kaynak: RFC-0025, ADR-033. Durum: **TANIMLI — K-129 ilk dilim**.

## Profil sahipliği

Resmî derleyici, runtime, CLI ve LSP aynı değişmez `KaynakSinirlari` profilini
kullanmak ZORUNDADIR. Program, paket veya ortam bu sınırları sessizce
yükseltemez. Sınırsız fallback ve başarılı görünen truncate YASAKTIR.

## Derleme

- Tek UTF-8 kaynak 8 MiB'ı ve 1.000.000 tokenı AŞAMAZ.
- Tek proje 4.096 `.dil` dosyası ve toplam 128 MiB kaynak metnini AŞAMAZ.
- Kaynak veya token aşımı S045'tir; parser/AST kurulmadan durmalıdır.

## Çalışma

- Çağrı derinliği 500'ü aşarsa C019 üretilir.
- Tek çalışma veya web isteği 10.000.000 cümle adımını AŞAMAZ.
- Tek liste/sözlük 1.000.000 öğeyi; tek eşzamanlı grup 1.024 görevi AŞAMAZ.
- `yaz`, soru istemi ve web yanıtı toplam 16 MiB veya 100.000 çıktı olayını
  aşarsa aşan olay IO'ya verilmeden C023 üretilir.
- Dil programının metin dosyası okuması 16 MiB'ı AŞAMAZ. Atomik append ve
  transaction ön-okuması aynı bounded reader'ı kullanmalıdır.

Uzun yaşayan web sunucusunda her istek bağımsız adım/çıktı bütçesi alır.

## LSP

LSP sunucusu en çok 256 açık belge ve toplam 128 MiB belge metni saklar. Tek
outbound JSON mesajı 8 MiB'ı aşamaz. Fazla belge sunucu durumuna eklenmez;
önceki belge sürümü korunur. Aşım S045 bildirimi veya kimlikli istekte JSON-RPC
`-32001` kaynak hatası olarak görünür.

## Henüz tamamlanmayan kapsam

K-129 tek değer/listenin bütün iç metinleriyle yaklaşık heap byte toplamını ve
aynı anda açık bağlantı sayısını henüz ölçmez. Bu alanlar B-025 açık kapsamıdır;
spec bu kaynakların sınırsız olduğuna dair güvence vermez.
