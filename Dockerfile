FROM ubuntu:18.04

RUN apt-get update -qq && apt-get install -y \
  git \
  cmake \
  gcc \
  g++ \
  pkg-config \
  libssl-dev \
  curl \
  llvm \
  clang \
  libpq-dev \
  ca-certificates

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
RUN chmod +x rustup.sh
RUN ./rustup.sh -y
# RUN rustup target add x86_64-unknown-linux-gnu

WORKDIR /rust/src
