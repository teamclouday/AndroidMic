
run:
	clear && cargo run --release

fix:
	cargo clippy --all --fix --allow-dirty --allow-staged
	cargo fmt --all

mine:
	clear && cargo run --release -- --mode TCP --channel 1 -f i16