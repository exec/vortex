# Vortex Development Container
FROM rust:1.75-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    pkg-config \
    libssl-dev \
    build-essential \
    jq \
    wget \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust tools for development
RUN cargo install \
    cargo-audit \
    cargo-watch \
    cargo-edit \
    cargo-expand

# Install GitHub CLI for CI/CD integration
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install gh -y

# Set up workspace
WORKDIR /workspace

# Configure git to be safe for any user
RUN git config --global --add safe.directory '*'

# Default command
CMD ["sleep", "infinity"]