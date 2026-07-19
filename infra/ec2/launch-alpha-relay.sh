#!/bin/bash
# Deploy a single, STABLE external relay node for the Lucas<->Josh alpha
# test (real cross-internet, not the disposable farm-sim fleet).
#
# Distinct from launch-farm-sim.sh in every way that matters for real usage:
# - ONE instance, not seven -- this is a rendezvous point, not a test topology.
# - Security group opens the P2P port to 0.0.0.0/0 (the whole internet),
#   not just the VPC CIDR -- real clients on home fiber/cellular/WiFi need
#   to reach it directly, they aren't other EC2 instances in the same VPC.
# - Separate tag key (Purpose=AlphaRelay, not FarmSim=true) so farm-sim's
#   own teardown script -- which is deliberately aggressive/iterate-happy --
#   can never match and terminate this by accident.
# - No casual teardown: `teardown` here requires typing the exact relay's
#   instance ID, not just "yes", because real people are relying on this
#   staying up.
#
# Instance type: originally t3.micro (the only type IAM allowed at first),
# escalated to m7i-flex.large after a live t3.micro build genuinely stalled
# -- confirmed via direct SSH inspection (2026-07-18) that CARGO_BUILD_JOBS=1
# does not prevent cargo from running one host-context and one target-context
# compile of uniffi_bindgen CONCURRENTLY (two ~230MB rustc processes, same
# PIDs, same start times, near-zero CPU-time growth across repeated checks
# over 3+ hours -- genuine thrashing stall, not OOM, not just slow).
# CARGO_BUILD_JOBS bounds jobs within cargo's own host/target unit graphs
# separately, not across them, so t3.micro's 913MB can't safely fit both
# concurrent copies regardless of that setting. m7i-flex.large's 8GB
# absorbs this without any swapping. ec2:CreateVpc/CreateSubnet remain
# denied -- this still reuses the account's existing default VPC.
#
# Usage:
#   ./launch-alpha-relay.sh launch   [region]
#   ./launch-alpha-relay.sh status   [region]
#   ./launch-alpha-relay.sh teardown [region]   # requires typing the instance ID to confirm
set -euo pipefail

export PATH="$PATH:/c/Users/SCM/AppData/Roaming/Python/Python314/Scripts"

ACTION=${1:-launch}
REGION=${2:-us-east-1}
INSTANCE_TYPE="m7i-flex.large"
KEY_NAME="scmessenger-farm-sim-key-v2"
GIT_REPO="https://github.com/Sovereign-Communication/SCMessenger.git"
SG_NAME="scmessenger-alpha-relay-sg"
STATE_FILE="$(dirname "$0")/alpha-relay-state.json"
USERDATA_FILE="$(dirname "$0")/alpha-relay-userdata.sh"
P2P_PORT=9001
HTTP_STATUS_PORT=9000
HEALTH_PORT=8080

log() { echo "[INFO] $*" >&2; }
ok()  { echo "[OK] $*" >&2; }
err() { echo "[ERROR] $*" >&2; }

discover_vpc_and_subnet() {
  log "Discovering default VPC..."
  VPC_ID=$(aws ec2 describe-vpcs --region "$REGION" \
    --filters "Name=isDefault,Values=true" \
    --query 'Vpcs[0].VpcId' --output text)
  if [ "$VPC_ID" = "None" ] || [ -z "$VPC_ID" ]; then
    err "No default VPC found in $REGION."
    exit 1
  fi
  ok "Default VPC: $VPC_ID"

  SUBNET_ID=$(aws ec2 describe-subnets --region "$REGION" \
    --filters "Name=vpc-id,Values=$VPC_ID" "Name=default-for-az,Values=true" \
    --query 'sort_by(Subnets,&AvailabilityZone)[0].SubnetId' --output text)
  if [[ ! "$SUBNET_ID" =~ ^subnet- ]]; then
    err "Could not resolve a default subnet: got '$SUBNET_ID'"
    exit 1
  fi
  ok "Subnet: $SUBNET_ID"
}

