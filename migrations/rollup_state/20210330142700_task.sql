CREATE TYPE task_status AS ENUM('inited', 'proving', 'proved');

CREATE TABLE task (
    task_id VARCHAR(30) NOT NULL,
    circuit VARCHAR(30) NOT NULL,
    block_id BIGINT NOT NULL, -- TODO: combine with task_id
    input jsonb NOT NULL,
    output jsonb DEFAULT NULL,
    public_input BYTEA DEFAULT NULL,
    proof BYTEA DEFAULT NULL,
    status task_status NOT NULL DEFAULT 'inited',
    prover_id VARCHAR(128) DEFAULT NULL,
    created_time TIMESTAMP(0) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_time TIMESTAMP(0) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (task_id)
);

-- CREATE INDEX task_idx_taskid ON task (task_id);
-- CREATE INDEX task_idx_status ON task (status);
-- CREATE UNIQUE INDEX task_prover_constraint ON task (task_id, prover_id);

CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
   NEW.updated_time = CURRENT_TIMESTAMP;
   RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_timestamp BEFORE UPDATE
ON task FOR EACH ROW EXECUTE PROCEDURE
update_timestamp();
