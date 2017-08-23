FROM ekidd/rust-musl-builder as builder
COPY src ./src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN ["cargo", "build" ,"--release"]

FROM alpine:3.6
ENV APP_DIR /app
WORKDIR $APP_DIR
RUN apk add --no-cache aspell aspell-fr vim

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/ruspell .
VOLUME $APP_DIR/input
VOLUME $APP_DIR/output

ENTRYPOINT ["./ruspell"]
