FROM node:24.22.3-bookworm-slim AS frontend
RUN npm install --global pnpm@11.25.0
WORKDIR /src/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

FROM rust:1.100.0-bookworm AS backend
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY backend/ ./backend/
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 coinz && useradd --uid 10001 --gid coinz --no-create-home coinz \
    && mkdir /data && chown coinz:coinz /data
WORKDIR /app
COPY --from=backend /src/target/release/coinz /app/coinz
COPY --from=frontend /src/frontend/build /app/frontend
ENV COINZ_BIND=0.0.0.0:3000 COINZ_DATA_DIR=/data COINZ_STATIC_DIR=/app/frontend
USER 10001:10001
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3000/api/health || exit 1
ENTRYPOINT ["/app/coinz"]
CMD ["serve"]
