FROM rust:1.76-bookworm

WORKDIR /usr/src/rust_axum_service

COPY . .

RUN cargo install cargo-watch

EXPOSE 7070

CMD ["cargo", "watch", "-x", "run"]