#!/bin/bash
# Per-node UserData template for farm-sim micro-instance topology.
# Placeholders (___NODE_NAME___, ___LISTEN_PORT___, ___BOOTSTRAP___, ___GIT_REPO___,
# ___GIT_REF___) are substituted by launch-farm-sim.sh before being passed to
# ec2 run-instances --user-data.
#
# Adds swap before building, and caps cargo's build parallelism via
# --build-arg (see docker/Dockerfile). Confirmed via live SSH into a
# building node on 2026-07-18: even with the Dockerfile's
# CARGO_PROFILE_RELEASE_LTO=false override, cargo runs multiple rustc
# processes concurrently by default, and on the original 1GiB t3.micro this
# pushed the instance into 2.4GB+ of active swap usage within ~30-40 minutes
# (load average 2.3 on a single vCPU) -- real, observed swap-thrashing, not
# a theoretical risk. CARGO_BUILD_JOBS=___CARGO_JOBS___ bounds peak
# concurrent memory directly; the swapfile size below is sized as
# defense-in-depth headroom on top of that, not as the primary fix.
set -ex
exec > /var/log/user-data.log 2>&1

fallocate -l ___SWAP_SIZE___ /swapfile
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

docker build --build-arg CARGO_BUILD_JOBS=___CARGO_JOBS___ -t scmessenger:latest -f docker/Dockerfile .
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
