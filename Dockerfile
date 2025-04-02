FROM rust:1.85 as builder

# RUN apk add --no-cache musl-dev gcc make
# RUN apk add --no-cache openssl openssl-dev openssl-libs-static pkgconfig

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release || true

COPY . .

RUN cargo build --release --bin stress

FROM debian:bookworm

RUN apt-get update && apt-get install -y chrony && rm -rf /var/lib/apt/lists/*

RUN echo "makestep 1 3" > /etc/chrony/chrony.conf && \
    echo "rtcsync" >> /etc/chrony/chrony.conf && \
    echo "local stratum 10" >> /etc/chrony/chrony.conf && \
    echo "allow" >> /etc/chrony/chrony.conf

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/stress .

CMD ./stress