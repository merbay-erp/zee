# ADR-117 — Sınırlı SQL dispatcher sahipliği ve log basıncı

- **Durum:** kabul, sınırlı SQL-only host pilotu
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-258, GB-079

Ürün talebi B1. Tam P09 kapanışı değildir.

Mevcut K246 süreç grubu sahibi doğrudan kullanılır; WNOWAIT leader PID yetkisini
alt grubun kapatılmasına kadar korur. Başarısız leader çıkışı, stdout'u açık
bırakan alt süreç bulunsa da heartbeat veya yeni işi sürdürmez. Yalnız verilen
Popen handles kapatılır; yabancı port/PID benimsenmez.

Mevcut K245 credential okuyucusu yalnız açık eşlenmiş değerleri, özel dosya kökü
ve nofollow denetimiyle aktarır. CA PEM ayrı sınırlı güven yapılandırmasıdır.
Log yazımı mevcut K246 PIPE_BUF sınırı ve nonblocking writer ile atomik kayıttır;
dolu pipe kaydı düşürür ve sayaçla ölçer; SQL ilerlemesini durdurmaz.

Handler yalnız mevcut SQL niyet tablosuna sabit sorgu ve veri yazar. Belirsiz
COMMIT/sinyal/timeout sonrası replay veya fail yazımı yapılmaz. Bilinen rollback
sonucu üç deneme/deadletter sınırına gider. 1..100 tick pilotu daemon/cron değildir.

Gerçek11 vaka native-proof.json içinde; final temiz kaynak kabulü ve gerçek ürün
handler benimsemesi açık kalır. Spec/RFC/ADR indeksleri, sürümler, güvenlik defteri,
backlog, korpus ve faz kaydı ortak entegrasyon dalgasında root sorumluluğundadır.

Review revision2: normal argparse invalidargument leaked literal argv to stderr;
MaskedParser.error now routes to fixed rejected JSON/exit2, disables abbreviations.
Actual --ticks canary and unknownoption canary both stderr0/noargvusage/oneJSON.
Review1 historical manifests/source/proof retained; new exact source native11 passed.

## K-309 — job dispatcher harness cleanup ve özgün PG sahibi: dar kapsamda kapalı

K-309 / GB-118 dar kapsamda kapalı: Job SQL dispatcher native kabul harness cleanup/sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/job11 frozen kabulü; özgün PG/readiness/cleanup ve yalnız ilk vaka iki host0 gözlemi doğrulandı. Baseline2, child-free tek beklenen failure ve kontrollü standalone44/ordinary3 ayrı kapsamda korunur. Diğer metrics/effect iki PG tüketicisi açık; genel P09 kısmi ve39plan statüsü mevcut değerlerindedir.

**Gerçek kaynak kabulü:** Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96, tree 1f055acaa194c9eb47c51f25c74f85cf8803b273; tam15 kaynak kapısı 1248.429sn ve temiz tested-source commit/özgün owners doğrulandı. Tam faz998/1023 geçti,0 başarısız,25ignored kendi kaynak koşusudur. K309 source4+OPENdocs11 gerçek atomik uygulama ve bağımsız makbuzlarıyla bağlıdır. Standalone44(genericPG18+job26), ordinary3(oldmask2+tek44wrapper), baseline2 ve native-driver pure15 hazırlığı ayrı kapsamlardır; Cargo/native toplamına eklenmez. İlk combined3 fixture1 eksik üç public import bağımlılığı nedeniyle2maskfailure/exit1/reaped kaldı; fixture2 kopyaları tamamlandı, Proposal3 baytları değişmedi; kaynak/native kusuru olarak gösterilmez.

**Gerçek native11 kabulü:** Yalnız bu exact kaynakta canonical tek ignored Cargo sahibi/mevcut job11 geçti; native sürücü 111.174sn. Kaynak348/production175,11case/32assertion/35s handler ve bağımsız heartbeat, TLSrequire ile scoped plaintextCOMMIT/no-replay ayrımı, fixture20 ve dokuz safe tool/raw kaydı frozen/root/peer ile bağlandı. PHC bahanesiyle gizli/atlanan komut boşluğu kabul edilmez; kalan24 ignored Cargo vakası bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback ve readiness özgün probe kapanışı doğrulandı; private runtime kaldırıldı, özgün dış owner0/reaped/errors[]/cancelled[] ve sahipli cleanup sınırları korundu. Host normal0/reaped/stderr0 makbuzları yalnız ilk vakadaki iki canonical-host-actor gözlemidir; tüm worker/host/component için bağımsız normal0 sonucu değildir. Public host0/reaped/stderr0 makbuzları yalnız ilk vakadaki iki canonical-host-actor gözlemini kapsar; tüm worker/host/component için bağımsız normal0 sonucu iddia edilmez. Diğer özgün sahipler için kapanış/reaped oracleları kendi kapsamındadır.

