#!/bin/sh

set -x

sudo ip tuntap del tap0 mode tap
sudo ip link del br0 type bridge
sudo iptables -t nat -D POSTROUTING -o ens33 -j MASQUERADE