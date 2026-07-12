//! Golden cases — the KB oracle: author → publish → the deflection read serves it; a link is idempotent;
//! an edit bumps the revision; the feedback tally surfaces the deflection metric.

mod common;
use common::*;

use backbone_corpus::application::service::corpus_write_service::*;
use uuid::Uuid;

// CGC-1 — a published article linked to a target is served by the deflection read.
#[tokio::test]
async fn cgc1_published_article_is_served() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let issue = Uuid::new_v4();

    let art = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "How to reset your PIN".into(), body: "Steps...".into(),
    }).await.unwrap();
    svc.link_article(company, art, LinkTarget {
        target_module: "support".into(), target_type: "issue".into(), target_id: issue, category_key: Some("account".into()),
    }).await.unwrap();
    assert!(svc.publish_article(art).await.unwrap());

    let served = svc.suggest_for_target(company, "support", issue).await.unwrap();
    assert_eq!(served.len(), 1);
    assert_eq!(served[0].id, art);
    assert_eq!(served[0].title, "How to reset your PIN");
}

// CGC-2 — linking the same article to the same target twice is idempotent.
#[tokio::test]
async fn cgc2_link_idempotent() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let item = Uuid::new_v4();
    let art = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "Care guide".into(), body: "...".into(),
    }).await.unwrap();
    let t = || LinkTarget { target_module: "catalog".into(), target_type: "item".into(), target_id: item, category_key: None };
    let l1 = svc.link_article(company, art, t()).await.unwrap();
    let l2 = svc.link_article(company, art, t()).await.unwrap();
    assert_eq!(l1, l2, "the same (article,target) link is idempotent");
}

// CGC-3 — editing an article bumps the revision counter.
#[tokio::test]
async fn cgc3_edit_bumps_revision() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let art = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "v1".into(), body: "a".into(),
    }).await.unwrap();
    let r = svc.edit_article(art, "v2", "b").await.unwrap();
    assert_eq!(r, 2, "revision bumped 1 → 2");
}

// CGC-4 — the deflection metric (completeness): feedback tallies into a helpful ratio per article.
#[tokio::test]
async fn cgc4_feedback_tally_surfaces_deflection() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let art = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "Returns policy".into(), body: "...".into(),
    }).await.unwrap();
    svc.record_feedback(company, art, true, None).await.unwrap();
    svc.record_feedback(company, art, true, None).await.unwrap();
    svc.record_feedback(company, art, false, Some("unclear".into())).await.unwrap();

    let stats = svc.article_stats(company).await.unwrap();
    let s = stats.iter().find(|s| s.article_id == art).unwrap();
    assert_eq!(s.helpful, 2);
    assert_eq!(s.not_helpful, 1);
    assert_eq!(s.helpful_pct, Some(67), "2/3 helpful → 67%");
}
