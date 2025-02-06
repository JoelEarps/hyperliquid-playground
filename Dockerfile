ARG PLATFORM
ARG TARGET_APP

FROM --platform=${PLATFORM} rust:1.84.0-alpine AS build

ARG RUST_TARGET
WORKDIR /app

COPY ./hyperliquid-spike .

RUN rustup target add ${RUST_TARGET}-unknown-linux-musl
RUN cargo build --target ${RUST_TARGET}-unknown-linux-musl --release

FROM --platform=${PLATFORM} scratch AS prod
ARG RUST_TARGET

COPY --from=build /app/target/${RUST_TARGET}-unknown-linux-musl/release/hyperliquid-spike /opt/hyperliquid-spike

CMD ["/opt/hyperliquid-spike"]