#!/bin/sh

~/.cargo/bin/cross build --release --target armv7-unknown-linux-gnueabihf && rsync ../target/armv7-unknown-linux-gnueabihf/release/interface aahneji@192.168.0.10:~
