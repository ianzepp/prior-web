FROM rust:1.93-slim AS builder
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;; \
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;; \
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.5/cargo-leptos-${TARGET}.tar.gz" \
    | tar -xz --strip-components=1 -C /usr/local/bin

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY public ./public
COPY src ./src

RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
      aarch64) TOOLCHAIN="1.93-aarch64-unknown-linux-gnu" ;; \
      x86_64) TOOLCHAIN="1.93-x86_64-unknown-linux-gnu" ;; \
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    rustup target add wasm32-unknown-unknown --toolchain "$TOOLCHAIN" \
    && cargo leptos build --release

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
