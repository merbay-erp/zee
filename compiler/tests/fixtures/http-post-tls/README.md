# HTTP POST test TLS kimliği

Bu CA ve PKCS12 yalnız hermetik localhost testleri için yeni üretildi; hiçbir gerçek sunucu, müşteri veya sır içermez. CA yalnız unit test Agent’ında güvenilir, production trust store’a eklenmez. server.p12 yalnız bilinen `zee-fixture` test parolasını taşır, test private key’i herkesçe bilinir; üretimde kullanılmaz. Native TLS doğrulaması kapatılmaz. Sertifika 2036’ya kadar geçerlidir.
