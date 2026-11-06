# Built from the repository root: docker build -f backend/Dockerfile .
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY backend backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p scratchme-api && cp target/release/scratchme-api /scratchme-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 scratchme
COPY --from=build /scratchme-api /usr/local/bin/scratchme-api
USER scratchme
ENV SCRATCHME_BIND=0.0.0.0:8080 RUST_LOG=info
EXPOSE 8080
CMD ["scratchme-api"]
