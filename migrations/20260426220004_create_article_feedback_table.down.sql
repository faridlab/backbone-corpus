-- Down: drop corpus.article_feedback table
DROP TABLE IF EXISTS corpus.article_feedback CASCADE;
DROP FUNCTION IF EXISTS corpus.article_feedback_audit_timestamp() CASCADE;
