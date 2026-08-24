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
```

Fixture mTLS development tersedia di `certs/dev/`. Regenerasi dengan:

```bash
./certs/dev/generate.sh
```

Jalankan server dengan fixture development:

```bash
set -a; source .env.example; set +a
cargo run
```

Client harus memakai `certs/dev/ca.crt` sebagai CA dan salah satu pasangan
client certificate/key. Role disimpan di tabel `auth_subject_roles`, bukan di
metadata client. Migration juga men-seed `reader.example`, `writer.example`,
dan `admin.example`.

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

Test policy dengan output keputusan yang mudah dibaca:

```bash
cargo test --test rbac_policy -- --nocapture
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
