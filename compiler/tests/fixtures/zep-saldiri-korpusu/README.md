# `.zep` saldırı korpusu

Bu klasör, paket yolu doğrulayıcısının kalıcı ve gözden geçirilebilir saldırı
girdilerini taşır. `yollar-v1.tsv` içindeki Unicode değerleri görünmez veya
yanıltıcı karakterleri kaynak dosyaya doğrudan gömmemek için kod noktası olarak
yazılır.

Her veri satırı dört sekmeyle ayrılmış alandır: vaka adı, `tam` ya da `iç`
yerleştirme türü, boşlukla ayrılmış hex Unicode kod noktaları ve beklenen hata
parçası. Korpus; klasik traversal/ayraç saldırılarını, NFD yolu, Unicode 17.0
UTS #39 `/`, `\\`, `.`, `:` benzerlerini ve görünmez yön denetleyicilerini
kapsar. Satır sayısı testte ayrıca sabittir; vakanın yanlışlıkla silinmesi de
kapıyı kırar.

Yol saldırılarının yanında uzunluk, yinelenen/sırasız girdi ve fazladan byte
vakaları `tedarik.rs` birim testlerinde yapısal olarak üretilir. Korpus veya
reddetme kümesi değişirse RFC-0020, ADR-006/028 ve spec/18 aynı committe
güncellenmelidir.
