# dpf-pi

A GPU accelerated digital picture frame app for Raspberry Pi.

## Dependencies

- Rust 1.51.0
  - rustc 1.51.0 (2fd73fabe 2021-03-23)
  - cargo 1.51.0 (43b129a20 2021-03-16)


## Build

### Native build (on Raspberry Pi OS)

```
sudo apt install build-essential
cargo build --release
```

### Cross compile (on Ubuntu 20.04)
Some cross-compile tools are required. See [push.yml](.github/workflows/push.yml) for more details.

```
git clone --depth=1 https://github.com/raspberrypi/firmware /tmp/rpi-firmware
SYSROOT=/usr/arm-linux-gnueabihf VC_ROOT=/tmp/rpi-firmware/opt/vc cargo build --target arm-unknown-linux-gnueabihf
```

## Run
```
digiphoto
```

## License

[BSD 3-Clause License](LICENSE)