FROM rust:1.73-alpine3.18

WORKDIR /home

ADD . .

RUN apk add --no-cache musl-dev libressl-dev

RUN cargo build --release


FROM alpine:3.18

COPY --from=0 /home/target/release/lol-tracker /bot/lol-tracker

WORKDIR /bot

ENTRYPOINT ["/bot/lol-tracker"]
