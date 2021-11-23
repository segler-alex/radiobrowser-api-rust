FROM alpine:3.14
ADD . /app
WORKDIR /app
RUN apk update
RUN apk add rustup openssl-dev gcc g++
RUN rustup-init -y
ENV PATH="/root/.cargo/bin:$PATH"
RUN cargo build --release

FROM alpine:3.14
EXPOSE 8080
COPY --from=0 /app/target/release/radiobrowser-api-rust /usr/bin/
COPY --from=0 /app/static/ /usr/lib/radiobrowser/static/
COPY --from=0 /app/etc/config-example.toml /etc/radiobrowser/config.toml
COPY --from=0 /app/etc/*.csv /etc/radiobrowser/
RUN addgroup -S radiobrowser && \
 adduser -S -G radiobrowser radiobrowser && \
 apk add libgcc && \
 mkdir -p /var/log/radiobrowser/ && \
 chown -R radiobrowser:radiobrowser /var/log/radiobrowser/ && \
 chmod go+r /etc/radiobrowser/config.toml
ENV STATIC_FILES_DIR=/usr/lib/radiobrowser/static/
USER radiobrowser:radiobrowser
CMD [ "radiobrowser-api-rust", "-f", "/etc/radiobrowser/config.toml", "-vvv"]
