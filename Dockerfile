FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Dependencies and environment variables
RUN USER=root apt-get update && apt-get -y install libssl-dev libsodium-dev

ENV RUSTFLAGS='-C target-cpu=native'

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json && \
    mkdir templates  # We need this folder, otherwise templates can't be found

# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:trixie-slim AS runtime
ARG APP=/usr/src/app

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN apt-get update \
    && apt-get install -y libssl3 ca-certificates curl wget\
    && groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP} \
    && mkdir /app /data

# Ports
EXPOSE 8079

RUN chown -R $APP_USER:$APP_USER ${APP} /data

COPY --from=builder \
     /app/target/release/assoc-admin-bot \
     ${APP}/assoc-admin-bot

USER $APP_USER
WORKDIR ${APP}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["./assoc-admin-bot", "--check"]

CMD ["./assoc-admin-bot"]
