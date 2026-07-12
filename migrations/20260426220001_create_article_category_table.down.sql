-- Down: drop corpus.article_categories table
DROP TABLE IF EXISTS corpus.article_categories CASCADE;
DROP FUNCTION IF EXISTS corpus.article_categories_audit_timestamp() CASCADE;
