#!/bin/bash
# Deploy a single m7i-flex.large instance to run a headless Android emulator,
# for real Android-to-Android testing through the alpha-relay before
# involving Josh's physical device.
#
# Separate from both the farm-sim fleet (disposable, t3.micro) and the
# alpha-relay (stable, t3.micro, runs `scm relay`) -- this needs real RAM
# (8GB) for the emulator itself, which t3.micro's 1GiB cannot provide.
# m7i-flex.large only became launchable once the IAM policy was widened
# to allow it (2026-07-18); t3.micro remains the only type allowed before
# that change.
set -euo pipefail

export PATH="$PATH:/c/Users/SCM/AppData/Roaming/Python/Python314/Scripts"

ACTION=${1:-launch}
REGION=${2:-us-east-1}
INSTANCE_TYPE="m7i-flex.large"
KEY_NAME="scmessenger-farm-sim-key-v2"
SG_NAME="scmessenger-android-emulator-sg"
STATE_FILE="$(dirname "$0")/android-emulator-state.json"

log() { echo "[INFO] $*" >&2; }
ok()  { echo "[OK] $*" >&2; }
err() { echo "[ERROR] $*" >&2; }

discover_vpc_and_subnet() {
  VPC_ID=$(aws ec2 describe-vpcs --region "$REGION" \
    --filters "Name=isDefault,Values=true" --query 'Vpcs[0].VpcId' --output text)
  SUBNET_ID=$(aws ec2 describe-subnets --region "$REGION" \
    --filters "Name=vpc-id,Values=$VPC_ID" "Name=default-for-az,Values=true" \
    --query 'sort_by(Subnets,&AvailabilityZone)[0].SubnetId' --output text)
  ok "VPC: $VPC_ID / Subnet: $SUBNET_ID"
}

discover_ami() {
  AMI_ID=$(aws ec2 describe-images --region "$REGION" \
    --owners 099720109477 \
    --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" \
               "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
  ok "AMI: $AMI_ID"
}

ensure_security_group() {
  SG_ID=$(aws ec2 describe-security-groups --region "$REGION" \
    --filters "Name=group-name,Values=$SG_NAME" "Name=vpc-id,Values=$VPC_ID" \
    --query 'SecurityGroups[0].GroupId' --output text 2>/dev/null || echo "None")

  if [ "$SG_ID" != "None" ] && [ -n "$SG_ID" ]; then
    ok "Reusing SG: $SG_ID"
    return
  fi

  SG_ID=$(aws ec2 create-security-group --region "$REGION" \
    --group-name "$SG_NAME" \
    --description "Android emulator test node -- SSH + adb only" \
    --vpc-id "$VPC_ID" --query 'GroupId' --output text)

  aws ec2 authorize-security-group-ingress --region "$REGION" \
    --group-id "$SG_ID" --protocol tcp --port 22 --cidr 0.0.0.0/0 >/dev/null
  # adb over SSH tunnel only (5037 forwarded locally), no need to expose
  # emulator console/adb ports (5554-5585) to the internet directly.

  ok "Created SG: $SG_ID"
}

do_launch() {
  if [ -f "$STATE_FILE" ]; then
    err "State file exists -- an android-emulator node may already be running. Check '$0 status $REGION' first."
    exit 1
  fi

  discover_vpc_and_subnet
  discover_ami
  ensure_security_group

  local userdata='#!/bin/bash
set -ex
exec > /var/log/user-data.log 2>&1
apt-get update
apt-get install -y openjdk-17-jre-headless unzip qemu-kvm bridge-utils cpu-checker curl
usermod -aG kvm ubuntu || true
kvm-ok || echo "[WARN] KVM not available -- emulator will need software CPU emulation (-accel off), much slower"
echo "[OK] base packages installed"
'

  local instance_id
  instance_id=$(aws ec2 run-instances --region "$REGION" \
    --image-id "$AMI_ID" \
    --instance-type "$INSTANCE_TYPE" \
    --key-name "$KEY_NAME" \
    --subnet-id "$SUBNET_ID" \
    --security-group-ids "$SG_ID" \
    --block-device-mappings "[{\"DeviceName\":\"/dev/sda1\",\"Ebs\":{\"VolumeSize\":30,\"VolumeType\":\"gp3\",\"DeleteOnTermination\":true}}]" \
    --user-data "$userdata" \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=scm-android-emulator},{Key=Purpose,Value=AndroidEmulatorTest}]" \
    --query 'Instances[0].InstanceId' --output text)

  if [[ ! "$instance_id" =~ ^i-[0-9a-f]+$ ]]; then
    err "run-instances did not return a valid instance ID: '$instance_id'"
    exit 1
  fi
  ok "Instance: $instance_id"

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
  "public_ip": "$public_ip"
}
EOF

  ok "Launched: $instance_id ($public_ip)"
  echo "=== ANDROID EMULATOR NODE LAUNCHED ==="
  cat "$STATE_FILE"
}

do_status() {
  if [ ! -f "$STATE_FILE" ]; then
    err "No state file -- run '$0 launch' first"
    exit 1
  fi
  local instance_id
  instance_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['instance_id'])")
  aws ec2 describe-instances --region "$REGION" --instance-ids "$instance_id" \
    --query 'Reservations[0].Instances[0].[InstanceId,State.Name,PublicIpAddress]' --output table
}

do_teardown() {
  if [ ! -f "$STATE_FILE" ]; then
    err "No state file -- nothing to tear down"
    exit 0
  fi
  local instance_id sg_id
  instance_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['instance_id'])")
  sg_id=$(python -c "import json; print(json.load(open('$STATE_FILE'))['security_group_id'])")

  echo "[TEARDOWN] m7i-flex.large costs real money while running -- terminating $instance_id"
  read -p "Type 'yes' to confirm: " confirm
  if [ "$confirm" != "yes" ]; then
    echo "[CANCEL] cancelled"
    exit 0
  fi

  aws ec2 terminate-instances --region "$REGION" --instance-ids "$instance_id"
  aws ec2 wait instance-terminated --region "$REGION" --instance-ids "$instance_id"
  aws ec2 delete-security-group --region "$REGION" --group-id "$sg_id" || err "SG deletion failed, cleanup manually"
  rm -f "$STATE_FILE"
  ok "Teardown complete"
}

case "$ACTION" in
  launch)   do_launch ;;
  status)   do_status ;;
  teardown) do_teardown ;;
  *) err "Usage: $0 [launch|status|teardown] [region]"; exit 1 ;;
esac
