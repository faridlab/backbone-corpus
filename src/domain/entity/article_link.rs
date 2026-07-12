use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::AuditMetadata;

/// Strongly-typed ID for ArticleLink
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArticleLinkId(pub Uuid);

impl ArticleLinkId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for ArticleLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ArticleLinkId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for ArticleLinkId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<ArticleLinkId> for Uuid {
    fn from(id: ArticleLinkId) -> Self { id.0 }
}

impl AsRef<Uuid> for ArticleLinkId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for ArticleLinkId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArticleLink {
    pub id: Uuid,
    pub company_id: Uuid,
    pub article_id: Uuid,
    pub target_module: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub category_key: Option<String>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl ArticleLink {
    /// Create a builder for ArticleLink
    pub fn builder() -> ArticleLinkBuilder {
        ArticleLinkBuilder::default()
    }

    /// Create a new ArticleLink with required fields
    pub fn new(company_id: Uuid, article_id: Uuid, target_module: String, target_type: String, target_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            article_id,
            target_module,
            target_type,
            target_id,
            category_key: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> ArticleLinkId {
        ArticleLinkId(self.id)
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


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the category_key field (chainable)
    pub fn with_category_key(mut self, value: String) -> Self {
        self.category_key = Some(value);
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
                "article_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.article_id = v; }
                }
                "target_module" => {
                    if let Ok(v) = serde_json::from_value(value) { self.target_module = v; }
                }
                "target_type" => {
                    if let Ok(v) = serde_json::from_value(value) { self.target_type = v; }
                }
                "target_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.target_id = v; }
                }
                "category_key" => {
                    if let Ok(v) = serde_json::from_value(value) { self.category_key = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for ArticleLink {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "ArticleLink"
    }
}

impl backbone_core::PersistentEntity for ArticleLink {
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

impl backbone_orm::EntityRepoMeta for ArticleLink {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("article_id".to_string(), "uuid".to_string());
        m.insert("target_id".to_string(), "uuid".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["target_module", "target_type"]
    }
    fn relations() -> &'static [(&'static str, &'static str, &'static str)] {
        &[("article", "articles", "articleId")]
    }
}

/// Builder for ArticleLink entity
///
/// Provides a fluent API for constructing ArticleLink instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct ArticleLinkBuilder {
    company_id: Option<Uuid>,
    article_id: Option<Uuid>,
    target_module: Option<String>,
    target_type: Option<String>,
    target_id: Option<Uuid>,
    category_key: Option<String>,
}

impl ArticleLinkBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the article_id field (required)
    pub fn article_id(mut self, value: Uuid) -> Self {
        self.article_id = Some(value);
        self
    }

    /// Set the target_module field (required)
    pub fn target_module(mut self, value: String) -> Self {
        self.target_module = Some(value);
        self
    }

    /// Set the target_type field (required)
    pub fn target_type(mut self, value: String) -> Self {
        self.target_type = Some(value);
        self
    }

    /// Set the target_id field (required)
    pub fn target_id(mut self, value: Uuid) -> Self {
        self.target_id = Some(value);
        self
    }

    /// Set the category_key field (optional)
    pub fn category_key(mut self, value: String) -> Self {
        self.category_key = Some(value);
        self
    }

    /// Build the ArticleLink entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<ArticleLink, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let article_id = self.article_id.ok_or_else(|| "article_id is required".to_string())?;
        let target_module = self.target_module.ok_or_else(|| "target_module is required".to_string())?;
        let target_type = self.target_type.ok_or_else(|| "target_type is required".to_string())?;
        let target_id = self.target_id.ok_or_else(|| "target_id is required".to_string())?;

        Ok(ArticleLink {
            id: Uuid::new_v4(),
            company_id,
            article_id,
            target_module,
            target_type,
            target_id,
            category_key: self.category_key,
            metadata: AuditMetadata::default(),
        })
    }
}
