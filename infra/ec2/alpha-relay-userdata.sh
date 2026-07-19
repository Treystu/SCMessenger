#!/bin/bash
# UserData for the alpha-test external relay node.
#
# This is REAL infrastructure for a real cross-internet alpha test (Lucas's
# Windows CLI + Android emulator, Josh's Windows-compiled build + physical
# Android phone on cellular/WiFi) -- not the disposable farm-sim fleet.
# Runs `scm relay` (the purpose-built headless relay command: no stdin,
# runs forever, listed in cli/src/main.rs) rather than `scm start`.
#
# Escalated from t3.micro to m7i-flex.large (2026-07-18) after a live
# t3.micro build genuinely stalled: confirmed via direct SSH inspection that
# CARGO_BUILD_JOBS=1 does not stop cargo from running one host-context and
# one target-context compile of uniffi_bindgen CONCURRENTLY (two ~230MB
# rustc processes competing for a 913MB box -- CARGO_BUILD_JOBS bounds jobs
# within each unit graph separately, not across host vs target). m7i-flex
# .large's 8GB comfortably fits both without swapping, so CARGO_BUILD_JOBS
# is set to 2 (matching vCPU count, not 1) and swap is a much smaller
# defense-in-depth margin rather than the primary fix.
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
git clone --depth 1 --branch main https://github.com/Sovereign-Communication/SCMessenger.git SCMessenger
cd SCMessenger

docker build --build-arg CARGO_BUILD_JOBS=2 -t scmessenger:latest -f docker/Dockerfile .
docker builder prune -af || true

# --network host + explicit `scm relay ...` command: bypasses
# entrypoint.sh's `scm start`-only flag injection entirely (see
# docker/entrypoint.sh line 52 -- its --http-bind/--port auto-injection
# only fires when $2="start"). Explicit flags here instead:
#   --http-bind 0.0.0.0:8080   -- health check API (matches farm-sim convention)
#   --listen /ip4/0.0.0.0/tcp/9001 -- fixed P2P port, memorable for both
#                                     Lucas and Josh's bootstrap config
#   --http-port 9000           -- status/landing page
#   --name alpha-relay          -- shows up in logs/status cleanly
docker run -d \
  --name scm-alpha-relay \
  --network host \
  --restart unless-stopped \
  -e RUST_LOG=info,scmessenger=debug \
  scmessenger:latest \
  scm --http-bind 0.0.0.0:8080 relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000 --name alpha-relay

echo "[OK] alpha-relay user-data complete"
