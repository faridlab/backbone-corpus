-- Down: drop corpus.article_links table
DROP TABLE IF EXISTS corpus.article_links CASCADE;
DROP FUNCTION IF EXISTS corpus.article_links_audit_timestamp() CASCADE;
