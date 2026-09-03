# ADR-053 — Compiler semantic değişiklik beyanı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-155A, B-064

## Bağlam

K-155/ADR-052'nin ilk koruğu, `compiler/src` commit başlığında `fix`, `bug`,
`duzelt` benzeri kelimeler arayarak yeni regression fixture'ını zorunlu
kılıyordu. Bu mekanizma `fix(parser): ...`, “correction” veya bütünüyle başka
bir başlıkla atlanabiliyordu. Dolayısıyla provenance verisi doğru olsa da
“fixture'sız bug fix geçemez” iddiası mekanizmadan daha güçlüydü.

Commit mesajı geliştirici niyetini anlatabilir; semantic güvenlik sınırı
olamaz. Kapsama giren her kaynak değişikliği, başlığından bağımsız ve tekil
olarak sınıflandırılmalıdır.

## Karar

1. Başlangıç commit'i
   `1c73298dca6bdbe27fc652daeb940b53709e1dbc` sonrasındaki **her**
   `compiler/src` değiştiren commit,
   `docs/compiler-degisiklik-beyanlari-v1.tsv` içinde tam SHA ile tam bir kez
   bulunur. Commit mesajına bakılmaz.
2. `semantic-bugfix`, kanıt alanında `regression/v2.tsv` vaka kimliği taşır;
   vakanın `fixed_by` SHA'sı beyan commit'iyle exact eşleşir.
3. `semantic-change`, var olan bir `spec/`, `rfcs/` veya `adr/` Markdown
   kanıtına gider. Davranış değişikliği normatif bağ olmadan geçemez.
4. `maintenance`, semantic davranışın değişmediğine dair en az 40 karakterlik
   açık gerekçe ve `kanıt=-` taşır. Bu bir sessiz muafiyet değil, incelenebilir
   beyan kaydıdır.
5. Eksik/yinelenen beyan, bilinmeyen sınıf, kısa gerekçe, yanlış kanıt,
   fixture ile eşleşmeyen bugfix veya `compiler/src` değiştirmeyen sahipsiz
   beyan fail-closed reddedilir.
6. Bir commit kendi SHA'sını önceden bilemeyeceği için akış iki atomik adımdır:
   önce kaynak commit'i; sonra beyan ve gerekiyorsa fixture/provenance commit'i.

## Reddedilen seçenekler

- **Conventional Commit regex'ini genişletmek:** bilinen yazımları yakalar ama
  niyet farklı kelimeyle yeniden gizlenebilir.
- **Yalnız diff büyüklüğüne/dosya adına bakmak:** semantic etkiyi mekanik
  değişiklikten güvenilir biçimde ayıramaz.
- **Her kaynak değişikliğine regression fixture istemek:** refactor, ölçüm ve
  iç bakım commit'leri için sahte bug geçmişi üretir.
- **Serbest muafiyet yorumu:** tekillik, kapsam ve bayat kayıt denetlenemez.

## Sonuçlar

- K-155'in fazla güçlü cümlesi artık gerçek mekanizmayla eşittir: hiçbir
  `compiler/src` commit'i explicit semantic sınıf beyanı olmadan geçemez;
  `semantic-bugfix` ise exact fixture olmadan geçemez.
- Gerçek geçici Git deposu testi, anlamsız/masum görünen commit başlığıyla da
  eksik beyanı reddeder; exact beyan+fixture kabulünü ve provenance yeniden
  yazım reddini uçtan uca kanıtlar.
- Grammar, runtime, tanı anlamı, RFC ve normatif spec bu düzeltmede değişmez.