**Tarihsel kontrollü hazırlık:** Proposal3 dört harness/test kaynağı bağımsız static incelemede doğrulandı: korumalı handler kurulumu/restore, sticky iptal/ilk hata, bağımsız cleanup ve publication, özgün foreground PG/readiness adapter’ı. Mevcut11native vaka/32require assertion/35s heartbeat döngüsü ve TLSrequire/scopedCOMMIT ayrımı korundu. Bu kaynak/test/OPEN belge dilimi aynı atomik uygulama kapsamındadır; tam kaynak veya native kabul sonucu değildir. Gerçek standalone preflight44 (genericPG18+job-specific26) PASS/exit0/reaped, unittest0.331sn; yalnız inert sahiplik/cleanup/iptal/publication modelleridir, gerçek PG/native11 değildir. Değişmeyen mask baseline2/.099sn ayrı tarihsel kabul olarak korunur. Gerçek fixture2 combined ordinary3 PASS/exit0/reaped, unittest0.511sn: eski mask2+tek nested44 wrapper. İlk fixture1 koşusu3test/2maskfailure/exit1/reaped kaldı; root üç public production import bağımlılığını fixture’a ekledi, Proposal3 baytları değişmedi. Bu eksik fixture hazırlığı kaynak/native kusuru veya OS/PG sızıntısı değildir.44/3 sayıları toplanıp Cargo/native vaka yapılmaz. Exact Proposal3 `/private/tmp/zee-k300/k309-job-proposal-3/PREPARATION.json` SHA256 `2096a3744a2989830b6833748b7382dfd9f290a77fcd793f0139a2df250e6cbe`; static peer `29eed668c09d318f258d78a167df9525c6cc50a9af41161465ac475d91ee3070`; actual controlled peer `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559`; fixture2 `a7c866f4914d02db9b17b4b118a66bee6e9d78c48ac4b16844c84c3b33b8e309`. Dört source pini: `scripts/tests/job-dispatcher-native-test.py`=`ef6a3ab8f990c3148a09d3234fe8b8075b9fd12083742059139df2599be5f49d`, `scripts/tests/job-dispatcher-mask-test.py`=`92357774ae9a09d55a366d844a33711564df5a924c237bdb9051135529a4b21a`, `scripts/tests/job-pg-owner-preflight-test.py`=`158af9ca7f2a97d4499c36c3af5ed5befc73380195da98bdf30d1a76b80d630f`, `compiler/tests/job_dispatcher_testi.rs`=`bda79d9307573127ee62eff6931b5dad363bbf4f2b6c3c73052d00c5f8d5a3d1`. Baseline/odaklı saf modeller/ordinary wrapper/native11 ayrı kanıt kapsamlarıdır; Python yöntemleri Cargo veya native vaka toplamına eklenmez.

Child-free exact AST fake probe.close karşı örneği tek beklenen failure/exit1/reaped olarak tarihsel kalır: cluster.close ve proof.write callback denemeleri atlanmıştı; gerçek OS/PG sızıntısı veya diskte kalmış PASS gözlemi değildi. Baseline mask2 ayrı kabul olarak korunur.

Mevcut11 case kimliği/32require/35s sabit handler, en az6bağımsız heartbeat ve eski30s süresinin aşılması korunur. CLI TLSrequire/ownedCA sınırı ile yalnız owned loopback host-component COMMIT fault fixture’ındaki plaintext profil ayrıdır; unknown COMMIT/sinyal/timeout replay/fail yazımı üretmez. Fixture20 ve privacy-safe public tool kaydı sırası/tam envanter kontrolü ayrı exact source/native sözleşmesine bağlıdır; PHC gizliliği gerekçesiyle gizli/atlanan komut boşluğu veya ham credential verisi kabul edilmez.

