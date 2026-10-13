-- Down: drop the declared foreign keys for corpus module
ALTER TABLE IF EXISTS corpus.article_feedback DROP CONSTRAINT IF EXISTS fk_article_feedback_article_id;
ALTER TABLE IF EXISTS corpus.article_links DROP CONSTRAINT IF EXISTS fk_article_links_article_id;
ALTER TABLE IF EXISTS corpus.articles DROP CONSTRAINT IF EXISTS fk_articles_category_id;
