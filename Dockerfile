# Rust as the base image
FROM rust:1.58.1-slim-buster

# 1. Create a new empty shell project
RUN USER=root cargo new --bin stock
WORKDIR /stock

# 2. Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# 3. Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# 4. Now that the dependency is built, copy your source code
COPY ./src ./src

# 5. Build for release.
RUN rm ./target/release/deps/stock*
RUN cargo install --path .

CMD ["stock"]