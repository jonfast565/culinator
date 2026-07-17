-- Full-text search index for recipes. Filter columns live in a companion
-- table so structured predicates (allergens, active time, hydration) stay
-- queryable without fighting FTS5 typing.

CREATE VIRTUAL TABLE IF NOT EXISTS recipe_search USING fts5(
    title,
    ingredients,
    techniques,
    notes,
    section,
    recipe_id UNINDEXED,
    book_id UNINDEXED
);

CREATE TABLE IF NOT EXISTS recipe_search_filters (
    recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
    book_id TEXT,
    allergens_json TEXT NOT NULL DEFAULT '[]',
    max_active_minutes REAL,
    hydration_percent REAL
);

CREATE INDEX IF NOT EXISTS idx_recipe_search_filters_book
    ON recipe_search_filters(book_id);

CREATE TRIGGER IF NOT EXISTS recipe_search_after_delete
AFTER DELETE ON recipes
BEGIN
    DELETE FROM recipe_search WHERE recipe_id = old.id;
    DELETE FROM recipe_search_filters WHERE recipe_id = old.id;
END;

-- Backfill titles for existing recipes; full indexing runs on next save.
INSERT INTO recipe_search (title, ingredients, techniques, notes, section, recipe_id, book_id)
SELECT title, '', '', '', '', id, book_id FROM recipes
WHERE id NOT IN (SELECT recipe_id FROM recipe_search);

INSERT OR IGNORE INTO recipe_search_filters (recipe_id, book_id)
SELECT id, book_id FROM recipes;
