# ------------------------------------------------------------------------------
# Cargo Dependency Prepare Stage
# ------------------------------------------------------------------------------
FROM rust:latest as planner

WORKDIR lrds
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Cargo Cache Stage
# ------------------------------------------------------------------------------
FROM rust:latest as cacher
WORKDIR lrds
RUN cargo install cargo-chef
COPY --from=planner /lrds/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Cargo Build Stage 
# ------------------------------------------------------------------------------
FROM rust:latest as builder

WORKDIR lrds
COPY . .

COPY --from=cacher lrds/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release 


# ------------------------------------------------------------------------------
# Final Stage 
# ------------------------------------------------------------------------------
FROM debian:buster-slim as runtime

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=lrds

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER

COPY --from=builder /lrds/target/release/liftright-data-server /usr/local/bin 
RUN chown -R $APP_USER:$APP_USER /usr/local/bin

USER $APP_USER

EXPOSE 3030
ENTRYPOINT ["/usr/local/bin/liftright-data-server"]

