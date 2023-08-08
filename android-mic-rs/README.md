experimental project written in Rust

backend for Widows and Linux
support only UDP


how to build

- install rust: https://www.rust-lang.org/tools/install

```
cargo run --release
```


format code
```
cargo clippy --all --fix --allow-dirty --allow-staged
cargo fmt --all
```

on Linux, you need alsa dev dep

Fedora
```
sudo dnf install alsa-lib-devel
```

Debian
```
sudo apt install libasound2-dev
```