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

Untuk mengaktifkan mTLS, isi ketiga environment variable berikut dengan file
PEM. Ketiganya harus tersedia bersama-sama:

```bash
export GRPC_TLS_CERT=/path/to/server.crt
export GRPC_TLS_KEY=/path/to/server.key
export GRPC_TLS_CLIENT_CA=/path/to/ca.crt
export GRPC_RBAC_SUBJECTS='reader.example=reader;writer.example=writer;admin.example=admin'
```

Subject diambil dari Common Name sertifikat client. Role `reader` dapat
melakukan `read` dan `list`, `writer` juga dapat `create` dan `update`, dan
`admin` dapat semua operasi channel. Subject yang tidak terdaftar ditolak
dengan `PERMISSION_DENIED`; koneksi tanpa client certificate ditolak oleh TLS.

```bash
cargo run
```

Jalankan integration test CRUD terhadap server yang sedang berjalan:

```bash
TEST_GRPC_ENDPOINT=http://127.0.0.1:50051 cargo test --test channel_crud
```

Untuk menampilkan output setiap operasi CRUD:

```bash
TEST_GRPC_ENDPOINT=http://127.0.0.1:50051 cargo test --test channel_crud -- --nocapture
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
