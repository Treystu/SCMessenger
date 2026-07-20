#!/usr/bin/env bash
# Provision the SCMessenger farm-relay EC2 instance (FARM_FINAL_PLAN.md B4).
#
# Every action below is a plain, individually-reviewable `aws` CLI call - no
# Terraform state file, no black-box plan/apply. Read each command before
# running this for real; it is a DRY RUN (prints commands, executes nothing)
# unless you pass --apply.
#
# Prerequisites:
#   - AWS CLI v2 installed (https://aws.amazon.com/cli/) - not installed on
#     this machine as of this script's authoring; install it yourself.
#   - Credentials from infra/aws/set-aws-credentials.sh already written to
#     ~/.config/scmorc/aws.env (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY,
#     AWS_DEFAULT_REGION=us-east-1), using the scoped IAM policy at
#     iam-policy-scmessenger-relay.json (NOT AmazonEC2FullAccess).
#
# What this creates (all free-tier eligible, matches the IAM policy's limits):
#   - One security group allowing exactly the AD-3 port ladder: tcp/443,
#     tcp/80, udp/443 (relay listen ports) + tcp/22 (SSH, restricted to your
#     current public IP only, not 0.0.0.0/0).
#   - One t3.micro EC2 instance (free tier: 750 hrs/month), 20GB gp3 root
#     volume (well under the 30GB free allowance), Amazon Linux 2023 AMI.
#   - user_data that installs Docker and runs the scm-cli-node relay image
#     (cloud/mesh/Dockerfile.cli) with --http-bind 0.0.0.0:9876 for health
#     checks (B3, already landed on main).
#
# Usage:
#   bash infra/aws/provision-relay.sh           # dry run, prints every command
#   bash infra/aws/provision-relay.sh --apply   # actually runs them

set -euo pipefail

APPLY=false
if [ "${1:-}" = "--apply" ]; then
    APPLY=true
fi

if [ -f "$HOME/.config/scmorc/aws.env" ]; then
    set -a
    # shellcheck source=/dev/null
    source "$HOME/.config/scmorc/aws.env"
    set +a
fi

REGION="${AWS_DEFAULT_REGION:-us-east-1}"
INSTANCE_TYPE="t3.micro"
KEY_NAME="scmessenger-relay-key"
SG_NAME="scmessenger-relay-sg"
TAG_NAME="scmessenger-farm-relay"

run() {
    echo "+ $*"
    if $APPLY; then
        "$@"
    fi
}

echo "=== SCMessenger relay provisioning ($( $APPLY && echo APPLY || echo DRY-RUN ), region=$REGION) ==="
echo

if ! command -v aws >/dev/null 2>&1; then
    echo "[ERROR] aws CLI not found. Install it, run 'aws configure' or rely on"
    echo "        ~/.config/scmorc/aws.env, then re-run this script."
    exit 1
fi

MY_IP="$(curl -s https://checkip.amazonaws.com || echo '0.0.0.0')"
echo "[INFO] Your current public IP (for SSH restriction): $MY_IP"
echo

# 1. Key pair (only if it doesn't already exist)
echo "--- Step 1: SSH key pair ---"
if aws ec2 describe-key-pairs --key-names "$KEY_NAME" --region "$REGION" >/dev/null 2>&1; then
    echo "[INFO] Key pair '$KEY_NAME' already exists, skipping."
else
    run aws ec2 create-key-pair --key-name "$KEY_NAME" --region "$REGION" \
        --query 'KeyMaterial' --output text
    echo "[NOTE] Save the printed private key to a local .pem file yourself and"
    echo "       chmod 400 it - this script does not write it to disk for you."
fi
echo

# 2. Security group - exactly the AD-3 port ladder + restricted SSH
echo "--- Step 2: Security group ($SG_NAME) ---"
run aws ec2 create-security-group --group-name "$SG_NAME" --region "$REGION" \
    --description "SCMessenger farm relay - AD-3 port ladder only" \
    --tag-specifications "ResourceType=security-group,Tags=[{Key=Name,Value=$SG_NAME}]"

run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
    --protocol tcp --port 443 --cidr 0.0.0.0/0
run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
    --protocol tcp --port 80 --cidr 0.0.0.0/0
run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
    --protocol udp --port 443 --cidr 0.0.0.0/0
run aws ec2 authorize-security-group-ingress --group-name "$SG_NAME" --region "$REGION" \
    --protocol tcp --port 22 --cidr "${MY_IP}/32"
echo

# 3. Latest Amazon Linux 2023 AMI (free-tier eligible)
echo "--- Step 3: Resolve AMI ---"
AMI_ID="resolve:ssm:/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64"
echo "[INFO] Using SSM parameter alias for latest Amazon Linux 2023: $AMI_ID"
echo

# 4. user_data: install Docker, run the relay container with health checks
USER_DATA=$(cat <<'CLOUDINIT'
#!/bin/bash
dnf install -y docker
systemctl enable --now docker
docker pull scm-cli-node:latest 2>/dev/null || true
docker run -d --restart unless-stopped \
    --name scm-relay \
    -p 443:443 -p 80:80 -p 443:443/udp -p 9876:9876 \
    scm-cli-node:latest \
    scm relay --http-bind 0.0.0.0:9876
CLOUDINIT
)

# 5. Launch the instance
echo "--- Step 5: Launch instance ($INSTANCE_TYPE) ---"
run aws ec2 run-instances \
    --image-id "$AMI_ID" \
    --instance-type "$INSTANCE_TYPE" \
    --key-name "$KEY_NAME" \
    --security-groups "$SG_NAME" \
    --region "$REGION" \
    --block-device-mappings 'DeviceName=/dev/xvda,Ebs={VolumeSize=20,VolumeType=gp3}' \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$TAG_NAME}]" \
    --user-data "$USER_DATA"

echo
echo "=== Done. Next: note the returned instance's public IP/DNS, point your"
echo "    DDNS hostname at it (AD-2), and verify http://<ip>:9876/health ==="
