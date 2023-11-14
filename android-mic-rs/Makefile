
run:
	clear && cargo run --release

fix:
	cargo clippy --all --fix --allow-dirty --allow-staged
	cargo fmt --all

mine:
	clear && cargo run --release -- --ip 192.168.0.31 --mode TCP --channel 1 -f i16 -s 16000