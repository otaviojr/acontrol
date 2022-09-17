#!/bin/bash
export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig
cross build --release --target arm-unknown-linux-gnueabihf