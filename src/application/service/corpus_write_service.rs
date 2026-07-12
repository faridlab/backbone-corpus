//! The hand-authored corpus write path (user-owned; survives regen).
//!
//! A support-anchored knowledge base. The load-bearing logic is the **deflection read**: given a target
//! (a support Issue, a catalog Item), return the PUBLISHED articles linked to it — the self-service answer
//! that deflects a ticket. The publish gate is the invariant: a `draft` article must NEVER be served to a
//! customer (it is unreviewed content). Corpus reaches no other module; it is READ by consumers
//! (support/portal) via the polymorphic `ArticleLink` logical FK — zero normal Cargo edge.

use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum CorpusError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(&'static str),
    #[error("invalid state: {0}")]
    InvalidState(&'static str),
    #[error("invalid input: {0}")]
    Invalid(String),
}

pub struct NewCategory {
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
}

pub struct NewArticle {
    pub company_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub body: String,
}

pub struct LinkTarget {
    pub target_module: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub category_key: Option<String>,
}

/// A served article (the deflection read result) — only ever a PUBLISHED article.
#[derive(Debug, Clone, PartialEq)]
pub struct ArticleView {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub category_id: Option<Uuid>,
}

/// An article's deflection metric — how many readers found it helpful.
#[derive(Debug, Clone, PartialEq)]
pub struct ArticleStats {
    pub article_id: Uuid,
    pub title: String,
    pub helpful: i64,
    pub not_helpful: i64,
    /// helpful / (helpful + not_helpful) × 100, rounded; None when no feedback yet.
    pub helpful_pct: Option<i64>,
}

pub struct CorpusWriteService {
    pool: PgPool,
}

impl CorpusWriteService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_category(&self, c: NewCategory) -> Result<Uuid, CorpusError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO corpus.article_categories (id, company_id, code, name) VALUES ($1,$2,$3,$4)")
            .bind(id).bind(c.company_id).bind(&c.code).bind(&c.name)
            .execute(&self.pool).await?;
        Ok(id)
    }

    /// Author a new article — always starts as a `draft` (never served until published).
    pub async fn create_article(&self, a: NewArticle) -> Result<Uuid, CorpusError> {
        if a.title.trim().is_empty() {
            return Err(CorpusError::Invalid("title is required".into()));
        }
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO corpus.articles (id, company_id, category_id, title, body, status, revision)
               VALUES ($1,$2,$3,$4,$5,'draft'::article_status,1)"#,
        )
        .bind(id).bind(a.company_id).bind(a.category_id).bind(&a.title).bind(&a.body)
        .execute(&self.pool).await?;
        Ok(id)
    }

    /// Edit an article's content — bumps the revision. A published edit stays published (it is a live fix).
    pub async fn edit_article(&self, article_id: Uuid, title: &str, body: &str) -> Result<i32, CorpusError> {
        let rev: Option<i32> = sqlx::query_scalar(
            r#"UPDATE corpus.articles SET title=$2, body=$3, revision=revision+1
               WHERE id=$1 AND (metadata->>'deleted_at') IS NULL
               RETURNING revision"#,
        )
        .bind(article_id).bind(title).bind(body).fetch_optional(&self.pool).await?;
        rev.ok_or(CorpusError::NotFound("article"))
    }

    /// Publish a draft — the draft→published transition, stamping `published_at`. CAS-gated on `draft` so it
    /// happens once. After this the article is served on the deflection read.
    pub async fn publish_article(&self, article_id: Uuid) -> Result<bool, CorpusError> {
        let moved = sqlx::query(
            r#"UPDATE corpus.articles
               SET status='published'::article_status, published_at=$2
               WHERE id=$1 AND status='draft'::article_status AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(article_id).bind(Utc::now()).execute(&self.pool).await?;
        Ok(moved.rows_affected() == 1)
    }

    /// Retire a published article (archived → no longer served, kept for history).
    pub async fn archive_article(&self, article_id: Uuid) -> Result<bool, CorpusError> {
        let moved = sqlx::query(
            r#"UPDATE corpus.articles SET status='archived'::article_status
               WHERE id=$1 AND status='published'::article_status AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(article_id).execute(&self.pool).await?;
        Ok(moved.rows_affected() == 1)
    }

    /// Link an article to the thing it explains (a support Issue, a catalog Item) via the polymorphic
    /// logical FK. Idempotent per (article, target).
    pub async fn link_article(&self, company_id: Uuid, article_id: Uuid, t: LinkTarget) -> Result<Uuid, CorpusError> {
        let id = Uuid::new_v4();
        let inserted: Option<Uuid> = sqlx::query_scalar(
            r#"INSERT INTO corpus.article_links
                 (id, company_id, article_id, target_module, target_type, target_id, category_key)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               ON CONFLICT (article_id, target_module, target_id) WHERE (metadata->>'deleted_at') IS NULL
               DO NOTHING
               RETURNING id"#,
        )
        .bind(id).bind(company_id).bind(article_id).bind(&t.target_module).bind(&t.target_type)
        .bind(t.target_id).bind(&t.category_key).fetch_optional(&self.pool).await?;
        // On conflict the INSERT returns nothing — return the EXISTING link's id so a re-link is a true no-op.
        match inserted {
            Some(new_id) => Ok(new_id),
            None => Ok(sqlx::query_scalar(
                r#"SELECT id FROM corpus.article_links
                   WHERE article_id=$1 AND target_module=$2 AND target_id=$3 AND (metadata->>'deleted_at') IS NULL"#,
            )
            .bind(article_id).bind(&t.target_module).bind(t.target_id).fetch_one(&self.pool).await?),
        }
    }

    /// Record a reader's "was this helpful?" vote — the deflection metric.
    pub async fn record_feedback(&self, company_id: Uuid, article_id: Uuid, helpful: bool, note: Option<String>) -> Result<Uuid, CorpusError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO corpus.article_feedback (id, company_id, article_id, helpful, note) VALUES ($1,$2,$3,$4,$5)")
            .bind(id).bind(company_id).bind(article_id).bind(helpful).bind(&note)
            .execute(&self.pool).await?;
        Ok(id)
    }

    // ---- the deflection read path -------------------------------------------------------------------

    /// The seam read: the PUBLISHED articles linked to a given target (e.g. a support Issue). A draft or
    /// archived article is NEVER returned — serving unreviewed/retired content to a customer is the invariant
    /// this guards (maturity council). Reached purely through the polymorphic `ArticleLink` (zero edge).
    pub async fn suggest_for_target(
        &self,
        company_id: Uuid,
        target_module: &str,
        target_id: Uuid,
    ) -> Result<Vec<ArticleView>, CorpusError> {
        let rows = sqlx::query(
            r#"SELECT a.id, a.title, a.body, a.category_id
               FROM corpus.articles a
               JOIN corpus.article_links l ON l.article_id = a.id
               WHERE l.company_id = $1 AND l.target_module = $2 AND l.target_id = $3
                 AND a.status = 'published'::article_status
                 AND (a.metadata->>'deleted_at') IS NULL AND (l.metadata->>'deleted_at') IS NULL
               ORDER BY a.published_at DESC NULLS LAST"#,
        )
        .bind(company_id).bind(target_module).bind(target_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_view).collect())
    }

    /// Suggest published articles for a routing category (e.g. an issue's category) — the browse-by-topic
    /// deflection path.
    pub async fn suggest_for_category(
        &self,
        company_id: Uuid,
        target_module: &str,
        category_key: &str,
    ) -> Result<Vec<ArticleView>, CorpusError> {
        let rows = sqlx::query(
            r#"SELECT DISTINCT a.id, a.title, a.body, a.category_id, a.published_at
               FROM corpus.articles a
               JOIN corpus.article_links l ON l.article_id = a.id
               WHERE l.company_id = $1 AND l.target_module = $2 AND l.category_key = $3
                 AND a.status = 'published'::article_status
                 AND (a.metadata->>'deleted_at') IS NULL AND (l.metadata->>'deleted_at') IS NULL
               ORDER BY a.published_at DESC NULLS LAST"#,
        )
        .bind(company_id).bind(target_module).bind(category_key).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_view).collect())
    }

    /// The deflection METRIC (completeness council): each article's helpful/not-helpful tally + ratio, so an
    /// operator can see which articles actually deflect — the number that proves the KB earns its keep. The
    /// module holds the feedback; this exposes it without a consumer re-summing the raw votes.
    pub async fn article_stats(&self, company_id: Uuid) -> Result<Vec<ArticleStats>, CorpusError> {
        let rows = sqlx::query(
            r#"SELECT a.id, a.title,
                      COALESCE(SUM(CASE WHEN f.helpful THEN 1 ELSE 0 END),0) AS helpful,
                      COALESCE(SUM(CASE WHEN NOT f.helpful THEN 1 ELSE 0 END),0) AS not_helpful
               FROM corpus.articles a
               LEFT JOIN corpus.article_feedback f
                 ON f.article_id = a.id AND (f.metadata->>'deleted_at') IS NULL
               WHERE a.company_id = $1 AND (a.metadata->>'deleted_at') IS NULL
               GROUP BY a.id, a.title
               ORDER BY helpful DESC"#,
        )
        .bind(company_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(|r| {
            let helpful: i64 = r.get("helpful");
            let not_helpful: i64 = r.get("not_helpful");
            let total = helpful + not_helpful;
            let helpful_pct = if total > 0 { Some(((helpful as f64 / total as f64) * 100.0).round() as i64) } else { None };
            ArticleStats { article_id: r.get("id"), title: r.get("title"), helpful, not_helpful, helpful_pct }
        }).collect())
    }
}

fn row_to_view(r: &sqlx::postgres::PgRow) -> ArticleView {
    ArticleView { id: r.get("id"), title: r.get("title"), body: r.get("body"), category_id: r.get("category_id") }
}
