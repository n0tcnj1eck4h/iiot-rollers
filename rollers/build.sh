#!/bin/sh

~/.cargo/bin/cross build --release --target armv7-unknown-linux-gnueabih && rsync ../target/armv7-unknown-linux-gnueabihf/release/rollers aahneji@192.168.0.10:~
