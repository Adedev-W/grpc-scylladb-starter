CREATE KEYSPACE IF NOT EXISTS grpc_starter
WITH replication = {'class': 'NetworkTopologyStrategy', 'datacenter1': 1};

CREATE TABLE IF NOT EXISTS grpc_starter.channels (
    id bigint PRIMARY KEY,
    name text,
    created_at_unix_ms bigint
);