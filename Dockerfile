# ================ #
#    Base Stage    #
# ================ #

FROM rust:slim-bullseye as base

ENV DOCKER_BUILDKIT=0
WORKDIR usr/src/app

RUN apt-get update && \
        apt-get upgrade -y --no-install-recommends && \
#        apt-get install -y --no-install-recommends build-essential autoconf automake cmake libtool libssl-dev pkg-config && \
	    apt-get clean && \
        rm -rf /var/lib/apt/lists/*

COPY Cargo.toml .
COPY Cargo.lock .

# ================ #
#   Builder Stage  #
# ================ #

FROM base as builder

COPY src/ src/

RUN cargo build --release --locked

# ================ #
#   Runner Stage   #
# ================ #

FROM base as runner

COPY --from=builder usr/src/app/target/release/cdn .

CMD ["./cdn"]
