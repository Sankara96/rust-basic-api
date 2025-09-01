# ---- Build static binary
FROM rust:1.80-alpine AS build
RUN apk add --no-cache musl-dev pkgconfig
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && cargo build --release
COPY . .
ENV RUSTFLAGS="-C target-cpu=x86-64-v3"
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---- Final image (distroless)
FROM gcr.io/distroless/static-debian12:nonroot
WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/nuage-robo-api /app/nuage-robo-api
USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/app/nuage-robo-api"]
