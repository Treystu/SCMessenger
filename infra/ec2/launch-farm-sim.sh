#!/bin/bash
# Direct EC2 API deployment for the farm-sim 7-node topology.
#
# WHY NOT CLOUDFORMATION: the scmessenger-relay-orchestrator IAM user has
# cloudformation:ValidateTemplate, cloudformation:ListStacks, ec2:CreateVpc
# and ec2:CreateSubnet all explicitly denied (verified via dry-run/no-op
# probes on 2026-07-18). It DOES have ec2:RunInstances, ec2:TerminateInstances,
# ec2:CreateSecurityGroup, ec2:AuthorizeSecurityGroupIngress, ec2:CreateTags,
# ec2:DescribeImages/Vpcs/Subnets. So: no custom VPC (use the account's
# existing default VPC/subnets, which already span every AZ), no
# CloudFormation (call the EC2 API directly).
#
# Usage:
#   ./launch-farm-sim.sh launch   [region]   # create SG + 7 instances
#   ./launch-farm-sim.sh status   [region]   # show instance state from saved IDs
#   ./launch-farm-sim.sh teardown [region]   # terminate all 7 + delete SG
set -euo pipefail

export PATH="$PATH:/c/Users/SCM/AppData/Roaming/Python/Python314/Scripts"

ACTION=${1:-launch}
REGION=${2:-us-east-1}
INSTANCE_TYPE="t3.micro"
KEY_NAME="scmessenger-farm-sim-key"
GIT_REPO="https://github.com/Sovereign-Communication/SCMessenger.git"
GIT_REF="main"
SG_NAME="scmessenger-farm-sim-sg"
STATE_FILE="$(dirname "$0")/farm-sim-state.json"
USERDATA_TEMPLATE="$(dirname "$0")/node-userdata-template.sh"

log() { echo "[INFO] $*"; }
ok()  { echo "[OK] $*"; }
err() { echo "[ERROR] $*" >&2; }

render_userdata() {
  # args: node_name listen_port bootstrap
  local node_name=$1 listen_port=$2 bootstrap=$3
  sed \
    -e "s|___NODE_NAME___|${node_name}|g" \
    -e "s|___LISTEN_PORT___|${listen_port}|g" \
    -e "s|___BOOTSTRAP___|${bootstrap}|g" \
    -e "s|___GIT_REPO___|${GIT_REPO}|g" \
    -e "s|___GIT_REF___|${GIT_REF}|g" \
    "$USERDATA_TEMPLATE"
}

discover_vpc_and_subnets() {
  log "Discovering default VPC..."
  VPC_ID=$(aws ec2 describe-vpcs --region "$REGION" \
    --filters "Name=isDefault,Values=true" \
    --query 'Vpcs[0].VpcId' --output text)
  if [ "$VPC_ID" = "None" ] || [ -z "$VPC_ID" ]; then
    err "No default VPC found in $REGION. This script relies on the account's default VPC (ec2:CreateVpc is denied for this IAM user)."
    exit 1
  fi
  ok "Default VPC: $VPC_ID"

  log "Discovering default subnets (one per AZ, need 3)..."
  mapfile -t SUBNETS < <(aws ec2 describe-subnets --region "$REGION" \
    --filters "Name=vpc-id,Values=$VPC_ID" "Name=default-for-az,Values=true" \
    --query 'sort_by(Subnets,&AvailabilityZone)[*].SubnetId' --output text | tr '\t' '\n')

  if [ "${#SUBNETS[@]}" -lt 3 ]; then
    err "Need at least 3 default subnets, found ${#SUBNETS[@]}"
    exit 1
  fi
  SUBNET_A="${SUBNETS[0]}"
  SUBNET_B="${SUBNETS[1]}"
  SUBNET_C="${SUBNETS[2]}"
  ok "Subnet A: $SUBNET_A | Subnet B: $SUBNET_B | Subnet C: $SUBNET_C"
}

