# K-250–K-256 temiz teknik kontrol noktası

15 Ekim İstanbul tarihiyle kayıt edilen yerel kabul, exact temiz
`d42f14747006aa0c50335d3e7c8ea0a1026141a8` kaynak kopyasında alındı.
K-253–K-256 kaynak dalgası `ba719497669823033c665a6bdceeebde3bac8f3e`;
K-253 semantic-change/dogfood-change beyanı takip d42f147 commit'indedir.
K-250–K-252'nin önceki [021bff4 kabulü](incelemeler/urunlesme-20261014/common-021bff4/README.md)
özgün kaynak kimliğini korur.

| Kapı | Gerçek sonuç | Kanıt |
|---|---|---|
| Tam faz | 904/913 başarılı, 0 başarısız, 9 dedicated atlama; 190.348 sn faz toplamı | [Faz](incelemeler/urunlesme-20261014/common-d42f147/faz-test-matrisi.md) |
| Ayrı PostgreSQL | TLS/statement-fault8, göç1, parola/COMMIT11, SQL queue21; 41/41, sıfır atlama | [Bağımsız özet](incelemeler/urunlesme-20261014/common-d42f147/independent-review.json) |
| DB hazırlık | 13/13 native, strict wrapper normal fazda | [Bağımsız özet](incelemeler/urunlesme-20261014/common-d42f147/independent-review.json) |
| Native SIGTERM | Normal handler200, TERM sonrası mevcut handler200/exit0, yeni handler yok, idle exit0/125ms | [Actual drain](incelemeler/urunlesme-20261014/common-d42f147/native-drain.json) |
| Host ve görünüm | Credential14, worker25; Python3.9/3.13'ün her birinde TR–EN7 native+4 Zee | [Bağımsız özet](incelemeler/urunlesme-20261014/common-d42f147/independent-review.json) |
| Sürekli kapılar | fmt/spec205/clippy/güvenlik/sır ve semantic/core-freeze geçti | [Bağımsız özet](incelemeler/urunlesme-20261014/common-d42f147/independent-review.json) |

Bağımsız inceleme 63 ham dosya hashini doğruladı. Root ayrıca proof,
bütün 20 komut günlüğü ve ikili hashini yeniden doğruladı. Ham proof
SHA-256 `3c629371ccd9e19746398bc2ea5e38d96ec8c7276937ff6ee93c8df922b2685f`;
debug ikili SHA-256
`d5ce06ee02368e78f5335bf85a86bd4192c6ce0711234c996fdc18daae6db379`.
BuildJSON exact kaynak ve dirty=false, kaynak başlangıç/sonu temiz ve aynı;
owned fixture temizliği kayıtlıdır. Ham günlükler depo dışındaki kalıcı
kanıt dizinindedir. Alt Python senaryoları Cargo toplamına eklenmez.

K-256 capable-positive kontrolünde gerçek TLS1.1 bağlantısı, CA/hostname
doğrulaması ve sunucunun pg_stat_ssl gözlemi vardır; ardından native
minimum TLS1.2 yolu reddeder. K-254 ayrı lease satırını gerçek uzun
transaction sırasında yeniler; bu SMTP teslimi veya genel dispatcher
kabulü değildir. K-255'in temiz021bff4 LSP25/2 ölçümü kendi kaynak ve
makine kimliğini korur; bu kontrol noktasına yeniden etiketlenmez.

Zee/Rust/Go kanıt özeti temiz d42f147 üzerinde üç turda bayt bayt eş çıktı
üretti; [karşılaştırma tarihçesi](dogfood-karsilastirma.md) actual UTC
tarihini ve exact kaynak SHA'sını korur. Başlangıç zee ve gerçek
zee-cekirdek checkout'ları temiz tutuldu; çalışma ayrı onaylı daldadır.

K-253'ün final1800s soak kapısı ve final kaynakta RC ASan/Miri henüz
çalıştırılmadı. Metrik kanalı, devralınan listener/overlap ve operasyonel
dispatcher adaylarının kendi kod/test/karar kabulü sürer; bu rapor onları
tamamlanmış göstermez. İnsan, fiziksel platform, tam anonim P08 korpusu,
lisans, uzak CI ve release kapıları açık; yerel native kabul onların
yerine geçmez. Yeni iki-klon sürüm ikilisi bu kontrol noktasında üretilmedi.
