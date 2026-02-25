# Simple Dockerfile for local development/testing
FROM rust:1.75-bookworm as builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/custom-ha-service /usr/local/bin/custom-ha-service

ENV MQTT_BROKER=homeassistant
ENV MQTT_PORT=1883
ENV SUBSCRIBE_TOPIC=custom-service/in
ENV PUBLISH_TOPIC=custom-service/out
ENV CLIENT_ID=custom-ha-service

CMD ["/usr/local/bin/custom-ha-service"]
