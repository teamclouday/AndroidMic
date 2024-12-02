
run:
	clear && cargo run --release

fix:
	cargo clippy --all --fix --allow-dirty --allow-staged
	cargo fmt --all

# you need to install cargo packager
# cargo install cargo-packager
nsis:
	cargo packager --release --formats nsis --verbose