# backbone-corpus — Extension Guide

## Public surface (stable)
- **Write path** (`application::service::corpus_write_service::CorpusWriteService`): `create_category`,
  `create_article`, `edit_article`, `publish_article`, `archive_article`, `link_article`,
  `record_feedback`, with `NewCategory`/`NewArticle`/`LinkTarget`.
- **The deflection read**: `suggest_for_target(company_id, target_module, target_id)`,
  `suggest_for_category(company_id, target_module, category_key)` → `Vec<ArticleView>` (only published
  articles, ever).
- **The deflection metric read**: `article_stats(company_id) -> Vec<ArticleStats>` (per-article
  helpful/not_helpful/helpful_pct).
- **The link seam**: `ArticleLink` — a polymorphic `(target_module, target_type, target_id)` logical FK,
  no DB constraint, no Cargo dependency on the target crate.

## How a consuming module (support/portal) uses corpus
Call `link_article(company_id, article_id, LinkTarget { target_module: "support", target_type: "issue",
target_id: issue_id, category_key: Some(issue.category) })` when you want an article associated with a
record you own. To surface suggestions on that record, call
`suggest_for_target(company_id, "support", issue_id)` (or `suggest_for_category` for a browse-by-topic
list). Both reads only ever return `published` articles — you never need to check status yourself. Never
insert into `corpus.*` tables directly; corpus owns its own schema.

## Not a contract
- The generated CRUD endpoints per entity are convenience scaffolding. Do **not** flip an article's
  `status` or insert an `article_links` row through the generic PATCH surface — it bypasses the CAS gate
  on publish/archive and the idempotent-link uniqueness handling. Use `CorpusWriteService`.
- `// <<< CUSTOM` blocks preserve local edits only; not a cross-module extension point.

## Invariants a consumer must not break
- Only `published` articles are ever served on `suggest_for_target`/`suggest_for_category` — a draft or
  archived article must never reach a customer. Do not build a parallel read path that skips the status
  filter (e.g. querying `corpus.articles` directly).
- `link_article` is idempotent per `(article_id, target_module, target_id)` — safe to call on every
  ticket-raise or item-view without your own dedup.
- Corpus reaches no other module's tables or write path — attachments belong in `backbone-bucket`, public
  slug/SEO/publish-to-web belongs in the `backbone-seo` overlay. Never bake those fields onto `Article`.
