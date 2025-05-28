-- Add migration script here
CREATE TYPE status AS ENUM ('ExecutionCompleted', 'ProofGenerationInProgress', 'ProofGenerationFailed', 'ProofGenerationSuccessful');
CREATE TABLE nexus_blocks (
    block_hash BYTEA PRIMARY KEY,
    block_number BIGINT NOT NULL,
    block TEXT NOT NULL,
    jmt_version BIGINT NOT NULL,
    zkvm_inputs BYTEA NOT NULL,
    block_status TEXT NOT NULL
);

CREATE TABLE transaction_with_status (
    transaction_hash BYTEA PRIMARY KEY,
    transaction TEXT NOT NULL,
    status TEXT NOT NULL,
    block_hash BYTEA
);

CREATE TABLE proofs (
    block_hash BYTEA PRIMARY KEY,
    block_number BIGINT NOT NULL,
    proof BYTEA NOT NULL
);
