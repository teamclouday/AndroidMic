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
Options:
      --ip <IP>
  -m, --mode <CONNECTION MODE (UPD/TCP)>
  -h, --help                              Print help
  -V, --version                           Print version
```

example:
```
cargo run --release -- --ip 192.168.1.79 -m UDP
```



clear && cargo r -- -i 192.168.1.79


todo: 
- support multiple audio format: done
- stereo
- parse ipv4/v6
- choose output device
- release socket if necesseray
- stop audio when disconnect