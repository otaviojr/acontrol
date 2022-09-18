#!/bin/bash
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/lib/arm-linux-gnueabi/pkgconfig
export PKG_CONFIG_LIBDIR=/lib/arm-linux-gnueabi/pkgconfig

#export PKG_CONFIG_ALLOW_CROSS=1
#export PKG_CONFIG_PATH=/lib/arm-linux-gnueabihf/pkgconfig
#export PKG_CONFIG_LIBDIR=/lib/arm-linux-gnueabihf/pkgconfig

#cross build --release --target arm-unknown-linux-gnueabihf --verbose
cargo build --release --target arm-unknown-linux-gnueabi --verbose
#cargo build --release --target arm-unknown-linux-gnueabihf --verbose
