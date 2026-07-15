ALTER TABLE recipes ADD COLUMN properties_json TEXT NOT NULL DEFAULT '{}';

CREATE TABLE IF NOT EXISTS recipe_revisions (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    parent_revision_id TEXT REFERENCES recipe_revisions(id),
    version_label TEXT,
    source_text TEXT NOT NULL,
    source_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_recipe_revisions_recipe ON recipe_revisions(recipe_id, created_at DESC);

CREATE TABLE IF NOT EXISTS recipe_yields (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    output_resource_id TEXT REFERENCES resources(id),
    amount_json TEXT NOT NULL,
    finished_mass_grams REAL,
    portion_count REAL,
    properties_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS nutrient_definitions (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    canonical_unit TEXT NOT NULL,
    daily_value_json TEXT
);

CREATE TABLE IF NOT EXISTS resource_nutrients (
    resource_id TEXT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    nutrient_id TEXT NOT NULL REFERENCES nutrient_definitions(id),
    amount_per_100g REAL NOT NULL,
    source_name TEXT,
    source_record_id TEXT,
    snapshot_at TEXT,
    PRIMARY KEY(resource_id, nutrient_id)
);

CREATE TABLE IF NOT EXISTS allergens (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS resource_allergens (
    resource_id TEXT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    allergen_id TEXT NOT NULL REFERENCES allergens(id),
    relation TEXT NOT NULL DEFAULT 'contains',
    PRIMARY KEY(resource_id, allergen_id, relation)
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS recipe_tags (
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY(recipe_id, tag_id)
);

CREATE TABLE IF NOT EXISTS executions (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id),
    recipe_revision_id TEXT REFERENCES recipe_revisions(id),
    scale_factor REAL NOT NULL DEFAULT 1.0,
    status TEXT NOT NULL,
    scheduled_start TEXT,
    started_at TEXT,
    completed_at TEXT,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS execution_operations (
    execution_id TEXT NOT NULL REFERENCES executions(id) ON DELETE CASCADE,
    operation_id TEXT NOT NULL REFERENCES operations(id),
    status TEXT NOT NULL,
    scheduled_start TEXT,
    scheduled_end TEXT,
    actual_start TEXT,
    actual_end TEXT,
    notes TEXT,
    PRIMARY KEY(execution_id, operation_id)
);

CREATE TABLE IF NOT EXISTS execution_observations (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES executions(id) ON DELETE CASCADE,
    operation_id TEXT REFERENCES operations(id),
    resource_id TEXT REFERENCES resources(id),
    observed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    property_path TEXT NOT NULL,
    value_json TEXT NOT NULL,
    unit TEXT,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_resources_recipe ON resources(recipe_id);
CREATE INDEX IF NOT EXISTS idx_processes_recipe ON processes(recipe_id);
CREATE INDEX IF NOT EXISTS idx_operations_recipe ON operations(recipe_id);
CREATE INDEX IF NOT EXISTS idx_operations_process ON operations(process_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_predecessor ON operation_dependencies(predecessor_operation_id);
CREATE INDEX IF NOT EXISTS idx_bindings_resource ON operation_bindings(resource_id);
CREATE INDEX IF NOT EXISTS idx_executions_recipe ON executions(recipe_id, started_at DESC);
