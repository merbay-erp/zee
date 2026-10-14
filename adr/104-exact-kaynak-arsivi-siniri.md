# ADR-104 — Exact kaynak arşivi ve çıktı sınırı

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-243, GB-066
- **Karar zinciri:** ADR-010/068/087, K-224/K-241; P01/P37

## Doğrulanan sorun

Kaynak betiği dosya listesi, dosya byte'ları, tarih ve arşiv için `HEAD`i
tekrar çözüyordu. Kontrollü gerçek Git deposunda eşzamanlı HEAD değişimi
exit 0 ile eski manifest/yeni ZIP üretti. Salt metin önekiyle denetlenen
çıktı yolu `..` veya sembolik ebeveynle depo dışına yazabildi. Git'in quoted
UTF-8 dosya listesi de Türkçe adları doğru byte yoluyla okuyamadı.

## Karar

Başlangıç commit SHA'sı tek kez çözülür. İsim, NUL kaynak listesi, blob'lar,
epoch, arşiv comment'i ve doğrulama aynı SHA'yı kullanır. Kirli caller veya
sonradan değişen HEAD bu snapshot'ın içeriğini değiştirmez. Git listesinde
satır sonu gibi manifestte temsil edilemeyen yol varsa yayın başlamaz.

Çıktı ebeveyni canonical depo sınırında doğrulanır; kaçış/symlink hedefi,
izin verilmeyen metadata dizini ve kaynak dosyasını ezme reddedilir.
Metadata ve izlenen kaynakların harf/physical identity eşdeğerleri de
değerlendirilir; case alias ve hard-link olumsuzları mevcut Cargo vakasındadır.
Geçici arşiv/özet korumalı aynı filesystem alanında hazırlanır. Arşiv, saf yerel
Git archive/ZIP/hash karşılaştırmasıyla doğrulanmadan son çıktıya taşınmaz.
Yanlış ZIP comment'i, eksik/fazla/duplicate veya hash'i değişmiş kaynak,
manifest uyumsuzluğu ve build/kişisel girdiler başarı sayılamaz.

Mevcut ZIP + `.sha256` dosya sözleşmesi korunur. İki ayrı dosyanın tek
filesystem transaction'ında atomik yayımlandığı iddia edilmez; tüketici
hash'i doğrulamalıdır. Eşzamanlı aynı kullanıcı tarafından bütün dizin
ağacının değiştirilmesine genel sandbox/transaction garantisi verilmez.
Python3 standart kitaplığı ve mevcut kaynak karşılaştırma helper'ı yerel
doğrulama için kullanılır; uzak CI kaydı veya ağ erişimi gerektirilmez.

## Kanıt ve etki

Mevcut `dokuman_tazelik_testi::kaynak_arsivi_build_ve_kisisel_artifaktlari_disarida_tutar`
gerçek geçici Git kampanyasını yürütür: HEAD değişimi, kirli caller, UTF-8
yol, yasak yol, bozuk arşiv ve çıktı sınırı olumlu/olumsuz kanıtları vardır.
Üretim compiler, grammar, runtime/API, bağımlılıklar ve benchmark iş yükü
değişmez. Kaynak ZIP kabulü signed binary release veya uzak CI kabulü değildir.
