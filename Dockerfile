FROM rust:slim-buster AS buildstage
WORKDIR /build
RUN /bin/sh -c set -eux;\
    rustup component add rustfmt;\
    apt-get update;\
    apt-get install -y --no-install-recommends libsqlite3-dev;\
    rm -rf /var/lib/apt/lists/*;
COPY . /build/
RUN cargo build --release
FROM debian:buster-slim
COPY --from=buildstage /build/target/release/cloud-config /usr/bin/
RUN /bin/sh -c set -eux;\
    apt-get update;\
    apt-get install -y --no-install-recommends libsqlite3-0;\
    rm -rf /var/lib/apt/lists/*;
CMD ["cloud-config"]
