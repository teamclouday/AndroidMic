experimental project written in Rust

backend for Widows and Linux.
Technically, the code could run on Android

supported
- UDP
- TCP

how to build

- install rust: https://www.rust-lang.org/tools/install

```shell
cargo build --release
```


format code
```shell
cargo clippy --all --fix --allow-dirty --allow-staged
cargo fmt --all
```

on Linux, you need alsa dev dep

Fedora
```shell
sudo dnf install alsa-lib-devel
```

Debian
```shell
sudo apt install libasound2-dev
```

```shell
Usage: android-mic-rs.exe [OPTIONS] --ip <IP>

Options:
  -i, --ip <IP>                  example: -i 192.168.1.79
  -m, --mode <connection mode>   UDP or TCP [default: UDP]
  -f, --format <audio format>    i16 or i32 [default: i16]
  -d, --device <output device>   [default: 0]
  -c, --channel <channel count>  1 or 2
  -r, --sample <sample rate>
  -h, --help                     Print help
  -V, --version                  Print version
```

example:
```shell
./target/release/android-mic-rs.exe --ip 192.168.1.79 -m UDP
```


advanced:
```shell
clear && cargo run --release -- --ip 192.168.1.79 --mode UDP --channel 2 -f i16 --device 4
```

todo: 
- support multiple audio format: done but not tested
- support multiple sample: done but not tested
- choose output device: done
- stereo: implemented but have bugs
- parse ipv4/v6: done, no support for v6 tho
- release socket if necesseray: not needed i think
- detect TCP disconnect 
- try ASIO backend