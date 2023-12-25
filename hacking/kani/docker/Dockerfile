#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update -q && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    bash-completion \
    curl \
    python3-pip \
    sudo \
    man \
    procps \
    vim \
    && rm -rf /var/lib/apt/lists/*

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

ARG UID
ARG GID

RUN set -eux; \
    if ! getent group $GID; then \
        groupadd -g $GID x; \
    fi; \
    if ! getent passwd $UID; then \
        useradd -u $UID -g $GID -G sudo -m -p x x; \
    fi

ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo

RUN mkdir -p -m 0755 $RUSTUP_HOME $CARGO_HOME && chown $UID:$GID $RUSTUP_HOME $CARGO_HOME

USER $UID

# Optimize by matching rust-toolchain.toml
ENV DEFAULT_TOOLCHAIN=nightly-2023-08-02

RUN curl -sSf https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain $DEFAULT_TOOLCHAIN

ENV PATH=$CARGO_HOME/bin:$PATH

RUN cargo install --locked kani-verifier && cargo kani setup

WORKDIR /work
