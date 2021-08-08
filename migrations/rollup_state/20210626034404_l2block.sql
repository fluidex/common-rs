CREATE TYPE block_status AS ENUM('uncommited', 'commited', 'submitted', 'confirmed');

CREATE TABLE l2block (
    block_id BIGINT PRIMARY KEY,
    new_root VARCHAR(256) NOT NULL,
    status block_status NOT NULL DEFAULT 'uncommited',
    -- TODO: tx_hash
    detail jsonb NOT NULL,
    created_time TIMESTAMP(0) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
