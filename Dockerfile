ARG PLATFORM
ARG TARGET_APP

FROM --platform=${PLATFORM} rust:1.84.0-alpine AS build

RUN apk update && apk add musl-dev openssl-dev protobuf protobuf-dev grpc grpc-plugins

ARG RUST_TARGET
WORKDIR /app

COPY ./hyperliquid-spike .

#  Turn this into dev container?
FROM build as dev_build

RUN rustup target add ${RUST_TARGET}-unknown-linux-musl
RUN --mount=type=ssh RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --target ${RUST_TARGET}-unknown-linux-musl

CMD ["/app/target/${RUST_TARGET}-unknown-linux-musl/release/hyperliquid-spike"]

FROM build as prod_build

ARG RUST_TARGET

RUN rustup target add ${RUST_TARGET}-unknown-linux-musl
RUN --mount=type=ssh RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --target ${RUST_TARGET}-unknown-linux-musl --release

FROM --platform=${PLATFORM} scratch AS prod

ARG RUST_TARGET

COPY --from=prod_build /app/target/${RUST_TARGET}-unknown-linux-musl/release/hyperliquid-spike /opt/hyperliquid-spike

# TODO: Remove glibcc dynamic libs by using rust openssl implementation
COPY --from=prod_build /usr/lib/libgcc_s.so.1 /usr/lib/libgcc_s.so.1
COPY --from=prod_build /usr/lib/libssl.so.3 /usr/lib/libssl.so.3
COPY --from=prod_build /usr/lib/libcrypto.so.3 /usr/lib/libcrypto.so.3
COPY --from=prod_build /lib/ld-musl-aarch64.so.1 /lib/ld-musl-aarch64.so.1
COPY --from=prod_build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

CMD ["/opt/hyperliquid-spike"]

FROM alpine:latest AS debug

ARG RUST_TARGET

COPY --from=build /app/target/${RUST_TARGET}-unknown-linux-musl/release/hyperliquid-spike /opt/hyperliquid-spike

COPY --from=build /usr/lib/libgcc_s.so.1 /usr/lib/libgcc_s.so.1

CMD ["/bin/sh"]