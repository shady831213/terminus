#!/bin/sh

set -x
sudo ip link add br0 type bridge
sudo ip tuntap add dev tap0 mode tap user $(whoami)
sudo ip link set tap0 master br0
sudo ip link set dev br0 up
sudo ip link set dev tap0 up

sudo ifconfig br0 192.168.3.1
#echo 1 > /proc/sys/net/ipv4/ip_forward
sudo sysctl -w net.ipv4.ip_forward=1
sudo iptables -D FORWARD 1
sudo iptables -t nat -A POSTROUTING -o ens33 -j MASQUERADE