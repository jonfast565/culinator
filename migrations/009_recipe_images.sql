CREATE TABLE IF NOT EXISTS recipe_images (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    handle TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'cover',
    operation_symbol TEXT,
    media_type TEXT NOT NULL,
    file_name TEXT,
    data_base64 TEXT NOT NULL,
    byte_size INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (recipe_id, handle)
);
CREATE INDEX IF NOT EXISTS idx_recipe_images_recipe ON recipe_images(recipe_id);
