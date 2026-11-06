# Build image for packaging/build-appimage.sh.
#
# Ubuntu 22.04 keeps the AppImage compatible with glibc 2.35 and newer
# (Linux Mint 21+, Ubuntu 22.04+, Debian 12+). The host's newer glibc is not
# used, so the resulting binary does not require GLIBC_2.39.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        desktop-file-utils \
        file \
        libasound2-dev \
        libgtk-3-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.89.0 \
    && chmod -R a+rwX /usr/local/cargo /usr/local/rustup

# The wrapper mounts a named volume here so crate downloads survive rebuilds.
# The volume inherits this directory's permissions, which keeps it writable
# when the container runs as the host user.
RUN mkdir -p /usr/local/cargo/registry && chmod 777 /usr/local/cargo/registry

WORKDIR /src
