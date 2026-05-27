FROM rust:1.89-bookworm AS builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown

COPY . .

RUN cargo build -p server --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/migrations /app/migrations

EXPOSE 3001

ENV PORT=3001

CMD ["/app/server"]
