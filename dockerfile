FROM rust:1.76-bookworm AS builder

WORKDIR /usr/src/rust_axum_service

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

WORKDIR /usr/src/rust_axum_service

COPY --from=builder /usr/src/rust_axum_service/target/release/boraxpr .

CMD ["./boraxpr"]