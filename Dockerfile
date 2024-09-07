# Use the official Rust image for Linux Intel x86_64 as the builder
FROM --platform=linux/amd64 rust:1.72 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin readme-proxy
WORKDIR /readme-proxy

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Use a minimal base image for Linux Intel x86_64 to run the application
FROM --platform=linux/amd64 debian:buster-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /readme-proxy/target/release/readme-proxy /usr/local/bin/readme-proxy

# Expose the port your application will run on
EXPOSE 8080

# Run the binary
CMD ["readme-proxy"]
