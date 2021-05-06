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

## Usage example

### Start server
```
dpf-pi --host 192.168.2.3
```

### Show image
```
curl -XPOST 'http://192.168.2.3:3000/image/show?mode=aspect_fit' -H'Content-Type: image/png' --data-binary @'rust-logo-512x512.png'
```

## License

[BSD 3-Clause License](LICENSE)

- rust-logo-512x512.png: [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/)