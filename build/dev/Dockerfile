FROM rust:latest

WORKDIR /usr/src/onetime-rs

COPY onetime-rs .

RUN cargo install --path .

EXPOSE 8000

CMD ["onetime-rs"]

