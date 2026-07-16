# HelixCore multi-service image. Entrypoint selects binary via args[0].
# Build: docker build -t helixforge/helix-core:0.1.0 .
# Run:   docker run -e DATABASE_URL=... helixforge/helix-core:0.1.0 gateway

FROM rust:1.97-bookworm AS builder
WORKDIR /src
# Cache deps: copy manifests first
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .cargo ./.cargo
COPY crates ./crates
COPY services ./services
COPY projects ./projects
COPY tools ./tools
# The dev toolchain file pins stable-x86_64-pc-windows-msvc for local Windows builds.
# Use the stable toolchain already present in this Linux image and remove the
# toolchain file so rustup does not re-download components.
ENV RUSTUP_TOOLCHAIN=stable
ENV SQLX_OFFLINE=true
RUN rm -f rust-toolchain.toml
RUN cargo build --release \
    -p gateway \
    -p agent_hub \
    -p vault_service \
    -p billing_service \
    -p observability_service \
    -p auth_adapter \
    -p helix_db --bins

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -u 65532 -r -s /usr/sbin/nologin helix
WORKDIR /app
COPY --from=builder /src/target/release/gateway /app/bin/gateway
COPY --from=builder /src/target/release/agent_hub /app/bin/agent_hub
COPY --from=builder /src/target/release/vault_service /app/bin/vault_service
COPY --from=builder /src/target/release/billing_service /app/bin/billing_service
COPY --from=builder /src/target/release/observability_service /app/bin/observability_service
COPY --from=builder /src/target/release/auth_adapter /app/bin/auth_adapter
COPY --from=builder /src/target/release/helix-audit-rehash /app/bin/helix-audit-rehash
COPY --from=builder /src/target/release/helix-migrate /app/bin/helix-migrate
COPY deploy/docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh \
    && chown -R helix:helix /app
USER helix
ENV PATH="/app/bin:${PATH}"
EXPOSE 8080 8081 8082 8083 8084 8085
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["gateway"]
