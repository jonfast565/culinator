-- Recipe-level manual nutrition override and per-ingredient manual facts.
CREATE TABLE IF NOT EXISTS recipe_nutrition (
    recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
    manual_override INTEGER NOT NULL DEFAULT 0,
    facts_json TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS resource_nutrition_manual (
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    resource_symbol TEXT NOT NULL,
    facts_per_100g_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (recipe_id, resource_symbol)
);
