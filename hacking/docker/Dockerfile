#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    bash-completion \
    curl \
    git \
    sudo \
    man \
    procps \
    vim \
    rsync \
    && rm -rf /var/lib/apt/lists/*

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

# Convenient shell completion and aliases
RUN curl -sSL -o /etc/bash_completion.d/nix \
        https://raw.githubusercontent.com/hedning/nix-bash-completions/e6db3081fe1f221470a26e345a96855e5f09ddec/_nix

ARG UID
ARG GID

RUN set -eux; \
    if ! getent group $GID; then \
        groupadd -g $GID x; \
    fi; \
    if ! getent passwd $UID; then \
        useradd -u $UID -g $GID -G sudo -m -p x x; \
    fi;

RUN mkdir -m 0755 /nix && chown $UID:$GID /nix

USER $UID

WORKDIR /tmp

RUN curl -sSf https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain none

COPY bashrc_extra .
RUN dst=~/.bashrc_extra && cp bashrc_extra $dst && echo ". $dst" >> ~/.bashrc

COPY inputrc .
RUN cp inputrc ~/.inputrc

COPY setup.sh .

ARG STATEFUL

RUN if [ "$STATEFUL" = "1" ]; then bash setup.sh; fi

COPY nix.conf /etc/nix/

WORKDIR /work
