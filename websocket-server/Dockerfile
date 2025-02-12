# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.83.0
ARG APP_NAME=websocket-server

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

# Install host build dependencies
RUN apk add --no-cache clang lld musl-dev git

# Copy the entire workspace
COPY . .

# Build the specific package
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release -p ${APP_NAME} && \
    cp ./target/release/${APP_NAME} /bin/server

FROM alpine:3.18 AS final

# Create a non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/server /bin/

EXPOSE 8081

CMD ["/bin/server"] 