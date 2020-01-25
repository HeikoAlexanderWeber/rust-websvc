# BUILD image and setup
FROM rust:1.40-slim-stretch as build
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install \
    ca-certificates \
    pkg-config \
    libssl-dev \
    xutils-dev \
    curl \
    musl-dev \
    musl-tools \
    build-essential \
    -y

ENV PKG_CONFIG_ALLOW_CROSS=1

RUN rustup target add x86_64-unknown-linux-musl

# Init environment and cache dependencies
WORKDIR /home/app
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir src \
    && printf "// dummy file" > src/lib.rs \
    && RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl \
    && rm -rf src/

# Copy sourcecode and build application
COPY . .
RUN \
    RUSTFLAGS=-Clinker=musl-gcc \
    cargo build --release \
    --target=x86_64-unknown-linux-musl


# Setup RUN image
FROM alpine:3.11 as run
LABEL maintainer="Weber, Heiko Alexander <heiko.a.weber@gmail.com>"
EXPOSE 8080

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
RUN apk update && apk upgrade

# Add a new user (principal of least privilege)
RUN addgroup -g 1000 app && adduser -D -s /bin/sh -u 1000 -G app app
USER app

# Copy built application
WORKDIR /home/app
COPY --from=build \
    /home/app/target/x86_64-unknown-linux-musl/release/rust-websvc .

# Entrypoint
ENTRYPOINT [ "/home/app/rust-websvc" ]
