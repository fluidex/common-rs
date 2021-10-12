CREATE TYPE block_status AS ENUM('uncommited', 'commited', 'verified');

CREATE TABLE l2block (
    block_id BIGINT PRIMARY KEY,
    new_root VARCHAR(256) NOT NULL,
    status block_status NOT NULL DEFAULT 'uncommited',
    detail jsonb NOT NULL,
    l1_tx_hash: BYTEA,
    created_time TIMESTAMP(0) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
