-- Kitchen mode enhancements for recipe tries (executions).

ALTER TABLE executions ADD COLUMN title TEXT;
ALTER TABLE executions ADD COLUMN findings TEXT;

ALTER TABLE execution_operations ADD COLUMN operation_symbol TEXT;

CREATE INDEX IF NOT EXISTS idx_execution_operations_symbol
    ON execution_operations(execution_id, operation_symbol);

CREATE INDEX IF NOT EXISTS idx_execution_observations_execution
    ON execution_observations(execution_id, observed_at DESC);
