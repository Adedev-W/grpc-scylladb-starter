# grpc-scylladb-starter

A Rust backend template with a gRPC API, ScyllaDB persistence, mutual TLS,
and database-backed role-based access control (RBAC).

## Prerequisites

- Rust 1.85 or newer
- Docker Compose
- OpenSSL for generating development certificates

## Run locally

From this directory, generate local certificates and start ScyllaDB:

```bash
./certs/dev/generate.sh
docker compose up -d --wait
```

If a local Docker volume was created by an older version of the project,
remove it once so the UUID schema can be created:

```bash
docker compose down -v
docker compose up -d --wait
```

This removes local ScyllaDB data.

Start the server with mTLS and RBAC enabled:

```bash
set -a
source .env.example
set +a
cargo run
```

The server listens on `127.0.0.1:50051`. Roles are stored in the
`auth_subject_roles` table. The development certificates identify
`reader.example`, `writer.example`, and `admin.example` subjects.

For local CRUD testing without certificates, explicitly opt in to insecure
mode:

```bash
env -u GRPC_TLS_CERT -u GRPC_TLS_KEY -u GRPC_TLS_CLIENT_CA \
  GRPC_ALLOW_INSECURE=true cargo run
```

## Tests

Run deterministic checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib
cargo test --test rbac_policy -- --nocapture
```

With the server running in insecure mode, run the CRUD integration test:

```bash
TEST_GRPC_ENDPOINT=http://127.0.0.1:50051 \
  cargo test --test channel_crud -- --nocapture
```

With the mTLS server running, the integration tests use the generated
development certificates automatically:

```bash
TEST_GRPC_ENDPOINT=https://127.0.0.1:50051 \
  cargo test --test channel_crud -- --nocapture
TEST_GRPC_ENDPOINT=https://127.0.0.1:50051 \
  cargo test --test rbac_grpc -- --nocapture
```

The CRUD API uses UUID channel IDs and ScyllaDB paging tokens for listing.

## Project structure

- `src/domain`: business models and validation rules.
- `src/application`: use cases, authorization policy, and repository ports.
- `src/transport/grpc`: gRPC and protobuf adapters.
- `src/infrastructure/scylla`: ScyllaDB implementations.
- `proto`: source protobuf definitions compiled by `build.rs`.
- `migrations`: schema and development seed data.
- `tests`: integration and policy tests.

Development private keys are generated locally and ignored by Git. Never
commit production certificates, keys, or credentials.
