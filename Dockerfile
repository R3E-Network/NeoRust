FROM rust:latest

RUN apt-get update \
    && apt-get install -y libssl-dev gcc git make wget

RUN git clone https://github.com/baidu/rust-sgx-sdk.git
ENV RUST_SGX_SDK=/rust-sgx-sdk

WORKDIR /workspace