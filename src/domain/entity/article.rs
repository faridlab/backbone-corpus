use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::ArticleStatus;
use super::AuditMetadata;

/// Strongly-typed ID for Article
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArticleId(pub Uuid);

impl ArticleId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for ArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ArticleId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for ArticleId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<ArticleId> for Uuid {
    fn from(id: ArticleId) -> Self { id.0 }
}

impl AsRef<Uuid> for ArticleId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for ArticleId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Article {
    pub id: Uuid,
    pub company_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub status: ArticleStatus,
    pub revision: i32,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl Article {
    /// Create a builder for Article
    pub fn builder() -> ArticleBuilder {
        ArticleBuilder::default()
    }

    /// Create a new Article with required fields
    pub fn new(company_id: Uuid, title: String, body: String, status: ArticleStatus, revision: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            category_id: None,
            title,
            body,
            status,
            revision,
            published_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> ArticleId {
        ArticleId(self.id)
    }

    /// Get when this entity was created
    pub fn created_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.created_at.as_ref()
    }

    /// Get when this entity was last updated
    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.updated_at.as_ref()
    }

    /// Check if this entity is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.metadata.deleted_at.is_some()
    }

    /// Check if this entity is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.metadata.deleted_at.is_none()
    }

    /// Get when this entity was deleted
    pub fn deleted_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.deleted_at.as_ref()
    }

    /// Get who created this entity
    pub fn created_by(&self) -> Option<&Uuid> {
        self.metadata.created_by.as_ref()
    }

    /// Get who last updated this entity
    pub fn updated_by(&self) -> Option<&Uuid> {
        self.metadata.updated_by.as_ref()
    }

    /// Get who deleted this entity
    pub fn deleted_by(&self) -> Option<&Uuid> {
        self.metadata.deleted_by.as_ref()
    }

    /// Get the current status
    pub fn status(&self) -> &ArticleStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the category_id field (chainable)
    pub fn with_category_id(mut self, value: Uuid) -> Self {
        self.category_id = Some(value);
        self
    }

    /// Set the published_at field (chainable)
    pub fn with_published_at(mut self, value: DateTime<Utc>) -> Self {
        self.published_at = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "company_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.company_id = v; }
                }
                "category_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.category_id = v; }
                }
                "title" => {
                    if let Ok(v) = serde_json::from_value(value) { self.title = v; }
                }
                "body" => {
                    if let Ok(v) = serde_json::from_value(value) { self.body = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "revision" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revision = v; }
                }
                "published_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.published_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for Article {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "Article"
    }
}

impl backbone_core::PersistentEntity for Article {
    fn entity_id(&self) -> String {
        self.id.to_string()
    }
    fn set_entity_id(&mut self, id: String) {
        if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            self.id = uuid;
        }
    }
    fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.created_at
    }
    fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.created_at = Some(ts);
    }
    fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.updated_at
    }
    fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.updated_at = Some(ts);
    }
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.deleted_at
    }
    fn set_deleted_at(&mut self, ts: Option<chrono::DateTime<chrono::Utc>>) {
        self.metadata.deleted_at = ts;
    }
}

impl backbone_orm::EntityRepoMeta for Article {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("category_id".to_string(), "uuid".to_string());
        m.insert("status".to_string(), "article_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["title", "body"]
    }
    fn relations() -> &'static [(&'static str, &'static str, &'static str)] {
        &[("category", "article_categories", "categoryId")]
    }
}

/// Builder for Article entity
///
/// Provides a fluent API for constructing Article instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct ArticleBuilder {
    company_id: Option<Uuid>,
    category_id: Option<Uuid>,
    title: Option<String>,
    body: Option<String>,
    status: Option<ArticleStatus>,
    revision: Option<i32>,
    published_at: Option<DateTime<Utc>>,
}

impl ArticleBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the category_id field (optional)
    pub fn category_id(mut self, value: Uuid) -> Self {
        self.category_id = Some(value);
        self
    }

    /// Set the title field (required)
    pub fn title(mut self, value: String) -> Self {
        self.title = Some(value);
        self
    }

    /// Set the body field (required)
    pub fn body(mut self, value: String) -> Self {
        self.body = Some(value);
        self
    }

    /// Set the status field (default: `ArticleStatus::default()`)
    pub fn status(mut self, value: ArticleStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the revision field (default: `1`)
    pub fn revision(mut self, value: i32) -> Self {
        self.revision = Some(value);
        self
    }

    /// Set the published_at field (optional)
    pub fn published_at(mut self, value: DateTime<Utc>) -> Self {
        self.published_at = Some(value);
        self
    }

    /// Build the Article entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<Article, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let title = self.title.ok_or_else(|| "title is required".to_string())?;
        let body = self.body.ok_or_else(|| "body is required".to_string())?;

        Ok(Article {
            id: Uuid::new_v4(),
            company_id,
            category_id: self.category_id,
            title,
            body,
            status: self.status.unwrap_or(ArticleStatus::default()),
            revision: self.revision.unwrap_or(1),
            published_at: self.published_at,
            metadata: AuditMetadata::default(),
        })
    }
}
