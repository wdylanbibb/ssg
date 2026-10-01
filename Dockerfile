# syntax=docker/dockerfile:1.7

FROM rust:1.97.1-bookworm AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked \
    && strip target/release/ssg

FROM debian:bookworm-slim AS runtime

ARG UID=10001
ARG GID=10001

RUN groupadd --gid "${GID}" ssg \
    && useradd --uid "${UID}" --gid "${GID}" --no-create-home --shell /usr/sbin/nologin ssg \
    && mkdir -p /site /tmp/public \
    && chown -R ssg:ssg /tmp/public

COPY --from=builder /build/target/release/ssg /usr/local/bin/ssg

USER ssg:ssg
WORKDIR /site

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/ssg"]
CMD ["serve", "--source", "/site", "--output", "/tmp/public", "--address", "0.0.0.0:8080"]
