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

# build for release or dev depending on what is desired
RUN if [ ${RELEASE_TYPE} = "dev" ]; then cargo build --locked ; else ROCKET_ENV=prod cargo build --release --locked ; fi

# move that file up the tree
RUN if [ ${RELEASE_TYPE} = "dev" ]; then cp /open-stock/target/debug/open-stock . ; else cp /open-stock/target/release/open-stock . ; fi

# our final base
FROM rust:1.70.0

# copy the build artifact from the build stage
COPY --from=build /open-stock .
COPY --from=build /open-stock/Rocket.toml .

ARG PORT=8080
ARG SECRET_KEY="OPEN_STOCK_SECRET_KEY_TEMPLATE"
ARG AORIGIN="*"
ARG ENV="prod"

ENV ROCKET_PORT ${PORT}
ENV ROCKET_SECRET_KEY ${SECRET_KEY}
ENV ROCKET_ENV ${ENV}
ENV ACCESS_ORIGIN ${AORIGIN}

EXPOSE ${ROCKET_PORT}

# set the startup command to run your binary
ENTRYPOINT ["./open-stock"]
