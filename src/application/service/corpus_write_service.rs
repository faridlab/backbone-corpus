//! The hand-authored corpus write path (user-owned; survives regen).
//!
//! A support-anchored knowledge base. The load-bearing logic is the **deflection read**: given a target
//! (a support Issue, a catalog Item), return the PUBLISHED articles linked to it — the self-service answer
//! that deflects a ticket. The publish gate is the invariant: a `draft` article must NEVER be served to a
//! customer (it is unreviewed content). Corpus reaches no other module; it is READ by consumers
//! (support/portal) via the polymorphic `ArticleLink` logical FK — zero normal Cargo edge.

use backbone_orm::company_scope;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::infrastructure::persistence::{
    ArticleCategoryRepository, ArticleFeedbackRepository, ArticleLinkRepository, ArticleRepository,
    ArticleStatsRow, ArticleViewRow, NewArticleRow, NewCategoryRow, NewFeedbackRow, NewLinkRow,
};

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
    categories: ArticleCategoryRepository,
    articles: ArticleRepository,
    links: ArticleLinkRepository,
    feedback: ArticleFeedbackRepository,
}

impl CorpusWriteService {
    pub fn new(pool: PgPool) -> Self {
        let categories = ArticleCategoryRepository::new(pool.clone());
        let articles = ArticleRepository::new(pool.clone());
        let links = ArticleLinkRepository::new(pool.clone());
        let feedback = ArticleFeedbackRepository::new(pool.clone());
        Self { pool, categories, articles, links, feedback }
    }

    pub async fn create_category(&self, c: NewCategory) -> Result<Uuid, CorpusError> {
        // RLS scope (ADR-0008): company is on the DTO — bind it for the whole body so the insert runs
        // with `app.company_id` set and the WITH CHECK passes under the app role.
        let company = c.company_id;
        company_scope::with_company_scope(Some(company), async move {
            let id = Uuid::new_v4();
            self.categories.insert_category(&self.pool, &NewCategoryRow {
                id,
                company_id: c.company_id,
                code: &c.code,
                name: &c.name,
            }).await?;
            Ok(id)
        }).await
    }

    /// Author a new article — always starts as a `draft` (never served until published).
    pub async fn create_article(&self, a: NewArticle) -> Result<Uuid, CorpusError> {
        if a.title.trim().is_empty() {
            return Err(CorpusError::Invalid("title is required".into()));
        }
        // RLS scope (ADR-0008): company is on the DTO — same pattern as `create_category`.
        let company = a.company_id;
        company_scope::with_company_scope(Some(company), async move {
            let id = Uuid::new_v4();
            self.articles.insert_article(&self.pool, &NewArticleRow {
                id,
                company_id: a.company_id,
                category_id: a.category_id,
                title: &a.title,
                body: &a.body,
            }).await?;
            Ok(id)
        }).await
    }

    /// Edit an article's content — bumps the revision. A published edit stays published (it is a live fix).
    pub async fn edit_article(&self, article_id: Uuid, title: &str, body: &str) -> Result<i32, CorpusError> {
        // RLS scope (ADR-0008), ID-only pattern: identified by the article id alone — no company arg.
        // This rides the request-dedicated connection (which carries the caller's `app.company_id`), so
        // RLS fences the update; another company's article is simply not found.
        let rev = self.articles.update_content(&self.pool, article_id, title, body).await?;
        rev.ok_or(CorpusError::NotFound("article"))
    }

    /// Publish a draft — the draft→published transition, stamping `published_at`. CAS-gated on `draft` so it
    /// happens once. After this the article is served on the deflection read.
    pub async fn publish_article(&self, article_id: Uuid) -> Result<bool, CorpusError> {
        // RLS scope (ADR-0008), ID-only pattern — see `edit_article`.
        let moved = self.articles.mark_published(&self.pool, article_id, Utc::now()).await?;
        Ok(moved == 1)
    }

