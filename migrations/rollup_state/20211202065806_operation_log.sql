-- Add migration script here

CREATE TABLE operation_log (
    id BIGINT CHECK (id >= 0) NOT NULL PRIMARY KEY,
    time TIMESTAMP(0) NOT NULL,
    method TEXT NOT NULL,
    params TEXT NOT NULL
);
