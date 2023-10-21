#!/bin/bash

sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    clang \
    gcc \
    libssl-dev \
    llvm \
    make \
    pkg-config \
    tmux \
    xz-utils \
    ufw

# Install Rust

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source \$HOME/.cargo/env

# Set CARGO_TARGET_DIR to reuse intermediate artifacts
export CARGO_TARGET_DIR=./target

# Install snarkOS
# cargo clean
cargo install --path .

echo "=================================================="
echo " Attention - Please ensure ports 4133 and 3033"
echo "             are enabled on your local network."
echo ""
echo " Cloud Providers - Enable ports 4133 and 3033"
echo "                   in your network firewall"
echo ""
echo " Home Users - Enable port forwarding or NAT rules"
echo "              for 4133 and 3033 on your router."
echo "=================================================="

# Open ports on system
sudo ufw allow 4133/tcp
sudo ufw allow 3033/tcp