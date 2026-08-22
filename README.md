# grpc-scylladb-starter

Backend template dengan gRPC API dan ScyllaDB. Runtime menggunakan repository
ScyllaDB dan konfigurasi environment.

## Run server

Konfigurasi opsional:

```bash
export GRPC_ADDR=127.0.0.1:50051
export SCYLLA_NODES=127.0.0.1:9042
export SCYLLA_KEYSPACE=grpc_starter
```

```bash
cargo run
```

Jalankan integration test CRUD terhadap server yang sedang berjalan:

```bash
TEST_GRPC_ENDPOINT=http://127.0.0.1:50051 cargo test --test channel_crud
```

Pastikan ScyllaDB dan schema sudah aktif sebelum menjalankan server:

```bash
sudo docker compose up -d
cargo run
```

Test melakukan create, get, update, list, delete, lalu memastikan channel yang
dihapus menghasilkan status gRPC `NOT_FOUND`.

Struktur utama:

- `src/domain`: aturan bisnis dan model.
- `src/application`: use case dan repository port.
- `src/transport/grpc`: adapter protobuf/tonic.
- `src/infrastructure`: implementasi adapter eksternal.
- `src/bootstrap.rs`: dependency wiring.
