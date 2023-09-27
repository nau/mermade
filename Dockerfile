# Use the official Rust image as the build stage
FROM rust:1.72 as builder

# Set the working directory inside the Docker image
WORKDIR /usr/src/mermade

# Copy the Cargo.toml and Cargo.lock files to the image
COPY Cargo.toml Cargo.lock ./

# Copy the source code to the image
COPY src ./src

# Compile the application
RUN cargo build --release

# Use the official Debian image as the runtime stage
FROM debian:bullseye-slim
# Install necessary libraries
RUN apt-get update && \
  apt-get install -y openssl ca-certificates && \
  rm -rf /var/lib/apt/lists/*

# Copy the binary from the build stage to the runtime stage
COPY --from=builder /usr/src/mermade/target/release/mermade /usr/local/bin/mermade

# Set the binary as the entry point of the container
CMD ["mermade"]
