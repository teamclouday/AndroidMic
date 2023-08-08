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

``` shell
Options:
      --ip <IP>
  -m, --mode <CONNECTION MODE (UPD/TCP)>
  -h, --help                              Print help
  -V, --version                           Print version
```

example:
cargo run --release -- --ip 192.168.1.79 -m UDP
