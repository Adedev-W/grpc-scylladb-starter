CREATE TABLE IF NOT EXISTS grpc_starter.channels (
    id bigint PRIMARY KEY,
    name text,
    created_at_unix_ms bigint
);