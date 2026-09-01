# ADR-001 — Bootstrap implementasyon dili: Rust

- **Durum:** kabul
- **Tarih:** 31 Ağustos 2026; bağımlılık ilkesi revizyonu 1 Eylül 2026

## Bağlam

Stage 0 derleyicisi için dil seçimi (master plan bölüm 11/12). Ölçütler:
bellek güvenliği, tek ikili dağıtım (okullar/çevrimdışı), Unicode desteği,
uzun vadede Cranelift/LLVM köprüleri, bakım maliyeti.

## Karar

**Rust** — ve bilinçli bir ek kısıt: **küçük, gerekçeli ve kilitli dış
bağımlılık yüzeyi**. Dil yüzeyi/grammar, JSON, CSV ve tarih aritmetiği gibi
zee'ye özgü anlamlar elde yazılır. Kriptografi/işletim sistemi güvenliği ile
kanıtlanmış genel sayı altyapısı yeniden icat edilmez. Her yeni crate ayrı
gerekçe, `Cargo.lock` sabitlemesi, native+WASM derleme ve semver incelemesi
ister.

İlk bootstrap gerçekten sıfır bağımlılıkla başladı. K-088'de parola ve OS
rastgeleliği için güvenlik crate'leri; K-092'de keyfî hassasiyetli Ondalık
için `num-bigint`/`num-traits` eklendi. Bunlar dil sözdizimini veya gözlenebilir
platform davranışını dış kütüphaneye devretmez.

## Gerekçe

- Tek statik ikili: `dil` komutu kurulumsuz kopyalanabiliyor (bölüm 3:
  "10 dakikada Merhaba Dünya" hedefi).
- Küçük ve kilitli bağımlılık grafiği tedarik zinciri incelemesini yapılabilir
  tutar; kaynakta exact Ondalık ya da Argon2 gibi güvenlik çekirdeklerini
  yeniden yazmaktan daha düşük doğruluk riski taşır.
- Cranelift/LLVM ekosistemi Rust'ta birinci sınıf (Faz 4 hazırlığı).
- Maliyet: geliştirme hızı C++'a göre değil ama Python'a göre yavaş;
  determinizm ve tanı kalitesi kazancı bunu karşıladı (23 commit'te 24 golden
  program çalışır durumda).

## Sonuçlar

Self-hosting (Faz 7) başlayana dek derleyici Türkçe konuşan ama Rust yazılan
bir eser olacak; katkıcı dokümanı bu ikiliği açıklamalı. Bağımlılık eklemek
kolaylık gerekçesiyle normal yol değildir: dil semantiği içeride kalır,
kriptografi/TLS ve genel sayı ilkelleri uzman kitaplıktan alınabilir. TLS elle
yazılMAZ; aynı şekilde BigInt de ikinci kez icat edilmez.
