FROM rust:1.79-bookworm as builder

WORKDIR /app

COPY . .

RUN cargo build --release

# ---

FROM debian:bookworm-slim

WORKDIR /app

RUN apt update && apt install -y libssl-dev python3

RUN ln -s /usr/bin/python3 /usr/bin/python

COPY --from=builder /app/target/release/backend torii
COPY --from=builder /app/examples examples

ENTRYPOINT ["/app/torii"]
