# ---- Builder stage ----
FROM rust:1.88-alpine AS builder
WORKDIR /app

# Устанавливаем зависимости, необходимые для сборки, включая curl
RUN apk add --no-cache musl-dev pkgconfig openssl-dev curl
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --locked && rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release --locked

# ---- Runtime stage ----
FROM alpine:latest
WORKDIR /app

# Устанавливаем python3, pip и библиотеку requests
RUN apk add --no-cache python3 py3-requests

# Копируем собранный бинарник
COPY --from=builder /app/target/release/script-server /app/script-server

RUN mkdir -p /app/scripts

EXPOSE 3000
CMD ["/app/script-server"]