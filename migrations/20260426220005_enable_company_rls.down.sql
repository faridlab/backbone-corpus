-- Down: remove the company RLS fence for corpus module

-- Reverse the company RLS fence for corpus.article_categories
DROP POLICY IF EXISTS article_categories_company_isolation ON corpus.article_categories;
ALTER TABLE corpus.article_categories NO FORCE ROW LEVEL SECURITY;
ALTER TABLE corpus.article_categories DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for corpus.articles
DROP POLICY IF EXISTS articles_company_isolation ON corpus.articles;
ALTER TABLE corpus.articles NO FORCE ROW LEVEL SECURITY;
ALTER TABLE corpus.articles DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for corpus.article_links
DROP POLICY IF EXISTS article_links_company_isolation ON corpus.article_links;
ALTER TABLE corpus.article_links NO FORCE ROW LEVEL SECURITY;
ALTER TABLE corpus.article_links DISABLE ROW LEVEL SECURITY;

-- Reverse the company RLS fence for corpus.article_feedback
DROP POLICY IF EXISTS article_feedback_company_isolation ON corpus.article_feedback;
ALTER TABLE corpus.article_feedback NO FORCE ROW LEVEL SECURITY;
ALTER TABLE corpus.article_feedback DISABLE ROW LEVEL SECURITY;

