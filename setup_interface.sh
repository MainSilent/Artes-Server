#!/usr/bin/env bash
set -e

TUNNEL_NAME=$1
IP_ADDRESS=$2
NIC=$(ip -4 route ls | grep default | grep -Po '(?<=dev )(\S+)' | head -1)
DNS=$(ip -4 route ls | grep default | grep -Po '(?<=via )(\S+)' | head -1)

ip tuntap del "${TUNNEL_NAME}" mode tun
ip tuntap add "${TUNNEL_NAME}" mode tun
ip link set "${TUNNEL_NAME}" up
ip addr add ${IP_ADDRESS}/16 dev "${TUNNEL_NAME}"

echo 1 > /proc/sys/net/ipv4/ip_forward
iptables -A FORWARD -i $TUNNEL_NAME -o $NIC -j ACCEPT
iptables -A FORWARD -i $NIC -o $TUNNEL_NAME -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -t nat -A POSTROUTING -o $NIC -j MASQUERADE

# DNS route
iptables -t nat -A PREROUTING -p tcp --dport 53 -j DNAT --to-destination $DNS -i $TUNNEL_NAME
iptables -t nat -A PREROUTING -p udp --dport 53 -j DNAT --to-destination $DNS -i $TUNNEL_NAME