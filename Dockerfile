FROM rust:1
ADD . /root
WORKDIR /root
RUN cargo build --release

FROM ubuntu:bionic
EXPOSE 8080
RUN groupadd -r radiobrowser && \
 useradd --no-log-init -r -g radiobrowser radiobrowser && \
 apt-get update && \
 apt-get install -y libssl1.1 && \
 rm -rf /var/lib/apt/lists/* && \
 mkdir -p /usr/lib/radiobrowser/static/ && \
 mkdir -p /var/log/radiobrowser/ && \
 chown -R radiobrowser:radiobrowser /var/log/radiobrowser/
COPY --from=0 /root/target/release/radiobrowser-api-rust /usr/bin/
COPY --from=0 /root/static/ /usr/lib/radiobrowser/static/
COPY --from=0 /root/etc/config-example.toml /etc/radiobrowser.toml
ENV STATIC_FILES_DIR=/usr/lib/radiobrowser/static/
USER radiobrowser:radiobrowser
CMD [ "radiobrowser-api-rust", "-f", "/etc/radiobrowser.toml"]
