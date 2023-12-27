FROM nixos/nix:2.19.0 as bootstrap

RUN nix-channel --add https://nixos.org/channels/nixos-23.11 nixpkgs && nix-channel --update

COPY bootstrap/ bootstrap/

RUN nix-build bootstrap/ --out-link /tmp/bootstrap-system

FROM scratch

COPY --from=bootstrap --link /tmp/bootstrap-system/root/ /

RUN /nix-support/setup

# HACK
ENV USER=root

# HACK
ENV PATH=/root/.nix-profile/bin

RUN rm -r /nix-support

RUN chmod 1777 /tmp /var/tmp

RUN nix-channel --add https://nixos.org/channels/nixos-23.11 nixpkgs && nix-channel --update

COPY env/ /tmp/env/

RUN d=/tmp/env && bash $d/setup.sh && rm -r $d

VOLUME /nix

WORKDIR /work

# FROM bootstrap
# WORKDIR /work/hacking/docker
