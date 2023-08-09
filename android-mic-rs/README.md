experimental project written in Rust

backend for Widows and Linux.
Technically, the code could run on Android

supported
- UDP
- TCP

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
Usage: android-mic-rs.exe [OPTIONS] --ip <IP>

Options:
  -i, --ip <IP>                  example: -i 192.168.1.79
  -m, --mode <connection mode>   UDP or TCP [default: UDP]
  -f, --format <audio format>    i16 or i32 [default: i16]
  -o, --output <output device>   [default: 0]
  -c, --channel <channel count>  1 or 2
  -r, --sample <sample rate>     should not have default config because it depend on the divice
  -h, --help                     Print help
  -V, --version                  Print version
```

example:
```
cargo run --release -- --ip 192.168.1.79 -m UDP
```



clear && cargo r -- -i 192.168.1.79


todo: 
- support multiple audio format: done
- choose output device: done
- stereo: done
- parse ipv4/v6: done, no support for v6 tho
- release socket if necesseray: not needed i think
- stop audio when disconnect 
- try ASIO backend