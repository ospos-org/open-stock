# Rust as the base image
FROM rust:1.70.0 as build

# 1. Create a new empty shell project
RUN USER=root cargo new --bin open-stock
WORKDIR /open-stock

# 2. Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN apt-get -y update
RUN apt-get install -y libclang-dev libopencv-dev
# --locked
RUN cargo build --release
RUN rm src/*.rs

# 4. Now that the dependency is built, copy your source code
COPY ./src ./src
COPY ./Rocket.toml ./

# build for release
# RUN rm ./target/release/deps/stock* .
RUN cargo build --release --locked

# our final base
FROM rust:1.70.0

# copy the build artifact from the build stage
COPY --from=build /open-stock/target/release/open-stock .
COPY --from=build /open-stock/Rocket.toml .

ARG PORT=8080
ENV PORT ${PORT}
EXPOSE ${PORT}

# set the startup command to run your binary
ENTRYPOINT ["./open-stock"]