    /// Retire a published article (archived → no longer served, kept for history).
    pub async fn archive_article(&self, article_id: Uuid) -> Result<bool, CorpusError> {
        // RLS scope (ADR-0008), ID-only pattern — see `edit_article`.
        let moved = self.articles.mark_archived(&self.pool, article_id).await?;
        Ok(moved == 1)
    }

    /// Link an article to the thing it explains (a support Issue, a catalog Item) via the polymorphic
    /// logical FK. Idempotent per (article, target).
    pub async fn link_article(&self, company_id: Uuid, article_id: Uuid, t: LinkTarget) -> Result<Uuid, CorpusError> {
        // RLS scope (ADR-0008): company is on the parameter — bind it for the whole body so both the
        // upsert and the conflict re-read are fenced.
        company_scope::with_company_scope(Some(company_id), async move {
            let id = Uuid::new_v4();
            let inserted = self.links.claim_link(&self.pool, &NewLinkRow {
                id,
                company_id,
                article_id,
                target_module: &t.target_module,
                target_type: &t.target_type,
                target_id: t.target_id,
                category_key: t.category_key.as_deref(),
            }).await?;
            // On conflict the INSERT returns nothing — return the EXISTING link's id so a re-link is a true no-op.
            match inserted {
                Some(new_id) => Ok(new_id),
                None => Ok(self
                    .links
                    .fetch_link_id(&self.pool, article_id, &t.target_module, t.target_id)
                    .await?),
            }
        }).await
    }

    /// Record a reader's "was this helpful?" vote — the deflection metric.
    pub async fn record_feedback(&self, company_id: Uuid, article_id: Uuid, helpful: bool, note: Option<String>) -> Result<Uuid, CorpusError> {
        // RLS scope (ADR-0008): company is on the parameter.
        company_scope::with_company_scope(Some(company_id), async move {
            let id = Uuid::new_v4();
            self.feedback.insert_feedback(&self.pool, &NewFeedbackRow {
                id,
                company_id,
                article_id,
                helpful,
                note: note.as_deref(),
            }).await?;
            Ok(id)
        }).await
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
        // RLS scope (ADR-0008): read-only, company on the parameter — the explicit `l.company_id` filter
        // stays as defense-in-depth.
        let rows = company_scope::with_company_scope(
            Some(company_id),
            self.articles.suggest_for_target(&self.pool, company_id, target_module, target_id),
        ).await?;
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
        // RLS scope (ADR-0008): read-only, company on the parameter.
        let rows = company_scope::with_company_scope(
            Some(company_id),
            self.articles.suggest_for_category(&self.pool, company_id, target_module, category_key),
        ).await?;
        Ok(rows.iter().map(row_to_view).collect())
    }

    /// The deflection METRIC (completeness council): each article's helpful/not-helpful tally + ratio, so an
    /// operator can see which articles actually deflect — the number that proves the KB earns its keep. The
    /// module holds the feedback; this exposes it without a consumer re-summing the raw votes.
    pub async fn article_stats(&self, company_id: Uuid) -> Result<Vec<ArticleStats>, CorpusError> {
        // RLS scope (ADR-0008): read-only, company on the parameter.
        let rows = company_scope::with_company_scope(
            Some(company_id),
            self.articles.feedback_tallies(&self.pool, company_id),
        ).await?;
        // The ratio is derived here, not in SQL: the repo returns the raw tallies, the service shapes them.
        Ok(rows.into_iter().map(|r| {
            let ArticleStatsRow { article_id, title, helpful, not_helpful } = r;
            let total = helpful + not_helpful;
            let helpful_pct = if total > 0 { Some(((helpful as f64 / total as f64) * 100.0).round() as i64) } else { None };
            ArticleStats { article_id, title, helpful, not_helpful, helpful_pct }
        }).collect())
    }
}

fn row_to_view(r: &ArticleViewRow) -> ArticleView {
    ArticleView { id: r.id, title: r.title.clone(), body: r.body.clone(), category_id: r.category_id }
}
