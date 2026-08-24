#!/usr/bin/env bash
set -euo pipefail

output_dir="$(cd "$(dirname "$0")" && pwd)"
cd "$output_dir"
umask 077

openssl genrsa -out ca.key 4096
openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 \
  -subj "/CN=grpc-scylladb-dev-ca" -out ca.crt

openssl genrsa -out server.key 2048
openssl req -new -key server.key -subj "/CN=localhost" -out server.csr
printf 'subjectAltName = DNS:localhost, IP:127.0.0.1\nextendedKeyUsage = serverAuth\n' > server.ext
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out server.crt -days 825 -sha256 -extfile server.ext

for subject in reader.example writer.example admin.example; do
  openssl genrsa -out "$subject.key" 2048
  openssl req -new -key "$subject.key" -subj "/CN=$subject" -out "$subject.csr"
  printf 'extendedKeyUsage = clientAuth\n' > client.ext
  openssl x509 -req -in "$subject.csr" -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out "$subject.crt" -days 825 -sha256 -extfile client.ext
done

rm -f *.csr *.ext *.srl
printf 'Generated dev mTLS fixtures in %s\n' "$output_dir"
