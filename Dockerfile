# create docker file from rust image
FROM rust:1.73-bookworm

WORKDIR /home

ADD . .

RUN cargo build --release


FROM debian:bookworm

# copy over the build artifact
COPY --from=0 /home/target/release/lol-tracker /bot/lol-tracker

RUN apt update
RUN apt install -y libssl-dev curl

WORKDIR /bot

# run the binary
ENTRYPOINT ["/bot/lol-tracker"]
