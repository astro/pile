#!/usr/bin/env bash

export PATH=~/.cargo/bin:$PATH

#xargo build --target=thumbv7em-none-eabihf --release \
#    || exit 1
cargo rustc -j1 --target=thumbv7em-none-eabihf --release \
    || exit 1

killall openocd
sleep 0.1

BIN=target/thumbv7em-none-eabihf/release/rstripee
openocd \
    -f /usr/share/openocd/scripts/interface/stlink-v2-1.cfg \
    -f /usr/share/openocd/scripts/target/stm32f4x.cfg \
    -c init \
    -c "reset halt" \
    -c "flash write_image erase $BIN" \
    -c "reset run" &
sleep 3
gdb-multiarch $BIN
