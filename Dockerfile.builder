FROM rust:1.83-bookworm

# Install system dependencies
RUN apt-get update && apt-get install -y \
    g++ \
    make \
    pkg-config \
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxkbcommon-x11-0 \
    libwayland-dev \
    libxkbcommon-dev \
    wget \
    curl \
    ca-certificates \
    gnupg \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Install Docker CLI
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && \
    apt-get install -y docker-ce-cli && \
    rm -rf /var/lib/apt/lists/*

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# Install sccache
RUN apt-get update && apt-get install -y sccache && rm -rf /var/lib/apt/lists/*

# Switch to nightly toolchain to match rust-toolchain.toml
RUN rustup default nightly

# Install rustfmt and clippy first
RUN rustup component add rustfmt clippy

# Add wasm target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli
RUN cargo install -f wasm-bindgen-cli --version 0.2.129

# Install wasm-opt (binaryen)
RUN wget https://github.com/WebAssembly/binaryen/releases/download/version_124/binaryen-version_124-x86_64-linux.tar.gz && \
    tar -xzf binaryen-version_124-x86_64-linux.tar.gz && \
    cp binaryen-version_124/bin/wasm-opt /usr/local/bin/ && \
    rm -rf binaryen-version_124 binaryen-version_124-x86_64-linux.tar.gz

# Set up sccache
ENV RUSTC_WRAPPER=sccache
ENV CARGO_INCREMENTAL=0
ENV CARGO_TERM_COLOR=always

# Verify installations
RUN rustc --version && \
    cargo --version && \
    wasm-bindgen --version && \
    wasm-opt --version && \
    sccache --version && \
    node --version && \
    npm --version && \
    docker --version

WORKDIR /workspace

LABEL org.opencontainers.image.source=https://github.com/bascanada/alacod
LABEL org.opencontainers.image.description="Container image for the Rust build environment with wasm support"

