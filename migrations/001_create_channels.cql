CREATE KEYSPACE IF NOT EXISTS grpc_starter
WITH replication = {'class': 'NetworkTopologyStrategy', 'datacenter1': 1};

CREATE TABLE IF NOT EXISTS grpc_starter.channels (
    id uuid PRIMARY KEY,
    name text,
    created_at_unix_ms bigint
);

CREATE TABLE IF NOT EXISTS grpc_starter.auth_subject_roles (
    subject text PRIMARY KEY,
    role text
);

INSERT INTO grpc_starter.auth_subject_roles (subject, role)
VALUES ('reader.example', 'reader');

INSERT INTO grpc_starter.auth_subject_roles (subject, role)
VALUES ('writer.example', 'writer');

INSERT INTO grpc_starter.auth_subject_roles (subject, role)
VALUES ('admin.example', 'admin');
