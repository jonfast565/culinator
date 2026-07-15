CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    title TEXT NOT NULL,
    protocol_version TEXT NOT NULL,
    declared_type_json TEXT NOT NULL,
    source_text TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS type_declarations (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    symbol TEXT NOT NULL,
    base_type_json TEXT NOT NULL,
    states_json TEXT NOT NULL DEFAULT '{}',
    properties_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(recipe_id, symbol)
);

CREATE TABLE IF NOT EXISTS resources (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    symbol TEXT NOT NULL,
    resource_kind TEXT NOT NULL,
    declared_type_json TEXT NOT NULL,
    properties_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(recipe_id, symbol)
);

CREATE TABLE IF NOT EXISTS processes (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    parent_process_id TEXT REFERENCES processes(id),
    symbol TEXT NOT NULL,
    declared_type_json TEXT NOT NULL,
    properties_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(recipe_id, symbol)
);

CREATE TABLE IF NOT EXISTS operations (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    process_id TEXT REFERENCES processes(id),
    symbol TEXT NOT NULL,
    declared_type_json TEXT NOT NULL,
    labor_mode TEXT,
    duration_min_seconds INTEGER,
    duration_max_seconds INTEGER,
    properties_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(recipe_id, symbol)
);

CREATE TABLE IF NOT EXISTS operation_dependencies (
    operation_id TEXT NOT NULL REFERENCES operations(id) ON DELETE CASCADE,
    predecessor_operation_id TEXT NOT NULL REFERENCES operations(id),
    dependency_kind TEXT NOT NULL,
    minimum_lag_seconds INTEGER,
    maximum_lag_seconds INTEGER,
    optional INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(operation_id, predecessor_operation_id, dependency_kind)
);

CREATE TABLE IF NOT EXISTS operation_bindings (
    operation_id TEXT NOT NULL REFERENCES operations(id) ON DELETE CASCADE,
    resource_id TEXT NOT NULL REFERENCES resources(id),
    role TEXT NOT NULL,
    quantity_json TEXT,
    exclusive INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(operation_id, resource_id, role)
);

CREATE TABLE IF NOT EXISTS operation_requirements (
    id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL REFERENCES operations(id) ON DELETE CASCADE,
    source_expression TEXT NOT NULL,
    typed_ast_json TEXT
);

CREATE TABLE IF NOT EXISTS operation_effects (
    id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL REFERENCES operations(id) ON DELETE CASCADE,
    target_path TEXT NOT NULL,
    operator TEXT NOT NULL,
    value_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS servings (
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    symbol TEXT NOT NULL,
    declared_type_json TEXT NOT NULL,
    amount_json TEXT NOT NULL,
    mass_grams REAL,
    is_default INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(recipe_id, symbol)
);
