FROM rust:1.83-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --bin codegraph

FROM alpine:3.20
RUN apk add --no-cache ca-certificates curl
COPY --from=builder /app/target/release/codegraph /usr/local/bin/
EXPOSE 3000
CMD ["codegraph", "serve"]
