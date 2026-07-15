CREATE TABLE IF NOT EXISTS recipe_books (
    id TEXT PRIMARY KEY,
    symbol TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    protocol_version TEXT NOT NULL DEFAULT '0.3',
    declared_type_json TEXT NOT NULL DEFAULT '{"name":"RecipeBook","arguments":[]}',
    properties_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE recipes ADD COLUMN book_id TEXT REFERENCES recipe_books(id) ON DELETE SET NULL;
ALTER TABLE recipes ADD COLUMN book_position INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_recipes_book ON recipes(book_id, book_position, title);
