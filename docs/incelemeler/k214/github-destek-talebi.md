# GitHub destek talebi taslağı — ürün tarafı iletecek

Subject: Remove sensitive business metadata from cached PR refs after history rewrite — merbay-erp/zee

Hello GitHub Support,

We own https://github.com/merbay-erp/zee. Private customer/business identifiers,
customer domains, host aliases and deployment paths were unintentionally copied
from a private product request and acceptance logs into pull requests #5 and #7.
We made the repository private while cleaning it. There are zero forks; no actual
password, private key or customer IP is involved, according to our current review.

We have anonymized the affected content, rewritten the affected history with
 git-filter-repo, updated ordinary branch references, and anonymized PR titles/
bodies. We are retaining the repository privately. Please remove cached views,
PR head/merge references and otherwise inaccessible objects that still retain the
original sensitive business metadata, then confirm the extent of server-side
purging. Please tell us if this category requires a different support workflow.

Affected PRs:
- https://github.com/merbay-erp/zee/pull/5
  original head: 0a8d1ba197d01fbcf87927ee7a26b89682807767
  original merge: a0f36685356bfc5ce8cf6fb598e4b7cae65e50db
- https://github.com/merbay-erp/zee/pull/7
  original head: 179fe78f1c446621b584abb8ddb3a6e3e996d591
  original merge: 45b66f3574e0be020aeffc95580dd8db822e1379

Privacy baseline: 12c59218dc84e8e9d4610274c722985ce6a99dae.
The full original-to-anonymized commit mapping is attached as sha-esleme-v1.tsv.
Please inspect related PR #6 cached parent diffs as its branch also descended
from the affected history. Dependency-retention tags refer to earlier unaffected
source and should remain available for our pinned dependency.

Affected examples include the private request copy under dogfood/zee-cekirdek,
product provenance documents, acceptance logs and review metadata. We can supply
original identifiers privately if support needs them; they are intentionally
omitted from this public-safe request draft.

Old source IDs in historical build/benchmark provenance represent past evidence,
not new measurements. They do not authorize retention of the original identifying
content in cached commit/PR views. Older separately reviewed public references
outside this rewrite scope are awaiting an explicit owner decision.

Please confirm which PR references/caches were removed and whether additional
ref removal or garbage collection is required before we consider reopening.
Thank you.

Please also inspect associated Actions/check-run caches and artifacts linked
to original commit IDs; historical CI metadata is archived privately and
original evidence remains explicitly historical. Ordinary branch rewrites
do not by themselves prove deletion of all GitHub-side copies.