Yalnız bu exact kaynakta mevcut tek ignored Cargo sahibi/job11 sonucu kullanılabilir; kalan24 ignored Cargo vakası bu kampanyada çalışmış sayılmaz. K306 dispatcher4/HTTP7, K307 SQL9, K308 starter14 ve tarihsel job11 sonuçları kendi kaynaklarında kalır.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-gates-1/PROOF.json` | `0391ffe6ed48eb5cfa4bb87cb4eb3520a4cdb0b4fdb40bfc43cdeacf95a71d3f` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-gates-controller-1.json` | `2b719cbf8871dd4adbaf6f68256718b985843334b7368ea6ee277b8159aeeec6` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-commit-1.json` | `d6a0fc6a1dbf6011901434eedc7f375e470bf5ecb5033f5222f2cd558f39d8b9` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-commit-controller-1.json` | `9b85a6a205d8aa03d5b566b0f787fdb5365dda87bd9f42819a5c0b626e932614` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-1/PROOF.json` | `c1a13da0dee5cdd49b1924ae5f85d47fca56c59bed26eac5a884b68e53a01e44` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-controller-1.json` | `39b250307822f60c7a22a030862201e541fc3fec9ec3e66e45e861fbdf00cc4f` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-1/native/proof.json` | `4892c0280317d206eac97ad2abe1491ecaa05cdd2bfc8875fe2bffcecaf387a0` |
| root_native_audit | `/private/tmp/zee-k300/K309-NATIVE-ROOT-AUDIT-1.json` | `4cfeb107b94d7cc62b877c74cffd622f13624d67795dda45bae4db95e112efe5` |
| native_peer_review | `/private/tmp/zee-k300/K309-NATIVE-PEER-REVIEW-1.json` | `f03d38cccc6bdb4e3b6446ef7818a03c484bc6ad9045a77da0d7f90a9b5b2dd1` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-frozen-acceptance-1.json` | `957b1bc09c54be91331d5eab10233f3fd3b410564e17a293cc3d4ab268a46714` |
| source_pins | `/private/tmp/zee-k300/k309-source-pins.json` | `e772138605af3354e93791740260897bad62a8d0ce13c7790101f2674eaf5f0c` |
| closure_template | `/private/tmp/zee-k300/K309-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `534d93dc60ae2739b4746ff1a08a3c7c800f9d32c8056438890ea1f8367dbe43` |
| closure_template_review | `/private/tmp/zee-k300/K309-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `72feb8424e1a7dac1a316585264da93cf8cb15f93fe424e2e0291b94b3fe34c2` |
| K309-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K309-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `eea7ebd5b15cee1dea16846f2ffacc5a7220e5d56a3bd817c75e18a29e670653` |
| K309-JOB-PROPOSAL3-ACTUAL-MODELS-INDEPENDENT-REVIEW-2.json | `/private/tmp/zee-k300/K309-JOB-PROPOSAL3-ACTUAL-MODELS-INDEPENDENT-REVIEW-2.json` | `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559` |
| K309-NATIVE-JOB11-DRIVER-INDEPENDENT-STATIC-REVIEW-3.json | `/private/tmp/zee-k300/K309-NATIVE-JOB11-DRIVER-INDEPENDENT-STATIC-REVIEW-3.json` | `7af1483c3d4a857c4d021d791f8ab6c50bfb867edc0fb4776e307a32ae7fd985` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K309-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `9a9f30bd151ae85feafb77688d656a56ed6cc656fd992c06018a6554f1926dcf` |

Bu yalnız job diliminin dar kapanışıdır; metrics/effect iki PG tüketicisi açık kalır.39plan statüsü korunur; P09/P13/P20/P23/P36 kısmi/P37sürekli ve ürün/adoption/provider/fiziksel platform/dış proje/RCsoakIDE/C1K288P33K293 ret sınırları değişmez.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans ve compiler/LSP kalıcı durum değişmez. Ürün komutları/göç yolu değişmediğinden `docs/job-dispatcher.md`, ADR-114/126/139 ve mevcut kaynak-pinli rehberler bilinçli unchanged tutulur; ADR-117 dar harness güvenlik sınırını ve mevcut SQL-only pilot durumunu taşır. PHC/parola/cookie/request/cert/key/DSN/credential özel baytları veya hash’leri public kayda alınmaz; private runtime veya eski PID/kök aranmaz ve benimsenmez.
