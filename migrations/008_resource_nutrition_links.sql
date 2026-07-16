CREATE TABLE IF NOT EXISTS resource_nutrition_links (
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    resource_symbol TEXT NOT NULL,
    fdc_id INTEGER NOT NULL,
    food_description TEXT NOT NULL,
    linked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (recipe_id, resource_symbol)
);
CREATE INDEX IF NOT EXISTS idx_resource_nutrition_links_recipe
    ON resource_nutrition_links(recipe_id);
