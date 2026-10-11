// This directory's extension (hand-written; ADR-0031). The generated mod.rs (lib.rs for
// src/) pulls it in with include!, so its lines resolve against this directory.

// The hand-written corpus SQL's parameter/projection types (see the four `*_repository` modules, all
// declared `user_owned` in metaphor.codegen.yaml).
pub use article_category_repository::NewCategoryRow;
pub use article_feedback_repository::NewFeedbackRow;
pub use article_link_repository::NewLinkRow;
pub use article_repository::{ArticleStatsRow, ArticleViewRow, NewArticleRow};
