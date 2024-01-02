# experimental project written in Rust

Cli for Widows and Linux.
Technically, the code could run on Android

## Usage
```shell
Usage: android-mic [OPTIONS]

Options:
  -i, --ip <IP>                  example: -i 192.168.1.79
  -m, --mode <connection mode>   UDP or TCP [default: UDP]
  -f, --format <audio format>    i16, f32, ... [default: i16]
  -d, --device <output device>   [default: 0]
  -c, --channel <channel count>  1 or 2
  -s, --sample <sample rate>     16000, 44100, ...
      --info                     show supported audio config
  -h, --help                     Print help
  -V, --version                  Print version
```

## Build
1. install rust: https://www.rust-lang.org/tools/install

2. Build this thing:
  ```shell
  cargo build --release
  ```

## Deps
on Linux, you need alsa dev dep

Fedora
```shell
sudo dnf install alsa-lib-devel
```

Debian
```shell
sudo apt install libasound2-dev
```

## Todo 
- support multiple audio format: done but have bugs
- support multiple sample: done but not tested
- stereo support: done but laggy with my hardware. Need test
- choose output device: done
- choose input device
- args to specify endianess of the phone, and apply conversion if necessery
- add check to not create offset when using UDP, long format, multiple channels, ect...
- try ASIO backend