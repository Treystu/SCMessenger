#!/bin/bash
# Per-node UserData template for farm-sim micro-instance topology.
# Placeholders (___NODE_NAME___, ___LISTEN_PORT___, ___BOOTSTRAP___, ___GIT_REPO___,
# ___GIT_REF___) are substituted by launch-farm-sim.sh before being passed to
# ec2 run-instances --user-data.
#
# Runs on Ubuntu 22.04. Adds swap before building: t3.micro has only 1GiB RAM,
# and `cargo build --release` on this workspace (libp2p, tokio, rustls, etc.)
# can exceed that during linking. Without swap the OOM killer silently kills
# rustc/the linker mid-build.
set -ex
exec > /var/log/user-data.log 2>&1

fallocate -l 2G /swapfile
chmod 600 /swapfile
mkswap /swapfile
swapon /swapfile
echo '/swapfile none swap sw 0 0' >> /etc/fstab

apt-get update
apt-get install -y docker.io git curl jq
systemctl start docker
systemctl enable docker

cd /opt
git clone --depth 1 --branch ___GIT_REF___ ___GIT_REPO___ SCMessenger
cd SCMessenger

docker build -t scmessenger:latest -f docker/Dockerfile .
docker builder prune -af || true

# --network host: libp2p binds directly to the instance's real interface
# instead of fighting Docker NAT -- matches the "IP is the source of truth"
# ledger design in cli/src/ledger.rs.
docker run -d \
  --name scm-node \
  --network host \
  --restart unless-stopped \
  -e RUST_LOG=info,scmessenger=debug \
  -e NODE_NAME=___NODE_NAME___ \
  -e LISTEN_PORT=___LISTEN_PORT___ \
  -e SC_BOOTSTRAP_NODES=___BOOTSTRAP___ \
  scmessenger:latest

echo "[OK] user-data complete for ___NODE_NAME___"
