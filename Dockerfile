FROM rust:slim-buster AS buildstage
WORKDIR /build
ENV PROTOC_NO_VENDOR 1
RUN rustup component add rustfmt && \
    apt-get update && \
    apt-get install -y --no-install-recommends libsqlite3-dev protobuf-compiler && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY . /build/
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends libsqlite3-0 && \
    apt-get clean && \
    rm -rf  /var/log/*log /var/lib/apt/lists/* /var/log/apt/* /var/lib/dpkg/*-old /var/cache/debconf/*-old
RUN useradd -m chain
USER chain
COPY --from=buildstage /build/target/release/cloud-config /usr/bin/
CMD ["cloud-config"]
