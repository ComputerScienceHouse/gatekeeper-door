FROM rust:1.24

apt-get update -y
apt-get install -y gcc-arm-linux-gnueabihf clang libnfc-dev
apt-get clean -y all
rustup target add armv7-unknown-linux-gnueabihf
mkdir -p ~/.cargo
cat >>~/.cargo/config <<EOF
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
EOF

cargo build --target=armv7-unknown-linux-gnueabihf