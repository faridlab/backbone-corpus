# backbone-corpus — FSD

## Entities
Article (`company_id`, `category_id?` FK, `title`, `body` markdown, `status`, `revision` int
default 1, `published_at?`; index `(company_id, status)`, index `(category_id)`) ·
ArticleCategory (`company_id`, `code`, `name`; unique `(company_id, code)`) ·
ArticleLink (`company_id`, `article_id` FK, `target_module`, `target_type`, `target_id` logical,
`category_key?`; index `(company_id, target_module, target_id)` — the deflection read; index
`(company_id, target_module, category_key)`; unique `(article_id, target_module, target_id)`) ·
ArticleFeedback (`company_id`, `article_id` FK, `helpful` bool, `note?`; index `(article_id)`).
Enum: ArticleStatus {draft (default), published, archived}.

## Write path (`CorpusWriteService`, hand-authored, user-owned)
- `create_category(NewCategory)` → a taxonomy folder, unique per (company, code)
- `create_article(NewArticle)` → always `draft`, `revision=1`
- `edit_article(article_id, title, body)` → overwrites content, `revision += 1`; returns the new revision
- `publish_article(article_id)` → CAS `draft → published`, stamps `published_at`; returns `false` if not
  currently draft (already published, archived, or missing)
- `archive_article(article_id)` → CAS `published → archived`
- `link_article(company_id, article_id, LinkTarget)` → idempotent per `(article_id, target_module,
  target_id)`; a re-link returns the existing link's id
- `record_feedback(company_id, article_id, helpful, note?)` → one vote row

### The deflection read (the load-bearing logic)
- `suggest_for_target(company_id, target_module, target_id) -> Vec<ArticleView>` — the published articles
  linked to a target, newest-published first. Filters `status='published'` AND excludes soft-deleted
  articles/links — a draft or archived article is **never** returned (maturity council 2026-07-11, CIP-1
  proven-by-revert: drop the status filter and a draft leaks).
- `suggest_for_category(company_id, target_module, category_key) -> Vec<ArticleView>` — the browse-by-topic
  variant, same publish gate, joined on `ArticleLink.category_key`.

### The deflection metric (completeness council 2026-07-11)
- `article_stats(company_id) -> Vec<ArticleStats>` → every article's `helpful`/`not_helpful` tally +
  `helpful_pct` (helpful ÷ total × 100 rounded, `None` when no votes), ordered by `helpful` desc. The KB's
  own aggregation — a consumer never re-sums raw `article_feedback` rows.

Errors: `CorpusError {Db, NotFound, InvalidState, Invalid}`.

## Seam (logical FK — zero normal Cargo edge)
- **`ArticleLink` → any target module (proven, CSEAM-1):** an article links to a target purely by
  `(target_module, target_type, target_id)` — no DB FK, no Cargo dependency on the target crate. Proven: a
  published KB article linked to a REAL `backbone-support` `Issue` is served back by `suggest_for_target`
  for that issue's id. `backbone-support` is a **dev-dependency only** (test-time), never a normal
  dependency of the shipped library.
- **Outbound:** none — corpus has no event sink today; it is a pure read seam.

## Test oracle
`corpus_golden_cases` (4: CGC-1 a published, linked article is served by the deflection read; CGC-2 linking
the same article+target twice is idempotent; CGC-3 an edit bumps the revision counter; CGC-4 the feedback
tally surfaces the deflection metric — 2 helpful + 1 not-helpful → `helpful_pct=Some(67)`),
`integrity_probes` (3: CIP-1 a draft article is never served, published it IS; CIP-2 an archived article is
withdrawn from the deflection read; CIP-3 the deflection read is scoped to company + target),
`corpus_support_seam` (1: CSEAM-1 a REAL `backbone-support` Issue surfaces its linked, published article)
+ `scripts/corpus_support_seam_roundtrip.sh` (§5). **8 focused tests.**
