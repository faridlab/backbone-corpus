// This directory's extension (hand-written; ADR-0031). The generated mod.rs (lib.rs for
// src/) pulls it in with include!, so its lines resolve against this directory.

pub mod corpus_write_service;
pub use corpus_write_service::{
    ArticleStats, ArticleView, CorpusError, CorpusWriteService, LinkTarget, NewArticle, NewCategory,
};
