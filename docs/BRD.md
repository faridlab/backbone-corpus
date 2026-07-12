# backbone-corpus — BRD

## Documents
Article (the KB unit) · ArticleCategory (taxonomy) · ArticleLink (the polymorphic seam-bearer) ·
ArticleFeedback (the deflection metric). Own Postgres schema `corpus`. Posts **no GL**. Reaches no other
module — it is READ by consumers (support/portal) purely via `ArticleLink`'s logical FK.

## Business rules

**BR-1 (author as draft).** `create_article` always starts an article at `status=draft` — private,
editable, never served. There is no way to create a published article directly; publishing is a separate,
explicit transition.

**BR-2 (publish gate — the invariant).** `suggest_for_target` and `suggest_for_category` — the deflection
read consumers call — serve **only** `status='published'` articles. A `draft` or `archived` article must
never be returned, because it is shown to a customer as authoritative self-service help (maturity council
2026-07-11). `publish_article` is the CAS-gated `draft → published` transition (stamps `published_at`, once
only); `archive_article` withdraws a published article (`published → archived`).

**BR-3 (edit bumps revision).** `edit_article` overwrites title/body and increments `revision`. A published
article stays published across an edit — it is treated as a live fix, not a new draft; there is no stored
history of prior revisions (`revision` is a counter, not a log).

**BR-4 (link is idempotent).** `link_article` ties an article to a target
(`target_module`+`target_type`+`target_id`, optional `category_key`) via a unique
`(article_id, target_module, target_id)` constraint. Re-linking the same pair is a true no-op — it returns
the existing link's id, not a duplicate.

**BR-5 (feedback is the deflection metric).** `record_feedback` captures a reader's helpful/not-helpful
vote per article. `article_stats(company)` aggregates it into `helpful`, `not_helpful`, and `helpful_pct`
(rounded, `None` when no votes yet) per article — the number that proves the KB earns its keep, computed by
the module itself so no consumer re-sums raw votes across the boundary (completeness council 2026-07-11).

## Events
None yet — corpus is a read-mostly seam; consumers poll `suggest_for_target`/`suggest_for_category`
directly rather than subscribing to article lifecycle events.

## Deferred (with reason)
Editorial approval workflow, stored revision history, full-text search/ranking, view/impression analytics,
general CMS/publishing, file attachments (corpus-scoping.md §4, council parking lots — see PRD).
