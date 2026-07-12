//! The deflection seam against a REAL backbone-support Issue. A published KB article is linked to a genuine
//! `support.issues` row via the polymorphic `ArticleLink` logical FK, and corpus's `suggest_for_target`
//! serves it back — the ticket-deflection read path, proven end-to-end. ZERO normal Cargo edge: support is a
//! dev-dependency only; corpus reaches the issue purely by its logical id, never importing support's write
//! path into the shipped library.

mod common;
use common::*;

use backbone_corpus::application::service::corpus_write_service::*;
use backbone_support::application::service::support_write_service::{
    NewIssue, NewSla, NewSlaPriority, SupportWriteService,
};
use uuid::Uuid;

// CSEAM-1 — a real support Issue surfaces its linked, published KB article via the deflection read.
#[tokio::test]
async fn cseam1_real_issue_gets_suggested_articles() {
    let pool = pool().await;
    let company = Uuid::new_v4();

    // A REAL backbone-support issue.
    let support = SupportWriteService::new(pool.clone());
    let sla = support.create_sla(NewSla {
        company_id: company, name: "Standard".into(), is_default: true,
        priorities: vec![NewSlaPriority { priority: "high".into(), response_time_mins: 60, resolution_time_mins: 240 }],
    }).await.unwrap();
    let issue = support.raise_issue(NewIssue {
        company_id: company, customer_id: Some(Uuid::new_v4()), subject: "Card declined at checkout".into(),
        description: None, priority: "high".into(), sla_id: Some(sla),
    }, chrono::Utc::now()).await.unwrap();

    // A published KB article linked to that real issue.
    let corpus = CorpusWriteService::new(pool.clone());
    let art = corpus.create_article(NewArticle {
        company_id: company, category_id: None,
        title: "Why was my card declined?".into(), body: "Common reasons and fixes...".into(),
    }).await.unwrap();
    corpus.link_article(company, art, LinkTarget {
        target_module: "support".into(), target_type: "issue".into(), target_id: issue, category_key: Some("payments".into()),
    }).await.unwrap();
    corpus.publish_article(art).await.unwrap();

    // The deflection read serves the article for the real issue — via the polymorphic link, zero edge.
    let served = corpus.suggest_for_target(company, "support", issue).await.unwrap();
    assert_eq!(served.len(), 1, "the real issue surfaces its linked published article");
    assert_eq!(served[0].id, art);
    assert_eq!(served[0].title, "Why was my card declined?");

    // The issue is a genuine support row (proves it's the real module, not a stub).
    let subj: String = sqlx::query_scalar("SELECT subject FROM support.issues WHERE id=$1")
        .bind(issue).fetch_one(&pool).await.unwrap();
    assert_eq!(subj, "Card declined at checkout");
}
