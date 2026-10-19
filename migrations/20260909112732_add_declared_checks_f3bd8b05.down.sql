-- Down: drop the declared rule constraints for corpus module
ALTER TABLE IF EXISTS corpus.article_feedback DROP CONSTRAINT IF EXISTS ck_article_feedback_note_max_length__max_length;
ALTER TABLE IF EXISTS corpus.article_feedback DROP CONSTRAINT IF EXISTS ck_article_feedback_article_id_required__required;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_category_key_max_length__max_length;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_target_id_required__required;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_target_type_max_length__max_length;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_target_type_required__required;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_target_module_max_length__max_length;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_target_module_required__required;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS ck_article_links_article_id_required__required;
DO $$
BEGIN
    IF to_regclass('corpus.uq_article_links_target_id__unique') IS NOT NULL AND to_regclass('corpus.idx_article_links_article_id_target_module_target_id') IS NULL THEN
        ALTER INDEX corpus.uq_article_links_target_id__unique RENAME TO idx_article_links_article_id_target_module_target_id;
    END IF;
END $$;

ALTER TABLE IF EXISTS corpus.articles DROP CONSTRAINT IF EXISTS ck_articles_revision_non_negative__non_negative;
ALTER TABLE IF EXISTS corpus.articles DROP CONSTRAINT IF EXISTS ck_articles_body_max_length__max_length;
ALTER TABLE IF EXISTS corpus.articles DROP CONSTRAINT IF EXISTS ck_articles_body_required__required;
ALTER TABLE IF EXISTS corpus.articles DROP CONSTRAINT IF EXISTS ck_articles_title_max_length__max_length;
ALTER TABLE IF EXISTS corpus.articles DROP CONSTRAINT IF EXISTS ck_articles_title_required__required;
ALTER TABLE IF EXISTS corpus.article_categories DROP CONSTRAINT IF EXISTS ck_article_categories_name_max_length__max_length;
ALTER TABLE IF EXISTS corpus.article_categories DROP CONSTRAINT IF EXISTS ck_article_categories_name_required__required;
ALTER TABLE IF EXISTS corpus.article_categories DROP CONSTRAINT IF EXISTS ck_article_categories_code_max_length__max_length;
ALTER TABLE IF EXISTS corpus.article_categories DROP CONSTRAINT IF EXISTS ck_article_categories_code_required__required;
