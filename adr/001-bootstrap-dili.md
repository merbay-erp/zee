# ADR-001 — Bootstrap implementasyon dili: Rust

- **Durum:** kabul
- **Tarih:** 31 Ağustos 2026

## Bağlam

Stage 0 derleyicisi için dil seçimi (master plan bölüm 11/12). Ölçütler:
bellek güvenliği, tek ikili dağıtım (okullar/çevrimdışı), Unicode desteği,
uzun vadede Cranelift/LLVM köprüleri, bakım maliyeti.

## Karar

**Rust** — ve bilinçli bir ek kısıt: **v0'da sıfır dış bağımlılık** (`docx`
değil `Cargo.toml`da tek satır bağımlılık yok; JSON, CSV, tarih aritmetiği,
rastgelelik elle yazıldı).

## Gerekçe

- Tek statik ikili: `dil` komutu kurulumsuz kopyalanabiliyor (bölüm 3:
  "10 dakikada Merhaba Dünya" hedefi).
- Sıfır bağımlılık = sıfır tedarik zinciri riski (bölüm 18) + her ortamda
  aynı derleme (reproducible build hedefine zemin).
- Cranelift/LLVM ekosistemi Rust'ta birinci sınıf (Faz 4 hazırlığı).
- Maliyet: geliştirme hızı C++'a göre değil ama Python'a göre yavaş;
  determinizm ve tanı kalitesi kazancı bunu karşıladı (23 commit'te 24 golden
  program çalışır durumda).

## Sonuçlar

Self-hosting (Faz 7) başlayana dek derleyici Türkçe konuşan ama Rust yazılan
bir eser olacak; katkıcı dokümanı bu ikiliği açıklamalı. Sıfır-bağımlılık
kısıtı stdlib ağ fazında (Faz 5) yeniden değerlendirilir (TLS elle yazılMAZ).
