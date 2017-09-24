#!/usr/bin/env bash

set -e
set -x

DOWNLOAD_PATH=https://downloads.lede-project.org/releases/17.01.2/targets/brcm2708/bcm2708/

(
    SDK=lede-sdk-17.01.2-brcm2708-bcm2708_gcc-5.4.0_musl-1.1.16_eabi.Linux-x86_64
    [ -f ${SDK}.tar.xz ] || wget -c ${DOWNLOAD_PATH}/${SDK}.tar.xz
    [ -d ${SDK} ] || tar xfJ ${SDK}.tar.xz
    PATH=$PATH:$(pwd)/${SDK}/staging_dir/toolchain-arm_arm1176jzf-s+vfp_gcc-5.4.0_musl-1.1.16_eabi/bin

    cd ustriped
    rm -f ustriped *.o
    make ustriped CC=arm-openwrt-linux-gcc
    arm-openwrt-linux-strip ustriped
)

IMAGEBUILDER=lede-imagebuilder-17.01.2-brcm2708-bcm2708.Linux-x86_64
(
    [ -f ${IMAGEBUILDER}.tar.xz ] || wget -c ${DOWNLOAD_PATH}/${IMAGEBUILDER}.tar.xz
    [ -d ${IMAGEBUILDER} ] || tar xfJ ${IMAGEBUILDER}.tar.xz
    cd ${IMAGEBUILDER}

    mkdir -p files/usr/local/bin files/etc/init.d files/etc/rc.d files/etc/config
    cp ../ustriped/ustriped files/usr/local/bin/
    cat > files/etc/init.d/ustriped <<EOF
#!/bin/sh /etc/rc.common

USE_PROCD=1

start_service() {
  procd_open_instance
  procd_set_param command /usr/local/bin/ustriped
  procd_close_instance
}
EOF
    chmod a+x files/etc/init.d/ustriped
    ln -f -s ../init.d/ustriped files/etc/rc.d/S99ustriped
    cat > files/etc/config/network <<EOF
config interface 'loopback'
	option ifname 'lo'
	option proto 'static'
	option ipaddr '127.0.0.1'
	option netmask '255.0.0.0'

config interface 'lan'
	option ifname 'eth0'
	option proto 'dhcp'
	option peerdns '1'
EOF

    make -j4 image FILES=files/ PACKAGES="-dnsmasq -firewall -wpad-mini -odhcpd -odhcp6c kmod-spi-dev kmod-spi-bcm2835 kmod-random-bcm2835"
)

du -hs $(find ${IMAGEBUILDER}/build_dir/ -iname \*.img)
