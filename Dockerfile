FROM rust:trixie

WORKDIR /usr/src/djit

COPY . .


RUN cargo install --path .

CMD [ "djit" ]