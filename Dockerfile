FROM rust:1.93-slim AS builder
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev ca-certificates perl \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-leptos --locked
RUN rustup target add wasm32-unknown-unknown

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY public ./public
COPY src ./src

RUN cargo leptos build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/prior-web /app/prior-web
COPY --from=builder /app/public /app/public
COPY --from=builder /app/target/prior-web/site /app/target/prior-web/site

ENV PORT=3000
EXPOSE 3000

CMD ["./prior-web"]
