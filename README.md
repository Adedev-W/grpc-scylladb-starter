# grpc-scylladb-starter

Backend template dengan gRPC API dan boundary repository yang siap dihubungkan
ke ScyllaDB. Runtime saat ini memakai in-memory adapter sebagai vertical slice
awal; konfigurasi dan port database sudah dipisahkan untuk tahap adapter berikutnya.

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

Struktur utama:

- `src/domain`: aturan bisnis dan model.
- `src/application`: use case dan repository port.
- `src/transport/grpc`: adapter protobuf/tonic.
- `src/infrastructure`: implementasi adapter eksternal.
- `src/bootstrap.rs`: dependency wiring.
