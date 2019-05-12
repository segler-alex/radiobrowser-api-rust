FROM rust:1
ADD . /root
WORKDIR /root
RUN cargo install && cargo clean

EXPOSE 8080
ENV STATIC_FILES_DIR=./static/
CMD [ "radiobrowser-api-rust", "-f", "radiobrowser.toml"]
