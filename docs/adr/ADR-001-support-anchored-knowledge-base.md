# ADR-001 — The support-anchored knowledge base, the publish gate, and the polymorphic link seam

Status: accepted · 2026-07-11 · Domain (Tier 5; posts no GL)

## Context
`backbone-corpus` was a mis-scaffolded org-directory stub (8 `organization_*` models, duplicating
`backbone-organization`) squatting the wrong domain under the right name (`docs/erp/corpus-scoping.md`).
The classic support-cost lever is article deflection — answering a customer before they open a ticket —
and no BUILT module owned it: `backbone-support` has tickets but no answers, `backbone-portal` has a reader
but nothing to read. Corpus is rebuilt as a lean KB filling exactly that gap.

## Decision
1. **An article is always authored as `draft`.** There is no path to create a published article directly;
   `publish_article` is a separate, explicit, CAS-gated `draft → published` transition.
2. **The deflection read gates on `status='published'`, hard.** `suggest_for_target` and
   `suggest_for_category` — the only reads a consumer (support/portal) calls — filter
   `status='published'` and exclude soft-deleted rows. A draft is unreviewed content; serving it to a
   customer as authoritative help is the worst outcome for a support surface (maturity council 2026-07-11).
3. **Linking is a POLYMORPHIC LOGICAL FK, not a Cargo dependency.** `ArticleLink` ties an article to a
   target via `(target_module, target_type, target_id)` with no DB constraint. Corpus never imports the
   target module — proven by CSEAM-1 against a REAL `backbone-support` Issue; support is a dev-dependency
   only (test-time), zero normal edge.
4. **A link is idempotent.** Re-linking the same `(article, target)` pair is a true no-op, returning the
   existing link's id — a consumer can call `link_article` on every ticket-raise without dedup logic of
   its own.
5. **The deflection metric is a first-class read, not raw votes.** `article_stats` aggregates
   `ArticleFeedback` into helpful/not-helpful/`helpful_pct` per article inside the module — the number that
   proves the KB earns its keep (completeness council 2026-07-11).
6. **Posts no GL.** Corpus is read-mostly; there is no financial posting surface.

## Consequences
- Turn corpus off and support/portal simply serve no suggested articles — nothing else depends on it.
  Proven vs a REAL `backbone-support` Issue; survives regen (§5); zero normal Cargo edge.
- A draft can be authored and worked on freely without any risk of leaking to a customer, because the gate
  lives in the read, not in a permissions layer a caller could bypass.

## Parking lot (each with a gate)
- **A draft linked to an issue was servable** — FIXED (maturity council 2026-07-11): the deflection read
  gates on `status='published'`; a draft/archived article is never returned (CIP-1/CIP-2,
  proven-by-revert: dropping the filter makes the draft leak).
- **The deflection metric was recorded but unreadable** — FIXED (completeness council 2026-07-11):
  `record_feedback` had no corresponding read, so the module's whole justification (does the KB deflect
  tickets?) was unanswerable without a consumer re-summing raw votes across the boundary. Added
  `article_stats` (CGC-4, proven-by-revert).
- **No approval workflow / reviewer role** — publish is a single verb, not a two-eyes review. Gate: an
  editorial workflow when a customer needs enforced review separation.
- **No stored revision history** — `revision` is a counter; an edit overwrites the body. Gate: a revisions
  table when rollback/audit of article content is required.
- **No full-text search / ranking** — `suggest_*` return by recency, not relevance. Gate: a search index
  when article volume outgrows category + link routing.
- **No view/impression analytics** — `helpful_pct` is over voters, not viewers. Gate: an impression counter
  on the deflection read when funnel analytics are needed.
