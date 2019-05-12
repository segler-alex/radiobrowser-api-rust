FROM rust:1
ADD . /root
WORKDIR /root
RUN cargo install && cargo clean

EXPOSE 8080
CMD [ "radiobrowser-api-rust" ]
