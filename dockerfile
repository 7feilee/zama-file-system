FROM rust:1.80 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release 
RUN db/gen_files.sh

FROM debian:bookworm-slim

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    openssl \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/zama-file-system /usr/local/bin/
COPY --from=builder /usr/src/app/db /usr/src/app/db

RUN chmod +x /usr/local/bin/zama-file-system

ENTRYPOINT ["zama-file-system"]
CMD ["server"]