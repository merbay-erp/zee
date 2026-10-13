# GitHub destek talebi taslağı — ürün tarafı iletecek

Subject: Purge cached business identifiers after complete published-history rewrite — merbay-erp/zee

Hello GitHub Support,

We own https://github.com/merbay-erp/zee. Customer/business identifiers,
domains, host aliases and deployment paths appeared in product evidence and
acceptance records, including PRs #5/#7 and older product transitions.
The repository is private, has zero forks and no Pages deployment. Our current
review found no real password, customer IP, private key or credential involved.

With owner authorization we anonymized all ordinary published branches and
tags using git-filter-repo, preserved merge topology, refreshed source ownership
manifests, and removed all legacy customer-line exceptions from the CI gate.
Current PR titles/bodies/comments were checked; they contain no matches under
the reviewed policy. There are no GitHub releases. This is not proof that old
PR refs, cached diffs, check-run views or Actions artifacts have been deleted.

Please purge inaccessible original commit objects, cached commit/PR views and
managed PR head/merge refs retaining the original identifying content. All
seven PRs have ancestors within the rewritten source lineage; please inspect
https://github.com/merbay-erp/zee/pull/1 through /pull/7, including closed PRs.

Particularly affected:
- PR #5 original head 0a8d1ba197d01fbcf87927ee7a26b89682807767;
  original merge a0f36685356bfc5ce8cf6fb598e4b7cae65e50db.
- PR #7 original head 179fe78f1c446621b584abb8ddb3a6e3e996d591;
  original merge 45b66f3574e0be020aeffc95580dd8db822e1379.
- PR #6 previously anonymized head 5cc127f4fba8a78b232d9e5a5f6c1515b884b784
  also retained older product ancestors and has now been rewritten again.

Attach both commit maps: docs/incelemeler/k214/sha-esleme-v1.tsv (original
today source to current anonymous source) and docs/incelemeler/k215/sha-esleme-v1.tsv
(previous published lineage to current anonymous source). The ref mapping
and verification record are in docs/incelemeler/k215.

Dependency-retention tags in this public-source repository now retain the
anonymized package-source genealogy. Historical benchmark/build/artifact
records keep the SHA of the actual past measurement; these strings do not
require retaining the identifying original content in public cached objects.
An independent private product repository keeps its deployed source pin;
this support request concerns only merbay-erp/zee, not that private repository.

Please also inspect Actions logs/artifacts and check-run caches attached to
original commit IDs. We archived original metadata privately. We can supply
original identifiers through the private support channel if needed; they are
intentionally omitted here. Please advise whether this business-metadata
category needs an alternative support procedure.

Confirm exactly which refs/caches/objects were purged, whether further ref
removal or garbage collection is needed, and whether cached Actions copies
remain. We will keep the repository private pending confirmation and separate
owner approval before reopening. Thank you.
