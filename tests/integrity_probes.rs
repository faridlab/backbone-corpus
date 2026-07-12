//! Integrity probes — the KB invariants: only PUBLISHED articles are served (a draft must never leak), an
//! archived article is withdrawn, and the deflection read is company/target scoped.

mod common;
use common::*;

use backbone_corpus::application::service::corpus_write_service::*;
use uuid::Uuid;

fn link(module: &str, ttype: &str, target: Uuid) -> LinkTarget {
    LinkTarget { target_module: module.into(), target_type: ttype.into(), target_id: target, category_key: None }
}

// CIP-1 — MATURITY: a DRAFT article linked to a target is NOT served — serving unreviewed content to a
// customer is the invariant. Proven-by-revert: dropping the `status='published'` filter from the deflection
// read makes the draft leak.
#[tokio::test]
async fn cip1_draft_never_served() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let issue = Uuid::new_v4();
    // A DRAFT article, linked but never published.
    let draft = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "WIP — do not show".into(), body: "unreviewed".into(),
    }).await.unwrap();
    svc.link_article(company, draft, link("support", "issue", issue)).await.unwrap();

    let served = svc.suggest_for_target(company, "support", issue).await.unwrap();
    assert!(served.is_empty(), "a draft article must never be served on the deflection read");

    // Once published, it IS served.
    svc.publish_article(draft).await.unwrap();
    assert_eq!(svc.suggest_for_target(company, "support", issue).await.unwrap().len(), 1);
}

// CIP-2 — an archived article is withdrawn from the deflection read.
#[tokio::test]
async fn cip2_archived_withdrawn() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let issue = Uuid::new_v4();
    let art = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "Old answer".into(), body: "...".into(),
    }).await.unwrap();
    svc.link_article(company, art, link("support", "issue", issue)).await.unwrap();
    svc.publish_article(art).await.unwrap();
    assert_eq!(svc.suggest_for_target(company, "support", issue).await.unwrap().len(), 1);

    assert!(svc.archive_article(art).await.unwrap());
    assert!(svc.suggest_for_target(company, "support", issue).await.unwrap().is_empty(), "archived is withdrawn");
}

// CIP-3 — the deflection read is scoped: another company's / another target's articles are not served.
#[tokio::test]
async fn cip3_scoped_to_company_and_target() {
    let pool = pool().await;
    let svc = CorpusWriteService::new(pool.clone());
    let company = Uuid::new_v4();
    let issue_a = Uuid::new_v4();
    let issue_b = Uuid::new_v4();
    let art = svc.create_article(NewArticle {
        company_id: company, category_id: None, title: "For issue A".into(), body: "...".into(),
    }).await.unwrap();
    svc.link_article(company, art, link("support", "issue", issue_a)).await.unwrap();
    svc.publish_article(art).await.unwrap();

    assert_eq!(svc.suggest_for_target(company, "support", issue_a).await.unwrap().len(), 1);
    assert!(svc.suggest_for_target(company, "support", issue_b).await.unwrap().is_empty(), "not served for a different target");
    assert!(svc.suggest_for_target(Uuid::new_v4(), "support", issue_a).await.unwrap().is_empty(), "not served to another company");
}
