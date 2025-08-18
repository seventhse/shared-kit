convert:
  cargo tarpaulin --out html --all-targets

build-cli:
  cargo build -p shared-kit-cli --release --bins

shared-kit *args:
    just build-cli
    ./target/release/shared-kit {{args}}