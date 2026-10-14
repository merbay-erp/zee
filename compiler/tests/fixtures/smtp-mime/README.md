# K-232 adayı SMTP TLS test kimliği

Bu CA/PKCS12 yalnız localhost sahte SMTP testleri için 14 Ekim 2026'da
üretildi. Hiçbir gerçek sunucu, müşteri veya sır yoktur. PKCS12 parolası
herkesçe bilinen `zee-fixture`; private key sentetik test kimliğidir.
CA yalnız test connector'ına eklenir; production OS trust/hostname kontrolü
kapanmaz. SAN localhost/127.0.0.1/::1'dir.

Leaf 14 Ekim 2026–15 Kasım 2027 (397 gün) geçerlidir. Uzun 2036 HTTP test
leaf'i native_tls/macOS maximum-validity kapısından geçmedi; bu gerçek
negatif sonuç gizlenmedi ve doğrulama devre dışı bırakılmadı.
Expiry öncesi fixture yenilemesi ve exact kaynak fingerprint incelemesi
gerekir. Repo'ya almadan root tedarik/GB/test kayıtlarını aynı değişiklikte
hazırlar; genel private-key dedektörü kapatılmaz.
