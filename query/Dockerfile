FROM rust:1.76 as builder

RUN USER=root cargo new query
WORKDIR /query

# Build only the dependencies to cache them
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build
RUN rm src/*.rs

# Build for dev
COPY ./src ./src
RUN rm ./target/debug/deps/query*
RUN cargo build

# The final base image, only copying from builder
FROM debian:bookworm-slim

# Install OpenSSL 3 and other dependencies
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
# Verify the actual path of libssl.so.3
RUN find /usr -name libssl.so.3
# Set the shared library path
ENV LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu


COPY --from=builder /query/target/debug/query /usr/src/query


# Run the binary
CMD ["/usr/src/query"]