discover_ami() {
  log "Discovering latest Ubuntu 22.04 LTS AMI..."
  AMI_ID=$(aws ec2 describe-images --region "$REGION" \
    --owners 099720109477 \
    --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" \
               "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
  if [ "$AMI_ID" = "None" ] || [ -z "$AMI_ID" ]; then
    err "Could not resolve an Ubuntu 22.04 AMI"
    exit 1
  fi
  ok "AMI: $AMI_ID"
}

ensure_security_group() {
  log "Checking for existing security group '$SG_NAME'..."
  SG_ID=$(aws ec2 describe-security-groups --region "$REGION" \
    --filters "Name=group-name,Values=$SG_NAME" "Name=vpc-id,Values=$VPC_ID" \
    --query 'SecurityGroups[0].GroupId' --output text 2>/dev/null || echo "None")

  if [ "$SG_ID" != "None" ] && [ -n "$SG_ID" ]; then
    ok "Reusing existing security group: $SG_ID"
    return
  fi

  log "Creating security group '$SG_NAME'..."
  SG_ID=$(aws ec2 create-security-group --region "$REGION" \
    --group-name "$SG_NAME" \
    --description "Alpha-test external relay -- real internet-facing, not VPC-internal" \
    --vpc-id "$VPC_ID" \
    --query 'GroupId' --output text)

  if [[ ! "$SG_ID" =~ ^sg-[0-9a-f]+$ ]]; then
    err "create-security-group did not return a valid SG ID: '$SG_ID'"
    exit 1
  fi

  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol tcp --port 22 --cidr 0.0.0.0/0 >/dev/null
  # P2P port MUST be open to the whole internet -- Lucas (fiber) and Josh
  # (cellular/WiFi) are real remote clients, not other instances in this VPC.
  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol tcp --port $P2P_PORT --cidr 0.0.0.0/0 >/dev/null
  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol udp --port $P2P_PORT --cidr 0.0.0.0/0 >/dev/null
  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol tcp --port $HTTP_STATUS_PORT --cidr 0.0.0.0/0 >/dev/null
  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol tcp --port $HEALTH_PORT --cidr 0.0.0.0/0 >/dev/null

  ok "Created security group: $SG_ID (SSH + P2P:$P2P_PORT tcp/udp + status:$HTTP_STATUS_PORT + health:$HEALTH_PORT, all 0.0.0.0/0)"
}

do_launch() {
  if [ -f "$STATE_FILE" ]; then
    err "State file $STATE_FILE already exists -- an alpha-relay may already be running."
    err "Run '$0 status $REGION' to check, or teardown first if you really want a fresh one."
    exit 1
  fi

  discover_vpc_and_subnet
  discover_ami
  ensure_security_group

  local userdata_content
  userdata_content=$(cat "$USERDATA_FILE")

  log "Launching alpha-relay..."
  local instance_id
  instance_id=$(aws ec2 run-instances --region "$REGION" \
    --image-id "$AMI_ID" \
    --instance-type "$INSTANCE_TYPE" \
    --key-name "$KEY_NAME" \
    --subnet-id "$SUBNET_ID" \
    --security-group-ids "$SG_ID" \
    --block-device-mappings "[{\"DeviceName\":\"/dev/sda1\",\"Ebs\":{\"VolumeSize\":20,\"VolumeType\":\"gp3\",\"DeleteOnTermination\":true}}]" \
    --user-data "$userdata_content" \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=scm-alpha-relay},{Key=Purpose,Value=AlphaRelay}]" \
    --query 'Instances[0].InstanceId' --output text)

  if [[ ! "$instance_id" =~ ^i-[0-9a-f]+$ ]]; then
    err "run-instances did not return a valid instance ID: '$instance_id'"
    exit 1
  fi
  ok "Instance: $instance_id"

  log "Waiting for 'running' state..."
  aws ec2 wait instance-running --region "$REGION" --instance-ids "$instance_id"

  local public_ip
  public_ip=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$instance_id" \
    --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)

  cat > "$STATE_FILE" <<EOF
{
  "region": "$REGION",
  "vpc_id": "$VPC_ID",
  "security_group_id": "$SG_ID",
  "instance_id": "$instance_id",
  "public_ip": "$public_ip",
  "p2p_port": $P2P_PORT,
  "http_status_port": $HTTP_STATUS_PORT,
  "health_port": $HEALTH_PORT
}
EOF

  ok "State saved to $STATE_FILE"
  echo ""
  echo "=== ALPHA-TEST RELAY LAUNCHED ==="
  cat "$STATE_FILE"
  echo ""
  echo "[NOTE] Instance is running but still executing user-data (swap setup +"
  echo "        apt install + git clone + cargo build --release, serial build,"
  echo "        expect 45-90 min on a t3.micro). Poll with:"
  echo "        $0 status $REGION"
  echo ""
  echo "Once ready, both Lucas and Josh should bootstrap with:"
  echo "  SC_BOOTSTRAP_NODES=/ip4/${public_ip}/tcp/${P2P_PORT}"
}

do_status() {
  if [ ! -f "$STATE_FILE" ]; then
    err "No state file at $STATE_FILE -- run '$0 launch' first"
    exit 1
  fi
  local instance_id
  instance_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['instance_id'])")
  aws ec2 describe-instances --region "$REGION" --instance-ids "$instance_id" \
    --query 'Reservations[0].Instances[0].[InstanceId,State.Name,PublicIpAddress]' \
    --output table
}

do_teardown() {
  if [ ! -f "$STATE_FILE" ]; then
    err "No state file at $STATE_FILE -- nothing to tear down"
    exit 0
  fi
  local instance_id sg_id
  instance_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['instance_id'])")
  sg_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['security_group_id'])")

  echo "[TEARDOWN] This is the REAL alpha-test relay Lucas and Josh use -- not a disposable test node."
  echo "[TEARDOWN] Type the exact instance ID ($instance_id) to confirm you mean to terminate it:"
  read -r confirm
  if [ "$confirm" != "$instance_id" ]; then
    echo "[CANCEL] Input did not match instance ID -- teardown cancelled"
    exit 0
  fi

  aws ec2 terminate-instances --region "$REGION" --instance-ids "$instance_id"
  log "Waiting for termination..."
  aws ec2 wait instance-terminated --region "$REGION" --instance-ids "$instance_id"

  log "Deleting security group $sg_id..."
  if aws ec2 delete-security-group --region "$REGION" --group-id "$sg_id"; then
    ok "Security group deleted"
  else
    err "SG deletion failed -- $sg_id needs manual cleanup"
  fi

  rm -f "$STATE_FILE"
  ok "Teardown complete"
}

case "$ACTION" in
  launch)   do_launch ;;
  status)   do_status ;;
  teardown) do_teardown ;;
  *) err "Usage: $0 [launch|status|teardown] [region]"; exit 1 ;;
esac
