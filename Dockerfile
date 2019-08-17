FROM rust:1
ADD . /root
WORKDIR /root
RUN cargo build --release

FROM ubuntu:bionic
EXPOSE 8080
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=0 /root/target/release/radiobrowser-api-rust /usr/bin/
COPY --from=0 /root/static/ /root/static/
COPY --from=0 /root/radiobrowser.toml /etc/radiobrowser.toml
ENV STATIC_FILES_DIR=/root/static/
CMD [ "radiobrowser-api-rust", "-f", "/etc/radiobrowser.toml"]
