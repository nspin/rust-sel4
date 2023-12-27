FROM nixos/nix:2.19.0 as bootstrap

RUN nix-channel --add https://nixos.org/channels/nixos-23.11 nixpkgs && nix-channel --update

COPY bootstrap.nix .

RUN nix-build bootstrap.nix --out-link /tmp/bootstrap-system

FROM scratch

COPY --from=bootstrap --link /tmp/bootstrap-system/root/ /

RUN /nix-support/setup

ENV USER=root
ENV PATH=/root/.nix-profile/bin

RUN rm -r /nix-support

RUN chmod 1777 /tmp /var/tmp

VOLUME /nix

WORKDIR /work/hacking/docker

FROM bootstrap

WORKDIR /work/hacking/docker
