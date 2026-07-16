-- HACCP (Hazard Analysis and Critical Control Points) management tables.

CREATE TABLE haccp_plans (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_haccp_plans_recipe_id ON haccp_plans(recipe_id);

CREATE TABLE haccp_hazards (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES haccp_plans(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    hazard_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL,
    likelihood TEXT NOT NULL,
    preventive_measures TEXT,
    is_ccp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_haccp_hazards_plan_id ON haccp_hazards(plan_id);

CREATE TABLE haccp_ccps (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES haccp_plans(id) ON DELETE CASCADE,
    hazard_id TEXT REFERENCES haccp_hazards(id) ON DELETE SET NULL,
    position INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    operation_symbol TEXT,
    critical_limit TEXT NOT NULL,
    monitoring_procedure TEXT NOT NULL,
    monitoring_frequency TEXT,
    corrective_action TEXT NOT NULL,
    verification_procedure TEXT,
    responsible_party TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_haccp_ccps_plan_id ON haccp_ccps(plan_id);

CREATE TABLE haccp_monitoring_records (
    id TEXT PRIMARY KEY,
    ccp_id TEXT NOT NULL REFERENCES haccp_ccps(id) ON DELETE CASCADE,
    recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    measured_value TEXT NOT NULL,
    within_limit INTEGER NOT NULL,
    corrective_action_taken TEXT,
    recorded_by TEXT,
    notes TEXT
);

CREATE INDEX idx_haccp_monitoring_records_ccp_id ON haccp_monitoring_records(ccp_id);
