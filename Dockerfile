FROM clux/muslrust AS builder

# Set the working directory inside the Docker image
WORKDIR /usr/src/mermade

# Copy the Cargo.toml and Cargo.lock files to the image
COPY Cargo.toml Cargo.lock ./

# Copy the source code to the image
COPY src ./src

# Compile the application
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest

# Copy the static binary from the builder stage
COPY --from=builder /usr/src/mermade/target/x86_64-unknown-linux-musl/release/mermade /usr/local/bin/mermade

# Set the binary as the entry point of the container
ENTRYPOINT ["mermade"]
