# backbone-corpus — business flows & golden cases

## Flow: author → publish → deflection read

```
create_article (always draft)
   │
   ▼  link_article (target_module/type/id, idempotent per article+target)
   │
   ▼  publish_article (CAS draft→published, stamps published_at)
   │
   └▶ suggest_for_target(company, target_module, target_id)
            │
            ▼  JOIN article_links, WHERE status='published' AND not soft-deleted
            │
            └▶ ArticleView[] — the answer that deflects the ticket
```
Posts NO GL. A draft or archived article is filtered out of the read; it is never a permissions check a
caller could bypass — the gate lives in the query itself.

## Golden cases (`tests/corpus_golden_cases.rs`)
- **CGC-1 — published article is served.** Create → link → publish → `suggest_for_target` returns exactly
  that article (id + title match).
- **CGC-2 — link idempotent.** Linking the same article to the same target twice returns the same link id.
- **CGC-3 — edit bumps revision.** `edit_article` on a fresh article (`revision=1`) returns `2`.
- **CGC-4 — feedback tally surfaces the deflection metric.** 2 helpful + 1 not-helpful votes →
  `article_stats` reports `helpful=2, not_helpful=1, helpful_pct=Some(67)`. (completeness council 2026-07-11)

## Integrity probes (`tests/integrity_probes.rs`)
- **CIP-1 — draft never served (the invariant).** A draft article, linked but unpublished, is **not**
  returned by `suggest_for_target`; once published it IS. Proven-by-revert: dropping the
  `status='published'` filter from the deflection read makes the draft leak. (maturity council 2026-07-11)
- **CIP-2 — archived withdrawn.** A published, linked, served article stops being served once
  `archive_article` succeeds.
- **CIP-3 — scoped to company + target.** A published article linked to issue A is not served for issue B,
  nor to a different company.

## Seam (`tests/corpus_support_seam.rs`)
- **CSEAM-1 — a REAL support Issue gets its suggested article.** A genuine `backbone-support` Issue
  (raised via `SupportWriteService::raise_issue`) is linked to a published corpus article via the
  polymorphic `ArticleLink`; `suggest_for_target(company, "support", issue)` serves it back. The issue row
  is verified to exist in `support.issues` (proves it's the real module, not a stub). Zero normal Cargo
  edge — `backbone-support` is a dev-dependency only.

## §5 round-trip (`scripts/corpus_support_seam_roundtrip.sh`)
Regen (`--force`) leaves the seam files byte-identical; the oracle + seam re-run green.
