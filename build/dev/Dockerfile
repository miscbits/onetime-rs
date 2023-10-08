FROM rust:latest

WORKDIR /usr/src/onetime-rs

COPY onetime-rs .

RUN cargo install --path .

CMD ["onetime-rs"]

