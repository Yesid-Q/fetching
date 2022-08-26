FROM rust:1.63.0-buster

WORKDIR /usr/fetching

COPY . .

RUN cargo install --path .

CMD [ "fetching" ]