discover_ami() {
  log "Discovering latest Ubuntu 22.04 LTS AMI (Canonical, owner 099720109477)..."
  AMI_ID=$(aws ec2 describe-images --region "$REGION" \
    --owners 099720109477 \
    --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" \
               "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
  if [ "$AMI_ID" = "None" ] || [ -z "$AMI_ID" ]; then
    err "Could not resolve an Ubuntu 22.04 AMI in $REGION"
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
    --description "Farm-sim node SSH + inter-node P2P/health traffic" \
    --vpc-id "$VPC_ID" \
    --query 'GroupId' --output text)

  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol tcp --port 22 --cidr 0.0.0.0/0 >/dev/null

  # Intra-VPC (default VPC is 172.31.0.0/16) all-protocol rule: Phase 3
  # failure-injection tests use `tc netem` INSIDE each instance to simulate
  # latency/loss, not VPC-level firewalling, so this just needs to not be
  # in the way of P2P discovery (mDNS/QUIC/relay) and the 8080 health check.
  VPC_CIDR=$(aws ec2 describe-vpcs --region "$REGION" --vpc-ids "$VPC_ID" \
    --query 'Vpcs[0].CidrBlock' --output text)
  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol -1 --cidr "$VPC_CIDR" >/dev/null

  ok "Created security group: $SG_ID (VPC CIDR $VPC_CIDR open internally)"
}

