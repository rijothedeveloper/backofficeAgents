FROM rust:1-slim-trixie AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:trixie-slim

# curl is here only for the container health check.
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --no-create-home app
COPY --from=builder /build/target/release/gateway /usr/local/bin/gateway
USER app

EXPOSE 8080
CMD ["gateway"]