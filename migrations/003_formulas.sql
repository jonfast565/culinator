CREATE TABLE IF NOT EXISTS formulas (
    id TEXT PRIMARY KEY,
    recipe_id TEXT REFERENCES recipes(id) ON DELETE CASCADE,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    basis TEXT NOT NULL CHECK (basis IN ('bakers_percent','percent_of_total','absolute_mass')),
    properties_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(recipe_id, symbol)
);

CREATE TABLE IF NOT EXISTS formula_ingredients (
    id TEXT PRIMARY KEY,
    formula_id TEXT NOT NULL REFERENCES formulas(id) ON DELETE CASCADE,
    resource_id TEXT REFERENCES resources(id) ON DELETE SET NULL,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    stage TEXT NOT NULL DEFAULT 'final',
    basis TEXT NOT NULL CHECK (basis IN ('bakers_percent','percent_of_total','absolute_mass')),
    percentage REAL,
    mass_grams REAL,
    is_flour INTEGER NOT NULL DEFAULT 0,
    water_fraction REAL NOT NULL DEFAULT 0 CHECK (water_fraction >= 0 AND water_fraction <= 1),
    position INTEGER NOT NULL DEFAULT 0,
    properties_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(formula_id, symbol, stage)
);

CREATE TABLE IF NOT EXISTS formula_runs (
    id TEXT PRIMARY KEY,
    formula_id TEXT NOT NULL REFERENCES formulas(id) ON DELETE CASCADE,
    execution_id TEXT REFERENCES executions(id) ON DELETE SET NULL,
    target_mass_grams REAL NOT NULL,
    result_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_formulas_recipe ON formulas(recipe_id);
CREATE INDEX IF NOT EXISTS idx_formula_ingredients_formula ON formula_ingredients(formula_id, position);
CREATE INDEX IF NOT EXISTS idx_formula_runs_formula ON formula_runs(formula_id, created_at DESC);
