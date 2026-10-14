# zee — dışarı HTTP istemcisinde POST + başlık + gövde (14.10.2026)

Talep eden: ürün tarafı (zee-cekirdek). Uygulayacak: zee dil geliştirme (Codex).
Bağlam: [`2026-10-13-tek-merkez-ve-inceleme.md`](README.md) §3 madde 3 ("dışarı HTTP istemcisi yalnız GET").
(Bu belge açık depoya kopyalanabilir: müşteri/alan/sunucu adı ve gerçek anahtar içermez.)

## 1) Gerçek ürün ihtiyacı — herkese açık formlarda insan doğrulaması (Cloudflare Turnstile)

Ürünün herkese açık POST formları bugün yalnız bal küpü alanı + CSRF + oran sınırı ile korunuyor; sunucu tarafı insan
doğrulaması yapılamıyor, çünkü Turnstile jetonu ancak **POST** ile doğrulanır:

| Dosya (zee-cekirdek) | Rota | Bugünkü koruma |
|---|---|---|
| `cekirdek/kaynak/rota_site.dil:20` | `POST /iletisim-gonder` | `website` bal küpü (s.28), CSRF + `guvenliMesajKaydet` sınırı (s.33); kayıttan sonra yöneticiye e-posta |
| `moduller/pdf-katalog/kaynak/rota_sm_katalog.dil:11` | `POST /katalog-talep-gonder` | aynı; onaylanınca talep edene PDF ekli e-posta gider |
| `moduller/teklif-formu/kaynak/rota_sm_teklif.dil:6` | `POST /teklif-talep` | aynı |
| `moduller/randevu/kaynak/rota_sm_randevu.dil:7` | `POST /randevu-talep` | aynı |
| `moduller/kariyer/kaynak/rota_sm_kariyer.dil:6` | `POST /kariyer-basvuru` | aynı |
| `moduller/ebulten/kaynak/rota_sm_ebulten.dil:10` | `POST /ebulten-kayit` | aynı |

Kanıt: bir üretim sitesinin tasarım taslağında Turnstile istemci tarafı hazır (site anahtarı meta etiketi + açık render),
ama zee'ye geçişte sunucu doğrulaması yazılamadığı için canlı formdan çıkarıldı. Formlar e-posta tetiklediğinden
(yönetici bildirimi, katalog gönderimi) bot kaydı doğrudan e-posta gönderen itibarına dokunuyor.

### Gerekli dış istek (Cloudflare'in belgelenmiş uç noktası)
```http
POST https://challenges.cloudflare.com/turnstile/v0/siteverify
Content-Type: application/x-www-form-urlencoded

secret=<TURNSTILE_GIZLI_ANAHTAR>&response=<istemciden gelen cf-turnstile-response>&remoteip=<istemci adresi>
```
Yanıt JSON: `{"success": true|false, "error-codes": [...], "hostname": "...", "action": "...", "cdata": "..."}`.
Kabul koşulu: `success = true` ve `hostname` sitenin kendi kökü. Gizli anahtar ortam değişkeninden okunur
(örn. `<SITE>_TURNSTILE_SECRET`), kaynak koda/göçe/günlüğe girmez.

### Ürün tarafında istenen kullanım (biçim önerisidir; söz dizimi dil tarafının kararı)
```
POST "/iletisim-gonder" adresine istek geldiğinde
    herkese açık
    jeton istek ve "cf-turnstile-response" ve "" ile urunIstekDegeri olsun
    dogrulama jeton ile insanDogrula olsun          # ↓ ürün işlemi
    dogrulama "gecti" eşit değilse
        "Doğrulama başarısız." yanıtını 403 durumuyla gönder
    ...

işlem insan dogrula
    jetonu al
    gizli "TURNSTILE_SECRET" ortam değişkeni olsun
    govde form kodlaması {"secret": gizli, "response": jeton, "remoteip": istemci adresi} olsun
    yanit "https://challenges.cloudflare.com/turnstile/v0/siteverify" adresine POST et
          başlık "Content-Type" "application/x-www-form-urlencoded"
          gövde govde
          zaman aşımı 5 saniye
    ...  yanit.durum, yanit.gövde (JSON) → success + hostname denetimi
```

## 2) İkinci gerçek kullanım — portal bildirimi (bugün kabuk betiği ile geçiştiriliyor)
`sunucu/zee-yedek.sh:32-35` ve `araclar/zee-filo.py:53-57`: ürün portalına `POST …/heartbeat` ve rapor gönderimi,
`Authorization: Bearer <PORTAL_JETONU>` + `Content-Type: application/json`. zee'den yapılamadığı için curl/Python'da.
```http
POST https://<portal>/api/v1/projects/<proje>/environments/<ortam>/heartbeat
Authorization: Bearer <PORTAL_JETONU>
Content-Type: application/json

{"stage":"production","name":"Canlı","backupStatus":"basarili","lastBackupAt":"2026-10-14T01:00:00Z"}
```

## 3) Dil tarafından beklenen asgari yüzey
- Yöntem: en az GET + POST (PUT/DELETE gerekmez).
- İstek başlıkları (Authorization dahil) ve gövde (form-urlencoded + JSON).
- Yanıt: durum kodu + gövde metni (JSON ayrıştırma mevcut araçlarla).
- Zaman aşımı zorunlu/varsayılanlı (ürün değeri 5 sn); hata = başarısız sonuç, istisna değil.
- Güvenlik: yalnız `https://`; gizli değerler hata mesajına/günlüğe/izlere yazılmaz; yönlendirme izlenmez ya da sınırlı;
  yanıt boyutu sınırı; (öneri) site başına izinli konak listesi — form doğrulaması tek konakla sınırlı.
- Test: yerel sahte sunucuyla başarılı/başarısız/zaman aşımı/yanlış hostname senaryoları.

Ürün kabulü: `/iletisim-gonder` Turnstile ile; geçersiz jeton 403, geçerli jeton kayıt + bildirim, Cloudflare erişilemezse
**kapalı başarısızlık** (mesaj alınmaz, ziyaretçiye "sonra tekrar deneyin").
