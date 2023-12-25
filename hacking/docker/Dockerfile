#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM nixos/nix

RUN nix-channel --update

RUN nix-env -i -A nixpkgs.shadow -A nixpkgs.getent

RUN set -eux; \
    for f in passwd shadow group; do \
        dst=/etc/$f; \
        src=$(readlink --canonicalize-existing $dst); \
        rm $dst; \
        cp $src $dst; \
    done

ARG UID
ARG GID

RUN set -eux; \
    if ! getent passwd $UID && ! getent group $GID; then \
        groupadd sudo; \
        groupadd -g $GID x; \
        useradd -u $UID -g $GID -G sudo -m -p x x; \
    else \
        [ $UID = 0 -a $GID = 0 ]; \
    fi

# no setuid
# RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

RUN chown -R $UID:$GID /nix

USER $UID

ENV USER=x

RUN [ $(whoami) = $USER ]

# FROM debian:bookworm

# RUN apt-get update && apt-get install -y \
#     curl xz-utils \
#     && rm -rf /var/lib/apt/lists/*

# RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

# ENV RUSTUP_HOME=/opt/rustup
# ENV CARGO_HOME=/opt/cargo

# RUN mkdir -p -m 0755 /nix $RUSTUP_HOME $CARGO_HOME && chown $UID:$GID /nix $RUSTUP_HOME $CARGO_HOME

# USER $UID

# COPY nix.conf /etc/nix/

# RUN echo 'f=~/.nix-profile/etc/profile.d/nix-daemon.sh && [ -f $f ] && . $f && unset f ' >> ~/.bashrc

# WORKDIR /work

# ---

# # Convenient shell completion and aliases
# RUN curl -sSL -o /etc/bash_completion.d/nix \
#         https://raw.githubusercontent.com/hedning/nix-bash-completions/e6db3081fe1f221470a26e345a96855e5f09ddec/_nix

# RUN curl -sSf https://sh.rustup.rs | \
#         bash -s -- -y --no-modify-path --default-toolchain none

# COPY bashrc_extra .
# RUN dst=~/.bashrc_extra && cp bashrc_extra $dst && echo ". $dst" >> ~/.bashrc

# COPY inputrc .
# RUN cp inputrc ~/.inputrc


# WORKDIR /tmp

# COPY setup.sh .

# ARG STATEFUL

# RUN if [ "$STATEFUL" = "1" ]; then bash setup.sh; fi
