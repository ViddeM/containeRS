##################
### BASE STAGE ###
##################
FROM rust:1.77 as base

ENV TZ=Europe/Stockholm
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -y tzdata

WORKDIR /app

# Install build dependencies
RUN cargo install sqlx-cli --version 0.7.3
RUN cargo install --locked strip_cargo_version

RUN rustup target add x86_64-unknown-linux-musl

# Required by `brotli-sys` and `openssl`
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev

###########################
### STRIP-VERSION STAGE ###
###########################
FROM base AS strip-version

COPY Cargo.lock Cargo.toml ./
RUN strip_cargo_version

###################
### BUILD STAGE ###
###################
FROM base AS build

# compile dependencies first so that Docker caches them
RUN cargo init --bin
COPY --from=strip-version /app/Cargo.* /app/
RUN cargo build --release --target x86_64-unknown-linux-musl

# compile the app itself
COPY . .

RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl
RUN strip /app/target/x86_64-unknown-linux-musl/release/container-registry-rs

########################
### PRODUCTION STAGE ###
########################
FROM scratch
WORKDIR /

ENV PORT=8080
EXPOSE 8080

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/container-registry-rs /
COPY migrations /

CMD ["/container-registry-rs"]
