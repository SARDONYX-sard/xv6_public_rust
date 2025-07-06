FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# hadolint ignore=DL3008
RUN useradd -ms /bin/bash vscode && \
  apt-get update && apt-get install -y sudo --no-install-recommends && \
  apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  echo 'vscode ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
  build-essential \
  ca-certificates \
  curl \
  gdb \
  git \
  gnupg \
  grub-pc-bin \
  nasm \
  python3 \
  qemu-system-x86 \
  unzip  \
  vim && \
  apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN curl -sSfL https://git.io/.gdbinit -o "$HOME/.gdbinit"

# Rust
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
# hadolint ignore=SC1091
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --profile minimal && \
  . "$HOME/.cargo/env"

ENV PATH="/root/.cargo/bin:${PATH}"

USER vscode
WORKDIR /workspace