launch_node() {
  # args: node_name subnet_id listen_port bootstrap
  local node_name=$1 subnet_id=$2 listen_port=$3 bootstrap=$4
  log "Launching $node_name (subnet $subnet_id, port $listen_port, bootstrap='$bootstrap')..."

  local userdata_file
  userdata_file=$(mktemp)
  render_userdata "$node_name" "$listen_port" "$bootstrap" > "$userdata_file"

  local instance_id
  instance_id=$(aws ec2 run-instances --region "$REGION" \
    --image-id "$AMI_ID" \
    --instance-type "$INSTANCE_TYPE" \
    --key-name "$KEY_NAME" \
    --subnet-id "$subnet_id" \
    --security-group-ids "$SG_ID" \
    --block-device-mappings "[{\"DeviceName\":\"/dev/sda1\",\"Ebs\":{\"VolumeSize\":16,\"VolumeType\":\"gp3\",\"DeleteOnTermination\":true}}]" \
    --user-data "file://$userdata_file" \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=scm-${node_name}},{Key=FarmSim,Value=true},{Key=Node,Value=${node_name}}]" \
    --query 'Instances[0].InstanceId' --output text)

  rm -f "$userdata_file"

  # set -e does not reliably propagate out of a failure inside `$(...)`
  # when the substitution feeds `read <<<` (a known bash gotcha) -- so
  # explicitly validate the instance ID shape here rather than trusting
  # the caller's error handling to catch a failed run-instances call.
  if [[ ! "$instance_id" =~ ^i-[0-9a-f]+$ ]]; then
    err "run-instances for $node_name did not return a valid instance ID: '$instance_id'"
    exit 1
  fi

  local private_ip
  private_ip=$(aws ec2 describe-instances --region "$REGION" \
    --instance-ids "$instance_id" \
    --query 'Reservations[0].Instances[0].PrivateIpAddress' --output text)

  if [[ ! "$private_ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    err "Could not resolve a private IP for $node_name ($instance_id): got '$private_ip'"
    exit 1
  fi

  ok "$node_name -> $instance_id (private $private_ip)"
  echo "$instance_id:$private_ip"
}

do_launch() {
  discover_vpc_and_subnets
  discover_ami
  ensure_security_group

  # NOTE: `launch_node` runs inside a subshell whenever it's invoked via
  # `$(...)` (command substitution always forks one), so an `exit 1` inside
  # launch_node only kills that subshell -- it will NOT stop this script.
  # Capturing to a plain `var=$(...)` assignment first (rather than piping
  # straight into `read <<<`) makes the subshell's exit status visible to
  # `set -e` / an explicit `||`, so a failed run-instances call actually
  # halts the deploy instead of silently continuing with an empty ID.
  local result

  log "Launching relay1 (bootstrap root, no upstream)..."
  result=$(launch_node relay1 "$SUBNET_A" 4001 "") || { err "launch_node failed for relay1"; exit 1; }
  IFS=':' read -r RELAY1_ID RELAY1_IP <<< "$result"

  log "Launching relay2 (bootstraps off relay1)..."
  result=$(launch_node relay2 "$SUBNET_B" 4002 "/ip4/${RELAY1_IP}/tcp/4001") || { err "launch_node failed for relay2"; exit 1; }
  IFS=':' read -r RELAY2_ID RELAY2_IP <<< "$result"

  log "Launching user nodes alice/bob/carol/david (bootstrap off relay1)..."
  result=$(launch_node alice "$SUBNET_A" 0 "/ip4/${RELAY1_IP}/tcp/4001") || { err "launch_node failed for alice"; exit 1; }
  IFS=':' read -r ALICE_ID ALICE_IP <<< "$result"

  result=$(launch_node bob "$SUBNET_B" 0 "/ip4/${RELAY1_IP}/tcp/4001") || { err "launch_node failed for bob"; exit 1; }
  IFS=':' read -r BOB_ID BOB_IP <<< "$result"

  result=$(launch_node carol "$SUBNET_A" 0 "/ip4/${RELAY1_IP}/tcp/4001") || { err "launch_node failed for carol"; exit 1; }
  IFS=':' read -r CAROL_ID CAROL_IP <<< "$result"

  result=$(launch_node david "$SUBNET_B" 0 "/ip4/${RELAY1_IP}/tcp/4001") || { err "launch_node failed for david"; exit 1; }
  IFS=':' read -r DAVID_ID DAVID_IP <<< "$result"

  log "Launching eve (bootstraps off relay2 -- forces a 2-hop relay path)..."
  result=$(launch_node eve "$SUBNET_C" 0 "/ip4/${RELAY2_IP}/tcp/4002") || { err "launch_node failed for eve"; exit 1; }
  IFS=':' read -r EVE_ID EVE_IP <<< "$result"

  log "Waiting for all 7 instances to reach 'running' state..."
  aws ec2 wait instance-running --region "$REGION" --instance-ids \
    "$RELAY1_ID" "$RELAY2_ID" "$ALICE_ID" "$BOB_ID" "$CAROL_ID" "$DAVID_ID" "$EVE_ID"

  log "Fetching public IPs..."
  PUBLIC_IPS_JSON=$(aws ec2 describe-instances --region "$REGION" \
    --instance-ids "$RELAY1_ID" "$RELAY2_ID" "$ALICE_ID" "$BOB_ID" "$CAROL_ID" "$DAVID_ID" "$EVE_ID" \
    --query 'Reservations[*].Instances[0].[Tags[?Key==`Node`].Value|[0],InstanceId,PublicIpAddress,PrivateIpAddress]' \
    --output json)

  cat > "$STATE_FILE" <<EOF
{
  "region": "$REGION",
  "vpc_id": "$VPC_ID",
  "security_group_id": "$SG_ID",
  "nodes": $(echo "$PUBLIC_IPS_JSON" | python -c "
import json, sys
rows = json.load(sys.stdin)
print(json.dumps({r[0]: {'instance_id': r[1], 'public_ip': r[2], 'private_ip': r[3]} for r in rows}, indent=2))
")
}
EOF

  ok "State saved to $STATE_FILE"
  echo ""
  echo "=== FARM-SIM 7-NODE TOPOLOGY LAUNCHED ==="
  cat "$STATE_FILE"
  echo ""
  echo "[NOTE] Instances are running but still executing user-data (swap setup"
  echo "        + apt install + git clone + cargo build --release). Expect"
  echo "        15-25 minutes before the scm-node container is actually up on"
  echo "        each instance. Poll with: $0 status $REGION"
}

do_status() {
  if [ ! -f "$STATE_FILE" ]; then
    err "No state file at $STATE_FILE -- run '$0 launch' first"
    exit 1
  fi
  local ids
  ids=$(python -c "import json; d=json.load(open('$STATE_FILE')); print(' '.join(n['instance_id'] for n in d['nodes'].values()))")
  aws ec2 describe-instances --region "$REGION" --instance-ids $ids \
    --query 'Reservations[*].Instances[0].[Tags[?Key==`Node`].Value|[0],InstanceId,State.Name,PublicIpAddress]' \
    --output table
}

do_teardown() {
  if [ ! -f "$STATE_FILE" ]; then
    err "No state file at $STATE_FILE -- nothing to tear down"
    exit 0
  fi
  local ids sg_id
  ids=$(python -c "import json; d=json.load(open('$STATE_FILE')); print(' '.join(n['instance_id'] for n in d['nodes'].values()))")
  sg_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['security_group_id'])")

  echo "[TEARDOWN] About to terminate: $ids"
  read -p "Type 'yes' to confirm: " confirm
  if [ "$confirm" != "yes" ]; then
    echo "[CANCEL] Teardown cancelled"
    exit 0
  fi

  aws ec2 terminate-instances --region "$REGION" --instance-ids $ids
  log "Waiting for termination..."
  aws ec2 wait instance-terminated --region "$REGION" --instance-ids $ids

  log "Deleting security group $sg_id..."
  aws ec2 delete-security-group --region "$REGION" --group-id "$sg_id" || \
    log "SG deletion failed (may still be referenced) -- retry manually later"

  rm -f "$STATE_FILE"
  ok "Teardown complete"
}

case "$ACTION" in
  launch)   do_launch ;;
  status)   do_status ;;
  teardown) do_teardown ;;
  *) err "Usage: $0 [launch|status|teardown] [region]"; exit 1 ;;
esac
