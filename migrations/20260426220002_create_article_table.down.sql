-- Down: drop corpus.articles table
DROP TABLE IF EXISTS corpus.articles CASCADE;
DROP FUNCTION IF EXISTS corpus.articles_audit_timestamp() CASCADE;
