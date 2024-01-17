FROM rust:latest

WORKDIR /app
COPY src ./src/
COPY Cargo.toml ./
COPY Cargo.lock ./

RUN cargo build --release

COPY public ./public/
COPY templates ./templates/
COPY Rocket.toml ./

CMD cargo run --release
