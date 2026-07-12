# backbone-corpus — PRD

Domain (Tier 5) · the **support-anchored knowledge base** · posts no GL · the deflection surface support/portal read.

## Why
The classic support-cost lever is article deflection: answer the customer before they open a ticket.
`backbone-corpus` is a lean KB — author an article, publish it, link it to the thing it explains (a support
Issue, a catalog Item), and serve only the reviewed ones back on the deflection read. It replaces a
mis-scaffolded org-directory stub (8 `organization_*` models, duplicating `backbone-organization`) that
squatted this module name with the wrong domain entirely — none of those models survive (`docs/erp/corpus-scoping.md`).

## Scope (KEEP — corpus-scoping.md §4)
- **Article** — the KB unit: `title`, `body` (markdown), publish lifecycle (draft → published → archived),
  `category_id`, a monotonic `revision` counter, `published_at`.
- **ArticleCategory** — a browse/routing taxonomy folder (e.g. Billing, Returns).
- **ArticleLink** — the seam-bearer: a **polymorphic logical FK** (`target_module` + `target_type` +
  `target_id`, optional `category_key`) tying an article to the thing it explains — a support `Issue`, a
  `WarrantyClaim`, a catalog `Item`. No DB FK; corpus never imports the target module.
- **ArticleFeedback** — a reader's "was this helpful?" vote — the deflection metric that proves the KB
  earns its keep.
- **The deflection read** — `suggest_for_target` / `suggest_for_category`: given a target, return the
  **published** articles linked to it. This is the load-bearing logic; everything else exists to feed it.
- **The deflection metric read** — `article_stats`: per-article helpful/not-helpful tally + ratio, so an
  operator can see which articles actually deflect.

## Non-goals (CUT / DEFER — corpus-scoping.md §4, council parking lots)
- **Editorial approval workflow / two-eyes review** — `publish_article` is a single verb, not a
  reviewer-gated pipeline. Gate: a customer that needs enforced review separation.
- **Stored revision history** — `revision` is a monotonic counter; an edit overwrites the body in place, no
  rollback. Gate: a revisions table when content audit/rollback is required.
- **Full-text search / ranking** — `suggest_*` return by recency (`published_at DESC`), not relevance. Gate:
  a search index when article volume outgrows category + link routing.
- **View/impression analytics** — `article_stats` is over voters, not viewers; a low-vote article is
  indistinguishable from a low-traffic one. Gate: an impression counter on the deflection read.
- **General CMS / publishing** (pages, posts, blog, campaigns) — that's `backbone-seo` overlay territory
  (public slug/canonical/robots/is_published targets an `Article` like any other publishable entity);
  marketing/campaign automation is already cut for SMB elsewhere. Corpus is internal + self-service help,
  not a CMS.
- **File attachments** — reuse `backbone-bucket` (`stored_file`/`file_version`) via a logical FK; never a
  new blob store. Article body is markdown text, not a blob.

## Success criteria
- A published article linked to a target is served by the deflection read; a draft or archived article is
  **never** served — the invariant a customer-facing surface cannot violate (maturity council 2026-07-11).
- The deflection metric (`article_stats`) is answerable from the public surface without a consumer
  re-summing raw `article_feedback` votes across the module boundary (completeness council 2026-07-11).
- Proven against a REAL `backbone-support` Issue (CSEAM-1): zero normal Cargo edge; support is a
  dev-dependency only. Posts no GL.
