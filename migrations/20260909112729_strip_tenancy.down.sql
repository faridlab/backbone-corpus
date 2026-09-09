-- Hand-authored (user-owned). Not regenerated.
--
-- Best-effort restore sketch for the tenancy strip (ADR-0029). This is a breaking module
-- release against dev-stage databases: the down re-adds the company_id column as nullable
-- with its plain index and the company isolation policy shape, but restores NO data —
-- rows written after the strip (or after the decorator re-keyed them) carry org_unit_id
-- only. The composing service's tenancy decorator remains the live fence; treat this
-- down as a schema-shape sketch for archaeology, not a usable rollback.

ALTER TABLE corpus.article_categories ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE corpus.articles          ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE corpus.article_links     ADD COLUMN IF NOT EXISTS company_id uuid;
ALTER TABLE corpus.article_feedback  ADD COLUMN IF NOT EXISTS company_id uuid;

-- The strip also dropped the company-leading read/posture indexes (the (company_id, code)
-- unique among them); a UNIQUE re-add could fail on duplicate codes this sketch does not
-- restore, so only the plain column indexes come back.
CREATE INDEX IF NOT EXISTS idx_article_categories_company_id ON corpus.article_categories (company_id);
CREATE INDEX IF NOT EXISTS idx_articles_company_id           ON corpus.articles (company_id);
CREATE INDEX IF NOT EXISTS idx_article_links_company_id      ON corpus.article_links (company_id);
CREATE INDEX IF NOT EXISTS idx_article_feedback_company_id   ON corpus.article_feedback (company_id);
