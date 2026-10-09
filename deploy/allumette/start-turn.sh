#!/bin/sh
set -eu
: "${TURN_SECRET:?}" "${TURN_DOMAIN:?}" "${VPS_PUBLIC_IP:?}"
umask 077
# Keep the shared secret in a private runtime file rather than a command argument/log.
cat > /tmp/allumette-turn.conf <<CONFIG
listening-port=3478
external-ip=${VPS_PUBLIC_IP}
realm=${TURN_DOMAIN}
fingerprint
use-auth-secret
static-auth-secret=${TURN_SECRET}
min-port=49160
max-port=49260
no-tls
no-dtls
no-cli
no-multicast-peers
no-loopback-peers
denied-peer-ip=10.0.0.0-10.255.255.255
denied-peer-ip=172.16.0.0-172.31.255.255
denied-peer-ip=192.168.0.0-192.168.255.255
denied-peer-ip=169.254.0.0-169.254.255.255
user-quota=8
total-quota=100
log-file=stdout
simple-log
CONFIG
exec turnserver -c /tmp/allumette-turn.conf
