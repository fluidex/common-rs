-- Add migration script here

-- public data is used for submit block in L1 contract
ALTER TABLE task ADD public_data bytea;