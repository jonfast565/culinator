ALTER TABLE formula_ingredients ADD COLUMN is_reference INTEGER NOT NULL DEFAULT 0;
ALTER TABLE formula_ingredients ADD COLUMN scalable INTEGER NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS formula_conversions (
    id TEXT PRIMARY KEY,
    formula_id TEXT NOT NULL REFERENCES formulas(id) ON DELETE CASCADE,
    view TEXT NOT NULL CHECK (view IN ('reference','total')),
    reference_mass_grams REAL NOT NULL,
    total_mass_grams REAL NOT NULL,
    result_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_formula_conversions_formula ON formula_conversions(formula_id, created_at DESC);
