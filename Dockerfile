# Rust as the base image
FROM rust:1.67.0 as build

# 1. Create a new empty shell project
RUN USER=root cargo new --bin open-stock
WORKDIR /open-stock

# 2. Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN apt-get -y update
RUN apt-get install -y libclang-dev libopencv-dev
RUN cargo build --release --locked
RUN rm src/*.rs

# 4. Now that the dependency is built, copy your source code
COPY ./src ./src

# build for release
# RUN rm ./target/release/deps/stock*
RUN cargo build --release --locked

# our final base
FROM rust:1.67.0

# copy the build artifact from the build stage
COPY --from=build /open-stock/target/release/open-stock .

EXPOSE 8000

# set the startup command to run your binary
ENTRYPOINT ["./open-stock"]