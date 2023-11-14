# experimental project written in Rust

Cli for Widows and Linux.
Technically, the code could run on Android

## Usage
```shell
Usage: android-mic-rs.exe [OPTIONS] --ip <IP>

Options:
  -i, --ip <IP>                  example: -i 192.168.1.79
  -m, --mode <connection mode>   UDP or TCP [default: UDP]
  -f, --format <audio format>    i16, f32, ... [default: i16]
  -d, --device <output device>   [default: 0]
  -c, --channel <channel count>  1 or 2
  -s, --sample <sample rate>     
  -i, --info-audio        
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

## Notes
To get your local ip on Linux, you can use this command:
```
ifconfig -a
```
You should the use the ip which begin with `192.168.`.

## Todo 
- support multiple audio format: done but have bugs
- support multiple sample: done but not tested
- choose output device: done
- stereo: implemented but have bugs
- parse ipv4/v6: done, no support for v6 tho
- release socket if necesseray: not needed i think
- detect TCP disconnect: Done
- try ASIO backend