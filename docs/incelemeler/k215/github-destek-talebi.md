# GitHub destek talebi — kullanıcı gönderecek

K-224, 14 Ekim 2026. Bu taslak K-214 taslağının güncel özel depo kapsamıdır.
**Gönderilmedi; GitHub destek teyidi yok; GB-046 açık.** Özel ürün kimlikleri,
ham kaynaklar, müşteri/alan/sunucu adları ve sırlar bu taslakta yer almaz.
Kullanıcı gerekli ekleri özel destek kanalında ayrıca gözden geçirir.

Subject: Confirm cleanup of managed PR refs and caches after source-history anonymization — merbay-erp/zee

Hello GitHub Support,

We own https://github.com/merbay-erp/zee. Business identifiers previously
appeared in product evidence and acceptance records, including pull requests
and older source transitions. The repository is private and will remain
private while we await your confirmation and a separate owner decision.

With owner authorization, we anonymized ordinary published source branches
and tags, preserved merge topology, refreshed source ownership records, and
removed legacy identifier exceptions from our history-scanning policy. Local
ref/hash and policy-scan records are available. These checks do not establish
that GitHub-managed PR refs, cached diffs, check-run views, or Actions copies
were removed on your systems.

Please inspect pull requests 1 through 7, including closed pull requests,
and the original commit objects retained by their head/merge refs and caches:
https://github.com/merbay-erp/zee/pulls

The original-to-anonymized commit maps and ref-verification records are in:

- docs/incelemeler/k214/sha-esleme-v1.tsv
- docs/incelemeler/k215/sha-esleme-v1.tsv
- docs/incelemeler/k215/ref-esleme-v1.tsv
- docs/incelemeler/k215/tarama.json

These identify the exact source lineage and ordinary rewritten refs. Historical
benchmark and build records retain the actual SHA of each past measurement;
they do not authorize retention of identifying original content in cached
objects. A separate private product repository retains its deployed source
pin and is outside this request.

Please confirm precisely which managed refs, cached commit/PR views, check-run
copies, and Actions logs/artifacts were purged, whether further ref removal or
garbage collection is needed, and which retained copies remain. If this
business-metadata category requires a different procedure, please tell us
which procedure to use. We can provide the original identifying terms through
the private support channel if needed; they are intentionally omitted here.

We will recheck the result against the exact listed lineage. No public exposure
will follow from cleanup alone; reopening requires a separate owner decision.
Thank you.
